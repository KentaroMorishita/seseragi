use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiteralDecodeError {
    pub range: Range<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LiteralContext {
    String,
    Template,
}

pub fn decode_string_literal(raw: &str) -> Result<String, LiteralDecodeError> {
    let Some(content) = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return Err(LiteralDecodeError {
            range: 0..raw.len(),
        });
    };
    decode_text(content, LiteralContext::String).map_err(|error| LiteralDecodeError {
        range: error.range.start + 1..error.range.end + 1,
    })
}

pub(crate) fn decode_template_text(raw: &str) -> Result<String, LiteralDecodeError> {
    decode_text(raw, LiteralContext::Template)
}

fn decode_text(raw: &str, context: LiteralContext) -> Result<String, LiteralDecodeError> {
    let mut decoded = String::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        if context == LiteralContext::Template && raw[cursor..].starts_with("\r\n") {
            decoded.push('\n');
            cursor += 2;
            continue;
        }
        if !raw[cursor..].starts_with('\\') {
            let character = raw[cursor..]
                .chars()
                .next()
                .expect("cursor remains on a UTF-8 boundary");
            if character.is_control() && !(context == LiteralContext::Template && character == '\n')
            {
                return Err(LiteralDecodeError {
                    range: cursor..cursor + character.len_utf8(),
                });
            }
            decoded.push(character);
            cursor += character.len_utf8();
            continue;
        }

        let escape_start = cursor;
        cursor += 1;
        let Some(escaped) = raw[cursor..].chars().next() else {
            return Err(LiteralDecodeError {
                range: escape_start..raw.len(),
            });
        };
        cursor += escaped.len_utf8();
        match escaped {
            '\\' => decoded.push('\\'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            '0' => decoded.push('\0'),
            '"' if context == LiteralContext::String => decoded.push('"'),
            '`' if context == LiteralContext::Template => decoded.push('`'),
            '$' if context == LiteralContext::Template && raw[cursor..].starts_with('{') => {
                decoded.push_str("${");
                cursor += 1;
            }
            'u' if raw[cursor..].starts_with('{') => {
                cursor += 1;
                let digits_start = cursor;
                while cursor < raw.len() && raw.as_bytes()[cursor].is_ascii_hexdigit() {
                    cursor += 1;
                }
                let escape_end = raw[cursor..]
                    .find('}')
                    .map_or(cursor, |offset| cursor + offset + 1);
                if cursor == digits_start
                    || cursor - digits_start > 6
                    || !raw[cursor..].starts_with('}')
                {
                    return Err(LiteralDecodeError {
                        range: escape_start..escape_end,
                    });
                }
                let scalar = u32::from_str_radix(&raw[digits_start..cursor], 16).map_err(|_| {
                    LiteralDecodeError {
                        range: escape_start..cursor + 1,
                    }
                })?;
                let character = char::from_u32(scalar).ok_or_else(|| LiteralDecodeError {
                    range: escape_start..cursor + 1,
                })?;
                decoded.push(character);
                cursor += 1;
            }
            _ => {
                return Err(LiteralDecodeError {
                    range: escape_start..cursor,
                });
            }
        }
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_string_escapes_without_rewriting_source_spelling() {
        assert_eq!(
            decode_string_literal(r#""line\ncolumn\t\"quoted\"\\\u{03BB}\0""#),
            Ok("line\ncolumn\t\"quoted\"\\λ\0".to_owned())
        );
    }

    #[test]
    fn rejects_context_specific_and_invalid_string_escapes() {
        assert_eq!(
            decode_string_literal(r#""bad\q""#),
            Err(LiteralDecodeError { range: 4..6 })
        );
        assert_eq!(
            decode_string_literal(r#""bad\`""#),
            Err(LiteralDecodeError { range: 4..6 })
        );
        assert_eq!(
            decode_string_literal(r#""\u{110000}""#),
            Err(LiteralDecodeError { range: 1..11 })
        );
    }
}
