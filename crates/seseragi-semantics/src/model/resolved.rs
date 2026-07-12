use serde::{Deserialize, Serialize};
use seseragi_syntax::{InterfaceDependency, InterfaceExport, SurfaceDecl};

mod interface;
mod names;

pub use interface::{ResolvedInterface, ResolvedInterfaceDecl};
pub use names::{
    ResolveIssue, ResolvedReference, ResolvedScope, ResolvedSymbol, ScopeId, ScopeKind, SymbolKind,
    SymbolNamespace,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedModule {
    pub schema: u32,
    pub stage: String,
    pub source: String,
    pub module: String,
    pub dependencies: Vec<InterfaceDependency>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<ResolvedImport>,
    pub declarations: Vec<SurfaceDecl>,
    pub scopes: Vec<ResolvedScope>,
    pub symbols: Vec<ResolvedSymbol>,
    pub references: Vec<ResolvedReference>,
    pub issues: Vec<ResolveIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedImport {
    pub symbol: SymbolId,
    pub local_name: String,
    pub origin: seseragi_syntax::ByteSpan,
    pub export: InterfaceExport,
}
