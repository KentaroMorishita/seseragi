use crate::{TypedDoStatement, TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, TypeRef};

use super::{EffectFailureOrigin, EffectFunctionIssue, ExplicitFailureOrigin};
use crate::typed::semantic_types::{
    semantic_values_have_same_identity, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{application_argument_type_from_expr, effect_from_value_type};
use crate::typed::TypedResolution;

pub(super) fn explicit_failure_mismatch(
    body: &TypedExpr,
    declared_failure: Option<&TypeRef>,
    resolution: &TypedResolution<'_>,
) -> Option<EffectFunctionIssue> {
    let declared = declared_failure
        .map(|failure| resolution.semantic_value_from_type_ref(failure))
        .unwrap_or_else(|| resolution.semantic_value_from_typed_type(&named("Never")));
    let mut failures = Vec::new();
    collect_explicit_failures(body, &mut failures);
    failures.retain(|failure| {
        let actual = resolution.semantic_value_from_typed_type(&failure.failure);
        !is_standard_never(&actual) && !semantic_values_have_same_identity(&declared, &actual)
    });
    let first = failures.first()?;
    Some(EffectFunctionIssue::ExplicitFailureMismatch {
        primary: declared_failure.map(type_ref_span).unwrap_or(first.origin),
        declared: declared.type_ref,
        failures,
    })
}

fn collect_explicit_failures(expression: &TypedExpr, failures: &mut Vec<ExplicitFailureOrigin>) {
    match expression {
        TypedExpr::EffectCall { effect, origin, .. }
        | TypedExpr::EffectInvoke { effect, origin, .. } => {
            failures.push(ExplicitFailureOrigin {
                failure: effect.failure.clone(),
                origin: *origin,
            });
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => {
                        collect_explicit_failures(value, failures);
                    }
                    TypedDoStatement::PureLet { .. } => {}
                }
            }
            collect_explicit_failures(result, failures);
        }
        TypedExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_explicit_failures(then_branch, failures);
            collect_explicit_failures(else_branch, failures);
        }
        TypedExpr::Match { arms, .. } => {
            for arm in arms {
                collect_explicit_failures(&arm.body, failures);
            }
        }
        TypedExpr::Block { result, .. } => {
            collect_explicit_failures(result, failures);
        }
        _ => {
            if let Some(effect) =
                effect_from_value_type(&application_argument_type_from_expr(expression))
            {
                failures.push(ExplicitFailureOrigin {
                    failure: effect.failure,
                    origin: super::expression_origin(expression),
                });
            }
        }
    }
}

fn is_standard_never(value: &SemanticValueType) -> bool {
    matches!(value.key, SemanticTypeKey::Other)
        && matches!(
            &value.type_ref,
            TypedType::Named { name, arguments }
                if name == "Never" && arguments.is_empty()
        )
}

fn type_ref_span(type_ref: &TypeRef) -> ByteSpan {
    match type_ref {
        TypeRef::Named { span, .. }
        | TypeRef::Hole { span }
        | TypeRef::Record { span, .. }
        | TypeRef::Tuple { span, .. }
        | TypeRef::Function { span, .. } => *span,
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

pub(super) fn compact_failure_conflict(body: &TypedExpr) -> Option<EffectFunctionIssue> {
    let mut failures = Vec::new();
    collect_failures(body, &mut failures);
    let mut distinct = Vec::new();
    for failure in failures {
        if failure.failure_type == "Never"
            || distinct.iter().any(|existing: &EffectFailureOrigin| {
                existing.failure_identity == failure.failure_identity
            })
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
        TypedExpr::EffectCall { effect, origin, .. }
        | TypedExpr::EffectInvoke { effect, origin, .. } => {
            let failure = match &effect.failure {
                TypedType::Named { name, arguments } if arguments.is_empty() => {
                    Some((name.clone(), name.clone()))
                }
                TypedType::ExternalNamed {
                    name,
                    canonical,
                    arguments,
                } if arguments.is_empty() => Some((name.clone(), canonical.clone())),
                _ => None,
            };
            if let Some((failure_type, failure_identity)) = failure {
                failures.push(EffectFailureOrigin {
                    failure_type,
                    failure_identity,
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
        TypedExpr::Block {
            statements, result, ..
        } => {
            for statement in statements {
                let value = match statement {
                    crate::TypedBlockStatement::Let { value, .. } => value,
                    crate::TypedBlockStatement::Function { body, .. } => body,
                };
                collect_failures(value, failures);
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
        TypedExpr::FieldAccess { receiver, .. }
        | TypedExpr::OptionalFieldAccess { receiver, .. }
        | TypedExpr::Unary {
            operand: receiver, ..
        } => collect_failures(receiver, failures),
        TypedExpr::Lambda { body, .. } => collect_failures(body, failures),
        TypedExpr::Record { items, .. } => {
            for item in items {
                collect_failures(item.value(), failures);
            }
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::Float { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Template { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. }
        | TypedExpr::Call { .. }
        | TypedExpr::Tuple { .. }
        | TypedExpr::Array { .. }
        | TypedExpr::List { .. }
        | TypedExpr::ArrayComprehension { .. }
        | TypedExpr::ListComprehension { .. }
        | TypedExpr::Binary { .. }
        | TypedExpr::If { .. }
        | TypedExpr::MonadDo { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::compact_failure_conflict;
    use crate::{TypedDoStatement, TypedEffect, TypedExpr, TypedType};
    use seseragi_syntax::ByteSpan;

    #[test]
    fn distinguishes_same_spelling_external_failures_by_canonical_owner() {
        let body = do_with_failures(&["fixture/first::SharedError", "fixture/second::SharedError"]);

        let issue = compact_failure_conflict(&body).expect("distinct owners must conflict");
        let super::EffectFunctionIssue::CompactFailureConflict { failures, .. } = issue else {
            panic!("expected compact failure conflict");
        };
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].failure_type, "SharedError");
        assert_eq!(failures[1].failure_type, "SharedError");
    }

    #[test]
    fn deduplicates_external_failures_from_the_same_canonical_owner() {
        let body =
            do_with_failures(&["fixture/shared::SharedError", "fixture/shared::SharedError"]);

        assert_eq!(compact_failure_conflict(&body), None);
    }

    fn do_with_failures(canonicals: &[&str]) -> TypedExpr {
        TypedExpr::DoBlock {
            statements: canonicals
                .iter()
                .enumerate()
                .map(|(index, canonical)| TypedDoStatement::Effect {
                    value: effect_invoke(
                        external_failure(canonical),
                        ByteSpan {
                            start: index,
                            end: index + 1,
                        },
                    ),
                })
                .collect(),
            result: Box::new(effect_invoke(
                named("Never"),
                ByteSpan {
                    start: canonicals.len(),
                    end: canonicals.len() + 1,
                },
            )),
            origin: ByteSpan {
                start: 0,
                end: canonicals.len() + 1,
            },
        }
    }

    fn effect_invoke(failure: TypedType, origin: ByteSpan) -> TypedExpr {
        TypedExpr::EffectInvoke {
            callee: "fixture::operation".to_owned(),
            effect: TypedEffect {
                environment: TypedType::Record {
                    closed: true,
                    fields: Vec::new(),
                },
                failure,
                success: named("Unit"),
            },
            arguments: Vec::new(),
            evidence: Vec::new(),
            origin,
        }
    }

    fn external_failure(canonical: &str) -> TypedType {
        TypedType::ExternalNamed {
            name: "SharedError".to_owned(),
            canonical: canonical.to_owned(),
            arguments: Vec::new(),
        }
    }

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
