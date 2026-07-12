use crate::surface::{ByteSpan, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleHeader {
    pub schema: u32,
    pub module: String,
    pub source: String,
    pub names: Vec<ModuleHeaderName>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleHeaderName {
    pub symbol: String,
    pub namespace: String,
    pub name: String,
    pub visibility: Visibility,
    pub declaration_kind: String,
    pub declaration: ByteSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constructor_of: Option<String>,
}
