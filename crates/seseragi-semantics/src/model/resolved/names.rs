use super::SymbolId;
use serde::{Deserialize, Serialize};
use seseragi_syntax::ByteSpan;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScopeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeKind {
    Module,
    Declaration,
    Function,
    DoBlock,
    MatchArm,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolNamespace {
    Type,
    Value,
    Trait,
    Field,
    Module,
    Operator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolKind {
    Let,
    Function,
    EffectFunction,
    TypeParameter,
    Parameter,
    PatternBinding,
    Type,
    Constructor,
    Trait,
    ModuleImport,
    Imported,
    Prelude,
    Operator,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedScope {
    pub id: ScopeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSymbol {
    pub id: SymbolId,
    pub spelling: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical: Option<String>,
    pub namespace: SymbolNamespace,
    pub kind: SymbolKind,
    pub scope: ScopeId,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedReference {
    pub spelling: String,
    pub namespace: SymbolNamespace,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SymbolId>,
    pub origin: ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveIssue {
    pub code: String,
    #[serde(rename = "messageKey")]
    pub message_key: String,
    pub primary: ByteSpan,
}
