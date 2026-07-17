use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TemplateChunk {
    Text(Range<usize>),
    Interpolation {
        expression: Range<usize>,
        interpolation: Range<usize>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateScan {
    pub(crate) chunks: Vec<TemplateChunk>,
    pub(crate) closed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemplateDecodeError {
    pub(crate) range: Range<usize>,
}

pub(crate) fn template_end(source: &str, start: usize) -> usize {
    debug_assert!(source[start..].starts_with('`'));
    let mut cursor = start + 1;
    while cursor < source.len() {
        if source[cursor..].starts_with('\\') {
            cursor += 1;
            advance_char(source, &mut cursor);
            continue;
        }
        if source[cursor..].starts_with("${") {
            cursor += 2;
            let Some((_, after)) = interpolation_end(source, cursor) else {
                return source.len();
            };
            cursor = after;
            continue;
        }
        if source[cursor..].starts_with('`') {
            return cursor + 1;
        }
        advance_char(source, &mut cursor);
    }
    source.len()
}

pub(crate) fn scan_template(raw: &str) -> TemplateScan {
    if !raw.starts_with('`') {
        return TemplateScan {
            chunks: Vec::new(),
            closed: false,
        };
    }
    let mut chunks = Vec::new();
    let mut cursor = 1;
    let mut text_start = cursor;
    let mut closed = false;
    while cursor < raw.len() {
        if raw[cursor..].starts_with('\\') {
            cursor += 1;
            advance_char(raw, &mut cursor);
            continue;
        }
        if raw[cursor..].starts_with("${") {
            if text_start < cursor {
                chunks.push(TemplateChunk::Text(text_start..cursor));
            }
            let interpolation_start = cursor;
            cursor += 2;
            let expression_start = cursor;
            let Some((expression_end, after)) = interpolation_end(raw, cursor) else {
                chunks.push(TemplateChunk::Interpolation {
                    expression: expression_start..raw.len(),
                    interpolation: interpolation_start..raw.len(),
                });
                return TemplateScan {
                    chunks,
                    closed: false,
                };
            };
            chunks.push(TemplateChunk::Interpolation {
                expression: expression_start..expression_end,
                interpolation: interpolation_start..after,
            });
            cursor = after;
            text_start = cursor;
            continue;
        }
        if raw[cursor..].starts_with('`') {
            if text_start < cursor {
                chunks.push(TemplateChunk::Text(text_start..cursor));
            }
            cursor += 1;
            closed = cursor == raw.len();
            break;
        }
        advance_char(raw, &mut cursor);
    }
    TemplateScan { chunks, closed }
}

pub(crate) fn decode_template_text(raw: &str) -> Result<String, TemplateDecodeError> {
    let mut decoded = String::new();
    let mut cursor = 0;
    while cursor < raw.len() {
        if raw[cursor..].starts_with("\r\n") {
            decoded.push('\n');
            cursor += 2;
            continue;
        }
        if !raw[cursor..].starts_with('\\') {
            let character = raw[cursor..]
                .chars()
                .next()
                .expect("cursor remains on a UTF-8 boundary");
            if character.is_control() && character != '\n' {
                return Err(TemplateDecodeError {
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
            return Err(TemplateDecodeError {
                range: escape_start..raw.len(),
            });
        };
        cursor += escaped.len_utf8();
        match escaped {
            '\\' => decoded.push('\\'),
            '`' => decoded.push('`'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            '0' => decoded.push('\0'),
            '$' if raw[cursor..].starts_with('{') => {
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
                    return Err(TemplateDecodeError {
                        range: escape_start..escape_end,
                    });
                }
                let scalar = u32::from_str_radix(&raw[digits_start..cursor], 16).map_err(|_| {
                    TemplateDecodeError {
                        range: escape_start..cursor + 1,
                    }
                })?;
                let character = char::from_u32(scalar).ok_or_else(|| TemplateDecodeError {
                    range: escape_start..cursor + 1,
                })?;
                decoded.push(character);
                cursor += 1;
            }
            _ => {
                return Err(TemplateDecodeError {
                    range: escape_start..cursor,
                });
            }
        }
    }
    Ok(decoded)
}

fn interpolation_end(source: &str, start: usize) -> Option<(usize, usize)> {
    let mut cursor = start;
    let mut depth = 1usize;
    while cursor < source.len() {
        if source[cursor..].starts_with("//") {
            cursor += 2;
            while cursor < source.len() && !source[cursor..].starts_with('\n') {
                advance_char(source, &mut cursor);
            }
            continue;
        }
        let character = source[cursor..].chars().next()?;
        if character == '_' || unicode_ident::is_xid_start(character) {
            advance_char(source, &mut cursor);
            while cursor < source.len()
                && source[cursor..].chars().next().is_some_and(|character| {
                    character == '\'' || unicode_ident::is_xid_continue(character)
                })
            {
                advance_char(source, &mut cursor);
            }
            continue;
        }
        match character {
            '"' | '\'' => skip_quoted(source, &mut cursor, character),
            '`' => cursor = template_end(source, cursor),
            '{' => {
                depth += 1;
                cursor += 1;
            }
            '}' => {
                let end = cursor;
                cursor += 1;
                depth -= 1;
                if depth == 0 {
                    return Some((end, cursor));
                }
            }
            _ => advance_char(source, &mut cursor),
        }
    }
    None
}

fn skip_quoted(source: &str, cursor: &mut usize, delimiter: char) {
    *cursor += delimiter.len_utf8();
    while *cursor < source.len() {
        let Some(character) = source[*cursor..].chars().next() else {
            return;
        };
        *cursor += character.len_utf8();
        if character == '\\' {
            advance_char(source, cursor);
        } else if character == delimiter {
            return;
        }
    }
}

fn advance_char(source: &str, cursor: &mut usize) {
    if let Some(character) = source[*cursor..].chars().next() {
        *cursor += character.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_nested_expressions_without_splitting_the_outer_template() {
        let raw = "`before ${match value { Ready -> `nested ${name}` }} after`";
        let scan = scan_template(raw);

        assert!(scan.closed);
        assert_eq!(scan.chunks.len(), 3);
        let TemplateChunk::Interpolation { expression, .. } = &scan.chunks[1] else {
            panic!("expected interpolation");
        };
        assert_eq!(
            &raw[expression.clone()],
            "match value { Ready -> `nested ${name}` }"
        );
        assert_eq!(template_end(raw, 0), raw.len());
    }

    #[test]
    fn keeps_identifier_apostrophes_inside_interpolations() {
        let raw = "`before ${次の値'} after`";
        let scan = scan_template(raw);

        assert!(scan.closed);
        let TemplateChunk::Interpolation { expression, .. } = &scan.chunks[1] else {
            panic!("expected interpolation");
        };
        assert_eq!(&raw[expression.clone()], "次の値'");
        assert_eq!(template_end(raw, 0), raw.len());
    }

    #[test]
    fn decodes_template_escapes_and_normalizes_crlf() {
        assert_eq!(
            decode_template_text("line\\n\\${value}\\`\\\\\r\nnext"),
            Ok("line\n${value}`\\\nnext".to_owned())
        );
    }

    #[test]
    fn reports_the_exact_invalid_escape_range() {
        assert_eq!(
            decode_template_text("before \\q after"),
            Err(TemplateDecodeError { range: 7..9 })
        );
        assert_eq!(
            decode_template_text("\\u{110000}"),
            Err(TemplateDecodeError { range: 0..10 })
        );
    }
}
