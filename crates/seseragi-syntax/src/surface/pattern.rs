use crate::surface_model::{ByteSpan, SurfacePattern, SurfaceRecordPatternField};
use crate::token::{Token, TokenKind};

#[cfg(test)]
mod tests;

#[cfg(test)]
thread_local! { static SIGNIFICANT_TOKEN_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }

pub(super) fn parse_pattern_range(tokens: &[Token], start: usize, end: usize) -> SurfacePattern {
    let at = tokens
        .get(start)
        .map(|token| ByteSpan {
            start: token.start,
            end: token.end,
        })
        .unwrap_or(ByteSpan { start: 0, end: 0 });
    let Some(_frame) = crate::resource::enter(at) else {
        return SurfacePattern::Error { span: at };
    };
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

    // Constructor application is right-nested, but scanning each argument as a
    // fresh range rescans the same suffix and uses one native frame per prefix.
    // Consume the already indexed prefix once, then build its nodes backwards.
    let mut remaining = significant.as_slice();
    let mut constructors = Vec::new();
    let mut depth = crate::resource::PatternDepth::default();
    let mut argument = loop {
        let first = remaining[0];
        let current_span = ByteSpan {
            start: tokens[first].start,
            end: span.end,
        };
        if !depth.reserve(current_span) {
            return SurfacePattern::Error { span };
        }
        // An unqualified constructor followed by braces is a struct pattern,
        // not another constructor prefix. Its fields retain the same budget.
        if tokens[first].kind == TokenKind::IdentifierUpper
            && remaining.get(1).is_some_and(|index| {
                tokens[*index].kind == TokenKind::PunctuationBraceLeft
                    && tokens[last].kind == TokenKind::PunctuationBraceRight
            })
        {
            let record = parse_record_pattern(tokens, remaining[1], last, current_span);
            break Some(match record {
                SurfacePattern::Record { fields, .. } => SurfacePattern::Struct {
                    name: tokens[first].raw.clone(),
                    name_span: ByteSpan {
                        start: tokens[first].start,
                        end: tokens[first].end,
                    },
                    fields,
                    span: current_span,
                },
                _ => SurfacePattern::Error { span: current_span },
            });
        }
        let (name, name_span, consumed) = if tokens[first].kind == TokenKind::IdentifierUpper {
            (
                tokens[first].raw.clone(),
                ByteSpan {
                    start: tokens[first].start,
                    end: tokens[first].end,
                },
                1,
            )
        } else if tokens[first].kind == TokenKind::IdentifierLower
            && remaining
                .get(1)
                .is_some_and(|index| tokens[*index].kind == TokenKind::PunctuationDot)
            && remaining
                .get(2)
                .is_some_and(|index| tokens[*index].kind == TokenKind::IdentifierUpper)
        {
            let constructor_name = remaining[2];
            (
                format!("{}.{}", tokens[first].raw, tokens[constructor_name].raw),
                ByteSpan {
                    start: tokens[first].start,
                    end: tokens[constructor_name].end,
                },
                3,
            )
        } else {
            break Some(parse_non_constructor_pattern(
                tokens,
                remaining,
                current_span,
            ));
        };
        constructors.push((name, name_span, current_span));
        remaining = &remaining[consumed..];
        if remaining.is_empty() {
            break None;
        }
    };
    if matches!(argument, Some(SurfacePattern::Error { .. })) {
        return SurfacePattern::Error { span };
    }
    for (name, name_span, span) in constructors.into_iter().rev() {
        argument = Some(SurfacePattern::Constructor {
            name,
            name_span,
            argument: argument.map(Box::new),
            span,
        });
    }
    argument.unwrap()
}

fn parse_non_constructor_pattern(
    tokens: &[Token],
    significant: &[usize],
    span: ByteSpan,
) -> SurfacePattern {
    let first = significant[0];
    let last = *significant.last().unwrap();
    if tokens[first].kind == TokenKind::PunctuationParenLeft
        && tokens[last].kind == TokenKind::PunctuationParenRight
    {
        return parse_parenthesized_pattern(tokens, first, last, span);
    }
    if matches!(
        tokens[first].kind,
        TokenKind::PunctuationSquareLeft | TokenKind::PunctuationListLeft
    ) && tokens[last].kind == TokenKind::PunctuationSquareRight
    {
        return parse_collection_pattern(tokens, first, last, span);
    }
    if tokens[first].kind == TokenKind::PunctuationBraceLeft
        && tokens[last].kind == TokenKind::PunctuationBraceRight
    {
        return parse_record_pattern(tokens, first, last, span);
    }
    if significant.len() == 2
        && tokens[first].kind == TokenKind::OperatorArithmetic
        && tokens[first].raw == "-"
        && tokens[last].kind == TokenKind::LiteralInteger
    {
        return SurfacePattern::Integer {
            raw: format!("-{}", tokens[last].raw),
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
        TokenKind::LiteralChar => SurfacePattern::Char {
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

fn parse_collection_pattern(
    tokens: &[Token],
    open: usize,
    close: usize,
    span: ByteSpan,
) -> SurfacePattern {
    let is_list = tokens[open].kind == TokenKind::PunctuationListLeft;
    if significant_indices(tokens, open + 1, close).is_empty() {
        return collection_pattern(is_list, Vec::new(), None, span);
    }
    let mut ranges = split_top_level_commas(tokens, open + 1, close);
    if ranges
        .last()
        .is_some_and(|(start, end)| significant_indices(tokens, *start, *end).is_empty())
    {
        ranges.pop();
    }
    let mut elements = Vec::new();
    let mut rest = None;
    let range_count = ranges.len();
    for (index, (start, end)) in ranges.into_iter().enumerate() {
        let significant = significant_indices(tokens, start, end);
        let Some(first) = significant.first().copied() else {
            return SurfacePattern::Error { span };
        };
        if tokens[first].kind == TokenKind::PunctuationEllipsis {
            if index + 1 != range_count || significant.len() < 2 || rest.is_some() {
                return SurfacePattern::Error { span };
            }
            let rest_pattern = parse_pattern_range(tokens, first + 1, end);
            if !matches!(
                rest_pattern,
                SurfacePattern::Name { .. } | SurfacePattern::Wildcard { .. }
            ) {
                return SurfacePattern::Error { span };
            }
            rest = Some(Box::new(rest_pattern));
            continue;
        }
        let element = parse_pattern_range(tokens, start, end);
        if matches!(element, SurfacePattern::Error { .. }) {
            return SurfacePattern::Error { span };
        }
        elements.push(element);
    }
    collection_pattern(is_list, elements, rest, span)
}

fn collection_pattern(
    is_list: bool,
    elements: Vec<SurfacePattern>,
    rest: Option<Box<SurfacePattern>>,
    span: ByteSpan,
) -> SurfacePattern {
    if is_list {
        SurfacePattern::List {
            elements,
            rest,
            span,
        }
    } else {
        SurfacePattern::Array {
            elements,
            rest,
            span,
        }
    }
}

fn parse_parenthesized_pattern(
    tokens: &[Token],
    open: usize,
    close: usize,
    span: ByteSpan,
) -> SurfacePattern {
    let ranges = split_top_level_commas(tokens, open + 1, close);
    if ranges.len() == 1 {
        let (start, end) = ranges[0];
        return parse_pattern_range(tokens, start, end);
    }
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

fn parse_record_pattern(
    tokens: &[Token],
    open: usize,
    close: usize,
    span: ByteSpan,
) -> SurfacePattern {
    if significant_indices(tokens, open + 1, close).is_empty() {
        return SurfacePattern::Record {
            fields: Vec::new(),
            span,
        };
    }
    let mut ranges = split_top_level_commas(tokens, open + 1, close);
    if ranges
        .last()
        .is_some_and(|(start, end)| significant_indices(tokens, *start, *end).is_empty())
    {
        ranges.pop();
    }
    let fields = ranges
        .into_iter()
        .map(|(start, end)| parse_record_pattern_field(tokens, start, end))
        .collect::<Option<Vec<_>>>();
    fields.map_or(SurfacePattern::Error { span }, |fields| {
        SurfacePattern::Record { fields, span }
    })
}

fn parse_record_pattern_field(
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Option<SurfaceRecordPatternField> {
    let significant = significant_indices(tokens, start, end);
    let name_index = *significant.first()?;
    let name_token = tokens.get(name_index)?;
    if name_token.kind != TokenKind::IdentifierLower {
        return None;
    }
    let name_span = ByteSpan {
        start: name_token.start,
        end: name_token.end,
    };
    let mut cursor = 1usize;
    let optional = significant
        .get(cursor)
        .and_then(|index| tokens.get(*index))
        .is_some_and(|token| token.raw == "?");
    if optional {
        cursor += 1;
    }
    let pattern = if significant
        .get(cursor)
        .and_then(|index| tokens.get(*index))
        .is_some_and(|token| token.kind == TokenKind::PunctuationColon)
    {
        let pattern_start = significant.get(cursor + 1).copied()?;
        parse_pattern_range(tokens, pattern_start, end)
    } else {
        if cursor != significant.len() {
            return None;
        }
        SurfacePattern::Name {
            name: name_token.raw.clone(),
            name_span,
            span: name_span,
        }
    };
    if matches!(pattern, SurfacePattern::Error { .. }) {
        return None;
    }
    let last = *significant.last()?;
    Some(SurfaceRecordPatternField {
        name: name_token.raw.clone(),
        name_span,
        optional,
        pattern,
        span: ByteSpan {
            start: name_token.start,
            end: tokens.get(last)?.end,
        },
    })
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
    ranges.push((range_start, end));
    ranges
}

fn significant_indices(tokens: &[Token], start: usize, end: usize) -> Vec<usize> {
    (start..end)
        .filter(|index| {
            #[cfg(test)]
            SIGNIFICANT_TOKEN_VISITS.with(|visits| visits.set(visits.get() + 1));
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
