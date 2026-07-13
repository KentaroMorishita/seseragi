use crate::{LoadedModule, LocalPackageGraph, ModuleGraph, ModuleIdentity};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedLocalProject {
    packages: LocalPackageGraph,
    entry: ModuleIdentity,
    graph: ModuleGraph<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
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
