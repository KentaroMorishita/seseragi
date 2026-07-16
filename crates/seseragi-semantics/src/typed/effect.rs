use crate::{unit_type, TypedDoStatement, TypedEffect, TypedExpr, TypedRecordField, TypedType};
use seseragi_syntax::{SurfaceRequirement, TypeRef};

use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

pub(crate) fn typed_effect_from_surface(
    return_type: &Option<TypeRef>,
    requirements: &[SurfaceRequirement],
    failure: Option<&TypeRef>,
    inferred_contract: bool,
    body: &TypedExpr,
) -> TypedEffect {
    if inferred_contract {
        return infer_compact_effect(body);
    }

    explicit_effect(return_type, requirements, failure)
}

fn explicit_effect(
    return_type: &Option<TypeRef>,
    requirements: &[SurfaceRequirement],
    failure: Option<&TypeRef>,
) -> TypedEffect {
    let success = return_type
        .as_ref()
        .map(typed_type_from_type_ref)
        .unwrap_or_else(unit_type);

    TypedEffect {
        environment: TypedType::Record {
            closed: true,
            fields: requirements
                .iter()
                .map(explicit_environment_field)
                .collect(),
        },
        failure: failure
            .map(typed_type_from_type_ref)
            .unwrap_or_else(|| named_type("Never")),
        success,
    }
}

fn infer_compact_effect(body: &TypedExpr) -> TypedEffect {
    let mut requirements = Vec::new();
    let mut failure = named_type("Never");

    collect_effect_contract(body, &mut requirements, &mut failure);

    TypedEffect {
        environment: TypedType::Record {
            closed: true,
            fields: requirements,
        },
        failure,
        success: inferred_type_from_expr(body),
    }
}

fn collect_effect_contract(
    expr: &TypedExpr,
    requirements: &mut Vec<TypedRecordField>,
    failure: &mut TypedType,
) {
    match expr {
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => {
            if let TypedType::Record { fields, .. } = &effect.environment {
                for field in fields {
                    push_requirement_unique(requirements, field.clone());
                }
            }
            widen_failure_from_never(failure, effect.failure.clone());
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                if let Some(value) = do_statement_effect(statement) {
                    collect_effect_contract(value, requirements, failure);
                }
            }
            collect_effect_contract(result, requirements, failure);
        }
        TypedExpr::Match {
            scrutinee, arms, ..
        } => {
            collect_effect_contract(scrutinee, requirements, failure);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_effect_contract(guard, requirements, failure);
                }
                collect_effect_contract(&arm.body, requirements, failure);
            }
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. }
        | TypedExpr::Call { .. }
        | TypedExpr::Tuple { .. }
        | TypedExpr::Array { .. }
        | TypedExpr::ArrayComprehension { .. }
        | TypedExpr::Binary { .. }
        | TypedExpr::If { .. }
        | TypedExpr::MonadDo { .. } => {}
    }
}

fn do_statement_effect(statement: &TypedDoStatement) -> Option<&TypedExpr> {
    match statement {
        TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => Some(value),
        TypedDoStatement::PureLet { .. } => None,
    }
}

fn push_requirement_unique(fields: &mut Vec<TypedRecordField>, candidate: TypedRecordField) {
    if fields.iter().any(|field| field.name == candidate.name) {
        return;
    }
    fields.push(candidate);
}

fn widen_failure_from_never(current: &mut TypedType, candidate: TypedType) {
    if named_type_is(current, "Never") {
        *current = candidate;
    }
}

fn explicit_environment_field(requirement: &SurfaceRequirement) -> TypedRecordField {
    match requirement {
        SurfaceRequirement::Shorthand { name, .. } => environment_field(&lower_first(name), name),
        SurfaceRequirement::Field { name, type_ref, .. } => TypedRecordField {
            name: name.clone(),
            optional: false,
            type_ref: typed_type_from_type_ref(type_ref),
        },
    }
}

fn environment_field(field_name: &str, type_name: &str) -> TypedRecordField {
    TypedRecordField {
        name: field_name.to_owned(),
        optional: false,
        type_ref: named_type(type_name),
    }
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn named_type_is(type_ref: &TypedType, expected_name: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, .. } if name == expected_name)
}

fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}
