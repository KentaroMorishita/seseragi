use crate::{TypedDoStatement, TypedExpr, TypedType};

use super::{EffectFailureOrigin, EffectFunctionIssue};

pub(super) fn compact_failure_conflict(body: &TypedExpr) -> Option<EffectFunctionIssue> {
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
        TypedExpr::EffectCall { effect, origin, .. } => {
            if let TypedType::Named { name, arguments } = &effect.failure {
                if arguments.is_empty() {
                    failures.push(EffectFailureOrigin {
                        failure_type: name.clone(),
                        origin: *origin,
                    });
                }
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
