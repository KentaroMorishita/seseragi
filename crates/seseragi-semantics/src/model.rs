use serde::{Deserialize, Serialize};
use seseragi_syntax::{ByteSpan, Visibility};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedModule {
    pub schema: u32,
    pub source: String,
    pub module: String,
    pub declarations: Vec<ResolvedDecl>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum ResolvedDecl {
    Value {
        symbol: SymbolId,
        name: String,
        visibility: Visibility,
        declaration: ByteSpan,
    },
}

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
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum TypedDecl {
    Let {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        scheme: TypedScheme,
        value: TypedExpr,
    },
    EffectFn {
        symbol: String,
        visibility: Visibility,
        origin: ByteSpan,
        parameters: Vec<TypedParameter>,
        effect: TypedEffect,
        body: TypedExpr,
    },
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
    Record {
        closed: bool,
        fields: Vec<TypedRecordField>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedRecordField {
    pub name: String,
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
    EffectCall {
        operation: String,
        arguments: Vec<TypedExpr>,
        origin: ByteSpan,
    },
    DoBlock {
        statements: Vec<TypedExpr>,
        result: Box<TypedExpr>,
        origin: ByteSpan,
    },
}

pub(crate) fn unit_type() -> TypedType {
    TypedType::Named {
        name: "Unit".to_owned(),
        arguments: Vec::new(),
    }
}
