use crate::surface_model::{ByteSpan, SurfacePattern};
use crate::token::{Token, TokenKind};

pub(super) fn parse_pattern_range(tokens: &[Token], start: usize, end: usize) -> SurfacePattern {
    let significant = significant_indices(tokens, start, end);
    let Some(first) = significant.first().copied() else {
        let at = tokens.get(start).map(|token| token.start).unwrap_or(0);
        return SurfacePattern::Error {
            span: ByteSpan { start: at, end: at },
        };
    };
    let last = significant.last().copied().unwrap_or(first);
    let span = ByteSpan {
        start: tokens[first].start,
        end: tokens[last].end,
    };

    if tokens[first].kind == TokenKind::PunctuationParenLeft
        && tokens[last].kind == TokenKind::PunctuationParenRight
    {
        return parse_parenthesized_pattern(tokens, first, last, span);
    }
    let constructor = if tokens[first].kind == TokenKind::IdentifierUpper {
        Some((
            tokens[first].raw.clone(),
            ByteSpan {
                start: tokens[first].start,
                end: tokens[first].end,
            },
            first + 1,
        ))
    } else if tokens[first].kind == TokenKind::IdentifierLower
        && significant
            .get(1)
            .is_some_and(|index| tokens[*index].kind == TokenKind::PunctuationDot)
        && significant
            .get(2)
            .is_some_and(|index| tokens[*index].kind == TokenKind::IdentifierUpper)
    {
        let constructor_name = significant[2];
        Some((
            format!("{}.{}", tokens[first].raw, tokens[constructor_name].raw),
            ByteSpan {
                start: tokens[first].start,
                end: tokens[constructor_name].end,
            },
            constructor_name + 1,
        ))
    } else {
        None
    };
    if let Some((name, name_span, argument_start)) = constructor {
        let argument = (significant
            .last()
            .is_some_and(|last| *last >= argument_start))
        .then(|| parse_pattern_range(tokens, argument_start, end))
        .map(Box::new);
        if argument
            .as_deref()
            .is_some_and(|argument| matches!(argument, SurfacePattern::Error { .. }))
        {
            return SurfacePattern::Error { span };
        }
        return SurfacePattern::Constructor {
            name,
            name_span,
            argument,
            span,
        };
    }
    if significant.len() != 1 {
        return SurfacePattern::Error { span };
    }
    match tokens[first].kind {
        TokenKind::LiteralInteger => SurfacePattern::Integer {
            raw: tokens[first].raw.clone(),
            span,
        },
        TokenKind::LiteralString => SurfacePattern::String {
            raw: tokens[first].raw.clone(),
            span,
        },
        TokenKind::LiteralBoolean => SurfacePattern::Boolean {
            value: tokens[first].raw == "True",
            span,
        },
        TokenKind::IdentifierLower => SurfacePattern::Name {
            name: tokens[first].raw.clone(),
            name_span: span,
            span,
        },
        TokenKind::Wildcard => SurfacePattern::Wildcard { span },
        _ => SurfacePattern::Error { span },
    }
}

fn parse_parenthesized_pattern(
    tokens: &[Token],
    open: usize,
    close: usize,
    span: ByteSpan,
) -> SurfacePattern {
    let ranges = split_top_level_commas(tokens, open + 1, close);
    if ranges.len() < 2 || ranges.iter().any(|(start, end)| start == end) {
        return SurfacePattern::Error { span };
    }
    let elements = ranges
        .into_iter()
        .map(|(start, end)| parse_pattern_range(tokens, start, end))
        .collect::<Vec<_>>();
    if elements
        .iter()
        .any(|element| matches!(element, SurfacePattern::Error { .. }))
    {
        return SurfacePattern::Error { span };
    }
    SurfacePattern::Tuple { elements, span }
}

fn split_top_level_commas(tokens: &[Token], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut range_start = start;
    let mut paren_depth = 0usize;
    let mut square_depth = 0usize;
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            TokenKind::PunctuationParenLeft => paren_depth += 1,
            TokenKind::PunctuationParenRight => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => square_depth += 1,
            TokenKind::PunctuationSquareRight => square_depth = square_depth.saturating_sub(1),
            TokenKind::PunctuationBraceLeft => brace_depth += 1,
            TokenKind::PunctuationBraceRight => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::PunctuationComma
                if paren_depth == 0 && square_depth == 0 && brace_depth == 0 =>
            {
                ranges.push((range_start, index));
                range_start = index + 1;
            }
            _ => {}
        }
    }
    if ranges.is_empty() {
        return ranges;
    }
    ranges.push((range_start, end));
    ranges
}

fn significant_indices(tokens: &[Token], start: usize, end: usize) -> Vec<usize> {
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
