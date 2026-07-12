use seseragi_syntax::{ByteSpan, InterfaceExport, InterfaceOperator, ModuleInterface};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedModule {
    pub interface: ModuleInterface,
    pub dependencies: Vec<LinkedDependency>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkedDependency {
    pub specifier: String,
    pub origin: ByteSpan,
    pub interface: ModuleInterface,
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
            | Self::DuplicateImport { origin, .. }
            | Self::MissingNamespaceAlias { origin }
            | Self::UnsupportedImportNamespace { origin, .. } => *origin,
        }
    }
}
