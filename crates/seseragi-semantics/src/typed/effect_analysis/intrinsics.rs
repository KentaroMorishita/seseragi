use crate::{TypedDoStatement, TypedExpr, TypedType};

use super::super::{semantic_types::semantic_values_are_compatible, TypedResolution};
use super::{expression_origin, EffectFunctionIssue};

pub(super) fn invalid_intrinsic_issues(
    body: &TypedExpr,
    resolution: &TypedResolution<'_>,
) -> Vec<EffectFunctionIssue> {
    let mut issues = Vec::new();
    collect_intrinsic_issues(body, resolution, &mut issues);
    issues
}

fn collect_intrinsic_issues(
    expression: &TypedExpr,
    resolution: &TypedResolution<'_>,
    issues: &mut Vec<EffectFunctionIssue>,
) {
    match expression {
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
            ..
        } => {
            if operation == "std/effect::mapError" {
                collect_map_error_issues(arguments, resolution, issues);
            } else if operation == "std/effect::fromEither" {
                collect_from_either_issues(arguments, *origin, issues);
            }
            for argument in arguments {
                collect_intrinsic_issues(argument, resolution, issues);
            }
        }
        TypedExpr::EffectInvoke { arguments, .. } => {
            for argument in arguments {
                collect_intrinsic_issues(argument, resolution, issues);
            }
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => {
                        collect_intrinsic_issues(value, resolution, issues);
                    }
                    TypedDoStatement::PureLet { value, .. } => {
                        collect_intrinsic_issues(value, resolution, issues);
                    }
                }
            }
            collect_intrinsic_issues(result, resolution, issues);
        }
        TypedExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_intrinsic_issues(scrutinee, resolution, issues);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_intrinsic_issues(guard, resolution, issues);
                }
                collect_intrinsic_issues(&arm.body, resolution, issues);
            }
        }
        TypedExpr::Tuple { elements, .. }
        | TypedExpr::Array { elements, .. }
        | TypedExpr::Call {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_intrinsic_issues(element, resolution, issues);
            }
        }
        TypedExpr::Binary { left, right, .. } => {
            collect_intrinsic_issues(left, resolution, issues);
            collect_intrinsic_issues(right, resolution, issues);
        }
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_intrinsic_issues(condition, resolution, issues);
            collect_intrinsic_issues(then_branch, resolution, issues);
            collect_intrinsic_issues(else_branch, resolution, issues);
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. } => {}
    }
}

fn collect_from_either_issues(
    arguments: &[TypedExpr],
    origin: seseragi_syntax::ByteSpan,
    issues: &mut Vec<EffectFunctionIssue>,
) {
    let [source] = arguments else {
        issues.push(EffectFunctionIssue::IntrinsicArityMismatch {
            primary: origin,
            expected: 1,
            actual: arguments.len(),
        });
        return;
    };
    let actual = super::super::type_ref::inferred_type_from_expr(source);
    if matches!(actual, TypedType::Hole) {
        return;
    }
    if !matches!(
        &actual,
        TypedType::Named { name, arguments } if name == "Either" && arguments.len() == 2
    ) {
        issues.push(EffectFunctionIssue::FromEitherSourceNotEither {
            primary: expression_origin(source),
            actual,
        });
    }
}

fn collect_map_error_issues(
    arguments: &[TypedExpr],
    resolution: &TypedResolution<'_>,
    issues: &mut Vec<EffectFunctionIssue>,
) {
    let [mapper, source] = arguments else {
        return;
    };
    let mapper_type = super::super::type_ref::inferred_type_from_expr(mapper);
    let parameter = match &mapper_type {
        TypedType::Function { parameter, .. } => parameter.as_ref(),
        TypedType::Hole => return,
        actual => {
            issues.push(EffectFunctionIssue::MapErrorMapperNotFunction {
                primary: expression_origin(mapper),
                actual: actual.clone(),
            });
            return;
        }
    };
    let source_effect = match source {
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => effect,
        _ if matches!(
            super::super::type_ref::inferred_type_from_expr(source),
            TypedType::Hole
        ) =>
        {
            return
        }
        _ => {
            issues.push(EffectFunctionIssue::MapErrorSourceNotEffect {
                primary: expression_origin(source),
            });
            return;
        }
    };
    if !super::super::type_ref::typed_type_contains_hole(parameter)
        && !super::super::type_ref::typed_type_contains_hole(&source_effect.failure)
        && !semantic_values_are_compatible(
            &resolution.semantic_value_from_typed_type(&source_effect.failure),
            &resolution.semantic_value_from_typed_type(parameter),
        )
    {
        issues.push(EffectFunctionIssue::MapErrorFailureMismatch {
            primary: expression_origin(mapper),
            expected: source_effect.failure.clone(),
            actual: parameter.clone(),
        });
    }
}
