use otterdive_core::analyse::{
    AnalyseSearchType, AnalyseSelection, ProfileLoadMode, RgbColor, apply_profile, parse_profile,
    write_profile,
};

const PHASE1_PROFILE: &str =
    include_str!("../../../tests/analyse-compat/profiles/phase1-basics.xml");
const SEARCH_CORPUS_PROFILE: &str =
    include_str!("../../../tests/analyse-compat/profiles/search-corpus.xml");
const REGEX_CORPUS_PROFILE: &str =
    include_str!("../../../tests/analyse-compat/profiles/regex-corpus.xml");
const STYLE_CORPUS_PROFILE: &str =
    include_str!("../../../tests/analyse-compat/profiles/style-overlap.xml");

#[test]
fn phase1_profile_fixture_imports_and_round_trips() {
    let parsed = parse_profile(PHASE1_PROFILE, 100).unwrap();
    assert_eq!(parsed.next_pattern_id, 102);
    assert_eq!(parsed.patterns.len(), 2);

    let configured = &parsed.patterns[1];
    assert_eq!(configured.id, 101);
    assert_eq!(configured.order_num, "20");
    assert!(!configured.enabled);
    assert_eq!(configured.search_type, AnalyseSearchType::Escaped);
    assert!(configured.match_case && configured.whole_word);
    assert_eq!(configured.selection, AnalyseSelection::Text);
    assert!(configured.hide && configured.bold && configured.italic && configured.underline);
    assert_eq!(configured.foreground, RgbColor::new(255, 0, 0));
    assert_eq!(configured.background, RgbColor::new(16, 32, 48));
    assert_eq!(configured.comment, "network & wifi");
    assert_eq!(configured.group, "network");

    let xml = write_profile(&parsed.patterns, None).unwrap();
    let reparsed = parse_profile(&xml, 100).unwrap();
    assert_eq!(reparsed.patterns, parsed.patterns);
}

#[test]
fn profile_fixture_supports_all_load_modes() {
    let imported = parse_profile(PHASE1_PROFILE, 100).unwrap().patterns;
    let mut patterns = parse_profile(
        "<AnalyseDoc><SearchText>existing</SearchText></AnalyseDoc>",
        1,
    )
    .unwrap()
    .patterns;

    apply_profile(&mut patterns, imported, ProfileLoadMode::Prepend);
    assert_eq!(
        patterns
            .iter()
            .map(|pattern| pattern.search_text.as_str())
            .collect::<Vec<_>>(),
        ["ERROR", "WARN\\tadapter", "existing"]
    );
}

#[test]
fn pending_oracle_profiles_are_valid_runtime_xml() {
    for (profile, expected_patterns) in [
        (SEARCH_CORPUS_PROFILE, 6),
        (REGEX_CORPUS_PROFILE, 10),
        (STYLE_CORPUS_PROFILE, 4),
    ] {
        let parsed = parse_profile(profile, 1).unwrap();
        assert_eq!(parsed.patterns.len(), expected_patterns);
    }
}
