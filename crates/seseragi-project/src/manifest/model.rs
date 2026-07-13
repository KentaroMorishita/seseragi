use crate::{ModulePath, PackageName};
use semver::Version;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Manifest {
    pub package: ManifestPackage,
    pub layout: ManifestLayout,
    pub exports: BTreeMap<String, ModulePath>,
    pub run: Option<ManifestRun>,
    pub(crate) deferred: DeferredTables,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestPackage {
    pub name: PackageName,
    pub version: Version,
    pub language: LanguageRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageRequirement(String);

impl LanguageRequirement {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, version: &Version) -> bool {
        super::requirement::matches(&self.0, version)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestLayout {
    pub source: LayoutPath,
    pub tests: LayoutPath,
    pub benchmarks: LayoutPath,
    pub generated: LayoutPath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutPath(String);

impl LayoutPath {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestRun {
    pub entry: ModulePath,
    pub target: Option<TargetId>,
    pub signal_mode: SignalMode,
    pub shutdown_grace_ms: Option<u64>,
    pub hash_seed: RunSeed,
    pub random_seed: RunSeed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetId(String);

impl TargetId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalMode {
    Cancel,
    Forward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunSeed {
    Entropy,
    Fixed(i64),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct DeferredTables {
    pub dependencies: BTreeMap<String, toml::Value>,
    pub foreign: Option<toml::Table>,
    pub test: Option<toml::Table>,
    pub benchmark: Option<toml::Table>,
    pub tool: Option<toml::Table>,
}
