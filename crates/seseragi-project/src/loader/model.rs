use crate::{Manifest, ModuleGraph, ModuleIdentity, ModulePath, PackageIdentity};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedModule {
    identity: ModuleIdentity,
    source_path: PathBuf,
    source: String,
}

impl LoadedModule {
    pub(crate) const fn new(
        identity: ModuleIdentity,
        source_path: PathBuf,
        source: String,
    ) -> Self {
        Self {
            identity,
            source_path,
            source,
        }
    }

    pub const fn path(&self) -> &ModulePath {
        self.identity.path()
    }

    pub const fn identity(&self) -> &ModuleIdentity {
        &self.identity
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedPackage {
    root: PathBuf,
    source_root: PathBuf,
    manifest: Manifest,
    identity: PackageIdentity,
    entry: ModulePath,
    graph: ModuleGraph<ModulePath>,
    modules: BTreeMap<ModulePath, LoadedModule>,
}

impl LoadedPackage {
    pub(crate) const fn new(
        root: PathBuf,
        source_root: PathBuf,
        manifest: Manifest,
        identity: PackageIdentity,
        entry: ModulePath,
        graph: ModuleGraph<ModulePath>,
        modules: BTreeMap<ModulePath, LoadedModule>,
    ) -> Self {
        Self {
            root,
            source_root,
            manifest,
            identity,
            entry,
            graph,
            modules,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub const fn identity(&self) -> &PackageIdentity {
        &self.identity
    }

    pub const fn entry(&self) -> &ModulePath {
        &self.entry
    }

    pub const fn graph(&self) -> &ModuleGraph<ModulePath> {
        &self.graph
    }

    pub fn modules(&self) -> impl Iterator<Item = (&ModulePath, &LoadedModule)> {
        self.modules.iter()
    }

    pub fn module(&self, path: &ModulePath) -> Option<&LoadedModule> {
        self.modules.get(path)
    }
}
