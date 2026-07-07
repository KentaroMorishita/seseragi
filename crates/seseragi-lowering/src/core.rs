use crate::{source_span, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedDecl, TypedEffect, TypedExpr, TypedModule, TypedType};
use seseragi_syntax::Visibility;

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
    pub type_name: String,
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
    EffectOperation {
        operation: String,
        requirements: Vec<String>,
        failure: String,
        success: String,
        arguments: Vec<CoreExpr>,
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
            TypedDecl::EffectFn {
                symbol,
                visibility,
                origin,
                parameters,
                effect,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                parameters: parameters
                    .into_iter()
                    .map(|_| CoreParameter {
                        id: "unit".to_owned(),
                        kind: "implicit".to_owned(),
                        type_name: "Unit".to_owned(),
                    })
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
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: effect_requirements(&effect),
            failure: type_name(&effect.failure),
            success: type_name(&effect.success),
            arguments: arguments
                .into_iter()
                .map(|argument| lower_expr(source, argument))
                .collect(),
            origin: source_span(source, origin),
        },
        expr => lower_expr(source, expr),
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
        TypedExpr::EffectCall {
            operation,
            arguments,
            origin,
        } => CoreExpr::EffectOperation {
            operation: lower_effect_operation(&operation),
            requirements: Vec::new(),
            failure: "Never".to_owned(),
            success: "Unit".to_owned(),
            arguments: arguments
                .into_iter()
                .map(|argument| lower_expr(source, argument))
                .collect(),
            origin: source_span(source, origin),
        },
        TypedExpr::DoBlock { result, .. } => lower_expr(source, *result),
    }
}

fn lower_effect_operation(operation: &str) -> String {
    match operation {
        "std/prelude::println" => "console.println".to_owned(),
        other => other.to_owned(),
    }
}

fn effect_requirements(effect: &TypedEffect) -> Vec<String> {
    match &effect.environment {
        TypedType::Record { fields, .. } => fields.iter().map(|field| field.name.clone()).collect(),
        _ => Vec::new(),
    }
}

fn type_name(type_ref: &TypedType) -> String {
    match type_ref {
        TypedType::Named { name, .. } => name.clone(),
        TypedType::Record { .. } => "Record".to_owned(),
    }
}
