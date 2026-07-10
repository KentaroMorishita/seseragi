use crate::{source_span, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{
    known_effect_operation_by_semantic, TypedDecl, TypedDoStatement, TypedEffect, TypedExpr,
    TypedModule, TypedParameter,
};
use seseragi_syntax::Visibility;

mod types;

use types::lower_typed_type;
pub use types::{CoreRecordField, CoreType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<CoreBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<CoreFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreBinding {
    pub symbol: String,
    pub visibility: Visibility,
    pub origin: SourceSpan,
    pub value: CoreExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreFunction {
    pub symbol: String,
    pub visibility: Visibility,
    pub origin: SourceSpan,
    pub parameters: Vec<CoreParameter>,
    pub body: CoreExpr,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreParameter {
    pub id: String,
    pub kind: String,
    #[serde(rename = "type")]
    pub type_ref: CoreType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreExpr {
    Unit {
        origin: SourceSpan,
    },
    Int64 {
        value: String,
        origin: SourceSpan,
    },
    String {
        value: String,
        origin: SourceSpan,
    },
    Boolean {
        value: bool,
        origin: SourceSpan,
    },
    Variable {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Call {
        callee: String,
        arguments: Vec<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Binary {
        operator: String,
        left: Box<CoreExpr>,
        right: Box<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    EffectOperation {
        operation: String,
        requirements: CoreType,
        failure: CoreType,
        success: CoreType,
        arguments: Vec<CoreExpr>,
        origin: SourceSpan,
    },
    Sequence {
        statements: Vec<CoreStatement>,
        result: Box<CoreExpr>,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreStatement {
    Effect {
        value: CoreExpr,
    },
    Bind {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
    },
}

pub fn lower_typed_module(module: TypedModule) -> CoreModule {
    let mut bindings = Vec::new();
    let mut functions = Vec::new();

    for declaration in module.declarations {
        match declaration {
            TypedDecl::Let {
                symbol,
                visibility,
                origin,
                value,
                ..
            } => bindings.push(CoreBinding {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                value: lower_expr(&module.source, value),
            }),
            TypedDecl::Fn {
                symbol,
                visibility,
                origin,
                parameters,
                body,
                ..
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                parameters: parameters
                    .into_iter()
                    .map(|parameter| lower_parameter(&parameter))
                    .collect(),
                body: lower_expr(&module.source, body),
            }),
            TypedDecl::EffectFn {
                symbol,
                visibility,
                origin,
                inferred_contract: _,
                parameters,
                effect,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                parameters: parameters
                    .into_iter()
                    .map(|parameter| lower_parameter(&parameter))
                    .collect(),
                body: lower_effect_body(&module.source, effect, body),
            }),
        }
    }

    CoreModule {
        schema: module.schema,
        stage: "core-ir".to_owned(),
        module: module.module,
        bindings,
        functions,
    }
}

fn lower_effect_body(source: &str, effect: TypedEffect, body: TypedExpr) -> CoreExpr {
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
                arguments: arguments
                    .into_iter()
                    .map(|argument| lower_expr(source, argument))
                    .collect(),
                origin: source_span(source, origin),
            }
        }
        TypedExpr::DoBlock {
            statements,
            result,
            origin,
        } => {
            let mut statements = statements.into_iter();
            match (statements.next(), statements.next()) {
                (Some(TypedDoStatement::Effect { value }), None) => {
                    lower_effect_body(source, effect, value)
                }
                (Some(first), Some(second)) => {
                    let statements = std::iter::once(first)
                        .chain(std::iter::once(second))
                        .chain(statements)
                        .map(|statement| lower_effect_statement(source, effect.clone(), statement))
                        .collect();
                    CoreExpr::Sequence {
                        statements,
                        result: Box::new(lower_expr(source, *result)),
                        origin: source_span(source, origin),
                    }
                }
                (None, _) => lower_expr(source, *result),
                (Some(statement), None) => CoreExpr::Sequence {
                    statements: vec![lower_effect_statement(source, effect, statement)],
                    result: Box::new(lower_expr(source, *result)),
                    origin: source_span(source, origin),
                },
            }
        }
        expr => lower_expr(source, expr),
    }
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

fn lower_expr(source: &str, expr: TypedExpr) -> CoreExpr {
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
            arguments: arguments
                .into_iter()
                .map(|argument| lower_expr(source, argument))
                .collect(),
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
                arguments: arguments
                    .into_iter()
                    .map(|argument| lower_expr(source, argument))
                    .collect(),
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

fn lower_parameter(parameter: &TypedParameter) -> CoreParameter {
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

fn lower_effect_operation(operation: &str) -> String {
    match operation {
        "std/prelude::readLine" => "stdin.readLine".to_owned(),
        "std/prelude::print" => "console.print".to_owned(),
        "std/prelude::println" => "console.println".to_owned(),
        other => other.to_owned(),
    }
}

fn effect_operation_contract(operation: &str) -> Option<(CoreType, CoreType, CoreType)> {
    let operation = known_effect_operation_by_semantic(operation)?;
    Some((
        CoreType::Record {
            closed: true,
            fields: vec![CoreRecordField {
                name: operation.requirement_field.to_owned(),
                optional: false,
                type_ref: named_core_type(operation.requirement_type),
            }],
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
