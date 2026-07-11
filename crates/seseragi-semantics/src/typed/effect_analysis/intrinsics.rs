use crate::{TypedDoStatement, TypedExpr, TypedType};

use super::{expression_origin, EffectFunctionIssue};

pub(super) fn invalid_intrinsic_issues(body: &TypedExpr) -> Vec<EffectFunctionIssue> {
    let mut issues = Vec::new();
    collect_intrinsic_issues(body, &mut issues);
    issues
}

fn collect_intrinsic_issues(expression: &TypedExpr, issues: &mut Vec<EffectFunctionIssue>) {
    match expression {
        TypedExpr::EffectCall {
            operation,
            arguments,
            ..
        } => {
            if operation == "std/effect::mapError" {
                collect_map_error_issues(arguments, issues);
            }
            for argument in arguments {
                collect_intrinsic_issues(argument, issues);
            }
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                match statement {
                    TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => {
                        collect_intrinsic_issues(value, issues);
                    }
                    TypedDoStatement::PureLet { value, .. } => {
                        collect_intrinsic_issues(value, issues);
                    }
                }
            }
            collect_intrinsic_issues(result, issues);
        }
        TypedExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_intrinsic_issues(scrutinee, issues);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_intrinsic_issues(guard, issues);
                }
                collect_intrinsic_issues(&arm.body, issues);
            }
        }
        TypedExpr::Tuple { elements, .. }
        | TypedExpr::Call {
            arguments: elements,
            ..
        } => {
            for element in elements {
                collect_intrinsic_issues(element, issues);
            }
        }
        TypedExpr::Binary { left, right, .. } => {
            collect_intrinsic_issues(left, issues);
            collect_intrinsic_issues(right, issues);
        }
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            collect_intrinsic_issues(condition, issues);
            collect_intrinsic_issues(then_branch, issues);
            collect_intrinsic_issues(else_branch, issues);
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. } => {}
    }
}

fn collect_map_error_issues(arguments: &[TypedExpr], issues: &mut Vec<EffectFunctionIssue>) {
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
        TypedExpr::EffectCall { effect, .. } => effect,
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
        && *parameter != source_effect.failure
    {
        issues.push(EffectFunctionIssue::MapErrorFailureMismatch {
            primary: expression_origin(mapper),
            expected: source_effect.failure.clone(),
            actual: parameter.clone(),
        });
    }
}
