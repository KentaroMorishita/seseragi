use crate::{LoadedModule, LocalPackageGraph, ModuleGraph, ModuleIdentity};
use std::collections::BTreeMap;

/// Editor analysis keeps successfully linked program roots in one canonical
/// graph while retaining failures for roots which could not be discovered.
#[derive(Debug)]
pub struct LoadedLocalAnalysis {
    project: LoadedLocalProject,
    program_errors: BTreeMap<ModuleIdentity, super::LocalProjectLoadError>,
    failed_program_sources: BTreeMap<ModuleIdentity, LoadedModule>,
}

impl LoadedLocalAnalysis {
    pub(super) fn new(
        project: LoadedLocalProject,
        program_errors: BTreeMap<ModuleIdentity, super::LocalProjectLoadError>,
        failed_program_sources: BTreeMap<ModuleIdentity, LoadedModule>,
    ) -> Self {
        Self {
            project,
            program_errors,
            failed_program_sources,
        }
    }

    pub const fn project(&self) -> &LoadedLocalProject {
        &self.project
    }

    pub const fn program_errors(&self) -> &BTreeMap<ModuleIdentity, super::LocalProjectLoadError> {
        &self.program_errors
    }

    /// Canonically resolved source inputs for diagnostics only. These roots did
    /// not link successfully and are absent from the project's accepted graph.
    pub const fn failed_program_sources(&self) -> &BTreeMap<ModuleIdentity, LoadedModule> {
        &self.failed_program_sources
    }

    pub(super) fn into_project(self) -> LoadedLocalProject {
        self.project
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedLocalProject {
    packages: LocalPackageGraph,
    entry: ModuleIdentity,
    graph: ModuleGraph<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedLocalTests {
    packages: LocalPackageGraph,
    roots: Vec<ModuleIdentity>,
    graph: ModuleGraph<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedLocalDocuments {
    packages: LocalPackageGraph,
    roots: Vec<ModuleIdentity>,
    graph: ModuleGraph<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
}

impl LoadedLocalDocuments {
    pub(super) const fn new(
        packages: LocalPackageGraph,
        roots: Vec<ModuleIdentity>,
        graph: ModuleGraph<ModuleIdentity>,
        modules: BTreeMap<ModuleIdentity, LoadedModule>,
    ) -> Self {
        Self {
            packages,
            roots,
            graph,
            modules,
        }
    }

    pub const fn packages(&self) -> &LocalPackageGraph {
        &self.packages
    }
    pub fn roots(&self) -> impl Iterator<Item = &ModuleIdentity> {
        self.roots.iter()
    }
    pub const fn graph(&self) -> &ModuleGraph<ModuleIdentity> {
        &self.graph
    }
    pub fn modules(&self) -> impl Iterator<Item = (&ModuleIdentity, &LoadedModule)> {
        self.modules.iter()
    }
    pub fn module(&self, identity: &ModuleIdentity) -> Option<&LoadedModule> {
        self.modules.get(identity)
    }
}

impl LoadedLocalTests {
    pub(super) const fn new(
        packages: LocalPackageGraph,
        roots: Vec<ModuleIdentity>,
        graph: ModuleGraph<ModuleIdentity>,
        modules: BTreeMap<ModuleIdentity, LoadedModule>,
    ) -> Self {
        Self {
            packages,
            roots,
            graph,
            modules,
        }
    }

    pub const fn packages(&self) -> &LocalPackageGraph {
        &self.packages
    }

    pub fn roots(&self) -> impl Iterator<Item = &ModuleIdentity> {
        self.roots.iter()
    }

    pub const fn graph(&self) -> &ModuleGraph<ModuleIdentity> {
        &self.graph
    }

    pub fn modules(&self) -> impl Iterator<Item = (&ModuleIdentity, &LoadedModule)> {
        self.modules.iter()
    }

    pub fn module(&self, identity: &ModuleIdentity) -> Option<&LoadedModule> {
        self.modules.get(identity)
    }
}

impl LoadedLocalProject {
    pub(super) const fn new(
        packages: LocalPackageGraph,
        entry: ModuleIdentity,
        graph: ModuleGraph<ModuleIdentity>,
        modules: BTreeMap<ModuleIdentity, LoadedModule>,
    ) -> Self {
        Self {
            packages,
            entry,
            graph,
            modules,
        }
    }

    pub const fn packages(&self) -> &LocalPackageGraph {
        &self.packages
    }

    pub const fn entry(&self) -> &ModuleIdentity {
        &self.entry
    }

    pub const fn graph(&self) -> &ModuleGraph<ModuleIdentity> {
        &self.graph
    }

    pub fn modules(&self) -> impl Iterator<Item = (&ModuleIdentity, &LoadedModule)> {
        self.modules.iter()
    }

    pub fn module(&self, identity: &ModuleIdentity) -> Option<&LoadedModule> {
        self.modules.get(identity)
    }
}
