use crate::{TypedDoStatement, TypedExpr, TypedType};

use super::{EffectFailureOrigin, EffectFunctionIssue};

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
