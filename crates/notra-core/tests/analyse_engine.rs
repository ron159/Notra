use notra_core::analyse::{AnalyseEngine, AnalyseRunInput, CancellationToken, parse_profile};
use serde::Deserialize;

const FIXTURE_TEXT: &str =
    include_str!("../../../tests/analyse-compat/fixtures/phase2-literal.txt");
const FIXTURE_PROFILE: &str =
    include_str!("../../../tests/analyse-compat/profiles/phase2-literal.xml");
const EXPECTED_JSON: &str =
    include_str!("../../../tests/analyse-compat/expected/phase2-literal.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedResult {
    oracle_status: String,
    source_lines: Vec<usize>,
    total_matches: usize,
    lines: Vec<ExpectedLine>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpectedLine {
    line: usize,
    matching_patterns: Vec<u64>,
    segments: Vec<ExpectedSegment>,
}

#[derive(Debug, Deserialize)]
struct ExpectedSegment {
    start: usize,
    end: usize,
    pattern: u64,
    hidden: bool,
    bold: bool,
    foreground: String,
    background: String,
}

#[test]
fn phase2_literal_core_matches_the_structured_contract() {
    let profile = parse_profile(FIXTURE_PROFILE, 1).unwrap();
    let result = AnalyseEngine::default()
        .run(
            AnalyseRunInput {
                run_id: 10,
                document_id: 20,
                document_revision: 30,
                pattern_revision: 40,
                text: FIXTURE_TEXT,
                patterns: &profile.patterns,
            },
            &CancellationToken::new(),
        )
        .unwrap();
    let expected: ExpectedResult = serde_json::from_str(EXPECTED_JSON).unwrap();

    assert_eq!(expected.oracle_status, "provisional_not_binary_verified");
    assert_eq!(result.total_matches, expected.total_matches);
    assert!(result.pattern_errors.is_empty());
    assert_eq!(
        result
            .lines
            .iter()
            .map(|line| line.source_line)
            .collect::<Vec<_>>(),
        expected.source_lines
    );
    assert_eq!(result.lines.len(), expected.lines.len());

    for (actual_line, expected_line) in result.lines.iter().zip(&expected.lines) {
        assert_eq!(actual_line.source_line, expected_line.line);
        assert_eq!(
            actual_line
                .matches
                .iter()
                .map(|line_match| line_match.pattern_id)
                .collect::<Vec<_>>(),
            expected_line.matching_patterns
        );
        assert_eq!(
            actual_line.styled_segments.len(),
            expected_line.segments.len()
        );

        for (actual, expected) in actual_line
            .styled_segments
            .iter()
            .zip(&expected_line.segments)
        {
            assert_eq!(actual.start_byte_in_line, expected.start);
            assert_eq!(actual.end_byte_in_line, expected.end);
            assert_eq!(actual.pattern_id, Some(expected.pattern));
            assert_eq!(actual.style.hidden, expected.hidden);
            assert_eq!(actual.style.bold, expected.bold);
            assert_eq!(actual.style.foreground.to_hex(), expected.foreground);
            assert_eq!(actual.style.background.to_hex(), expected.background);
        }
    }
}
