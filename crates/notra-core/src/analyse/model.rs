pub type PatternId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);

    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn from_hex(value: &str) -> Option<Self> {
        let value = value.strip_prefix('#')?;
        if value.len() != 6 {
            return None;
        }
        Some(Self {
            red: u8::from_str_radix(&value[0..2], 16).ok()?,
            green: u8::from_str_radix(&value[2..4], 16).ok()?,
            blue: u8::from_str_radix(&value[4..6], 16).ok()?,
        })
    }

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AnalyseSearchType {
    #[default]
    Normal,
    Escaped,
    Regex,
    RegexMultiline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnalyseSelection {
    Text,
    #[default]
    Line,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysePattern {
    pub id: PatternId,
    pub order_num: String,
    pub enabled: bool,
    pub search_text: String,
    pub search_type: AnalyseSearchType,
    pub match_case: bool,
    pub whole_word: bool,
    pub selection: AnalyseSelection,
    pub hide: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub foreground: RgbColor,
    pub background: RgbColor,
    pub comment: String,
    pub group: String,
}

impl Default for AnalysePattern {
    fn default() -> Self {
        Self {
            id: 0,
            order_num: String::new(),
            enabled: true,
            search_text: String::new(),
            search_type: AnalyseSearchType::Normal,
            match_case: false,
            whole_word: false,
            selection: AnalyseSelection::Line,
            hide: false,
            bold: false,
            italic: false,
            underline: false,
            foreground: RgbColor::BLACK,
            background: RgbColor::WHITE,
            comment: String::new(),
            group: String::new(),
        }
    }
}

impl AnalysePattern {
    pub fn change_from(&self, previous: &Self) -> PatternChange {
        if self.id != previous.id
            || self.enabled != previous.enabled
            || self.search_text != previous.search_text
            || self.search_type != previous.search_type
            || self.match_case != previous.match_case
            || self.whole_word != previous.whole_word
        {
            return PatternChange::Search;
        }

        if self.selection != previous.selection
            || self.hide != previous.hide
            || self.bold != previous.bold
            || self.italic != previous.italic
            || self.underline != previous.underline
            || self.foreground != previous.foreground
            || self.background != previous.background
        {
            return PatternChange::Presentation;
        }

        if self.order_num != previous.order_num
            || self.comment != previous.comment
            || self.group != previous.group
        {
            return PatternChange::Metadata;
        }

        PatternChange::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternChange {
    None,
    Metadata,
    Presentation,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileLoadMode {
    Replace,
    Append,
    Prepend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSpan {
    pub line: usize,
    pub start_byte_in_line: usize,
    pub end_byte_in_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMatch {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub line_spans: Vec<LineSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysePatternError {
    pub pattern_id: PatternId,
    pub kind: AnalysePatternErrorKind,
    pub message: String,
}

impl AnalysePatternError {
    pub fn new(
        pattern_id: PatternId,
        kind: AnalysePatternErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            pattern_id,
            kind,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AnalysePatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pattern {}: {}", self.pattern_id, self.message)
    }
}

impl std::error::Error for AnalysePatternError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysePatternErrorKind {
    EmptyPattern,
    InvalidExtended,
    InvalidRegex,
    RegexBackendUnavailable,
    Cancelled,
    Backend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternResult {
    pub pattern_id: PatternId,
    pub matches: Vec<RawMatch>,
    pub error: Option<AnalysePatternError>,
    pub search_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinePatternMatch {
    pub pattern_id: PatternId,
    pub spans: Vec<LineSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveStyle {
    pub hidden: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub foreground: RgbColor,
    pub background: RgbColor,
}

impl Default for EffectiveStyle {
    fn default() -> Self {
        Self {
            hidden: false,
            bold: false,
            italic: false,
            underline: false,
            foreground: RgbColor::BLACK,
            background: RgbColor::WHITE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSegment {
    pub start_byte_in_line: usize,
    pub end_byte_in_line: usize,
    pub pattern_id: Option<PatternId>,
    pub style: EffectiveStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedLineResult {
    pub source_line: usize,
    pub text: String,
    pub matches: Vec<LinePatternMatch>,
    pub styled_segments: Vec<StyledSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyseResult {
    pub run_id: u64,
    pub document_id: u64,
    pub document_revision: u64,
    pub pattern_revision: u64,
    pub lines: Vec<MergedLineResult>,
    pub total_matches: usize,
    pub pattern_errors: Vec<AnalysePatternError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonacoRangeDto {
    pub start_line: u32,
    pub start_column_utf16: u32,
    pub end_line: u32,
    pub end_column_utf16: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_defaults_match_the_compatibility_spec() {
        let pattern = AnalysePattern::default();
        assert!(pattern.enabled);
        assert_eq!(pattern.search_type, AnalyseSearchType::Normal);
        assert_eq!(pattern.selection, AnalyseSelection::Line);
        assert_eq!(pattern.foreground, RgbColor::BLACK);
        assert_eq!(pattern.background, RgbColor::WHITE);
    }

    #[test]
    fn changes_are_classified_by_required_recomputation() {
        let previous = AnalysePattern::default();
        assert_eq!(previous.change_from(&previous), PatternChange::None);

        let mut metadata = previous.clone();
        metadata.comment = "network".to_owned();
        assert_eq!(metadata.change_from(&previous), PatternChange::Metadata);

        let mut presentation = previous.clone();
        presentation.bold = true;
        assert_eq!(
            presentation.change_from(&previous),
            PatternChange::Presentation
        );

        let mut search = previous.clone();
        search.match_case = true;
        assert_eq!(search.change_from(&previous), PatternChange::Search);
    }

    #[test]
    fn rgb_hex_is_strict_and_canonical() {
        assert_eq!(
            RgbColor::from_hex("#0a80FF"),
            Some(RgbColor::new(10, 128, 255))
        );
        assert_eq!(RgbColor::new(10, 128, 255).to_hex(), "#0A80FF");
        assert_eq!(RgbColor::from_hex("0A80FF"), None);
        assert_eq!(RgbColor::from_hex("#FFF"), None);
    }
}
