use crate::{Manifest, ModuleGraph, PackageIdentity};
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
