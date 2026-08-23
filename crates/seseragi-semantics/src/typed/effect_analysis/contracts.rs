use crate::{TypedDoStatement, TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceRequirement, TypeRef};

use super::{
    EffectFailureOrigin, EffectFunctionIssue, ExplicitEnvironmentOrigin, ExplicitFailureOrigin,
};
use crate::typed::effect::explicit_environment;
use crate::typed::semantic_types::{
    semantic_values_have_same_identity, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::{application_argument_type_from_expr, effect_from_value_type};
use crate::typed::TypedResolution;

pub(super) fn explicit_success_mismatch(
    body: &TypedExpr,
    declared_success: Option<&TypeRef>,
    function: ByteSpan,
    resolution: &TypedResolution<'_>,
) -> Option<EffectFunctionIssue> {
    let declared = declared_success
        .map(|success| resolution.semantic_value_from_type_ref(success))
        .unwrap_or_else(|| resolution.semantic_value_from_typed_type(&named("Unit")));
    let effect = effect_from_value_type(&application_argument_type_from_expr(body))?;
    let actual = resolution.semantic_value_from_typed_type(&effect.success);
    if is_standard_never(&actual) || semantic_values_have_same_identity(&declared, &actual) {
        return None;
    }
    Some(EffectFunctionIssue::ExplicitSuccessMismatch {
        primary: declared_success.map(type_ref_span).unwrap_or(function),
        declared: declared.type_ref,
        actual: actual.type_ref,
        origin: result_origin(body),
    })
}

pub(super) fn explicit_environment_mismatch(
    body: &TypedExpr,
    declared_success: Option<&TypeRef>,
    requirements: &[SurfaceRequirement],
    function: ByteSpan,
    resolution: &TypedResolution<'_>,
) -> Option<EffectFunctionIssue> {
    let declared = explicit_environment(requirements, resolution);
    let mut operations = Vec::new();
    collect_explicit_environments(body, &mut operations);
    operations
        .retain(|operation| !environment_satisfies(&declared, &operation.environment, resolution));
    (!operations.is_empty()).then_some(EffectFunctionIssue::ExplicitEnvironmentMismatch {
        primary: requirements
            .first()
            .map(requirement_span)
            .or_else(|| declared_success.map(type_ref_span))
            .unwrap_or(function),
        declared,
        operations,
    })
}

fn requirement_span(requirement: &SurfaceRequirement) -> ByteSpan {
    match requirement {
        SurfaceRequirement::Shorthand { span, .. } | SurfaceRequirement::Field { span, .. } => {
            *span
        }
    }
}

fn environment_satisfies(
    declared: &TypedType,
    actual: &TypedType,
    resolution: &TypedResolution<'_>,
) -> bool {
    let (
        TypedType::Record {
            fields: declared_fields,
            ..
        },
        TypedType::Record {
            fields: actual_fields,
            ..
        },
    ) = (declared, actual)
    else {
        return false;
    };
    actual_fields.iter().all(|actual_field| {
        declared_fields.iter().any(|declared_field| {
            declared_field.name == actual_field.name
                && declared_field.optional == actual_field.optional
                && environment_field_types_have_same_identity(
                    &declared_field.type_ref,
                    &actual_field.type_ref,
                    resolution,
                )
        })
    })
}

fn environment_field_types_have_same_identity(
    declared: &TypedType,
    actual: &TypedType,
    resolution: &TypedResolution<'_>,
) -> bool {
    if semantic_values_have_same_identity(
        &resolution.semantic_value_from_typed_type(declared),
        &resolution.semantic_value_from_typed_type(actual),
    ) {
        return true;
    }

    // Shorthand requirements such as `with Clock` retain only their local
    // spelling, while a re-exported standard Effect contract carries the
    // canonical external identity (`std/clock::Clock`). Match that narrow
    // hydrated/unhydrated pair without weakening explicitly typed fields.
    matches!(
        (declared, actual),
        (
            TypedType::Named { name: declared_name, arguments: declared_arguments },
            TypedType::ExternalNamed { name: actual_name, canonical, arguments: actual_arguments },
        ) if declared_arguments.is_empty()
            && actual_arguments.is_empty()
            && declared_name == actual_name
            && canonical.ends_with(&format!("::{declared_name}"))
    )
}

fn collect_explicit_environments(
    expression: &TypedExpr,
    environments: &mut Vec<ExplicitEnvironmentOrigin>,
) {
    match expression {
        TypedExpr::EffectCall { effect, origin, .. }
        | TypedExpr::EffectInvoke { effect, origin, .. } => {
            environments.push(ExplicitEnvironmentOrigin {
                environment: effect.environment.clone(),
                origin: *origin,
            });
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => {
                        collect_explicit_environments(value, environments);
                    }
                    TypedDoStatement::PureLet { .. } => {}
                }
            }
            collect_explicit_environments(result, environments);
        }
        TypedExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_explicit_environments(then_branch, environments);
            collect_explicit_environments(else_branch, environments);
        }
        TypedExpr::Match { arms, .. } => {
            for arm in arms {
                collect_explicit_environments(&arm.body, environments);
            }
        }
        TypedExpr::Block { result, .. } => {
            collect_explicit_environments(result, environments);
        }
        _ => {
            if let Some(effect) =
                effect_from_value_type(&application_argument_type_from_expr(expression))
            {
                environments.push(ExplicitEnvironmentOrigin {
                    environment: effect.environment,
                    origin: super::expression_origin(expression),
                });
            }
        }
    }
}

fn result_origin(expression: &TypedExpr) -> ByteSpan {
    match expression {
        TypedExpr::DoBlock { result, .. } | TypedExpr::Block { result, .. } => {
            result_origin(result)
        }
        _ => super::expression_origin(expression),
    }
}

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
        | TypeRef::Function { span, .. }
        | TypeRef::RequirementMerge { span, .. } => *span,
    }
}

fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

pub(super) fn compact_failure_conflict(
    body: &TypedExpr,
    resolution: &TypedResolution<'_>,
) -> Option<EffectFunctionIssue> {
    let mut failures = Vec::new();
    collect_failures(body, &mut failures);
    let mut distinct: Vec<(SemanticValueType, EffectFailureOrigin)> = Vec::new();
    for failure in failures {
        let semantic = resolution.semantic_value_from_typed_type(&failure.failure);
        if is_standard_never(&semantic)
            || distinct
                .iter()
                .any(|(existing, _)| semantic_values_have_same_identity(existing, &semantic))
        {
            continue;
        }
        distinct.push((semantic, failure));
    }
    let primary = distinct.get(1)?.1.origin;
    Some(EffectFunctionIssue::CompactFailureConflict {
        primary,
        failures: distinct.into_iter().map(|(_, failure)| failure).collect(),
    })
}

fn collect_failures(expression: &TypedExpr, failures: &mut Vec<EffectFailureOrigin>) {
    match expression {
        TypedExpr::EffectCall { effect, origin, .. }
        | TypedExpr::EffectInvoke { effect, origin, .. } => {
            failures.push(EffectFailureOrigin {
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
                        collect_failures(value, failures);
                    }
                    TypedDoStatement::PureLet { .. } => {}
                }
            }
            collect_failures(result, failures);
        }
        TypedExpr::Block { result, .. } => collect_failures(result, failures),
        TypedExpr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_failures(then_branch, failures);
            collect_failures(else_branch, failures);
        }
        TypedExpr::Match { arms, .. } => {
            for arm in arms {
                collect_failures(&arm.body, failures);
            }
        }
        _ => {
            if let Some(effect) =
                effect_from_value_type(&application_argument_type_from_expr(expression))
            {
                failures.push(EffectFailureOrigin {
                    failure: effect.failure,
                    origin: super::expression_origin(expression),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::compact_failure_conflict;
    use crate::typed::TypedResolution;
    use crate::{TypedDoStatement, TypedEffect, TypedExpr, TypedType};
    use seseragi_syntax::ByteSpan;

    #[test]
    fn distinguishes_same_spelling_external_failures_by_canonical_owner() {
        let body = do_with_failures(&["fixture/first::SharedError", "fixture/second::SharedError"]);
        let resolved = crate::resolve_module("fixture/main.ssrg", "");
        let resolution = TypedResolution::new(&resolved);

        let issue =
            compact_failure_conflict(&body, &resolution).expect("distinct owners must conflict");
        let super::EffectFunctionIssue::CompactFailureConflict { failures, .. } = issue else {
            panic!("expected compact failure conflict");
        };
        assert_eq!(failures.len(), 2);
        assert_eq!(
            failures[0].failure,
            external_failure("fixture/first::SharedError")
        );
        assert_eq!(
            failures[1].failure,
            external_failure("fixture/second::SharedError")
        );
    }

    #[test]
    fn deduplicates_external_failures_from_the_same_canonical_owner() {
        let body =
            do_with_failures(&["fixture/shared::SharedError", "fixture/shared::SharedError"]);
        let resolved = crate::resolve_module("fixture/main.ssrg", "");
        let resolution = TypedResolution::new(&resolved);

        assert_eq!(compact_failure_conflict(&body, &resolution), None);
    }

    #[test]
    fn includes_parameterized_external_arguments_in_failure_identity() {
        let body = TypedExpr::DoBlock {
            statements: vec![
                TypedDoStatement::Effect {
                    value: effect_invoke(
                        parameterized_external_failure("Never"),
                        ByteSpan { start: 0, end: 1 },
                    ),
                },
                TypedDoStatement::Effect {
                    value: effect_invoke(
                        parameterized_external_failure("String"),
                        ByteSpan { start: 1, end: 2 },
                    ),
                },
            ],
            result: Box::new(effect_invoke(named("Never"), ByteSpan { start: 2, end: 3 })),
            origin: ByteSpan { start: 0, end: 3 },
        };
        let resolved = crate::resolve_module("fixture/main.ssrg", "");
        let resolution = TypedResolution::new(&resolved);

        let issue = compact_failure_conflict(&body, &resolution)
            .expect("different generic arguments must conflict");
        let super::EffectFunctionIssue::CompactFailureConflict { failures, .. } = issue else {
            panic!("expected compact failure conflict");
        };
        assert_eq!(failures.len(), 2);
    }

    #[test]
    fn deduplicates_the_same_parameterized_external_failure() {
        let failure = parameterized_external_failure("Never");
        let body = TypedExpr::DoBlock {
            statements: vec![TypedDoStatement::Effect {
                value: effect_invoke(failure.clone(), ByteSpan { start: 0, end: 1 }),
            }],
            result: Box::new(effect_invoke(failure, ByteSpan { start: 1, end: 2 })),
            origin: ByteSpan { start: 0, end: 2 },
        };
        let resolved = crate::resolve_module("fixture/main.ssrg", "");
        let resolution = TypedResolution::new(&resolved);

        assert_eq!(compact_failure_conflict(&body, &resolution), None);
    }

    #[test]
    fn collects_failures_from_branches_and_effect_typed_results() {
        let dom_error = external_failure("std/web/dom::DomError");
        let runtime_error = parameterized_external_failure("Never");
        let body = TypedExpr::DoBlock {
            statements: vec![TypedDoStatement::Effect {
                value: effect_invoke(dom_error, ByteSpan { start: 0, end: 1 }),
            }],
            result: Box::new(TypedExpr::If {
                condition: Box::new(TypedExpr::Boolean {
                    value: true,
                    type_ref: named("Bool"),
                    origin: ByteSpan { start: 1, end: 2 },
                }),
                then_branch: Box::new(effect_value(
                    runtime_error.clone(),
                    ByteSpan { start: 2, end: 3 },
                )),
                else_branch: Box::new(effect_value(
                    runtime_error.clone(),
                    ByteSpan { start: 3, end: 4 },
                )),
                type_ref: effect_type(runtime_error),
                origin: ByteSpan { start: 1, end: 4 },
            }),
            origin: ByteSpan { start: 0, end: 4 },
        };
        let resolved = crate::resolve_module("fixture/main.ssrg", "");
        let resolution = TypedResolution::new(&resolved);

        let issue = compact_failure_conflict(&body, &resolution)
            .expect("branch result failure must conflict with the statement failure");
        let super::EffectFunctionIssue::CompactFailureConflict { failures, .. } = issue else {
            panic!("expected compact failure conflict");
        };
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[1].origin, ByteSpan { start: 2, end: 3 });
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

    fn parameterized_external_failure(argument: &str) -> TypedType {
        TypedType::ExternalNamed {
            name: "DomRuntimeError".to_owned(),
            canonical: "std/web/dom::DomRuntimeError".to_owned(),
            arguments: vec![named(argument)],
        }
    }

    fn effect_value(failure: TypedType, origin: ByteSpan) -> TypedExpr {
        TypedExpr::Variable {
            name: "fixture::effect".to_owned(),
            evidence: Vec::new(),
            type_ref: effect_type(failure),
            origin,
        }
    }

    fn effect_type(failure: TypedType) -> TypedType {
        TypedType::Named {
            name: "Effect".to_owned(),
            arguments: vec![
                TypedType::Record {
                    closed: true,
                    fields: Vec::new(),
                },
                failure,
                named("Unit"),
            ],
        }
    }

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }
}
