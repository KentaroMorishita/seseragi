use crate::effect_ops::known_effect_operation_by_surface;
use seseragi_syntax::{ByteSpan, SurfaceDecl, Token, TokenKind};

mod do_block;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectFailureOrigin {
    pub(crate) failure_type: String,
    pub(crate) origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EffectFunctionIssue {
    CompactContractClause {
        primary: ByteSpan,
    },
    MissingDoResult {
        primary: ByteSpan,
    },
    CompactFailureConflict {
        primary: ByteSpan,
        failures: Vec<EffectFailureOrigin>,
    },
    DoStatementNotEffect {
        primary: ByteSpan,
    },
    BindValueNotEffect {
        primary: ByteSpan,
        bind: ByteSpan,
    },
    CompactBodyNotEffect {
        primary: ByteSpan,
    },
}

pub(crate) fn analyze_effect_function(
    declaration: &SurfaceDecl,
    tokens: &[Token],
) -> Vec<EffectFunctionIssue> {
    let SurfaceDecl::EffectFn {
        inferred_contract,
        span,
        ..
    } = declaration
    else {
        return Vec::new();
    };

    let bind_issues = do_block::invalid_bind_issues(tokens, *span);
    if !bind_issues.is_empty() {
        return bind_issues;
    }
    if let Some(issue) = do_block::missing_result_issue(tokens, *span) {
        return vec![issue];
    }
    if !inferred_contract {
        return do_block::invalid_explicit_statement_issues(tokens, *span);
    }
    if let Some(clause) = compact_contract_clause(tokens, *span) {
        return vec![EffectFunctionIssue::CompactContractClause {
            primary: token_span(clause),
        }];
    }

    let Some(operation) = compact_effect_body_operation(tokens, *span) else {
        return Vec::new();
    };
    if is_known_effect_surface_operation(operation) {
        return compact_failure_conflict(tokens, *span)
            .into_iter()
            .collect();
    }
    if operation.kind == TokenKind::KeywordDo {
        return match compact_do_unknown_statement_operation(tokens, *span, operation) {
            Some(statement) => vec![EffectFunctionIssue::CompactBodyNotEffect {
                primary: token_span(statement),
            }],
            None => compact_failure_conflict(tokens, *span)
                .into_iter()
                .collect(),
        };
    }
    vec![EffectFunctionIssue::CompactBodyNotEffect {
        primary: token_span(operation),
    }]
}

fn compact_failure_conflict(tokens: &[Token], span: ByteSpan) -> Option<EffectFunctionIssue> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    let mut failures = Vec::new();
    for token in tokens[equals_index + 1..]
        .iter()
        .take_while(|token| token.end <= span.end)
    {
        let Some(operation) = known_effect_operation_by_surface(token.raw.as_str()) else {
            continue;
        };
        if operation.failure_type == "Never"
            || failures
                .iter()
                .any(|failure: &EffectFailureOrigin| failure.failure_type == operation.failure_type)
        {
            continue;
        }
        failures.push(EffectFailureOrigin {
            failure_type: operation.failure_type.to_owned(),
            origin: token_span(token),
        });
    }
    let primary = failures.get(1)?.origin;
    Some(EffectFunctionIssue::CompactFailureConflict { primary, failures })
}

fn compact_effect_body_operation(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens[equals_index + 1..]
        .iter()
        .find(|token| token.end <= span.end && is_significant(token))
}

fn compact_contract_clause(tokens: &[Token], span: ByteSpan) -> Option<&Token> {
    let equals_index = tokens
        .iter()
        .position(|token| token.start >= span.start && token.end <= span.end && token.raw == "=")?;
    tokens.iter().take(equals_index).find(|token| {
        token.start >= span.start
            && (matches!(token.kind, TokenKind::KeywordWith | TokenKind::KeywordFails)
                || token.raw == "where")
    })
}

fn compact_do_unknown_statement_operation<'tokens>(
    tokens: &'tokens [Token],
    span: ByteSpan,
    do_token: &Token,
) -> Option<&'tokens Token> {
    let left_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "{")?;
    let right_brace = tokens
        .iter()
        .skip_while(|token| token.start <= do_token.start)
        .find(|token| token.end <= span.end && token.raw == "}")?;
    tokens
        .iter()
        .skip_while(|token| token.start <= left_brace.start)
        .find(|token| {
            token.end <= right_brace.start
                && token.kind == TokenKind::IdentifierLower
                && !is_known_effect_surface_operation(token)
                && !is_do_bind_target(tokens, token)
        })
}

fn is_do_bind_target(tokens: &[Token], candidate: &Token) -> bool {
    tokens
        .iter()
        .skip_while(|token| token.start <= candidate.start)
        .find(|token| is_significant(token))
        .is_some_and(|token| token.kind == TokenKind::OperatorBind)
}

pub(super) fn is_known_effect_surface_operation(token: &Token) -> bool {
    known_effect_operation_by_surface(token.raw.as_str()).is_some()
}

pub(super) fn is_significant(token: &Token) -> bool {
    !matches!(
        token.kind,
        TokenKind::TriviaComment
            | TokenKind::TriviaNewline
            | TokenKind::TriviaSpace
            | TokenKind::Eof
    )
}

pub(super) fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
