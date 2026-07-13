use serde::{Deserialize, Serialize};
use seseragi_syntax::{InterfaceDependency, InterfaceExport, SurfaceDecl};

mod instances;
mod interface;
mod names;

pub use instances::ResolvedDependencyInstance;
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_instances: Vec<ResolvedDependencyInstance>,
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
    pub specifier: String,
    pub module: String,
    pub local_name: String,
    pub origin: seseragi_syntax::ByteSpan,
    pub in_scope: bool,
    pub export: InterfaceExport,
    /// Nominal types referenced by a public callable scheme, resolved while
    /// the provider's final interface is still available. `None` means the
    /// callable scheme could not be resolved without ambiguity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_type_bindings: Option<Vec<crate::ExternalTypeBinding>>,
}
