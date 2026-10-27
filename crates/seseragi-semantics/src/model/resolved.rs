use serde::{Deserialize, Serialize};
use seseragi_syntax::{InterfaceDependency, InterfaceExport, SurfaceDecl, SurfaceForeignModule};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub foreign_modules: Vec<SurfaceForeignModule>,
    pub declarations: Vec<SurfaceDecl>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deprecations: Vec<seseragi_syntax::DeclarationDeprecation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespace_deprecations: Vec<ResolvedNamespaceDeprecation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operator_deprecations: Vec<seseragi_syntax::InterfaceOperatorDeprecation>,
    pub scopes: Vec<ResolvedScope>,
    pub symbols: Vec<ResolvedSymbol>,
    pub references: Vec<ResolvedReference>,
    pub issues: Vec<ResolveIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedNamespaceDeprecation {
    pub metadata: seseragi_syntax::InterfaceNamespaceDeprecation,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub own_declaration: Option<seseragi_syntax::ByteSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reexport_origin: Option<seseragi_syntax::ByteSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedImport {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reexported_as: Option<String>,
    pub symbol: SymbolId,
    pub specifier: String,
    pub module: String,
    pub local_name: String,
    pub origin: seseragi_syntax::ByteSpan,
    pub in_scope: bool,
    pub export: InterfaceExport,
    /// Nominal receiver that owns an automatically retained inherent method.
    /// The method is not introduced into lexical value scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_owner: Option<SymbolId>,
    /// Nominal types referenced by a public callable scheme, resolved while
    /// the provider's final interface is still available. `None` means the
    /// callable scheme could not be resolved without ambiguity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_type_bindings: Option<Vec<crate::ExternalTypeBinding>>,
    /// Trait names referenced by an imported callable scheme, resolved while
    /// the provider's final interface is still available. The binding is only
    /// needed for non-standalone traits; standard traits keep their registry
    /// identity in the typed stage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheme_trait_bindings: Option<Vec<crate::ExternalTraitBinding>>,
    /// Trait names referenced by an imported public trait contract, resolved
    /// in the provider interface before consumer-side method validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_trait_bindings: Option<Vec<crate::ExternalTraitBinding>>,
}
