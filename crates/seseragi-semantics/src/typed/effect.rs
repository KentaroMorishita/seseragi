use crate::{
    effect_ops::known_effect_operation_by_semantic, unit_type, TypedDoStatement, TypedEffect,
    TypedExpr, TypedRecordField, TypedType,
};
use seseragi_syntax::{ByteSpan, Token, TokenKind, TypeRef};

use super::expr::{find_type_name_after, lower_first};
use super::type_ref::{inferred_type_from_expr, typed_type_from_type_ref};

pub(crate) fn typed_effect_from_surface(
    return_type: &Option<TypeRef>,
    inferred_contract: bool,
    tokens: &[Token],
    span: ByteSpan,
    body: &TypedExpr,
) -> TypedEffect {
    if inferred_contract {
        return infer_compact_effect(body);
    }

    explicit_effect(return_type, tokens, span)
}

fn explicit_effect(return_type: &Option<TypeRef>, tokens: &[Token], span: ByteSpan) -> TypedEffect {
    let with_type = find_type_name_after(tokens, span, TokenKind::KeywordWith);
    let failure = find_type_name_after(tokens, span, TokenKind::KeywordFails)
        .unwrap_or_else(|| "Never".to_owned());
    let success = return_type
        .as_ref()
        .map(typed_type_from_type_ref)
        .unwrap_or_else(unit_type);

    TypedEffect {
        environment: TypedType::Record {
            closed: true,
            fields: with_type
                .map(explicit_environment_field)
                .into_iter()
                .collect::<Vec<_>>(),
        },
        failure: named_type(&failure),
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
        TypedExpr::EffectCall { operation, .. } => {
            let Some(operation) = known_effect_operation_by_semantic(operation) else {
                return;
            };
            if let Some((field_name, type_name)) = operation.requirement {
                push_requirement_unique(requirements, environment_field(field_name, type_name));
            }
            widen_failure_from_never(failure, named_type(operation.failure_type));
        }
        TypedExpr::DoBlock {
            statements, result, ..
        } => {
            for statement in statements {
                collect_effect_contract(do_statement_value(statement), requirements, failure);
            }
            collect_effect_contract(result, requirements, failure);
        }
        TypedExpr::Unit { .. }
        | TypedExpr::Integer { .. }
        | TypedExpr::String { .. }
        | TypedExpr::Boolean { .. }
        | TypedExpr::Variable { .. }
        | TypedExpr::Call { .. }
        | TypedExpr::Binary { .. }
        | TypedExpr::If { .. } => {}
    }
}

fn do_statement_value(statement: &TypedDoStatement) -> &TypedExpr {
    match statement {
        TypedDoStatement::Effect { value } | TypedDoStatement::Bind { value, .. } => value,
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

fn explicit_environment_field(name: String) -> TypedRecordField {
    environment_field(&lower_first(&name), &name)
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
