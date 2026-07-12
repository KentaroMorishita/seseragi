use seseragi_syntax::{
    ByteSpan, InterfaceExport, InterfaceOperator, ModuleHeader, ModuleInterface,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedModule {
    pub header: ModuleHeader,
    pub interface: ModuleInterface,
    pub dependencies: Vec<LinkedDependency>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedDependency {
    pub specifier: String,
    pub origin: ByteSpan,
    pub interface: ModuleInterface,
    /// Same-package declaration names retained for namespace-member privacy
    /// diagnostics. External dependencies expose only their public interface.
    pub header: Option<ModuleHeader>,
    pub imports: Vec<LinkedImport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkedImport {
    Symbol {
        local_name: String,
        origin: ByteSpan,
        export: InterfaceExport,
    },
    Namespace {
        local_name: String,
        origin: ByteSpan,
        module: String,
    },
    Operator {
        spelling: String,
        origin: ByteSpan,
        export: InterfaceExport,
        operator: InterfaceOperator,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkError {
    UnresolvedSpecifier {
        specifier: String,
        origin: ByteSpan,
    },
    MissingExport {
        module: String,
        name: String,
        origin: ByteSpan,
    },
    PrivateExport {
        module: String,
        source: String,
        name: String,
        namespaces: Vec<String>,
        origin: ByteSpan,
        declarations: Vec<ByteSpan>,
    },
    DuplicateImport {
        namespace: String,
        local_name: String,
        origin: ByteSpan,
    },
    MissingNamespaceAlias {
        origin: ByteSpan,
    },
    UnsupportedImportNamespace {
        namespace: String,
        origin: ByteSpan,
    },
}

impl LinkError {
    pub const fn origin(&self) -> ByteSpan {
        match self {
            Self::UnresolvedSpecifier { origin, .. }
            | Self::MissingExport { origin, .. }
            | Self::PrivateExport { origin, .. }
            | Self::DuplicateImport { origin, .. }
            | Self::MissingNamespaceAlias { origin }
            | Self::UnsupportedImportNamespace { origin, .. } => *origin,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LinkTargetError {
    ModuleMismatch {
        header: String,
        interface: String,
    },
    SourceMismatch {
        header: String,
        interface: String,
    },
    MissingPublicExport {
        module: String,
        namespace: String,
        name: String,
        declaration: ByteSpan,
    },
}
