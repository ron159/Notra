#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedError {
    UnpairedHighSurrogate { offset: usize, value: u16 },
    UnexpectedLowSurrogate { offset: usize, value: u16 },
}

impl std::fmt::Display for ExtendedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnpairedHighSurrogate { offset, value } => write!(
                f,
                "unpaired high surrogate \\u{value:04X} at character {offset}"
            ),
            Self::UnexpectedLowSurrogate { offset, value } => write!(
                f,
                "unexpected low surrogate \\u{value:04X} at character {offset}"
            ),
        }
    }
}

impl std::error::Error for ExtendedError {}

pub fn translate_analyse_extended(input: &str) -> Result<String, ExtendedError> {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < chars.len() {
        if chars[index] != '\\' || index + 1 == chars.len() {
            output.push(chars[index]);
            index += 1;
            continue;
        }

        let escape = chars[index + 1];
        match escape {
            'r' => output.push('\r'),
            'n' => output.push('\n'),
            '0' => output.push('\0'),
            't' => output.push('\t'),
            '\\' => output.push('\\'),
            'b' | 'o' | 'd' | 'x' => {
                let (radix, width) = match escape {
                    'b' => (2, 8),
                    'o' => (8, 3),
                    'd' => (10, 3),
                    'x' => (16, 2),
                    _ => unreachable!(),
                };
                if let Some(value) = parse_fixed_digits(&chars, index + 2, width, radix) {
                    output.push(char::from_u32(value).expect("numeric escape is within Unicode"));
                    index += width + 2;
                    continue;
                }
                output.push('\\');
                output.push(escape);
            }
            'u' => {
                let Some(value) = parse_fixed_digits(&chars, index + 2, 4, 16) else {
                    output.push('\\');
                    output.push('u');
                    index += 2;
                    continue;
                };
                let value = value as u16;
                if (0xD800..=0xDBFF).contains(&value) {
                    let low_start = index + 6;
                    let low = if chars.get(low_start) == Some(&'\\')
                        && chars.get(low_start + 1) == Some(&'u')
                    {
                        parse_fixed_digits(&chars, low_start + 2, 4, 16).map(|value| value as u16)
                    } else {
                        None
                    };
                    let Some(low @ 0xDC00..=0xDFFF) = low else {
                        return Err(ExtendedError::UnpairedHighSurrogate {
                            offset: index,
                            value,
                        });
                    };
                    let scalar =
                        0x10000 + (((u32::from(value) - 0xD800) << 10) | (u32::from(low) - 0xDC00));
                    output.push(char::from_u32(scalar).expect("valid surrogate pair"));
                    index += 12;
                    continue;
                }
                if (0xDC00..=0xDFFF).contains(&value) {
                    return Err(ExtendedError::UnexpectedLowSurrogate {
                        offset: index,
                        value,
                    });
                }
                output.push(char::from_u32(u32::from(value)).expect("valid Unicode scalar"));
                index += 6;
                continue;
            }
            _ => {
                output.push('\\');
                output.push(escape);
            }
        }
        index += 2;
    }

    Ok(output)
}

fn parse_fixed_digits(chars: &[char], start: usize, width: usize, radix: u32) -> Option<u32> {
    let digits = chars.get(start..start + width)?;
    digits.iter().try_fold(0, |value, digit| {
        digit.to_digit(radix).map(|digit| value * radix + digit)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_simple_and_numeric_escapes() {
        let translated =
            translate_analyse_extended(r"\r\n\0\t\\|\b01000001\o101\d065\x41\u0041").unwrap();
        assert_eq!(translated, "\r\n\0\t\\|AAAAA");
    }

    #[test]
    fn accepts_upper_and_lower_hex_digits() {
        assert_eq!(translate_analyse_extended(r"\xAf\u4e2D").unwrap(), "¯中");
    }

    #[test]
    fn invalid_and_truncated_escapes_remain_literal() {
        for input in [
            r"\q", r"\b01012", r"\o89", r"\d1x3", r"\xG1", r"\u12Z4", r"tail\",
        ] {
            assert_eq!(translate_analyse_extended(input).unwrap(), input);
        }
    }

    #[test]
    fn combines_utf16_surrogate_pairs() {
        assert_eq!(translate_analyse_extended(r"\uD83D\uDE00").unwrap(), "😀");
    }

    #[test]
    fn rejects_unpaired_surrogates_without_losing_data() {
        assert!(matches!(
            translate_analyse_extended(r"\uD83D"),
            Err(ExtendedError::UnpairedHighSurrogate { .. })
        ));
        assert!(matches!(
            translate_analyse_extended(r"\uDE00"),
            Err(ExtendedError::UnexpectedLowSurrogate { .. })
        ));
    }

    #[test]
    fn preserves_unicode_and_mixed_sequences() {
        assert_eq!(
            translate_analyse_extended(r"中文\t한글\n日本語").unwrap(),
            "中文\t한글\n日本語"
        );
    }
}
