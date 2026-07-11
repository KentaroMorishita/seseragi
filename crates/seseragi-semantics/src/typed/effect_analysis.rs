use crate::{effect_ops::known_effect_operation_by_semantic, TypedDoStatement, TypedExpr};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceExpr, Token, TokenKind};

use super::effect_body::typed_effect_body;
use super::functions::typed_parameters_from_surface;
use super::TypedResolution;

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
    resolution: &TypedResolution<'_>,
) -> Vec<EffectFunctionIssue> {
    let SurfaceDecl::EffectFn {
        inferred_contract,
        parameters,
        requirements,
        failure,
        constraints,
        body,
        span,
        ..
    } = declaration
    else {
        return Vec::new();
    };
    let Some(surface_body) = body.as_ref() else {
        return Vec::new();
    };
    let typed_parameters = typed_parameters_from_surface(parameters);
    let typed_body = typed_effect_body(surface_body, &typed_parameters, resolution);

    let bind_issues = invalid_bind_issues(&typed_body);
    if !bind_issues.is_empty() {
        return bind_issues;
    }
    if let Some(issue) = missing_result_issue(surface_body) {
        return vec![issue];
    }
    if *inferred_contract
        && (!requirements.is_empty() || failure.is_some() || !constraints.is_empty())
    {
        let primary = compact_contract_clause(tokens, *span).unwrap_or(*span);
        return vec![EffectFunctionIssue::CompactContractClause { primary }];
    }

    let statement_issues = invalid_statement_issues(&typed_body, *inferred_contract);
    if !statement_issues.is_empty() {
        return statement_issues;
    }
    if !is_effect_expression(&typed_body) {
        let issue = if *inferred_contract {
            EffectFunctionIssue::CompactBodyNotEffect {
                primary: expression_origin(&typed_body),
            }
        } else {
            EffectFunctionIssue::DoStatementNotEffect {
                primary: expression_origin(&typed_body),
            }
        };
        return vec![issue];
    }
    if !inferred_contract {
        return Vec::new();
    }

    compact_failure_conflict(&typed_body).into_iter().collect()
}

fn invalid_bind_issues(body: &TypedExpr) -> Vec<EffectFunctionIssue> {
    let TypedExpr::DoBlock { statements, .. } = body else {
        return Vec::new();
    };
    statements
        .iter()
        .filter_map(|statement| {
            let TypedDoStatement::Bind { value, origin, .. } = statement else {
                return None;
            };
            (!is_effect_expression(value)).then(|| EffectFunctionIssue::BindValueNotEffect {
                primary: expression_origin(value),
                bind: *origin,
            })
        })
        .collect()
}

fn invalid_statement_issues(body: &TypedExpr, inferred_contract: bool) -> Vec<EffectFunctionIssue> {
    let TypedExpr::DoBlock {
        statements, result, ..
    } = body
    else {
        return Vec::new();
    };
    let mut issues = statements
        .iter()
        .filter_map(|statement| {
            let TypedDoStatement::Effect { value } = statement else {
                return None;
            };
            (!is_effect_expression(value)).then(|| EffectFunctionIssue::DoStatementNotEffect {
                primary: expression_origin(value),
            })
        })
        .collect::<Vec<_>>();
    if !is_effect_expression(result) {
        issues.push(if inferred_contract {
            EffectFunctionIssue::CompactBodyNotEffect {
                primary: expression_origin(result),
            }
        } else {
            EffectFunctionIssue::DoStatementNotEffect {
                primary: expression_origin(result),
            }
        });
    }
    issues
}

fn missing_result_issue(body: &SurfaceExpr) -> Option<EffectFunctionIssue> {
    let SurfaceExpr::Do {
        result: None, span, ..
    } = body
    else {
        return None;
    };
    let point = span.end.saturating_sub(1);
    Some(EffectFunctionIssue::MissingDoResult {
        primary: ByteSpan {
            start: point,
            end: point,
        },
    })
}

fn compact_failure_conflict(body: &TypedExpr) -> Option<EffectFunctionIssue> {
    let mut failures = Vec::new();
    collect_failures(body, &mut failures);
    let mut distinct = Vec::new();
    for failure in failures {
        if failure.failure_type == "Never"
            || distinct
                .iter()
                .any(|existing: &EffectFailureOrigin| existing.failure_type == failure.failure_type)
        {
            continue;
        }
        distinct.push(failure);
    }
    let primary = distinct.get(1)?.origin;
    Some(EffectFunctionIssue::CompactFailureConflict {
        primary,
        failures: distinct,
    })
}

fn collect_failures(expression: &TypedExpr, failures: &mut Vec<EffectFailureOrigin>) {
    match expression {
        TypedExpr::EffectCall {
            operation, origin, ..
        } => {
            if let Some(operation) = known_effect_operation_by_semantic(operation) {
                failures.push(EffectFailureOrigin {
                    failure_type: operation.failure_type.to_owned(),
                    origin: *origin,
                });
            }
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => {
                        collect_failures(value, failures);
                    }
                    TypedDoStatement::PureLet { .. } => {}
                }
            }
            collect_failures(result, failures);
        }
        TypedExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_failures(scrutinee, failures);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_failures(guard, failures);
                }
                collect_failures(&arm.body, failures);
            }
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. }
        | TypedExpr::Call { .. }
        | TypedExpr::Tuple { .. }
        | TypedExpr::Binary { .. }
        | TypedExpr::If { .. } => {}
    }
}

fn is_effect_expression(expression: &TypedExpr) -> bool {
    matches!(
        expression,
        TypedExpr::EffectCall { .. } | TypedExpr::DoBlock { .. }
    )
}

fn compact_contract_clause(tokens: &[Token], span: ByteSpan) -> Option<ByteSpan> {
    tokens
        .iter()
        .find(|token| {
            token.start >= span.start
                && token.end <= span.end
                && (matches!(token.kind, TokenKind::KeywordWith | TokenKind::KeywordFails)
                    || token.raw == "where")
        })
        .map(token_span)
}

fn expression_origin(expression: &TypedExpr) -> ByteSpan {
    match expression {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::Boolean { origin, .. }
        | TypedExpr::Variable { origin, .. }
        | TypedExpr::Call { origin, .. }
        | TypedExpr::Tuple { origin, .. }
        | TypedExpr::Binary { origin, .. }
        | TypedExpr::If { origin, .. }
        | TypedExpr::Match { origin, .. }
        | TypedExpr::EffectCall { origin, .. }
        | TypedExpr::DoBlock { origin, .. } => *origin,
    }
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
