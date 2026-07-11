use crate::source_span;
use seseragi_semantics::{
    known_effect_operation_by_semantic, TypedDoStatement, TypedEffect, TypedExpr, TypedParameter,
};

use super::decision::lower_match;
use super::types::lower_typed_type;
use super::{CoreExpr, CoreParameter, CoreRecordField, CoreStatement, CoreType};

pub(super) fn lower_effect_body(source: &str, effect: TypedEffect, body: TypedExpr) -> CoreExpr {
    match body {
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
        } => {
            let (requirements, failure, success) = effect_operation_contract(&operation)
                .unwrap_or_else(|| {
                    (
                        lower_typed_type(effect.environment.clone()),
                        lower_typed_type(effect.failure.clone()),
                        lower_typed_type(effect.success.clone()),
                    )
                });
            CoreExpr::EffectOperation {
                operation: lower_effect_operation(&operation),
                requirements,
                failure,
                success,
                arguments: lower_exprs(source, arguments),
                origin: source_span(source, origin),
            }
        }
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => {
            let statements = statements
                .into_iter()
                .map(|statement| lower_effect_statement(source, effect.clone(), statement))
                .collect::<Vec<_>>();
            if statements.is_empty() {
                lower_expr(source, *result)
            } else {
                CoreExpr::Sequence {
                    statements,
                    result: Box::new(lower_expr(source, *result)),
                    origin: source_span(source, origin),
                }
            }
        }
        expr => lower_expr(source, expr),
    }
}

pub(super) fn lower_expr(source: &str, expr: TypedExpr) -> CoreExpr {
    match expr {
        TypedExpr::Unit { origin, .. } => CoreExpr::Unit {
            origin: source_span(source, origin),
        },
        TypedExpr::Integer { value, origin, .. } => CoreExpr::Int64 {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::String { value, origin, .. } => CoreExpr::String {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Boolean { value, origin, .. } => CoreExpr::Boolean {
            value,
            origin: source_span(source, origin),
        },
        TypedExpr::Variable {
            name,
            type_ref,
            origin,
        } => CoreExpr::Variable {
            name,
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Call {
            callee,
            arguments,
            type_ref,
            origin,
        } => CoreExpr::Call {
            callee,
            arguments: lower_exprs(source, arguments),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Tuple {
            elements,
            type_ref,
            origin,
        } => CoreExpr::Tuple {
            elements: lower_exprs(source, elements),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Binary {
            operator,
            left,
            right,
            type_ref,
            origin,
        } => CoreExpr::Binary {
            operator,
            left: Box::new(lower_expr(source, *left)),
            right: Box::new(lower_expr(source, *right)),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::If {
            condition,
            then_branch,
            else_branch,
            type_ref,
            origin,
        } => CoreExpr::If {
            condition: Box::new(lower_expr(source, *condition)),
            then_branch: Box::new(lower_expr(source, *then_branch)),
            else_branch: Box::new(lower_expr(source, *else_branch)),
            type_ref: lower_typed_type(type_ref),
            origin: source_span(source, origin),
        },
        TypedExpr::Match {
            scrutinee,
            arms,
            exhaustive,
            type_ref,
            origin,
        } => lower_match(source, *scrutinee, arms, exhaustive, type_ref, origin),
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
        } => {
            let (requirements, failure, success) = effect_operation_contract(&operation)
                .unwrap_or_else(|| {
                    (
                        empty_requirement_record(),
                        named_core_type("Never"),
                        named_core_type("Unit"),
                    )
                });
            CoreExpr::EffectOperation {
                operation: lower_effect_operation(&operation),
                requirements,
                failure,
                success,
                arguments: lower_exprs(source, arguments),
                origin: source_span(source, origin),
            }
        }
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => CoreExpr::Sequence {
            statements: statements
                .into_iter()
                .map(|statement| lower_expr_statement(source, statement))
                .collect(),
            result: Box::new(lower_expr(source, *result)),
            origin: source_span(source, origin),
        },
    }
}

pub(super) fn lower_parameter(parameter: &TypedParameter) -> CoreParameter {
    match parameter {
        TypedParameter::ImplicitUnit { type_ref } => CoreParameter {
            id: "unit".to_owned(),
            kind: "implicit".to_owned(),
            type_ref: lower_typed_type(type_ref.clone()),
        },
        TypedParameter::Named { name, type_ref, .. } => CoreParameter {
            id: name.clone(),
            kind: "named".to_owned(),
            type_ref: lower_typed_type(type_ref.clone()),
        },
    }
}

fn lower_exprs(source: &str, expressions: Vec<TypedExpr>) -> Vec<CoreExpr> {
    expressions
        .into_iter()
        .map(|expression| lower_expr(source, expression))
        .collect()
}

fn lower_effect_statement(
    source: &str,
    effect: TypedEffect,
    statement: TypedDoStatement,
) -> CoreStatement {
    match statement {
        TypedDoStatement::Effect { value } => CoreStatement::Effect {
            value: lower_effect_body(source, effect, value),
        },
        TypedDoStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::Bind {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_effect_body(source, effect, value),
            origin: source_span(source, origin),
        },
    }
}

fn lower_expr_statement(source: &str, statement: TypedDoStatement) -> CoreStatement {
    match statement {
        TypedDoStatement::Effect { value } => CoreStatement::Effect {
            value: lower_expr(source, value),
        },
        TypedDoStatement::Bind {
            name,
            type_ref,
            value,
            origin,
        } => CoreStatement::Bind {
            name,
            type_ref: lower_typed_type(type_ref),
            value: lower_expr(source, value),
            origin: source_span(source, origin),
        },
    }
}

fn lower_effect_operation(operation: &str) -> String {
    match operation {
        "std/prelude::readLine" => "stdin.readLine".to_owned(),
        "std/prelude::print" => "console.print".to_owned(),
        "std/prelude::println" => "console.println".to_owned(),
        "std/effect::succeed" => "effect.succeed".to_owned(),
        other => other.to_owned(),
    }
}

fn effect_operation_contract(operation: &str) -> Option<(CoreType, CoreType, CoreType)> {
    let operation = known_effect_operation_by_semantic(operation)?;
    Some((
        CoreType::Record {
            closed: true,
            fields: operation
                .requirement
                .map(|(field_name, type_name)| CoreRecordField {
                    name: field_name.to_owned(),
                    optional: false,
                    type_ref: named_core_type(type_name),
                })
                .into_iter()
                .collect(),
        },
        named_core_type(operation.failure_type),
        CoreType::Named {
            name: operation.success_type.to_owned(),
            arguments: operation
                .success_type_arguments
                .iter()
                .map(|name| named_core_type(name))
                .collect(),
        },
    ))
}

fn named_core_type(name: &str) -> CoreType {
    CoreType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}

fn empty_requirement_record() -> CoreType {
    CoreType::Record {
        closed: true,
        fields: Vec::new(),
    }
}
