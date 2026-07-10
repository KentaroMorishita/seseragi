use seseragi_syntax::{ByteSpan, Token, TokenKind};

use super::{is_known_effect_surface_operation, is_significant, token_span, EffectFunctionIssue};

pub(super) fn invalid_bind_issues(tokens: &[Token], span: ByteSpan) -> Vec<EffectFunctionIssue> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| {
            token.start >= span.start
                && token.end <= span.end
                && token.kind == TokenKind::OperatorBind
        })
        .filter_map(|(index, bind)| {
            let operation = tokens[index + 1..]
                .iter()
                .find(|token| token.end <= span.end && token.kind == TokenKind::IdentifierLower)?;
            (!is_known_effect_surface_operation(operation)).then(|| {
                EffectFunctionIssue::BindValueNotEffect {
                    primary: token_span(operation),
                    bind: ByteSpan {
                        start: bind.start,
                        end: operation.end,
                    },
                }
            })
        })
        .collect()
}

pub(super) fn missing_result_issue(
    tokens: &[Token],
    span: ByteSpan,
) -> Option<EffectFunctionIssue> {
    let (left_brace_index, right_brace_index) = do_brace_indices(tokens, span)?;
    let contents = &tokens[left_brace_index + 1..right_brace_index];
    let significant = contents.iter().any(is_significant);
    let final_operation_index = contents.iter().rposition(is_known_effect_surface_operation);
    let final_operation_is_bound = final_operation_index.is_some_and(|operation_index| {
        let line_start = contents[..operation_index]
            .iter()
            .rposition(|token| token.kind == TokenKind::TriviaNewline)
            .map_or(0, |index| index + 1);
        contents[line_start..operation_index]
            .iter()
            .any(|token| token.kind == TokenKind::OperatorBind)
    });
    (!significant || final_operation_is_bound).then(|| EffectFunctionIssue::MissingDoResult {
        primary: ByteSpan {
            start: tokens[right_brace_index].start,
            end: tokens[right_brace_index].start,
        },
    })
}

pub(super) fn invalid_explicit_statement_issues(
    tokens: &[Token],
    span: ByteSpan,
) -> Vec<EffectFunctionIssue> {
    let Some((left_brace_index, right_brace_index)) = do_brace_indices(tokens, span) else {
        return Vec::new();
    };
    let mut issues = Vec::new();
    let mut line_start = left_brace_index + 1;
    for line_end in (left_brace_index + 1..=right_brace_index).filter(|index| {
        *index == right_brace_index || tokens[*index].kind == TokenKind::TriviaNewline
    }) {
        if let Some(issue) = invalid_explicit_line_issue(tokens, line_start, line_end) {
            issues.push(issue);
        }
        line_start = line_end + 1;
    }
    issues
}

fn invalid_explicit_line_issue(
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Option<EffectFunctionIssue> {
    let significant = tokens[start..end]
        .iter()
        .filter(|token| is_significant(token))
        .collect::<Vec<_>>();
    let first = significant.first().copied()?;
    if significant
        .iter()
        .any(|token| token.kind == TokenKind::OperatorBind)
        || is_known_effect_surface_operation(first)
        || first.kind != TokenKind::IdentifierLower
    {
        return None;
    }
    Some(EffectFunctionIssue::DoStatementNotEffect {
        primary: token_span(first),
    })
}

fn do_brace_indices(tokens: &[Token], span: ByteSpan) -> Option<(usize, usize)> {
    let do_index = tokens.iter().position(|token| {
        token.start >= span.start && token.end <= span.end && token.kind == TokenKind::KeywordDo
    })?;
    let left_brace_index = tokens[do_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "{")
        .map(|index| do_index + 1 + index)?;
    let right_brace_index = tokens[left_brace_index + 1..]
        .iter()
        .position(|token| token.end <= span.end && token.raw == "}")
        .map(|index| left_brace_index + 1 + index)?;
    Some((left_brace_index, right_brace_index))
}
