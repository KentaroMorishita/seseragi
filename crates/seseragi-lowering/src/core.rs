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
pub use instances::{
    CoreInstance, CoreInstanceConstraint, CoreInstanceEvidence, CoreInstanceImplementation,
    CoreInstanceMethod, CoreShowPayloadEvidence,
};
pub use types::{CoreRecordField, CoreType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModule {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_type_bindings: Vec<ExternalTypeBinding>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_dependencies: Vec<CoreModuleDependency>,
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
pub struct CoreModuleDependency {
    pub specifier: String,
    pub module: String,
    pub origin: SourceSpan,
    pub imports: Vec<CoreModuleImport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreModuleImport {
    pub namespace: String,
    pub imported: String,
    pub local: String,
    pub canonical: String,
    pub origin: SourceSpan,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_constructor_parameters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<CoreInstanceConstraint>,
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
#[serde(rename_all = "camelCase")]
pub struct CoreCallEvidence {
    pub constraint: CoreInstanceConstraint,
    pub evidence: CoreInstanceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreTraitDispatch {
    pub trait_identity: String,
    pub method: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreComprehensionClause {
    Generator {
        pattern: CorePattern,
        source: CoreExpr,
        evidence: CoreCallEvidence,
        origin: SourceSpan,
    },
    Guard {
        condition: CoreExpr,
        origin: SourceSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CorePattern {
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Binding {
        name: String,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Wildcard {
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Constructor {
        symbol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument: Option<Box<CorePattern>>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Tuple {
        elements: Vec<CorePattern>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Invalid {
        origin: SourceSpan,
    },
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
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Call {
        callee: String,
        arguments: Vec<CoreExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trait_dispatch: Option<CoreTraitDispatch>,
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
    Array {
        elements: Vec<CoreExpr>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    ArrayComprehension {
        element: Box<CoreExpr>,
        clauses: Vec<CoreComprehensionClause>,
        #[serde(rename = "type")]
        type_ref: CoreType,
        origin: SourceSpan,
    },
    Binary {
        operator: String,
        left: Box<CoreExpr>,
        right: Box<CoreExpr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        evidence: Vec<CoreCallEvidence>,
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
    EffectInvoke {
        callee: String,
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
    MonadDo {
        statements: Vec<CoreMonadDoStatement>,
        result: Box<CoreExpr>,
        evidence: CoreCallEvidence,
        #[serde(rename = "type")]
        type_ref: CoreType,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreMonadDoStatement {
    Expression {
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
    let module_dependencies = module
        .module_dependencies
        .into_iter()
        .map(|dependency| CoreModuleDependency {
            specifier: dependency.specifier,
            module: dependency.module,
            origin: source_span(&module.source, dependency.origin),
            imports: dependency
                .imports
                .into_iter()
                .map(|import| CoreModuleImport {
                    namespace: import.namespace,
                    imported: import.imported,
                    local: import.local,
                    canonical: import.canonical,
                    origin: source_span(&module.source, import.origin),
                })
                .collect(),
        })
        .collect();
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
                type_constructor_parameters,
                scheme,
                parameters,
                body,
            } => functions.push(CoreFunction {
                symbol,
                visibility,
                origin: source_span(&module.source, origin),
                type_parameters: scheme.type_parameters,
                type_constructor_parameters,
                constraints: scheme
                    .constraints
                    .into_iter()
                    .map(instances::lower_constraint)
                    .collect(),
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
                type_parameters: Vec::new(),
                type_constructor_parameters: Vec::new(),
                constraints: Vec::new(),
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
        module_dependencies,
        adts,
        instances,
        bindings,
        functions,
    }
}
