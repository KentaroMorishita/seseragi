//! Folding and selection ranges.
//!
//! Both features read only tokens and the surface syntax, never the typed
//! analysis, so they keep working while a document does not type-check or
//! does not even parse (spec 12.6).

use crate::features::range_json;
use serde_json::{json, Value};
use seseragi_source::{LineIndex, PositionEncoding};
use seseragi_syntax::{lex, parse_surface_ast, ByteSpan, Token, TokenKind};

/// One matched pair of `{}`, `()` or `[]`, as byte offsets of the two
/// delimiters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Bracket {
    open: usize,
    close: usize,
}

fn significant_tokens(source: &str) -> Vec<Token> {
    lex("", source)
        .tokens
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenKind::TriviaSpace | TokenKind::Eof))
        .collect()
}

/// Pairs brackets with a stack. An unmatched delimiter in unfinished source
/// is skipped instead of discarding the pairs around it.
fn brackets(tokens: &[Token]) -> Vec<Bracket> {
    let mut stack: Vec<(TokenKind, usize)> = Vec::new();
    let mut pairs = Vec::new();
    for token in tokens {
        let closing = match token.kind {
            TokenKind::PunctuationBraceLeft
            | TokenKind::PunctuationParenLeft
            | TokenKind::PunctuationListLeft
            | TokenKind::PunctuationSquareLeft => {
                stack.push((token.kind, token.start));
                continue;
            }
            TokenKind::PunctuationBraceRight => TokenKind::PunctuationBraceLeft,
            TokenKind::PunctuationParenRight => TokenKind::PunctuationParenLeft,
            TokenKind::PunctuationSquareRight => TokenKind::PunctuationSquareLeft,
            _ => continue,
        };
        let opener = stack.iter().rposition(|(kind, _)| {
            *kind == closing
                || (closing == TokenKind::PunctuationSquareLeft
                    && *kind == TokenKind::PunctuationListLeft)
        });
        if let Some(index) = opener {
            let (_, open) = stack[index];
            stack.truncate(index);
            pairs.push(Bracket {
                open,
                close: token.start,
            });
        }
    }
    pairs
}

pub(crate) fn folding_ranges(source: &str) -> Value {
    let index = LineIndex::new(source);
    // `try_locate` counts lines from 1; LSP lines count from 0.
    let line = |offset: usize| {
        index
            .try_locate(offset)
            .ok()
            .and_then(|position| u32::try_from(position.line.checked_sub(1)?).ok())
    };
    let mut ranges: Vec<(u32, u32, Option<&str>)> = Vec::new();
    let tokens = significant_tokens(source);

    // Blocks: `match`, `do`, records, lists and argument lists. The closing
    // delimiter stays visible, so a fold ends on the line before it.
    for pair in brackets(&tokens) {
        if let (Some(start), Some(end)) = (line(pair.open), line(pair.close)) {
            if end > start + 1 {
                ranges.push((start, end - 1, None));
            }
        }
    }

    let surface = parse_surface_ast("", source);
    for declaration in &surface.declarations {
        let span = declaration.span();
        if let (Some(start), Some(end)) = (line(span.start), line(span.end.saturating_sub(1))) {
            if end > start {
                ranges.push((start, end, None));
            }
        }
    }

    // A run of two or more import lines folds as one group.
    let mut imports = surface
        .imports
        .iter()
        .filter_map(|import| {
            Some((
                line(import.span.start)?,
                line(import.span.end.saturating_sub(1))?,
            ))
        })
        .collect::<Vec<_>>();
    imports.sort();
    push_runs(&mut ranges, &imports, "imports");

    // Consecutive comment lines, and a block comment that spans lines.
    let mut comments = Vec::new();
    for token in tokens
        .iter()
        .filter(|token| token.kind == TokenKind::TriviaComment)
    {
        if let (Some(start), Some(end)) = (line(token.start), line(token.end.saturating_sub(1))) {
            comments.push((start, end));
        }
    }
    push_runs(&mut ranges, &comments, "comment");

    ranges.sort();
    ranges.dedup_by(|later, earlier| later.0 == earlier.0 && later.1 == earlier.1);
    Value::Array(
        ranges
            .into_iter()
            .map(|(start, end, kind)| {
                let mut range = json!({"startLine": start, "endLine": end});
                if let Some(kind) = kind {
                    range["kind"] = json!(kind);
                }
                range
            })
            .collect(),
    )
}

/// Merges line ranges whose lines touch into runs and keeps the runs that
/// cover more than one line.
fn push_runs<'kind>(
    ranges: &mut Vec<(u32, u32, Option<&'kind str>)>,
    lines: &[(u32, u32)],
    kind: &'kind str,
) {
    let mut run: Option<(u32, u32)> = None;
    for &(start, end) in lines {
        run = match run {
            Some((first, last)) if start <= last + 1 => Some((first, last.max(end))),
            Some((first, last)) => {
                if last > first {
                    ranges.push((first, last, Some(kind)));
                }
                Some((start, end))
            }
            None => Some((start, end)),
        };
    }
    if let Some((first, last)) = run {
        if last > first {
            ranges.push((first, last, Some(kind)));
        }
    }
}

pub(crate) fn selection_ranges(
    source: &str,
    positions: &[usize],
    encoding: PositionEncoding,
) -> Value {
    let tokens = significant_tokens(source);
    let pairs = brackets(&tokens);
    let surface = parse_surface_ast("", source);
    let declarations = surface
        .declarations
        .iter()
        .map(|declaration| declaration.span())
        .chain(surface.imports.iter().map(|import| import.span))
        .collect::<Vec<_>>();
    Value::Array(
        positions
            .iter()
            .map(|&position| {
                selection_chain(source, &tokens, &pairs, &declarations, position, encoding)
            })
            .collect(),
    )
}

/// Builds the chain from the token under the cursor out to the whole
/// document: token, bracket contents, the bracketed expression, enclosing
/// brackets in turn, the declaration, then the file.
fn selection_chain(
    source: &str,
    tokens: &[Token],
    pairs: &[Bracket],
    declarations: &[ByteSpan],
    position: usize,
    encoding: PositionEncoding,
) -> Value {
    let mut spans = Vec::new();
    if let Some(token) = tokens.iter().find(|token| {
        token.start <= position
            && position < token.end
            && !matches!(token.kind, TokenKind::TriviaNewline)
    }) {
        spans.push(ByteSpan {
            start: token.start,
            end: token.end,
        });
    }
    for pair in pairs {
        if pair.open < position && position <= pair.close {
            if let Some(inner) = trimmed(source, pair.open + 1, pair.close) {
                spans.push(inner);
            }
        }
        if pair.open <= position && position <= pair.close {
            spans.push(ByteSpan {
                start: pair.open,
                end: pair.close + 1,
            });
        }
    }
    spans.extend(
        declarations
            .iter()
            .filter(|span| span.start <= position && position <= span.end)
            .copied(),
    );
    spans.push(ByteSpan {
        start: 0,
        end: source.len(),
    });
    spans.retain(|span| span.start <= position && position <= span.end);
    spans.sort_by_key(|span| (span.end - span.start, std::cmp::Reverse(span.start)));
    spans.dedup();
    // Each parent must contain its child; drop spans that only overlap.
    let mut chain: Vec<ByteSpan> = Vec::new();
    for span in spans {
        if chain.last().is_none_or(|child| {
            span.start <= child.start && child.end <= span.end && span != *child
        }) {
            chain.push(span);
        }
    }
    let mut selection = Value::Null;
    for span in chain.iter().rev() {
        let Some(range) = range_json(source, span.start, span.end, encoding) else {
            continue;
        };
        selection = if selection.is_null() {
            json!({"range": range})
        } else {
            json!({"range": range, "parent": selection})
        };
    }
    if selection.is_null() {
        let empty = range_json(source, position, position, encoding).unwrap_or(Value::Null);
        selection = json!({"range": empty});
    }
    selection
}

fn trimmed(source: &str, start: usize, end: usize) -> Option<ByteSpan> {
    let text = source.get(start..end)?;
    let leading = text.len() - text.trim_start().len();
    let trailing = text.len() - text.trim_end().len();
    (leading + trailing < text.len()).then_some(ByteSpan {
        start: start + leading,
        end: end - trailing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_blocks_declarations_imports_and_comment_runs() {
        let source = "import * as a from \"std/list\"\nimport * as b from \"std/maybe\"\n\n// one\n// two\npub fn value x: Int -> Int =\n  match x {\n    0 -> 1\n    _ -> 2\n  }\n";
        let folds = folding_ranges(source);
        let folds = folds.as_array().unwrap();
        assert!(
            folds.contains(&json!({"startLine": 0, "endLine": 1, "kind": "imports"})),
            "{folds:?}"
        );
        assert!(folds.contains(&json!({"startLine": 3, "endLine": 4, "kind": "comment"})));
        assert!(folds.contains(&json!({"startLine": 5, "endLine": 9})));
        assert!(folds.contains(&json!({"startLine": 6, "endLine": 8})));
    }

    #[test]
    fn folds_unfinished_source_without_a_parse() {
        let source = "pub fn value -> Int = {\n  let a = 1\n  let b = (\n    a +\n";
        let folds = folding_ranges(source);
        // The unclosed blocks give no fold, and the request still succeeds.
        assert!(folds.is_array());
        let closed = "pub fn broken -> Int = {\n  let a = (\n    1 +\n  )\n  a +\n}\n";
        let folds = folding_ranges(closed);
        assert!(
            folds
                .as_array()
                .unwrap()
                .contains(&json!({"startLine": 0, "endLine": 4})),
            "{folds:?}"
        );
    }

    #[test]
    fn selection_grows_from_the_token_to_the_document() {
        let source = "pub fn value x: Int -> Int = add (x + 1) 2\n";
        let position = source.find("x + 1").unwrap();
        let selections = selection_ranges(source, &[position], PositionEncoding::Utf16);
        let mut ranges = Vec::new();
        let mut current = &selections[0];
        while !current.is_null() {
            let range = &current["range"];
            ranges.push((
                range["start"]["character"].as_u64().unwrap(),
                range["end"]["character"].as_u64().unwrap(),
                range["end"]["line"].as_u64().unwrap(),
            ));
            current = &current["parent"];
        }
        let column = position as u64;
        assert_eq!(ranges[0], (column, column + 1, 0));
        assert_eq!(ranges[1], (column, column + 5, 0));
        assert_eq!(ranges[2], (column - 1, column + 6, 0));
        assert_eq!(ranges.last().unwrap(), &(0, 0, 1));
    }
}
