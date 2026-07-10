use serde::{Deserialize, Serialize};
use seseragi_syntax::{ByteSpan, InterfaceScheme, InterfaceType, Visibility};

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
    Constructor {
        symbol: SymbolId,
        name: String,
        owner: String,
        visibility: Visibility,
        scheme: InterfaceScheme,
        declaration: ByteSpan,
    },
    Type {
        symbol: SymbolId,
        name: String,
        visibility: Visibility,
        #[serde(skip_serializing_if = "Option::is_none")]
        declaration_kind: Option<String>,
        declaration: ByteSpan,
        #[serde(skip_serializing_if = "Option::is_none")]
        representation: Option<InterfaceType>,
    },
    Operator {
        symbol: SymbolId,
        name: String,
        visibility: Visibility,
        #[serde(skip_serializing_if = "Option::is_none")]
        fixity: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        precedence: Option<u32>,
        declaration: ByteSpan,
    },
    Instance {
        symbol: SymbolId,
        trait_name: String,
        head: InterfaceType,
        declaration: ByteSpan,
    },
}
