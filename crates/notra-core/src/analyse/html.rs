use std::fmt::Write;

use super::{EffectiveStyle, MergedLineResult};

pub fn write_result_html(
    lines: &[MergedLineResult],
    show_line_numbers: bool,
    font_size: u16,
) -> String {
    let mut output = format!(
        "<meta charset=\"utf-8\"><pre style=\"margin:0;white-space:pre-wrap;font-family:Consolas,monospace;font-size:{}px\">",
        font_size.clamp(8, 72)
    );
    let line_digits = lines
        .last()
        .map(|line| line.source_line.to_string().len())
        .unwrap_or(1);
    for (line_index, line) in lines.iter().enumerate() {
        if show_line_numbers {
            write_text(
                &mut output,
                &format!("{:>line_digits$}: ", line.source_line),
            );
        }
        write_styled_line(&mut output, line);
        if line_index + 1 < lines.len() {
            output.push('\n');
        }
    }
    output.push_str("</pre>");
    output
}

fn write_styled_line(output: &mut String, line: &MergedLineResult) {
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
        write_style(output, &segment.style);
        write_text(
            output,
            &line.text[segment.start_byte_in_line..segment.end_byte_in_line],
        );
        output.push_str("</span>");
        byte_offset = segment.end_byte_in_line;
    }
    write_text(output, &line.text[byte_offset..]);
}

fn write_style(output: &mut String, style: &EffectiveStyle) {
    output.push_str("<span style=\"");
    if style.hidden {
        output.push_str("visibility:hidden;");
    }
    if style.bold {
        output.push_str("font-weight:700;");
    }
    if style.italic {
        output.push_str("font-style:italic;");
    }
    if style.underline {
        output.push_str("text-decoration:underline;");
    }
    write!(
        output,
        "color:{};background-color:{}\">",
        style.foreground.to_hex(),
        style.background.to_hex()
    )
    .expect("writing HTML into a String cannot fail");
}

fn write_text(output: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            character => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyse::{RgbColor, StyledSegment};

    #[test]
    fn writes_safe_unicode_html_with_styles_and_source_lines() {
        let line = MergedLineResult {
            source_line: 12,
            text: "错误 <x>".to_owned(),
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

        let html = write_result_html(&[line], true, 12);
        assert!(html.starts_with("<meta charset=\"utf-8\"><pre"));
        assert!(html.contains("12: "));
        assert!(html.contains("visibility:hidden;"));
        assert!(html.contains("font-weight:700;"));
        assert!(html.contains("color:#FF0000;"));
        assert!(html.contains("错误</span> &lt;x&gt;"));
        assert!(html.ends_with("</pre>"));
    }
}
