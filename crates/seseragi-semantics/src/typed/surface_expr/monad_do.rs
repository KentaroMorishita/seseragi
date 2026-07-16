use crate::{TypedConstraint, TypedExpr, TypedMonadDoStatement, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceDoItem, SurfaceExpr, SurfacePattern};
use std::collections::BTreeMap;

use super::{
    type_surface_expression, PureExpressionContext, SemanticTypeKey, SurfaceExpressionAnalysis,
};
use crate::typed::pure_issues::{MonadDoIssue, PureCallIssue};
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_monad_do(
    items: &[SurfaceDoItem],
    result: Option<&SurfaceExpr>,
    origin: ByteSpan,
    base_context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let Some(expected) = base_context.expected().cloned() else {
        return invalid_monad_do(
            origin,
            MonadDoIssue::ResultTypeNotMonadic {
                expression: origin,
                actual: TypedType::Hole,
            },
        );
    };
    let Some((constructor, _)) = monad_parts(&expected.type_ref) else {
        return invalid_monad_do(
            origin,
            MonadDoIssue::ResultTypeNotMonadic {
                expression: origin,
                actual: expected.type_ref,
            },
        );
    };
    let constraint = TypedConstraint {
        name: "Monad".to_owned(),
        arguments: vec![constructor.clone()],
    };
    let identity = base_context.trait_identity("Monad");
    let evidence = identity
        .as_ref()
        .and_then(|identity| {
            base_context
                .select_call_evidence(&[constraint.clone()], &[Some(identity.clone())])
                .ok()
        })
        .and_then(|mut evidence| (evidence.len() == 1).then(|| evidence.remove(0)));
    let Some(evidence) = evidence else {
        return invalid_do(
            origin,
            PureCallIssue::MissingInstance {
                callee: origin,
                constraint,
            },
        );
    };

    let mut locals = BTreeMap::new();
    let mut statements = Vec::new();
    let mut merged =
        SurfaceExpressionAnalysis::valid_with_semantic_type(hole(origin), expected.key.clone());

    for item in items {
        let context = base_context.with_locals(locals.clone()).without_expected();
        match item {
            SurfaceDoItem::Expression { value, .. } => {
                let analysis = type_surface_expression(value, &context);
                if let Some(issue) = monad_type_issue(
                    value.span(),
                    &constructor,
                    &inferred_type_from_expr(&analysis.value),
                ) {
                    merged.monad_do_issue = merged.monad_do_issue.take().or(Some(issue));
                }
                statements.push(TypedMonadDoStatement::Expression {
                    value: analysis.value.clone(),
                });
                merged.merge_issues_from(analysis);
            }
            SurfaceDoItem::Bind {
                pattern,
                value,
                span,
            } => {
                let analysis = type_surface_expression(value, &context);
                let actual = inferred_type_from_expr(&analysis.value);
                let payload = monad_payload(&constructor, &actual);
                if payload.is_none() {
                    merged.monad_do_issue = merged
                        .monad_do_issue
                        .take()
                        .or_else(|| monad_type_issue(value.span(), &constructor, &actual));
                }
                match (binding(pattern, base_context), payload) {
                    (Some((symbol, name)), Some(type_ref)) => {
                        locals.insert(
                            symbol,
                            base_context.semantic_value_from_typed_type(&type_ref),
                        );
                        statements.push(TypedMonadDoStatement::Bind {
                            name,
                            type_ref,
                            value: analysis.value.clone(),
                            origin: *span,
                        });
                    }
                    (None, Some(_)) if matches!(pattern, SurfacePattern::Wildcard { .. }) => {
                        statements.push(TypedMonadDoStatement::Expression {
                            value: analysis.value.clone(),
                        });
                    }
                    _ => {
                        let issue = binding_pattern_issue(pattern);
                        merged.monad_do_issue = merged.monad_do_issue.take().or(Some(issue));
                    }
                }
                merged.merge_issues_from(analysis);
            }
            SurfaceDoItem::Let {
                pattern,
                value,
                span,
            } => {
                let analysis = type_surface_expression(value, &context);
                let type_ref = inferred_type_from_expr(&analysis.value);
                if let Some((symbol, name)) = binding(pattern, base_context) {
                    locals.insert(
                        symbol,
                        base_context.semantic_value_from_typed_type(&type_ref),
                    );
                    statements.push(TypedMonadDoStatement::PureLet {
                        name,
                        type_ref,
                        value: analysis.value.clone(),
                        origin: *span,
                    });
                } else {
                    let issue = binding_pattern_issue(pattern);
                    merged.monad_do_issue = merged.monad_do_issue.take().or(Some(issue));
                }
                merged.merge_issues_from(analysis);
            }
        }
    }

    let Some(result) = result else {
        merged.monad_do_issue =
            merged
                .monad_do_issue
                .take()
                .or(Some(MonadDoIssue::MissingFinalExpression {
                    do_block: origin,
                }));
        return merged;
    };
    let result_context = base_context
        .with_locals(locals)
        .with_expected(Some(expected.clone()));
    let result_analysis = type_surface_expression(result, &result_context);
    let result_type = inferred_type_from_expr(&result_analysis.value);
    if let Some(issue) = monad_type_issue(result.span(), &constructor, &result_type) {
        merged.monad_do_issue = merged.monad_do_issue.take().or(Some(issue));
    }
    let result_value = result_analysis.value.clone();
    merged.merge_issues_from(result_analysis);
    merged.value = TypedExpr::MonadDo {
        statements,
        result: Box::new(result_value),
        evidence,
        type_ref: expected.type_ref,
        origin,
    };
    merged
}

fn binding(
    pattern: &SurfacePattern,
    context: &PureExpressionContext<'_>,
) -> Option<(crate::SymbolId, String)> {
    let SurfacePattern::Name {
        name, name_span, ..
    } = pattern
    else {
        return None;
    };
    let symbol = context.binding_symbol(*name_span)?;
    Some((symbol, name.clone()))
}

fn monad_type_issue(
    origin: ByteSpan,
    constructor: &TypedType,
    actual: &TypedType,
) -> Option<MonadDoIssue> {
    monad_payload(constructor, actual)
        .is_none()
        .then(|| MonadDoIssue::ConstructorMismatch {
            expression: origin,
            expected: apply_constructor(constructor, TypedType::Hole),
            actual: actual.clone(),
        })
}

fn binding_pattern_issue(pattern: &SurfacePattern) -> MonadDoIssue {
    match pattern {
        SurfacePattern::Integer { .. }
        | SurfacePattern::String { .. }
        | SurfacePattern::Boolean { .. }
        | SurfacePattern::Constructor { .. } => MonadDoIssue::RefutableBindPattern {
            pattern: pattern.span(),
        },
        SurfacePattern::Tuple { .. } | SurfacePattern::Error { .. } => {
            MonadDoIssue::UnsupportedBindPattern {
                pattern: pattern.span(),
            }
        }
        SurfacePattern::Name { .. } | SurfacePattern::Wildcard { .. } => {
            unreachable!("supported do binding patterns are handled before diagnostics")
        }
    }
}

fn monad_payload(constructor: &TypedType, applied: &TypedType) -> Option<TypedType> {
    let (actual_constructor, payload) = monad_parts(applied)?;
    (actual_constructor == *constructor).then_some(payload)
}

fn monad_parts(type_ref: &TypedType) -> Option<(TypedType, TypedType)> {
    match type_ref {
        TypedType::Named { name, arguments } if !arguments.is_empty() => Some((
            TypedType::Named {
                name: name.clone(),
                arguments: arguments[..arguments.len() - 1].to_vec(),
            },
            arguments.last()?.clone(),
        )),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } if !arguments.is_empty() => Some((
            TypedType::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: arguments[..arguments.len() - 1].to_vec(),
            },
            arguments.last()?.clone(),
        )),
        _ => None,
    }
}

fn apply_constructor(constructor: &TypedType, argument: TypedType) -> TypedType {
    match constructor {
        TypedType::Named { name, arguments } => {
            let mut applied = arguments.clone();
            applied.push(argument);
            TypedType::Named {
                name: name.clone(),
                arguments: applied,
            }
        }
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => {
            let mut applied = arguments.clone();
            applied.push(argument);
            TypedType::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: applied,
            }
        }
        _ => TypedType::Hole,
    }
}

fn invalid_do(origin: ByteSpan, issue: PureCallIssue) -> SurfaceExpressionAnalysis {
    let mut analysis =
        SurfaceExpressionAnalysis::valid_with_semantic_type(hole(origin), SemanticTypeKey::Invalid);
    analysis.pure_call_issue = Some(issue);
    analysis
}

fn invalid_monad_do(origin: ByteSpan, issue: MonadDoIssue) -> SurfaceExpressionAnalysis {
    let mut analysis =
        SurfaceExpressionAnalysis::valid_with_semantic_type(hole(origin), SemanticTypeKey::Invalid);
    analysis.monad_do_issue = Some(issue);
    analysis
}

fn hole(origin: ByteSpan) -> TypedExpr {
    TypedExpr::Variable {
        name: String::new(),
        evidence: Vec::new(),
        type_ref: TypedType::Hole,
        origin,
    }
}
