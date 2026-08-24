use crate::PackageName;
use semver::Version;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lockfile {
    pub schema: u64,
    pub language: Version,
    pub standard_library: Version,
    pub unicode: String,
    pub timezone_database: String,
    pub root: String,
    pub packages: Vec<LockPackage>,
    pub providers: Vec<LockProviderSelection>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockPackage {
    pub id: String,
    pub name: PackageName,
    pub version: Version,
    pub source_kind: LockSourceKind,
    pub source: String,
    pub manifest_digest: String,
    pub content_digest: String,
    pub dependencies: Vec<LockDependency>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockSourceKind {
    Workspace,
    Path,
    Registry,
}

impl LockSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Path => "path",
            Self::Registry => "registry",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockDependency {
    pub import: String,
    pub package: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockProviderSelection {
    pub field: String,
    pub service: String,
    pub required_contract: String,
    pub provider_contract: String,
    pub provider: String,
    pub package_version: String,
    pub package_source: String,
    pub package_digest: String,
    pub artifact_digest: String,
    pub backend: String,
    pub backend_abi_major: u64,
    pub target: String,
    pub entry_module: String,
    pub entry_export: String,
    pub runtime_features: Vec<String>,
    pub host_packages: Vec<LockHostPackage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LockHostPackage {
    pub name: String,
    pub version: String,
    pub source: String,
    pub content_digest: String,
}

impl LockPackage {
    pub fn canonical_id(&self) -> String {
        format!(
            "{}@{}#{}:{}",
            self.name.as_str(),
            self.version,
            self.source_kind.as_str(),
            self.source
        )
    }
}
