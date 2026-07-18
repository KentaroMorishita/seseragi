use crate::{unit_type, KnownEffectOperation, TypedEffect, TypedExpr, TypedRecordField, TypedType};

use super::super::type_ref::{
    application_argument_type_from_expr, effect_from_value_type, inferred_type_from_expr,
};

pub(super) fn operation_effect(
    operation: KnownEffectOperation,
    arguments: &[TypedExpr],
) -> TypedEffect {
    match operation.surface_name {
        "mapError" => map_error_effect(arguments),
        "fromEither" => from_either_effect(arguments),
        _ => standard_operation_effect(operation, arguments),
    }
}

fn standard_operation_effect(
    operation: KnownEffectOperation,
    arguments: &[TypedExpr],
) -> TypedEffect {
    let argument_type = || {
        arguments
            .first()
            .map(inferred_type_from_expr)
            .unwrap_or(TypedType::Hole)
    };
    let success = match operation.surface_name {
        "succeed" => arguments
            .first()
            .map(inferred_type_from_expr)
            .unwrap_or_else(unit_type),
        _ => operation_success_type(operation),
    };
    TypedEffect {
        environment: empty_environment_with(
            operation
                .requirement
                .map(|(name, type_name)| TypedRecordField {
                    name: name.to_owned(),
                    optional: false,
                    type_ref: named_type(type_name),
                })
                .into_iter()
                .collect(),
        ),
        failure: if operation.surface_name == "fail" {
            argument_type()
        } else {
            named_type(operation.failure_type)
        },
        success,
    }
}

fn map_error_effect(arguments: &[TypedExpr]) -> TypedEffect {
    let source = arguments.get(1).and_then(expression_effect);
    let mapped_failure = arguments
        .first()
        .map(inferred_type_from_expr)
        .and_then(|type_ref| match type_ref {
            TypedType::Function { result, .. } => Some(*result),
            _ => None,
        })
        .unwrap_or(TypedType::Hole);
    TypedEffect {
        environment: source
            .as_ref()
            .map(|effect| effect.environment.clone())
            .unwrap_or(TypedType::Hole),
        failure: mapped_failure,
        success: source
            .as_ref()
            .map(|effect| effect.success.clone())
            .unwrap_or(TypedType::Hole),
    }
}

fn from_either_effect(arguments: &[TypedExpr]) -> TypedEffect {
    let (failure, success) = arguments
        .first()
        .map(inferred_type_from_expr)
        .and_then(either_arguments)
        .unwrap_or((TypedType::Hole, TypedType::Hole));
    TypedEffect {
        environment: empty_environment_with(Vec::new()),
        failure,
        success,
    }
}

fn either_arguments(type_ref: TypedType) -> Option<(TypedType, TypedType)> {
    let TypedType::Named { name, arguments } = type_ref else {
        return None;
    };
    let [failure, success] = arguments.as_slice() else {
        return None;
    };
    (name == "Either").then(|| (failure.clone(), success.clone()))
}

fn expression_effect(expression: &TypedExpr) -> Option<TypedEffect> {
    match expression {
        TypedExpr::EffectCall { effect, .. } | TypedExpr::EffectInvoke { effect, .. } => {
            Some(effect.clone())
        }
        _ => effect_from_value_type(&application_argument_type_from_expr(expression)),
    }
}

fn operation_success_type(operation: KnownEffectOperation) -> TypedType {
    TypedType::Named {
        name: operation.success_type.to_owned(),
        arguments: operation
            .success_type_arguments
            .iter()
            .map(|name| named_type(name))
            .collect(),
    }
}

fn empty_environment_with(fields: Vec<TypedRecordField>) -> TypedType {
    TypedType::Record {
        closed: true,
        fields,
    }
}

fn named_type(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
