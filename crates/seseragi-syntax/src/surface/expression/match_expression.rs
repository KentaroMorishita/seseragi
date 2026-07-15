use super::{find_matching_brace, parse_expression_range, ExpressionParser};
use crate::line_continuation::starts_with_operator;
use crate::surface::pattern::parse_pattern_range;
use crate::surface_model::{ByteSpan, SurfaceExpr, SurfaceMatchArm};
use crate::token::{Token, TokenKind};

pub(super) fn parse(parser: &mut ExpressionParser<'_>, match_token: &Token) -> Option<SurfaceExpr> {
    let scrutinee = parser.parse_expr_bp(0, &[TokenKind::PunctuationBraceLeft])?;
    parser.skip_trivia();
    let open = parser.cursor;
    if parser.kind_at_cursor() != Some(TokenKind::PunctuationBraceLeft) {
        return None;
    }
    let close = find_matching_brace(parser.tokens, open, parser.end)?;
    let arms = parse_arms(parser.tokens, open + 1, close);
    parser.cursor = close + 1;
    Some(SurfaceExpr::Match {
        scrutinee: Box::new(scrutinee),
        arms,
        span: ByteSpan {
            start: match_token.start,
            end: parser.tokens[close].end,
        },
    })
}

fn parse_arms(tokens: &[Token], start: usize, end: usize) -> Vec<SurfaceMatchArm> {
    split_arm_ranges(tokens, start, end)
        .into_iter()
        .map(|(start, end)| parse_arm(tokens, start, end))
        .collect()
}

fn parse_arm(tokens: &[Token], start: usize, end: usize) -> SurfaceMatchArm {
    let significant = significant_indices(tokens, start, end);
    let first = significant.first().copied().unwrap_or(start);
    let last = significant.last().copied().unwrap_or(first);
    let arrow = find_top_level(tokens, start, end, TokenKind::OperatorArrow);
    let guard_start =
        arrow.and_then(|arrow| find_top_level(tokens, start, arrow, TokenKind::KeywordWhen));
    let pattern_end = guard_start.or(arrow).unwrap_or(end);
    let pattern = parse_pattern_range(tokens, start, pattern_end);
    let guard = guard_start.map(|when| {
        let guard_end = arrow.unwrap_or(end);
        expression_or_error(tokens, when + 1, guard_end, tokens[when].end)
    });
    let body = arrow.map_or_else(
        || error_expression(tokens.get(last).map_or(0, |token| token.end)),
        |arrow| expression_or_error(tokens, arrow + 1, end, tokens[arrow].end),
    );
    let span = if significant.is_empty() {
        pattern.span()
    } else {
        ByteSpan {
            start: tokens[first].start,
            end: tokens[last].end,
        }
    };
    SurfaceMatchArm {
        pattern,
        guard,
        body,
        span,
    }
}

fn split_arm_ranges(tokens: &[Token], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut range_start = start;
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut square_depth = 0usize;

    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            TokenKind::PunctuationBraceLeft => brace_depth += 1,
            TokenKind::PunctuationBraceRight => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::PunctuationParenLeft => paren_depth += 1,
            TokenKind::PunctuationParenRight => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::PunctuationSquareLeft => square_depth += 1,
            TokenKind::PunctuationSquareRight => square_depth = square_depth.saturating_sub(1),
            TokenKind::PunctuationSemicolon
                if brace_depth == 0 && paren_depth == 0 && square_depth == 0 =>
            {
                push_non_empty_range(tokens, &mut ranges, range_start, index);
                range_start = index + 1;
            }
            TokenKind::TriviaNewline
                if brace_depth == 0
                    && paren_depth == 0
                    && square_depth == 0
                    && should_split_at_newline(tokens, range_start, index, end) =>
            {
                push_non_empty_range(tokens, &mut ranges, range_start, index);
                range_start = index + 1;
            }
            _ => {}
        }
    }
    push_non_empty_range(tokens, &mut ranges, range_start, end);
    ranges
}

fn should_split_at_newline(tokens: &[Token], start: usize, newline: usize, end: usize) -> bool {
    if starts_with_operator(tokens, newline + 1, end) {
        return false;
    }
    let Some(arrow) = find_top_level(tokens, start, newline, TokenKind::OperatorArrow) else {
        return !significant_indices(tokens, start, newline).is_empty();
    };
    if parse_expression_range(tokens, arrow + 1, newline).is_some() {
        return true;
    }

    next_logical_line_has_arm_arrow(tokens, newline + 1, end)
}

fn next_logical_line_has_arm_arrow(tokens: &[Token], start: usize, end: usize) -> bool {
    let line_start = (start..end).find(|index| tokens.get(*index).is_some_and(is_significant));
    let Some(line_start) = line_start else {
        return false;
    };
    let line_end = tokens
        .iter()
        .enumerate()
        .take(end)
        .skip(line_start)
        .find_map(|(index, token)| {
            matches!(
                token.kind,
                TokenKind::TriviaNewline | TokenKind::PunctuationSemicolon
            )
            .then_some(index)
        })
        .unwrap_or(end);
    find_top_level(tokens, line_start, line_end, TokenKind::OperatorArrow).is_some()
}

fn find_top_level(
    tokens: &[Token],
    start: usize,
    end: usize,
    expected: TokenKind,
) -> Option<usize> {
    let mut brace_depth = 0usize;
    let mut paren_depth = 0usize;
    let mut square_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        let kind = token.kind;
        if kind == expected && brace_depth == 0 && paren_depth == 0 && square_depth == 0 {
            return Some(index);
        }
        match kind {
            TokenKind::PunctuationBraceLeft => brace_depth += 1,
            TokenKind::PunctuationBraceRight => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::PunctuationParenLeft => paren_depth += 1,
            TokenKind::PunctuationParenRight => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::PunctuationSquareLeft => square_depth += 1,
            TokenKind::PunctuationSquareRight => square_depth = square_depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn expression_or_error(tokens: &[Token], start: usize, end: usize, at: usize) -> SurfaceExpr {
    parse_expression_range(tokens, start, end).unwrap_or_else(|| error_expression(at))
}

fn error_expression(at: usize) -> SurfaceExpr {
    SurfaceExpr::Error {
        span: ByteSpan { start: at, end: at },
    }
}

fn push_non_empty_range(
    tokens: &[Token],
    ranges: &mut Vec<(usize, usize)>,
    start: usize,
    end: usize,
) {
    if !significant_indices(tokens, start, end).is_empty() {
        ranges.push((start, end));
    }
}

fn significant_indices(tokens: &[Token], start: usize, end: usize) -> Vec<usize> {
    (start..end)
        .filter(|index| tokens.get(*index).is_some_and(is_significant))
        .collect()
}

fn is_significant(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::PunctuationSemicolon
    )
}
