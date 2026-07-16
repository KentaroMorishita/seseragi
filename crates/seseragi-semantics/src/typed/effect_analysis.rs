use crate::{TypedDoStatement, TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceDecl, SurfaceExpr, Token, TokenKind};

use super::effect_body::analyze_effect_body;
use super::functions::typed_parameters_from_surface;
use super::pure_issues::{ArrayIssue, PureCallIssue, RangeIssue};
use super::TypedResolution;

mod contracts;
mod intrinsics;

use contracts::compact_failure_conflict;
use intrinsics::invalid_intrinsic_issues;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EffectFailureOrigin {
    pub(crate) failure_type: String,
    pub(crate) failure_identity: String,
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
    MapErrorMapperNotFunction {
        primary: ByteSpan,
        actual: TypedType,
    },
    MapErrorSourceNotEffect {
        primary: ByteSpan,
    },
    MapErrorFailureMismatch {
        primary: ByteSpan,
        expected: TypedType,
        actual: TypedType,
    },
    IntrinsicArityMismatch {
        primary: ByteSpan,
        expected: usize,
        actual: usize,
    },
    FromEitherSourceNotEither {
        primary: ByteSpan,
        actual: TypedType,
    },
    Call(PureCallIssue),
    Array(ArrayIssue),
    Range(RangeIssue),
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
    let body_analysis = analyze_effect_body(surface_body, &typed_parameters, resolution);
    let typed_body = body_analysis.value;

    if !body_analysis.array_issues.is_empty() {
        return body_analysis
            .array_issues
            .into_iter()
            .map(EffectFunctionIssue::Array)
            .collect();
    }
    if !body_analysis.range_issues.is_empty() {
        return body_analysis
            .range_issues
            .into_iter()
            .map(EffectFunctionIssue::Range)
            .collect();
    }

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

    if !body_analysis.call_issues.is_empty() {
        return body_analysis
            .call_issues
            .into_iter()
            .map(EffectFunctionIssue::Call)
            .collect();
    }

    let intrinsic_issues = invalid_intrinsic_issues(&typed_body, resolution);
    if !intrinsic_issues.is_empty() {
        return intrinsic_issues;
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

fn is_effect_expression(expression: &TypedExpr) -> bool {
    matches!(
        expression,
        TypedExpr::EffectCall { .. } | TypedExpr::EffectInvoke { .. } | TypedExpr::DoBlock { .. }
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

pub(super) fn expression_origin(expression: &TypedExpr) -> ByteSpan {
    match expression {
        TypedExpr::Unit { origin, .. }
        | TypedExpr::Integer { origin, .. }
        | TypedExpr::String { origin, .. }
        | TypedExpr::Boolean { origin, .. }
        | TypedExpr::Variable { origin, .. }
        | TypedExpr::Call { origin, .. }
        | TypedExpr::Tuple { origin, .. }
        | TypedExpr::Array { origin, .. }
        | TypedExpr::List { origin, .. }
        | TypedExpr::ArrayComprehension { origin, .. }
        | TypedExpr::ListComprehension { origin, .. }
        | TypedExpr::Binary { origin, .. }
        | TypedExpr::If { origin, .. }
        | TypedExpr::Match { origin, .. }
        | TypedExpr::EffectCall { origin, .. }
        | TypedExpr::EffectInvoke { origin, .. }
        | TypedExpr::DoBlock { origin, .. }
        | TypedExpr::MonadDo { origin, .. } => *origin,
    }
}

fn token_span(token: &Token) -> ByteSpan {
    ByteSpan {
        start: token.start,
        end: token.end,
    }
}
