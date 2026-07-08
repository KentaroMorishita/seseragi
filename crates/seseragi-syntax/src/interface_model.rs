use crate::surface::{ByteSpan, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleInterface {
    pub schema: u32,
    pub module: String,
    pub source: String,
    pub dependencies: Vec<InterfaceDependency>,
    pub exports: Vec<InterfaceExport>,
    pub operators: Vec<InterfaceOperator>,
    pub instances: Vec<InterfaceInstance>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceDependency {
    pub specifier: String,
    pub module: String,
    pub origin: ByteSpan,
    pub imports: Vec<InterfaceImport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceImport {
    pub namespace: String,
    pub name: String,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceExport {
    pub symbol: String,
    pub namespace: String,
    pub name: String,
    pub visibility: Visibility,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_kind: Option<String>,
    pub declaration: ByteSpan,
    pub scheme: InterfaceScheme,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representation: Option<InterfaceType>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceScheme {
    pub type_parameters: Vec<String>,
    pub constraints: Vec<InterfaceConstraint>,
    #[serde(rename = "type")]
    pub type_ref: InterfaceType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceConstraint {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum InterfaceType {
    Named {
        name: String,
        arguments: Vec<InterfaceType>,
    },
    TypeConstructor {
        name: String,
        arity: u32,
    },
    Function {
        parameter: Box<InterfaceType>,
        result: Box<InterfaceType>,
    },
    Apply {
        constructor: String,
        arguments: Vec<InterfaceType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceOperator {
    pub symbol: String,
    pub spelling: String,
    pub fixity: String,
    pub precedence: u32,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceInstance {
    #[serde(rename = "trait")]
    pub trait_name: String,
    pub type_parameters: Vec<String>,
    pub head: InterfaceType,
    pub constraints: Vec<InterfaceConstraint>,
    pub origin: ByteSpan,
}
