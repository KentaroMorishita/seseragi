use crate::{layout, FormatOptions};
use seseragi_syntax::{lex, CstArtifact, CstNode, Token, TokenKind, TokenStream};
use std::ops::Range;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormatEdit {
    pub range: Range<usize>,
    pub text: String,
}

fn anchor(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaSpace | TokenKind::TriviaNewline | TokenKind::Eof
    )
}

fn smallest(node: &CstNode, first: usize, last: usize) -> Option<&CstNode> {
    if node.start_token > first || node.end_token <= last {
        return None;
    }
    node.children
        .iter()
        .filter_map(|child| smallest(child, first, last))
        .min_by_key(|child| child.end_token - child.start_token)
        .or_else(|| (node.kind.starts_with("complete-") || node.kind == "top-decl").then_some(node))
}

/// Projects canonical document layout onto complete CST nodes. Tokens outside
/// those nodes are never replaced; comments and literal spelling are anchors.
pub fn format_cst_range(
    tokens: &TokenStream,
    cst: &CstArtifact,
    range: Range<usize>,
    options: FormatOptions,
) -> Vec<FormatEdit> {
    let source = tokens.reconstructed_text();
    if range.start > range.end
        || range.end > source.len()
        || !source.is_char_boundary(range.start)
        || !source.is_char_boundary(range.end)
    {
        return vec![];
    }
    let original: Vec<_> = tokens
        .tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| anchor(t))
        .collect();
    let selected: Vec<_> = original
        .iter()
        .filter(|(_, t)| {
            if range.is_empty() {
                t.start <= range.start && range.start < t.end
            } else {
                t.start < range.end && range.start < t.end
            }
        })
        .collect();
    if selected.is_empty() {
        return vec![];
    }
    let formatted = layout::format_valid_module(tokens, cst, options.line_width);
    let formatted_tokens = lex(&tokens.source, &formatted);
    let anchors: Vec<_> = formatted_tokens
        .tokens
        .iter()
        .filter(|t| anchor(t))
        .collect();
    if original.len() != anchors.len()
        || original
            .iter()
            .zip(&anchors)
            .any(|((_, a), b)| a.kind != b.kind || a.raw != b.raw)
    {
        return vec![];
    }
    let mut edits = vec![];
    for top in &cst.root.children {
        let mut intersection = selected
            .iter()
            .filter(|(index, _)| top.start_token <= *index && *index < top.end_token);
        let Some(first) = intersection.next() else {
            continue;
        };
        let last = intersection.next_back().unwrap_or(first);
        let (begin, end) = (first.0, last.0);
        let Some(node) = smallest(top, begin, end) else {
            continue;
        };
        if has_error(node)
            || cst.errors.iter().any(|error| {
                error.start_token < node.end_token && node.start_token < error.end_token
            })
            || cst.missing.iter().any(|missing| {
                node.start_token <= missing.at_token && missing.at_token <= node.end_token
            })
        {
            continue;
        }
        let Some(a) = original.iter().position(|(i, _)| *i >= node.start_token) else {
            continue;
        };
        let Some(b) = original.iter().rposition(|(i, _)| *i < node.end_token) else {
            continue;
        };
        if a > b {
            continue;
        }
        let mut start = original[a].1.start;
        let mut mapped_start = anchors[a].start;
        let end = original[b].1.end;
        let mapped_end = anchors[b].end;
        let line_start = source[..start].rfind('\n').map_or(0, |i| i + 1);
        let mapped_line_start = formatted[..mapped_start].rfind('\n').map_or(0, |i| i + 1);
        if source[line_start..start].trim().is_empty()
            && formatted[mapped_line_start..mapped_start].trim().is_empty()
        {
            start = line_start;
            mapped_start = mapped_line_start;
        }
        let old = &source[start..end];
        let new = &formatted[mapped_start..mapped_end];
        let mut prefix = old
            .bytes()
            .zip(new.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while !old.is_char_boundary(prefix) || !new.is_char_boundary(prefix) {
            prefix -= 1;
        }
        let mut suffix = old[prefix..]
            .bytes()
            .rev()
            .zip(new[prefix..].bytes().rev())
            .take_while(|(a, b)| a == b)
            .count();
        while !old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix)
        {
            suffix -= 1;
        }
        if old != new {
            edits.push(FormatEdit {
                range: start + prefix..end - suffix,
                text: new[prefix..new.len() - suffix].to_owned(),
            });
        }
    }
    edits
}

fn has_error(node: &CstNode) -> bool {
    node.kind == "complete-error" || node.children.iter().any(has_error)
}
