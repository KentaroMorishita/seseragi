use super::parse_expression_range;
use crate::line_continuation::starts_with_operator;
use crate::surface::pattern::parse_pattern_range;
use crate::surface_model::{ByteSpan, SurfaceDoItem, SurfaceExpr};
use crate::token::{Token, TokenKind};

pub(super) fn parse_do_contents(
    tokens: &[Token],
    open: usize,
    close: usize,
) -> (Vec<SurfaceDoItem>, Option<SurfaceExpr>) {
    let segments = split_segments(tokens, open + 1, close);
    let mut items = Vec::new();
    let mut result = None;
    let segment_count = segments.len();

    for (position, (start, end)) in segments.into_iter().enumerate() {
        let is_last = position + 1 == segment_count;
        match parse_segment(tokens, start, end) {
            Some(ParsedSegment::Bind(item) | ParsedSegment::Let(item)) => items.push(item),
            Some(ParsedSegment::Expression(expression)) if is_last => result = Some(expression),
            Some(ParsedSegment::Expression(expression)) => {
                let span = expression.span();
                items.push(SurfaceDoItem::Expression {
                    value: expression,
                    span,
                });
            }
            None => {}
        }
    }

    (items, result)
}

enum ParsedSegment {
    Bind(SurfaceDoItem),
    Let(SurfaceDoItem),
    Expression(SurfaceExpr),
}

fn parse_segment(tokens: &[Token], start: usize, end: usize) -> Option<ParsedSegment> {
    let significant = significant_indices(tokens, start, end);
    let first = *significant.first()?;
    let last = *significant.last()?;
    let span = ByteSpan {
        start: tokens[first].start,
        end: tokens[last].end,
    };

    if tokens[first].kind != TokenKind::KeywordFor {
        if let Some(bind) = top_level_token(tokens, start, end, TokenKind::OperatorBind) {
            let value = expression_or_error(tokens, bind + 1, end, tokens[bind].end);
            return Some(ParsedSegment::Bind(SurfaceDoItem::Bind {
                pattern: parse_pattern_range(tokens, start, bind),
                value,
                span,
            }));
        }
    }

    if tokens[first].kind == TokenKind::KeywordLet {
        let equals = significant
            .iter()
            .copied()
            .find(|index| tokens[*index].kind == TokenKind::OperatorEquals);
        let (pattern_end, value_start, error_at) = equals
            .map(|equals| (equals, equals + 1, tokens[equals].end))
            .unwrap_or((end, end, tokens[first].end));
        let value = expression_or_error(tokens, value_start, end, error_at);
        return Some(ParsedSegment::Let(SurfaceDoItem::Let {
            pattern: parse_pattern_range(tokens, first + 1, pattern_end),
            value,
            span,
        }));
    }

    Some(ParsedSegment::Expression(expression_or_error(
        tokens,
        start,
        end,
        tokens[first].start,
    )))
}

fn expression_or_error(tokens: &[Token], start: usize, end: usize, error_at: usize) -> SurfaceExpr {
    parse_expression_range(tokens, start, end).unwrap_or(SurfaceExpr::Error {
        span: ByteSpan {
            start: error_at,
            end: error_at,
        },
    })
}

pub(super) fn split_segments(tokens: &[Token], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut segment_start = start;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut square_depth = 0usize;

    for index in start..end {
        match tokens[index].kind {
            TokenKind::PunctuationBraceLeft => brace_depth += 1,
            TokenKind::PunctuationBraceRight => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::PunctuationParenLeft => paren_depth += 1,
            TokenKind::PunctuationParenRight => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => square_depth += 1,
            TokenKind::PunctuationSquareRight => square_depth = square_depth.saturating_sub(1),
            TokenKind::PunctuationSemicolon
                if brace_depth == 0 && paren_depth == 0 && square_depth == 0 =>
            {
                push_non_empty_segment(tokens, &mut segments, segment_start, index);
                segment_start = index + 1;
            }
            TokenKind::TriviaNewline
                if brace_depth == 0
                    && paren_depth == 0
                    && square_depth == 0
                    && !starts_with_operator(tokens, index + 1, end)
                    && segment_is_complete(tokens, segment_start, index) =>
            {
                push_non_empty_segment(tokens, &mut segments, segment_start, index);
                segment_start = index + 1;
            }
            _ => {}
        }
    }
    push_non_empty_segment(tokens, &mut segments, segment_start, end);
    segments
}

fn segment_is_complete(tokens: &[Token], start: usize, end: usize) -> bool {
    let significant = significant_indices(tokens, start, end);
    let Some(first) = significant.first().copied() else {
        return false;
    };
    if tokens[first].kind != TokenKind::KeywordFor {
        if let Some(bind) = top_level_token(tokens, start, end, TokenKind::OperatorBind) {
            return parse_expression_range(tokens, bind + 1, end).is_some();
        }
    }
    if tokens[first].kind == TokenKind::KeywordLet {
        return significant
            .iter()
            .copied()
            .find(|index| tokens[*index].kind == TokenKind::OperatorEquals)
            .is_some_and(|equals| parse_expression_range(tokens, equals + 1, end).is_some());
    }
    if tokens[first].kind == TokenKind::KeywordFn {
        return top_level_token(tokens, start, end, TokenKind::OperatorEquals)
            .is_some_and(|equals| parse_expression_range(tokens, equals + 1, end).is_some());
    }
    parse_expression_range(tokens, start, end).is_some()
}

fn top_level_token(
    tokens: &[Token],
    start: usize,
    end: usize,
    expected: TokenKind,
) -> Option<usize> {
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut square_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        if token.kind == expected && brace_depth == 0 && paren_depth == 0 && square_depth == 0 {
            return Some(index);
        }
        match token.kind {
            TokenKind::PunctuationBraceLeft => brace_depth += 1,
            TokenKind::PunctuationBraceRight => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::PunctuationParenLeft => paren_depth += 1,
            TokenKind::PunctuationParenRight => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => square_depth += 1,
            TokenKind::PunctuationSquareRight => square_depth = square_depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn push_non_empty_segment(
    tokens: &[Token],
    segments: &mut Vec<(usize, usize)>,
    start: usize,
    end: usize,
) {
    if significant_indices(tokens, start, end).is_empty() {
        return;
    }
    segments.push((start, end));
}

pub(super) fn significant_indices(tokens: &[Token], start: usize, end: usize) -> Vec<usize> {
    (start..end)
        .filter(|index| {
            tokens.get(*index).is_some_and(|token| {
                !matches!(
                    token.kind,
                    TokenKind::TriviaComment
                        | TokenKind::TriviaNewline
                        | TokenKind::TriviaSpace
                        | TokenKind::PunctuationSemicolon
                )
            })
        })
        .collect()
}
