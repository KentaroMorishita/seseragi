use crate::{source_span, SourceSpan};
use serde::{Deserialize, Serialize};
use seseragi_semantics::{ExternalTypeBinding, TypedDecl, TypedModule};
use seseragi_syntax::Visibility;

mod adt;
mod decision;
mod expr;
mod instances;
mod types;

use adt::{lower_adt, AdtDeclInput};
pub use adt::{CoreAdt, CoreAdtVariant};
pub use decision::{
    CoreDecisionBinding, CoreDecisionBranch, CoreDecisionProjection, CoreDecisionTest,
};
use expr::{lower_effect_body, lower_expr, lower_parameter};
use instances::lower_instances;
pub use instances::{CoreInstance, CoreInstanceConstraint, CoreInstanceImplementation};
pub use types::{CoreRecordField, CoreType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_type_bindings: Vec<ExternalTypeBinding>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adts: Vec<CoreAdt>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instances: Vec<CoreInstance>,
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
    Tuple {
        elements: Vec<CoreExpr>,
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
    If {
        condition: Box<CoreExpr>,
        then_branch: Box<CoreExpr>,
        else_branch: Box<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Decision {
        scrutinee: Box<CoreExpr>,
        scrutinee_type: CoreType,
        branches: Vec<CoreDecisionBranch>,
        exhaustive: bool,
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
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        value: CoreExpr,
        origin: SourceSpan,
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
    let instances = lower_instances(&module.source, module.instances);
    let mut adts = Vec::new();
    let mut bindings = Vec::new();
    let mut functions = Vec::new();

    for declaration in module.declarations {
        match declaration {
            TypedDecl::Adt {
                symbol,
                name,
                visibility,
                opaque,
                type_parameters,
                variants,
                origin,
            } => adts.push(lower_adt(
                &module.source,
                AdtDeclInput {
                    symbol,
                    name,
                    visibility,
                    opaque,
                    type_parameters,
                    variants,
                    origin,
                },
            )),
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
                effect: _,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                parameters: parameters
                    .into_iter()
                    .map(|parameter| lower_parameter(&parameter))
                    .collect(),
                body: lower_effect_body(&module.source, body),
            }),
        }
    }

    CoreModule {
        schema: module.schema,
        stage: "core-ir".to_owned(),
        module: module.module,
        external_type_bindings: module.external_type_bindings,
        adts,
        instances,
        bindings,
        functions,
    }
}
