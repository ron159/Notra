use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use fancy_regex::{Regex as FancyRegex, RegexBuilder as FancyRegexBuilder};
use regex::{Regex, RegexBuilder};

use super::{
    AnalysePattern, AnalysePatternError, AnalysePatternErrorKind, AnalyseSearchType, LineSpan,
    MonacoRangeDto, PatternId, RawMatch, translate_analyse_extended,
};

const LITERAL_CHUNK_BYTES: usize = 64 * 1024;
const REGEX_BACKTRACK_LIMIT: usize = 1_000_000;
const UNIVERSAL_NEWLINE_PATTERN: &str =
    r"(?:\r\n|[\n\r\x0B\x0C\u{0085}\u{2028}\u{2029}])";

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

pub trait AnalyseMatcherBackend: Send + Sync {
    fn compile(
        &self,
        pattern: &AnalysePattern,
    ) -> Result<Box<dyn CompiledAnalyseMatcher>, AnalysePatternError>;
}

pub trait CompiledAnalyseMatcher: Send + Sync {
    fn find_all(
        &self,
        text: &str,
        line_index: &LineIndex,
        cancel: &CancellationToken,
    ) -> Result<Vec<RawMatch>, AnalysePatternError>;
}

pub struct AnalyseMatcherRouter {
    regex_backend: Option<Arc<dyn AnalyseMatcherBackend>>,
}

impl Default for AnalyseMatcherRouter {
    fn default() -> Self {
        Self::with_regex_backend(Arc::new(FunctionalRegexBackend))
    }
}

impl AnalyseMatcherRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_regex_backend(regex_backend: Arc<dyn AnalyseMatcherBackend>) -> Self {
        Self {
            regex_backend: Some(regex_backend),
        }
    }

    pub fn without_regex_backend() -> Self {
        Self {
            regex_backend: None,
        }
    }
}

impl AnalyseMatcherBackend for AnalyseMatcherRouter {
    fn compile(
        &self,
        pattern: &AnalysePattern,
    ) -> Result<Box<dyn CompiledAnalyseMatcher>, AnalysePatternError> {
        match pattern.search_type {
            AnalyseSearchType::Normal | AnalyseSearchType::Escaped => {
                compile_literal_pattern(pattern)
            }
            AnalyseSearchType::Regex | AnalyseSearchType::RegexMultiline => self
                .regex_backend
                .as_ref()
                .ok_or_else(|| {
                    AnalysePatternError::new(
                        pattern.id,
                        AnalysePatternErrorKind::RegexBackendUnavailable,
                        "Regex backend is not configured",
                    )
                })?
                .compile(pattern),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FunctionalRegexBackend;

impl AnalyseMatcherBackend for FunctionalRegexBackend {
    fn compile(
        &self,
        pattern: &AnalysePattern,
    ) -> Result<Box<dyn CompiledAnalyseMatcher>, AnalysePatternError> {
        if pattern.search_text.is_empty() {
            return Err(AnalysePatternError::new(
                pattern.id,
                AnalysePatternErrorKind::EmptyPattern,
                "search text is empty",
            ));
        }

        let search_text = translate_universal_newlines(&pattern.search_text);
        let flags = match pattern.search_type {
            AnalyseSearchType::Regex => "m",
            AnalyseSearchType::RegexMultiline => "ms",
            AnalyseSearchType::Normal | AnalyseSearchType::Escaped => {
                return Err(AnalysePatternError::new(
                    pattern.id,
                    AnalysePatternErrorKind::Backend,
                    "functional Regex backend received a non-Regex pattern",
                ));
            }
        };
        let search_text = format!("(?{flags}:{search_text})");
        let mut builder = FancyRegexBuilder::new(&search_text);
        builder
            .case_insensitive(!pattern.match_case)
            .backtrack_limit(REGEX_BACKTRACK_LIMIT);
        let regex = builder.build().map_err(|error| {
            AnalysePatternError::new(
                pattern.id,
                AnalysePatternErrorKind::InvalidRegex,
                error.to_string(),
            )
        })?;

        Ok(Box::new(FunctionalRegexMatcher {
            pattern_id: pattern.id,
            regex,
            whole_word: pattern.whole_word,
        }))
    }
}

struct FunctionalRegexMatcher {
    pattern_id: PatternId,
    regex: FancyRegex,
    whole_word: bool,
}

impl CompiledAnalyseMatcher for FunctionalRegexMatcher {
    fn find_all(
        &self,
        text: &str,
        line_index: &LineIndex,
        cancel: &CancellationToken,
    ) -> Result<Vec<RawMatch>, AnalysePatternError> {
        self.check_cancelled(cancel)?;
        let mut matches = Vec::new();
        for found in self.regex.find_iter(text) {
            self.check_cancelled(cancel)?;
            let found = found.map_err(|error| {
                AnalysePatternError::new(
                    self.pattern_id,
                    AnalysePatternErrorKind::Backend,
                    error.to_string(),
                )
            })?;
            if self.whole_word && !is_whole_word(text, found.start(), found.end()) {
                continue;
            }
            if let Some(raw_match) = line_index.raw_match(found.start(), found.end()) {
                matches.push(raw_match);
            }
        }
        Ok(matches)
    }
}

impl FunctionalRegexMatcher {
    fn check_cancelled(&self, cancel: &CancellationToken) -> Result<(), AnalysePatternError> {
        if cancel.is_cancelled() {
            Err(AnalysePatternError::new(
                self.pattern_id,
                AnalysePatternErrorKind::Cancelled,
                "Analyse run was cancelled",
            ))
        } else {
            Ok(())
        }
    }
}

fn translate_universal_newlines(pattern: &str) -> String {
    let mut translated = String::with_capacity(pattern.len());
    let mut characters = pattern.chars().peekable();
    let mut in_class = false;

    while let Some(character) = characters.next() {
        match character {
            '[' if !in_class => {
                in_class = true;
                translated.push(character);
            }
            ']' if in_class => {
                in_class = false;
                translated.push(character);
            }
            '\\' => {
                let mut slash_count = 1;
                while characters.peek() == Some(&'\\') {
                    characters.next();
                    slash_count += 1;
                }
                if !in_class && slash_count % 2 == 1 && characters.peek() == Some(&'R') {
                    characters.next();
                    translated.extend(std::iter::repeat_n('\\', slash_count - 1));
                    translated.push_str(UNIVERSAL_NEWLINE_PATTERN);
                } else {
                    translated.extend(std::iter::repeat_n('\\', slash_count));
                }
            }
            _ => translated.push(character),
        }
    }

    translated
}

fn compile_literal_pattern(
    pattern: &AnalysePattern,
) -> Result<Box<dyn CompiledAnalyseMatcher>, AnalysePatternError> {
    let search_text = match pattern.search_type {
        AnalyseSearchType::Normal => pattern.search_text.clone(),
        AnalyseSearchType::Escaped => {
            translate_analyse_extended(&pattern.search_text).map_err(|error| {
                AnalysePatternError::new(
                    pattern.id,
                    AnalysePatternErrorKind::InvalidExtended,
                    error.to_string(),
                )
            })?
        }
        AnalyseSearchType::Regex | AnalyseSearchType::RegexMultiline => unreachable!(),
    };
    if search_text.is_empty() {
        return Err(AnalysePatternError::new(
            pattern.id,
            AnalysePatternErrorKind::EmptyPattern,
            "search text is empty",
        ));
    }

    let regex = RegexBuilder::new(&regex::escape(&search_text))
        .case_insensitive(!pattern.match_case)
        .build()
        .map_err(|error| {
            AnalysePatternError::new(
                pattern.id,
                AnalysePatternErrorKind::Backend,
                error.to_string(),
            )
        })?;
    let overlap_bytes = search_text.chars().count().saturating_mul(4).max(1);
    Ok(Box::new(LiteralCompiledMatcher {
        pattern_id: pattern.id,
        regex,
        whole_word: pattern.whole_word,
        overlap_bytes,
    }))
}

struct LiteralCompiledMatcher {
    pattern_id: u64,
    regex: Regex,
    whole_word: bool,
    overlap_bytes: usize,
}

impl CompiledAnalyseMatcher for LiteralCompiledMatcher {
    fn find_all(
        &self,
        text: &str,
        line_index: &LineIndex,
        cancel: &CancellationToken,
    ) -> Result<Vec<RawMatch>, AnalysePatternError> {
        let mut matches = Vec::new();
        let mut chunk_start = 0;
        let mut next_search_start = 0;

        while chunk_start < text.len() {
            self.check_cancelled(cancel)?;
            let primary_end = floor_char_boundary(
                text,
                chunk_start
                    .saturating_add(LITERAL_CHUNK_BYTES)
                    .min(text.len()),
            );
            let primary_end = if primary_end == chunk_start {
                next_char_boundary(text, chunk_start)
            } else {
                primary_end
            };
            let search_end = ceil_char_boundary(
                text,
                primary_end
                    .saturating_add(self.overlap_bytes)
                    .min(text.len()),
            );

            let search_start = chunk_start.max(next_search_start);
            for found in self.regex.find_iter(&text[search_start..search_end]) {
                self.check_cancelled(cancel)?;
                let start = search_start + found.start();
                let end = search_start + found.end();
                if primary_end < text.len() && start >= primary_end {
                    break;
                }
                next_search_start = end;
                if self.whole_word && !is_whole_word(text, start, end) {
                    continue;
                }
                if let Some(raw_match) = line_index.raw_match(start, end) {
                    matches.push(raw_match);
                }
            }

            chunk_start = primary_end;
        }

        Ok(matches)
    }
}

impl LiteralCompiledMatcher {
    fn check_cancelled(&self, cancel: &CancellationToken) -> Result<(), AnalysePatternError> {
        if cancel.is_cancelled() {
            Err(AnalysePatternError::new(
                self.pattern_id,
                AnalysePatternErrorKind::Cancelled,
                "Analyse run was cancelled",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct LineIndex {
    text_len: usize,
    lines: Vec<LineRecord>,
}

#[derive(Debug, Clone, Copy)]
struct LineRecord {
    start: usize,
    content_end: usize,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut lines = Vec::new();
        let mut line_start = 0;
        let mut index = 0;

        while index < bytes.len() {
            match bytes[index] {
                b'\r' => {
                    lines.push(LineRecord {
                        start: line_start,
                        content_end: index,
                    });
                    index += usize::from(bytes.get(index + 1) == Some(&b'\n')) + 1;
                    line_start = index;
                }
                b'\n' => {
                    lines.push(LineRecord {
                        start: line_start,
                        content_end: index,
                    });
                    index += 1;
                    line_start = index;
                }
                _ => index += 1,
            }
        }
        lines.push(LineRecord {
            start: line_start,
            content_end: text.len(),
        });

        Self {
            text_len: text.len(),
            lines,
        }
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_text<'a>(&self, text: &'a str, line: usize) -> Option<&'a str> {
        if text.len() != self.text_len {
            return None;
        }
        let line = self.lines.get(line.checked_sub(1)?)?;
        text.get(line.start..line.content_end)
    }

    pub fn raw_match(&self, start: usize, end: usize) -> Option<RawMatch> {
        if start > end || end > self.text_len {
            return None;
        }
        let start_line_index = self.line_index_at(start);
        let end_line_index = if start == end {
            start_line_index
        } else {
            self.line_index_at(end - 1)
        };
        let mut line_spans = Vec::with_capacity(end_line_index - start_line_index + 1);

        for line_index in start_line_index..=end_line_index {
            let line = self.lines[line_index];
            let span_start = start.max(line.start).min(line.content_end);
            let span_end = end.min(line.content_end).max(span_start);
            line_spans.push(LineSpan {
                line: line_index + 1,
                start_byte_in_line: span_start - line.start,
                end_byte_in_line: span_end - line.start,
            });
        }

        Some(RawMatch {
            start_byte: start,
            end_byte: end,
            start_line: start_line_index + 1,
            end_line: end_line_index + 1,
            line_spans,
        })
    }

    pub fn monaco_range(&self, text: &str, raw_match: &RawMatch) -> Option<MonacoRangeDto> {
        if text.len() != self.text_len
            || raw_match.start_byte > raw_match.end_byte
            || raw_match.end_byte > self.text_len
            || !text.is_char_boundary(raw_match.start_byte)
            || !text.is_char_boundary(raw_match.end_byte)
        {
            return None;
        }

        let start_line_index = self.line_index_at(raw_match.start_byte);
        let end_line_index = self.line_index_at(raw_match.end_byte);
        let start_line = self.lines[start_line_index];
        let end_line = self.lines[end_line_index];
        let start_offset = raw_match.start_byte.min(start_line.content_end);
        let end_offset = raw_match.end_byte.min(end_line.content_end);

        Some(MonacoRangeDto {
            start_line: u32::try_from(start_line_index + 1).ok()?,
            start_column_utf16: utf16_column(text, start_line.start, start_offset)?,
            end_line: u32::try_from(end_line_index + 1).ok()?,
            end_column_utf16: utf16_column(text, end_line.start, end_offset)?,
        })
    }

    fn line_index_at(&self, offset: usize) -> usize {
        self.lines
            .partition_point(|line| line.start <= offset.min(self.text_len))
            .saturating_sub(1)
    }

}

fn is_whole_word(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
}

fn is_word_char(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

fn floor_char_boundary(text: &str, mut offset: usize) -> usize {
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn ceil_char_boundary(text: &str, mut offset: usize) -> usize {
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    offset
}

fn next_char_boundary(text: &str, offset: usize) -> usize {
    text[offset..]
        .chars()
        .next()
        .map_or(text.len(), |character| offset + character.len_utf8())
}

fn utf16_column(text: &str, line_start: usize, offset: usize) -> Option<u32> {
    let code_units = text.get(line_start..offset)?.encode_utf16().count();
    u32::try_from(code_units).ok()?.checked_add(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(search_text: &str) -> AnalysePattern {
        AnalysePattern {
            id: 7,
            search_text: search_text.to_owned(),
            ..Default::default()
        }
    }

    #[test]
    fn line_index_handles_mixed_endings() {
        let text = "one\r\ntwo\rthree\nfour";
        let index = LineIndex::new(text);
        assert_eq!(index.line_count(), 4);
        assert_eq!(index.line_text(text, 1), Some("one"));
        assert_eq!(index.line_text(text, 2), Some("two"));
        assert_eq!(index.line_text(text, 3), Some("three"));
        assert_eq!(index.line_text(text, 4), Some("four"));
    }

    #[test]
    fn raw_match_splits_multiline_byte_spans() {
        let text = "前置A\r\nB结尾";
        let index = LineIndex::new(text);
        let start = text.find('A').unwrap();
        let end = text.find('结').unwrap();
        let found = index.raw_match(start, end).unwrap();
        assert_eq!(found.start_line, 1);
        assert_eq!(found.end_line, 2);
        assert_eq!(found.line_spans.len(), 2);
        assert_eq!(found.line_spans[0].start_byte_in_line, "前置".len());
        assert_eq!(found.line_spans[0].end_byte_in_line, "前置A".len());
        assert_eq!(found.line_spans[1].start_byte_in_line, 0);
        assert_eq!(found.line_spans[1].end_byte_in_line, 1);
    }

    #[test]
    fn normal_search_is_case_insensitive_and_supports_whole_word() {
        let mut pattern = pattern("cat");
        pattern.whole_word = true;
        let matcher = AnalyseMatcherRouter::new().compile(&pattern).unwrap();
        let text = "Cat scatter cat_ cat";
        let found = matcher
            .find_all(text, &LineIndex::new(text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(&text[found[0].start_byte..found[0].end_byte], "Cat");
    }

    #[test]
    fn escaped_search_can_cross_crlf() {
        let mut pattern = pattern(r"A\r\nB");
        pattern.search_type = AnalyseSearchType::Escaped;
        let matcher = AnalyseMatcherRouter::new().compile(&pattern).unwrap();
        let text = "before A\r\nB after";
        let found = matcher
            .find_all(text, &LineIndex::new(text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].start_line, 1);
        assert_eq!(found[0].end_line, 2);
    }

    #[test]
    fn literal_matcher_finds_a_match_across_chunk_boundary() {
        let needle = "跨界pattern";
        let prefix = "a".repeat(LITERAL_CHUNK_BYTES - 3);
        let text = format!("{prefix}{needle} tail");
        let matcher = AnalyseMatcherRouter::new()
            .compile(&pattern(needle))
            .unwrap();
        let found = matcher
            .find_all(&text, &LineIndex::new(&text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(&text[found[0].start_byte..found[0].end_byte], needle);
    }

    #[test]
    fn chunking_preserves_global_non_overlapping_match_semantics() {
        let prefix = "x".repeat(LITERAL_CHUNK_BYTES - 1);
        let text = format!("{prefix}aaaa");
        let matcher = AnalyseMatcherRouter::new().compile(&pattern("aa")).unwrap();
        let found = matcher
            .find_all(&text, &LineIndex::new(&text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].start_byte, LITERAL_CHUNK_BYTES - 1);
        assert_eq!(found[1].start_byte, LITERAL_CHUNK_BYTES + 1);
    }

    #[test]
    fn converts_utf8_byte_offsets_to_monaco_utf16_columns() {
        let text = "中😀e\u{301}\n한";
        let index = LineIndex::new(text);
        let start = text.find('😀').unwrap();
        let end = text.find('\n').unwrap();
        let raw_match = index.raw_match(start, end).unwrap();
        assert_eq!(
            index.monaco_range(text, &raw_match),
            Some(MonacoRangeDto {
                start_line: 1,
                start_column_utf16: 2,
                end_line: 1,
                end_column_utf16: 6,
            })
        );
    }

    #[test]
    fn cancelled_literal_search_stops_before_scanning() {
        let matcher = AnalyseMatcherRouter::new()
            .compile(&pattern("missing"))
            .unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = matcher
            .find_all("text", &LineIndex::new("text"), &cancel)
            .unwrap_err();
        assert_eq!(error.kind, AnalysePatternErrorKind::Cancelled);
    }

    #[test]
    fn regex_backend_can_be_explicitly_disabled() {
        let mut pattern = pattern("foo.*bar");
        pattern.search_type = AnalyseSearchType::Regex;
        let error = match AnalyseMatcherRouter::without_regex_backend().compile(&pattern) {
            Ok(_) => panic!("Regex compiled without a backend"),
            Err(error) => error,
        };
        assert_eq!(error.kind, AnalysePatternErrorKind::RegexBackendUnavailable);
    }

    #[test]
    fn functional_regex_supports_anchors_lookaround_backreferences_and_posix_classes() {
        let text = "ok\nERROR wlan0\nfoobar\nfoo middle foo\ndigits 123";
        for (search_text, expected) in [
            (r"^ERROR", "ERROR"),
            (r"(?<=ERROR )wlan0", "wlan0"),
            (r"foo(?=bar)", "foo"),
            (r"(foo).*\1", "foo middle foo"),
            (r"[[:digit:]]+", "123"),
        ] {
            let mut pattern = pattern(search_text);
            pattern.search_type = AnalyseSearchType::Regex;
            pattern.match_case = true;
            let matcher = AnalyseMatcherRouter::new().compile(&pattern).unwrap();
            let found = matcher
                .find_all(text, &LineIndex::new(text), &CancellationToken::new())
                .unwrap();
            assert!(
                found.iter().any(|found| {
                    text.get(found.start_byte..found.end_byte) == Some(expected)
                }),
                "pattern {search_text:?} did not find {expected:?}"
            );
        }
    }

    #[test]
    fn regex_multiline_controls_dot_newline_matching() {
        let text = "left\nmiddle\nright";
        let mut normal = pattern("left.*right");
        normal.search_type = AnalyseSearchType::Regex;
        normal.match_case = true;
        let normal = AnalyseMatcherRouter::new().compile(&normal).unwrap();
        assert!(
            normal
                .find_all(text, &LineIndex::new(text), &CancellationToken::new())
                .unwrap()
                .is_empty()
        );

        let mut multiline = pattern("left.*right");
        multiline.search_type = AnalyseSearchType::RegexMultiline;
        multiline.match_case = true;
        let multiline = AnalyseMatcherRouter::new().compile(&multiline).unwrap();
        let found = multiline
            .find_all(text, &LineIndex::new(text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].start_line, 1);
        assert_eq!(found[0].end_line, 3);
    }

    #[test]
    fn regex_supports_universal_and_zero_length_matches() {
        let text = "a\r\nb\nc";
        let mut newline = pattern(r"\R");
        newline.search_type = AnalyseSearchType::Regex;
        let newline = AnalyseMatcherRouter::new().compile(&newline).unwrap();
        let found = newline
            .find_all(text, &LineIndex::new(text), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 2);
        assert_eq!(&text[found[0].start_byte..found[0].end_byte], "\r\n");

        let mut zero_length = pattern(r"(?=.)");
        zero_length.search_type = AnalyseSearchType::Regex;
        let zero_length = AnalyseMatcherRouter::new().compile(&zero_length).unwrap();
        let found = zero_length
            .find_all("ab", &LineIndex::new("ab"), &CancellationToken::new())
            .unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|found| found.start_byte == found.end_byte));
        assert!(found.iter().all(|found| found.start_line == 1));
    }

    #[test]
    fn regex_reports_invalid_patterns_and_honors_pre_cancel() {
        let mut invalid = pattern("(");
        invalid.search_type = AnalyseSearchType::Regex;
        let error = match AnalyseMatcherRouter::new().compile(&invalid) {
            Ok(_) => panic!("invalid Regex compiled"),
            Err(error) => error,
        };
        assert_eq!(error.kind, AnalysePatternErrorKind::InvalidRegex);

        let mut valid = pattern("text");
        valid.search_type = AnalyseSearchType::Regex;
        let matcher = AnalyseMatcherRouter::new().compile(&valid).unwrap();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let error = matcher
            .find_all("text", &LineIndex::new("text"), &cancel)
            .unwrap_err();
        assert_eq!(error.kind, AnalysePatternErrorKind::Cancelled);
    }
}
