use crate::{Manifest, ModuleGraph, ModulePath, PackageIdentity};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPackageManifest {
    root: PathBuf,
    identity: PackageIdentity,
    manifest: Manifest,
}

impl LocalPackageManifest {
    pub(super) const fn new(root: PathBuf, identity: PackageIdentity, manifest: Manifest) -> Self {
        Self {
            root,
            identity,
            manifest,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn identity(&self) -> &PackageIdentity {
        &self.identity
    }

    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPackageGraph {
    root: PackageIdentity,
    graph: ModuleGraph<PackageIdentity>,
    packages: BTreeMap<PackageIdentity, LocalPackageManifest>,
}

impl LocalPackageGraph {
    pub(super) const fn new(
        root: PackageIdentity,
        graph: ModuleGraph<PackageIdentity>,
        packages: BTreeMap<PackageIdentity, LocalPackageManifest>,
    ) -> Self {
        Self {
            root,
            graph,
            packages,
        }
    }

    pub const fn root(&self) -> &PackageIdentity {
        &self.root
    }

    pub const fn graph(&self) -> &ModuleGraph<PackageIdentity> {
        &self.graph
    }

    pub fn packages(&self) -> impl Iterator<Item = (&PackageIdentity, &LocalPackageManifest)> {
        self.packages.iter()
    }

    pub fn package(&self, identity: &PackageIdentity) -> Option<&LocalPackageManifest> {
        self.packages.get(identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedPackageImport {
    dependency_key: String,
    export_key: String,
    package: PackageIdentity,
    module: ModulePath,
}

impl ResolvedPackageImport {
    pub(super) const fn new(
        dependency_key: String,
        export_key: String,
        package: PackageIdentity,
        module: ModulePath,
    ) -> Self {
        Self {
            dependency_key,
            export_key,
            package,
            module,
        }
    }

    pub fn dependency_key(&self) -> &str {
        &self.dependency_key
    }

    pub fn export_key(&self) -> &str {
        &self.export_key
    }

    pub const fn package(&self) -> &PackageIdentity {
        &self.package
    }

    pub const fn module(&self) -> &ModulePath {
        &self.module
    }
}
