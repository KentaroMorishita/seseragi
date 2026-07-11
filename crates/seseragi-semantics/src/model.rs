use serde::{Deserialize, Serialize};
use seseragi_syntax::{
    ByteSpan, InterfaceDependency, InterfaceExport, InterfaceInstance, InterfaceOperator,
    Visibility,
};

mod resolved;

pub use resolved::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModule {
    pub schema: u32,
    pub stage: String,
    pub source: String,
    pub module: String,
    pub declarations: Vec<TypedDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedModuleInterface {
    pub schema: u32,
    pub stage: String,
    pub module: String,
    pub source: String,
    pub dependencies: Vec<InterfaceDependency>,
    pub exports: Vec<InterfaceExport>,
    pub operators: Vec<InterfaceOperator>,
    pub instances: Vec<InterfaceInstance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDecl {
    Adt {
        symbol: String,
        name: String,
        visibility: Visibility,
        opaque: bool,
        type_parameters: Vec<String>,
        variants: Vec<TypedAdtVariant>,
        origin: ByteSpan,
    },
    Let {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        value: TypedExpr,
    },
    Fn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        parameters: Vec<TypedParameter>,
        body: TypedExpr,
    },
    EffectFn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        #[serde(default, skip_serializing_if = "is_false")]
        inferred_contract: bool,
        parameters: Vec<TypedParameter>,
        effect: TypedEffect,
        body: TypedExpr,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedAdtVariant {
    pub symbol: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<TypedType>,
    pub scheme: TypedScheme,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedScheme {
    pub type_parameters: Vec<String>,
    pub constraints: Vec<TypedConstraint>,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedType {
    Named {
        name: String,
        arguments: Vec<TypedType>,
    },
    Hole,
    Record {
        closed: bool,
        fields: Vec<TypedRecordField>,
    },
    Tuple {
        elements: Vec<TypedType>,
    },
    Function {
        parameter: Box<TypedType>,
        result: Box<TypedType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedRecordField {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_ref: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedParameter {
    ImplicitUnit {
        #[serde(rename = "type")]
        type_ref: TypedType,
    },
    Named {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedEffect {
    pub environment: TypedType,
    pub failure: TypedType,
    pub success: TypedType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedExpr {
    Unit {
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Variable {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Call {
        callee: String,
        arguments: Vec<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Tuple {
        elements: Vec<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Binary {
        operator: String,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    If {
        condition: Box<TypedExpr>,
        then_branch: Box<TypedExpr>,
        else_branch: Box<TypedExpr>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Match {
        scrutinee: Box<TypedExpr>,
        arms: Vec<TypedMatchArm>,
        #[serde(default, skip_serializing_if = "is_false")]
        exhaustive: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    EffectCall {
        operation: String,
        effect: TypedEffect,
        arguments: Vec<TypedExpr>,
        origin: ByteSpan,
    },
    DoBlock {
        statements: Vec<TypedDoStatement>,
        result: Box<TypedExpr>,
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guard: Option<TypedExpr>,
    pub body: TypedExpr,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedPattern {
    Integer {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    String {
        value: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Boolean {
        value: bool,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Wildcard {
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Binding {
        symbol: SymbolId,
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Constructor {
        symbol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument: Option<Box<TypedPattern>>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Tuple {
        elements: Vec<TypedPattern>,
        #[serde(rename = "type")]
        type_ref: TypedType,
        origin: ByteSpan,
    },
    Invalid {
        origin: ByteSpan,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDoStatement {
    Effect {
        value: TypedExpr,
    },
    PureLet {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        value: TypedExpr,
        origin: ByteSpan,
    },
    Bind {
        name: String,
        #[serde(rename = "type")]
        type_ref: TypedType,
        value: TypedExpr,
        origin: ByteSpan,
    },
}

pub(crate) fn unit_type() -> TypedType {
    TypedType::Named {
        name: "Unit".to_owned(),
        arguments: Vec::new(),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}
