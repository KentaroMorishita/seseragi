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

pub(crate) fn template_end(source: &str, start: usize) -> usize {
    debug_assert!(source[start..].starts_with('`'));
    scan_nested(source, start + 1, ScanContext::Template).map_or(source.len(), |(_, after)| after)
}

/// A template that starts with `` `[ `` and closes on the same line.
///
/// `` `[ `` is otherwise a List literal. The closer is a later backtick on
/// that same line, outside a string, a character literal, and a `//` comment.
/// A backtick followed by `[` is the next List literal. Another backtick
/// later on the line means a real template follows the list.
pub(crate) fn single_line_bracket_template_end(source: &str, start: usize) -> Option<usize> {
    if !source[start..].starts_with("`[") {
        return None;
    }
    let line_end = source[start..]
        .find(['\n', '\r'])
        .map(|offset| start + offset)
        .unwrap_or(source.len());
    let mut cursor = start + 1;
    while cursor < line_end {
        let rest = &source[cursor..line_end];
        if rest.starts_with("//") {
            return None;
        }
        if rest.starts_with('"') || rest.starts_with('\'') {
            let delimiter = rest.chars().next()?;
            cursor = quoted_end_on_line(source, cursor, delimiter, line_end)?;
            continue;
        }
        if rest.starts_with('`') {
            let after = cursor + 1;
            // `` `[ `` on the same line is the next List literal, not this template's closer.
            if source[after..].starts_with('[') {
                return None;
            }
            if source[after..line_end].contains('`') {
                return None;
            }
            return Some(after);
        }
        advance_char(source, &mut cursor);
    }
    None
}

/// End of a quote that opens at `start` and closes before `line_end`.
fn quoted_end_on_line(
    source: &str,
    start: usize,
    delimiter: char,
    line_end: usize,
) -> Option<usize> {
    let mut cursor = start + delimiter.len_utf8();
    while cursor < line_end {
        let character = source[cursor..].chars().next()?;
        cursor += character.len_utf8();
        if character == '\\' {
            let escaped = source.get(cursor..)?.chars().next()?;
            cursor += escaped.len_utf8();
            if cursor > line_end {
                return None;
            }
        } else if character == delimiter {
            return Some(cursor);
        }
    }
    None
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

fn interpolation_end(source: &str, start: usize) -> Option<(usize, usize)> {
    scan_nested(source, start, ScanContext::Interpolation { depth: 1 })
}

enum ScanContext {
    Template,
    Interpolation { depth: usize },
}

/// Template lexing precedes the syntax budget. Keep nested templates and
/// interpolation braces on an explicit heap stack, rather than the host stack.
fn scan_nested(source: &str, start: usize, initial: ScanContext) -> Option<(usize, usize)> {
    let mut cursor = start;
    let mut contexts = vec![initial];
    while cursor < source.len() {
        match contexts.last_mut()? {
            ScanContext::Template => {
                if source[cursor..].starts_with('\\') {
                    cursor += 1;
                    advance_char(source, &mut cursor);
                } else if source[cursor..].starts_with("${") {
                    cursor += 2;
                    contexts.push(ScanContext::Interpolation { depth: 1 });
                } else if source[cursor..].starts_with('`') {
                    let end = cursor;
                    cursor += 1;
                    contexts.pop();
                    if contexts.is_empty() {
                        return Some((end, cursor));
                    }
                } else {
                    advance_char(source, &mut cursor);
                }
            }
            ScanContext::Interpolation { depth } => {
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
                    '`' => {
                        cursor += 1;
                        contexts.push(ScanContext::Template);
                    }
                    '{' => {
                        *depth += 1;
                        cursor += 1;
                    }
                    '}' => {
                        let end = cursor;
                        cursor += 1;
                        *depth -= 1;
                        if *depth == 0 {
                            contexts.pop();
                            if contexts.is_empty() {
                                return Some((end, cursor));
                            }
                        }
                    }
                    _ => advance_char(source, &mut cursor),
                }
            }
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
    use crate::literal::{decode_template_text, LiteralDecodeError};

    #[test]
    fn finds_a_same_line_template_that_starts_with_a_list_open() {
        let source = "fn a -> String = `[a]`\nfn b -> List<Int> = `[1, 2]\n";
        let open = source.find("`[").unwrap();
        assert_eq!(
            single_line_bracket_template_end(source, open),
            Some(source.find('\n').unwrap())
        );
        let list = source.rfind("`[").unwrap();
        assert_eq!(single_line_bracket_template_end(source, list), None);
    }

    #[test]
    fn keeps_a_list_when_a_backtick_is_quoted_or_commented() {
        let quoted = "fn values -> List<String> = `[\"`\"]\n";
        let open = quoted.find("`[").unwrap();
        assert_eq!(single_line_bracket_template_end(quoted, open), None);

        let comment = "fn values -> List<Int> = `[1, 2] // note `\n";
        let open = comment.find("`[").unwrap();
        assert_eq!(single_line_bracket_template_end(comment, open), None);

        let later = "fn values -> List<String> = `[\"${\"]\nfn later -> String = `later`\n";
        let open = later.find("`[").unwrap();
        assert_eq!(single_line_bracket_template_end(later, open), None);
    }

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
            Err(LiteralDecodeError { range: 7..9 })
        );
        assert_eq!(
            decode_template_text("\\u{110000}"),
            Err(LiteralDecodeError { range: 0..10 })
        );
    }
}
