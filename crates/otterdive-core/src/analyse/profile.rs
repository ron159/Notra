use std::collections::BTreeMap;

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::writer::Writer;

use super::{
    AnalysePattern, AnalyseSearchType, AnalyseSelection, PatternId, ProfileLoadMode, RgbColor,
};

const ROOT_ELEMENT: &[u8] = b"AnalyseDoc";
const PATTERN_ELEMENT: &[u8] = b"SearchText";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedProfile {
    pub patterns: Vec<AnalysePattern>,
    pub next_pattern_id: PatternId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    Xml(String),
    MissingRoot,
    InvalidRoot(String),
    UnknownEntity(String),
    PatternIdOverflow,
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(message) => write!(f, "invalid Analyse profile XML: {message}"),
            Self::MissingRoot => write!(f, "Analyse profile has no AnalyseDoc root element"),
            Self::InvalidRoot(name) => {
                write!(f, "Analyse profile root must be AnalyseDoc, found {name}")
            }
            Self::UnknownEntity(name) => write!(f, "unknown XML entity: &{name};"),
            Self::PatternIdOverflow => write!(f, "pattern ID range is exhausted"),
        }
    }
}

impl std::error::Error for ProfileError {}

pub fn parse_profile(
    xml: &str,
    first_pattern_id: PatternId,
) -> Result<ParsedProfile, ProfileError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut patterns = Vec::new();
    let mut next_pattern_id = first_pattern_id;
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut current_pattern: Option<AnalysePattern> = None;

    loop {
        match reader.read_event().map_err(xml_error)? {
            Event::Start(element) => {
                if depth == 0 {
                    if root_seen {
                        return Err(ProfileError::Xml("multiple root elements".to_owned()));
                    }
                    if element.name().as_ref() != ROOT_ELEMENT {
                        return Err(ProfileError::InvalidRoot(
                            String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                        ));
                    }
                    root_seen = true;
                } else if depth == 1 && element.name().as_ref() == PATTERN_ELEMENT {
                    current_pattern = Some(pattern_from_element(
                        &element,
                        reader.decoder(),
                        next_pattern_id,
                    )?);
                    next_pattern_id = next_pattern_id
                        .checked_add(1)
                        .ok_or(ProfileError::PatternIdOverflow)?;
                }
                depth += 1;
            }
            Event::Empty(element) => {
                if depth == 0 {
                    if root_seen {
                        return Err(ProfileError::Xml("multiple root elements".to_owned()));
                    }
                    if element.name().as_ref() != ROOT_ELEMENT {
                        return Err(ProfileError::InvalidRoot(
                            String::from_utf8_lossy(element.name().as_ref()).into_owned(),
                        ));
                    }
                    root_seen = true;
                } else if depth == 1 && element.name().as_ref() == PATTERN_ELEMENT {
                    patterns.push(pattern_from_element(
                        &element,
                        reader.decoder(),
                        next_pattern_id,
                    )?);
                    next_pattern_id = next_pattern_id
                        .checked_add(1)
                        .ok_or(ProfileError::PatternIdOverflow)?;
                }
            }
            Event::Text(text) if current_pattern.is_some() && depth == 2 => {
                current_pattern
                    .as_mut()
                    .expect("checked above")
                    .search_text
                    .push_str(&text.xml10_content().map_err(xml_error)?);
            }
            Event::CData(text) if current_pattern.is_some() && depth == 2 => {
                current_pattern
                    .as_mut()
                    .expect("checked above")
                    .search_text
                    .push_str(&text.decode().map_err(xml_error)?);
            }
            Event::GeneralRef(reference) if current_pattern.is_some() && depth == 2 => {
                let value = if let Some(value) = reference.resolve_char_ref().map_err(xml_error)? {
                    value.to_string()
                } else {
                    let name = reference.decode().map_err(xml_error)?;
                    resolve_predefined_entity(&name)
                        .ok_or_else(|| ProfileError::UnknownEntity(name.into_owned()))?
                        .to_owned()
                };
                current_pattern
                    .as_mut()
                    .expect("checked above")
                    .search_text
                    .push_str(&value);
            }
            Event::End(element) => {
                if depth == 0 {
                    return Err(ProfileError::Xml("unexpected closing element".to_owned()));
                }
                if depth == 2
                    && element.name().as_ref() == PATTERN_ELEMENT
                    && let Some(pattern) = current_pattern.take()
                {
                    patterns.push(pattern);
                }
                depth -= 1;
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if !root_seen {
        return Err(ProfileError::MissingRoot);
    }

    Ok(ParsedProfile {
        patterns,
        next_pattern_id,
    })
}

pub fn write_profile(
    patterns: &[AnalysePattern],
    hits: Option<&BTreeMap<PatternId, usize>>,
) -> Result<String, ProfileError> {
    let defaults = AnalysePattern::default();
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_error)?;

    let mut root = BytesStart::new("AnalyseDoc");
    root.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    root.push_attribute(("xsi:noNamespaceSchemaLocation", "./AnalyseDoc.xsd"));
    writer.write_event(Event::Start(root)).map_err(xml_error)?;

    for pattern in patterns {
        let mut attributes: Vec<(&str, String)> = Vec::new();
        if let Some(hits) = hits.and_then(|hits| hits.get(&pattern.id)) {
            attributes.push(("hits", hits.to_string()));
        }
        if pattern.order_num != defaults.order_num {
            attributes.push(("orderNum", pattern.order_num.clone()));
        }
        if pattern.enabled != defaults.enabled {
            attributes.push(("doSearch", bool_value(pattern.enabled).to_owned()));
        }
        if pattern.search_type != defaults.search_type {
            attributes.push((
                "searchType",
                search_type_value(pattern.search_type).to_owned(),
            ));
        }
        if pattern.match_case != defaults.match_case {
            attributes.push(("matchCase", bool_value(pattern.match_case).to_owned()));
        }
        if pattern.whole_word != defaults.whole_word {
            attributes.push(("wholeWord", bool_value(pattern.whole_word).to_owned()));
        }
        if pattern.selection != defaults.selection {
            attributes.push(("select", selection_value(pattern.selection).to_owned()));
        }
        if pattern.hide != defaults.hide {
            attributes.push(("hide", bool_value(pattern.hide).to_owned()));
        }
        if pattern.bold != defaults.bold {
            attributes.push(("bold", bool_value(pattern.bold).to_owned()));
        }
        if pattern.italic != defaults.italic {
            attributes.push(("italic", bool_value(pattern.italic).to_owned()));
        }
        if pattern.underline != defaults.underline {
            attributes.push(("underlined", bool_value(pattern.underline).to_owned()));
        }
        if pattern.foreground != defaults.foreground {
            attributes.push(("color", color_value(pattern.foreground)));
        }
        if pattern.background != defaults.background {
            attributes.push(("bgColor", color_value(pattern.background)));
        }
        if !pattern.comment.is_empty() {
            attributes.push(("comment", pattern.comment.clone()));
        }
        if !pattern.group.is_empty() {
            attributes.push(("group", pattern.group.clone()));
        }

        let mut element = BytesStart::new("SearchText");
        for (key, value) in &attributes {
            element.push_attribute((*key, value.as_str()));
        }
        writer
            .write_event(Event::Start(element))
            .map_err(xml_error)?;
        writer
            .write_event(Event::Text(BytesText::new(&pattern.search_text)))
            .map_err(xml_error)?;
        writer
            .write_event(Event::End(BytesEnd::new("SearchText")))
            .map_err(xml_error)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("AnalyseDoc")))
        .map_err(xml_error)?;
    String::from_utf8(writer.into_inner()).map_err(|error| ProfileError::Xml(error.to_string()))
}

pub fn apply_profile(
    existing: &mut Vec<AnalysePattern>,
    mut imported: Vec<AnalysePattern>,
    mode: ProfileLoadMode,
) {
    match mode {
        ProfileLoadMode::Replace => *existing = imported,
        ProfileLoadMode::Append => existing.append(&mut imported),
        ProfileLoadMode::Prepend => {
            imported.append(existing);
            *existing = imported;
        }
    }
}

fn pattern_from_element(
    element: &BytesStart<'_>,
    decoder: quick_xml::encoding::Decoder,
    id: PatternId,
) -> Result<AnalysePattern, ProfileError> {
    let mut pattern = AnalysePattern {
        id,
        ..Default::default()
    };

    for attribute in element.attributes() {
        let attribute = attribute.map_err(xml_error)?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, decoder)
            .map_err(xml_error)?;
        match attribute.key.as_ref() {
            b"orderNum" => pattern.order_num = value.into_owned(),
            b"doSearch" => pattern.enabled = parse_bool(&value),
            b"searchType" => pattern.search_type = parse_search_type(&value),
            b"matchCase" => pattern.match_case = parse_bool(&value),
            b"wholeWord" => pattern.whole_word = parse_bool(&value),
            b"select" => pattern.selection = parse_selection(&value),
            b"hide" => pattern.hide = parse_bool(&value),
            b"bold" => pattern.bold = parse_bool(&value),
            b"italic" => pattern.italic = parse_bool(&value),
            b"underlined" => pattern.underline = parse_bool(&value),
            b"color" => pattern.foreground = parse_color(&value, RgbColor::BLACK),
            b"bgColor" => pattern.background = parse_color(&value, RgbColor::WHITE),
            b"comment" => pattern.comment = value.into_owned(),
            b"group" => pattern.group = value.into_owned(),
            _ => {}
        }
    }

    Ok(pattern)
}

fn parse_bool(value: &str) -> bool {
    value == "true"
}

fn bool_value(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn parse_search_type(value: &str) -> AnalyseSearchType {
    match value {
        "escaped" => AnalyseSearchType::Escaped,
        "regex" => AnalyseSearchType::Regex,
        "rgx_multiline" => AnalyseSearchType::RegexMultiline,
        _ => AnalyseSearchType::Normal,
    }
}

fn search_type_value(value: AnalyseSearchType) -> &'static str {
    match value {
        AnalyseSearchType::Normal => "normal",
        AnalyseSearchType::Escaped => "escaped",
        AnalyseSearchType::Regex => "regex",
        AnalyseSearchType::RegexMultiline => "rgx_multiline",
    }
}

fn parse_selection(value: &str) -> AnalyseSelection {
    match value {
        "text" => AnalyseSelection::Text,
        _ => AnalyseSelection::Line,
    }
}

fn selection_value(value: AnalyseSelection) -> &'static str {
    match value {
        AnalyseSelection::Text => "text",
        AnalyseSelection::Line => "line",
    }
}

fn parse_color(value: &str, fallback: RgbColor) -> RgbColor {
    RgbColor::from_hex(value)
        .or_else(|| {
            NAMED_COLORS
                .iter()
                .find(|(name, _)| *name == value)
                .map(|(_, color)| *color)
        })
        .unwrap_or(fallback)
}

fn color_value(color: RgbColor) -> String {
    NAMED_COLORS
        .iter()
        .find(|(_, value)| *value == color)
        .map(|(name, _)| (*name).to_owned())
        .unwrap_or_else(|| color.to_hex())
}

fn xml_error(error: impl std::fmt::Display) -> ProfileError {
    ProfileError::Xml(error.to_string())
}

const NAMED_COLORS: &[(&str, RgbColor)] = &[
    ("black", RgbColor::new(0x00, 0x00, 0x00)),
    ("red", RgbColor::new(0xFF, 0x00, 0x00)),
    ("darkRed", RgbColor::new(0x80, 0x00, 0x00)),
    ("deepPurple", RgbColor::new(0x87, 0x13, 0x97)),
    ("darkBlue", RgbColor::new(0x00, 0x00, 0x80)),
    ("darkGreen", RgbColor::new(0x00, 0x80, 0x00)),
    ("darkGrey", RgbColor::new(0x40, 0x40, 0x40)),
    ("liteRed", RgbColor::new(0xFF, 0x60, 0x60)),
    ("brown", RgbColor::new(0x80, 0x40, 0x00)),
    ("purple", RgbColor::new(0x80, 0x00, 0xFF)),
    ("blue", RgbColor::new(0x00, 0x00, 0xFF)),
    ("blueGreen", RgbColor::new(0x00, 0x80, 0x80)),
    ("grey", RgbColor::new(0x80, 0x80, 0x80)),
    ("orange", RgbColor::new(0xFF, 0x80, 0x00)),
    ("beige", RgbColor::new(0xC0, 0x80, 0x40)),
    ("pink", RgbColor::new(0xFF, 0x00, 0xFF)),
    ("liteBlue", RgbColor::new(0x6C, 0xA8, 0xE0)),
    ("green", RgbColor::new(0x00, 0xBE, 0x00)),
    ("liteGrey", RgbColor::new(0xC0, 0xC0, 0xC0)),
    ("darkYellow", RgbColor::new(0xFF, 0xC0, 0x00)),
    ("liteBeige", RgbColor::new(0xDB, 0xB7, 0x93)),
    ("litePink", RgbColor::new(0xFF, 0x99, 0xDD)),
    ("cyan", RgbColor::new(0x00, 0xFF, 0xFF)),
    ("liteGreen", RgbColor::new(0x00, 0xFF, 0x00)),
    ("white", RgbColor::new(0xFF, 0xFF, 0xFF)),
    ("yellow", RgbColor::new(0xFF, 0xFF, 0x00)),
    ("offWhite", RgbColor::new(0xFF, 0xFB, 0xF0)),
    ("veryLitePurple", RgbColor::new(0xEF, 0xD8, 0xE9)),
    ("veryLiteBlue", RgbColor::new(0xC4, 0xF9, 0xFD)),
    ("veryLiteGrey", RgbColor::new(0xE0, 0xE0, 0xE0)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults_and_preserves_search_whitespace() {
        let parsed = parse_profile(
            "<AnalyseDoc><SearchText>  ERROR\t timeout  </SearchText></AnalyseDoc>",
            10,
        )
        .unwrap();
        assert_eq!(parsed.next_pattern_id, 11);
        assert_eq!(parsed.patterns.len(), 1);
        assert_eq!(parsed.patterns[0].id, 10);
        assert_eq!(parsed.patterns[0].search_text, "  ERROR\t timeout  ");
        assert_eq!(
            parsed.patterns[0],
            AnalysePattern {
                id: 10,
                search_text: "  ERROR\t timeout  ".to_owned(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn parses_runtime_attributes_and_ignores_unknown_attributes() {
        let parsed = parse_profile(
            r##"<AnalyseDoc><SearchText orderNum="10" doSearch="false" searchType="rgx_multiline" matchCase="true" wholeWord="true" select="text" hide="true" bold="true" italic="true" underlined="true" color="red" bgColor="#102030" comment="a &amp; b" group="network" hits="4" future="ignored">ERROR&lt;wifi</SearchText></AnalyseDoc>"##,
            1,
        )
        .unwrap();
        let pattern = &parsed.patterns[0];
        assert_eq!(pattern.order_num, "10");
        assert!(!pattern.enabled);
        assert_eq!(pattern.search_type, AnalyseSearchType::RegexMultiline);
        assert!(pattern.match_case && pattern.whole_word);
        assert_eq!(pattern.selection, AnalyseSelection::Text);
        assert!(pattern.hide && pattern.bold && pattern.italic && pattern.underline);
        assert_eq!(pattern.foreground, RgbColor::new(255, 0, 0));
        assert_eq!(pattern.background, RgbColor::new(16, 32, 48));
        assert_eq!(pattern.comment, "a & b");
        assert_eq!(pattern.group, "network");
        assert_eq!(pattern.search_text, "ERROR<wifi");
    }

    #[test]
    fn writes_runtime_compatible_xml_and_optional_hits() {
        let pattern = AnalysePattern {
            id: 7,
            enabled: false,
            search_text: "a < b & c".to_owned(),
            search_type: AnalyseSearchType::RegexMultiline,
            selection: AnalyseSelection::Text,
            foreground: RgbColor::new(255, 0, 0),
            background: RgbColor::new(1, 2, 3),
            comment: "network & wifi".to_owned(),
            ..Default::default()
        };
        let hits = BTreeMap::from([(7, 3)]);
        let xml = write_profile(&[pattern], Some(&hits)).unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("hits=\"3\""));
        assert!(xml.contains("searchType=\"rgx_multiline\""));
        assert!(xml.contains("color=\"red\""));
        assert!(xml.contains("bgColor=\"#010203\""));
        assert!(xml.contains("comment=\"network &amp; wifi\""));
        assert!(xml.contains("a &lt; b &amp; c"));
    }

    #[test]
    fn round_trip_preserves_pattern_semantics() {
        let patterns = vec![
            AnalysePattern {
                id: 40,
                search_text: "first".to_owned(),
                ..Default::default()
            },
            AnalysePattern {
                id: 41,
                order_num: "20".to_owned(),
                search_text: "second\\nline".to_owned(),
                search_type: AnalyseSearchType::Escaped,
                bold: true,
                group: "g".to_owned(),
                ..Default::default()
            },
        ];
        let xml = write_profile(&patterns, None).unwrap();
        let reparsed = parse_profile(&xml, 40).unwrap();
        assert_eq!(reparsed.patterns, patterns);
    }

    #[test]
    fn load_modes_preserve_import_order() {
        let pattern = |id| AnalysePattern {
            id,
            search_text: id.to_string(),
            ..Default::default()
        };
        let imported = vec![pattern(3), pattern(4)];

        let mut appended = vec![pattern(1), pattern(2)];
        apply_profile(&mut appended, imported.clone(), ProfileLoadMode::Append);
        assert_eq!(
            appended.iter().map(|p| p.id).collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );

        let mut prepended = vec![pattern(1), pattern(2)];
        apply_profile(&mut prepended, imported.clone(), ProfileLoadMode::Prepend);
        assert_eq!(
            prepended.iter().map(|p| p.id).collect::<Vec<_>>(),
            [3, 4, 1, 2]
        );

        let mut replaced = vec![pattern(1), pattern(2)];
        apply_profile(&mut replaced, imported, ProfileLoadMode::Replace);
        assert_eq!(replaced.iter().map(|p| p.id).collect::<Vec<_>>(), [3, 4]);
    }

    #[test]
    fn rejects_non_analyse_roots() {
        assert!(matches!(
            parse_profile("<Other />", 1),
            Err(ProfileError::InvalidRoot(_))
        ));
    }

    #[test]
    fn rejects_multiple_roots() {
        assert!(matches!(
            parse_profile("<AnalyseDoc/><AnalyseDoc/>", 1),
            Err(ProfileError::Xml(_))
        ));
    }
}
