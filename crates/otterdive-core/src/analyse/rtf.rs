use super::{EffectiveStyle, MergedLineResult, RgbColor};

pub fn write_result_rtf(lines: &[MergedLineResult], show_line_numbers: bool, font_size: u16) -> String {
    let mut colors = vec![RgbColor::BLACK, RgbColor::WHITE];
    for segment in lines.iter().flat_map(|line| &line.styled_segments) {
        push_unique(&mut colors, segment.style.foreground);
        push_unique(&mut colors, segment.style.background);
    }

    let mut output = String::from("{\\rtf1\\ansi\\ansicpg65001\\deff0");
    output.push_str("{\\fonttbl{\\f0\\fmodern Consolas;}}");
    output.push_str("{\\colortbl;");
    for color in &colors {
        output.push_str(&format!(
            "\\red{}\\green{}\\blue{};",
            color.red, color.green, color.blue
        ));
    }
    output.push('}');
    output.push_str(&format!("\\f0\\fs{}\\cf1\\highlight2 ", font_size.clamp(8, 72) * 2));

    let line_digits = lines
        .last()
        .map(|line| line.source_line.to_string().len())
        .unwrap_or(1);
    for (line_index, line) in lines.iter().enumerate() {
        if show_line_numbers {
            write_text(&mut output, &format!("{:>line_digits$}: ", line.source_line));
        }
        write_styled_line(&mut output, line, &colors);
        if line_index + 1 < lines.len() {
            output.push_str("\\line\n");
        }
    }
    output.push('}');
    output
}

fn write_styled_line(output: &mut String, line: &MergedLineResult, colors: &[RgbColor]) {
    let mut byte_offset = 0usize;
    for segment in &line.styled_segments {
        if segment.start_byte_in_line < byte_offset
            || segment.start_byte_in_line > segment.end_byte_in_line
            || segment.end_byte_in_line > line.text.len()
            || !line.text.is_char_boundary(segment.start_byte_in_line)
            || !line.text.is_char_boundary(segment.end_byte_in_line)
        {
            continue;
        }
        write_text(output, &line.text[byte_offset..segment.start_byte_in_line]);
        write_style(output, &segment.style, colors);
        write_text(
            output,
            &line.text[segment.start_byte_in_line..segment.end_byte_in_line],
        );
        output.push_str("\\v0\\b0\\i0\\ulnone\\cf1\\highlight2 ");
        byte_offset = segment.end_byte_in_line;
    }
    write_text(output, &line.text[byte_offset..]);
}

fn write_style(output: &mut String, style: &EffectiveStyle, colors: &[RgbColor]) {
    if style.hidden {
        output.push_str("\\v ");
    }
    if style.bold {
        output.push_str("\\b ");
    }
    if style.italic {
        output.push_str("\\i ");
    }
    if style.underline {
        output.push_str("\\ul ");
    }
    let foreground = color_index(colors, style.foreground);
    let background = color_index(colors, style.background);
    output.push_str(&format!("\\cf{foreground}\\highlight{background} "));
}

fn write_text(output: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '\\' | '{' | '}' => {
                output.push('\\');
                output.push(character);
            }
            '\t' => output.push_str("\\tab "),
            '\r' => {}
            '\n' => output.push_str("\\line "),
            character if character.is_ascii() => output.push(character),
            character => {
                for unit in character.encode_utf16(&mut [0; 2]) {
                    output.push_str(&format!("\\u{}?", *unit as i16));
                }
            }
        }
    }
}

fn push_unique(colors: &mut Vec<RgbColor>, color: RgbColor) {
    if !colors.contains(&color) {
        colors.push(color);
    }
}

fn color_index(colors: &[RgbColor], color: RgbColor) -> usize {
    colors.iter().position(|candidate| *candidate == color).unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::StyledSegment;

    #[test]
    fn writes_unicode_styles_hidden_text_and_source_lines() {
        let line = MergedLineResult {
            source_line: 12,
            text: "错误 {x}".to_owned(),
            matches: Vec::new(),
            styled_segments: vec![StyledSegment {
                start_byte_in_line: 0,
                end_byte_in_line: "错误".len(),
                pattern_id: Some(1),
                style: EffectiveStyle {
                    hidden: true,
                    bold: true,
                    foreground: RgbColor::new(255, 0, 0),
                    ..Default::default()
                },
            }],
        };

        let rtf = write_result_rtf(&[line], true, 12);
        assert!(rtf.starts_with("{\\rtf1"));
        assert!(rtf.contains("12: "));
        assert!(rtf.contains("\\red255\\green0\\blue0;"));
        assert!(rtf.contains("\\v "));
        assert!(rtf.contains("\\b "));
        assert!(rtf.contains("\\u"));
        assert!(rtf.contains("\\{x\\}"));
    }
}
