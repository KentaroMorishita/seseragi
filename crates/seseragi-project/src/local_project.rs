mod error;
mod import;
mod model;

#[cfg(test)]
mod tests;

pub use error::LocalProjectLoadError;
pub use model::LoadedLocalProject;

use crate::loader::audit;
use crate::loader::filesystem;
use crate::{
    discover_local_package_graph, is_standard_module, LoadedModule, LocalPackageGraph, ModuleGraph,
    ModuleIdentity, ModuleRoot, PackageIdentity, PackageLoadError,
};
use import::resolve_import;
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_local_project(
    root: impl AsRef<Path>,
) -> Result<LoadedLocalProject, LocalProjectLoadError> {
    let packages = discover_local_package_graph(root).map_err(LocalProjectLoadError::Packages)?;
    let root = packages.root().clone();
    let root_manifest = packages
        .package(&root)
        .expect("discovered package graph contains its root");
    let entry_path = root_manifest
        .manifest()
        .run
        .as_ref()
        .ok_or_else(|| LocalProjectLoadError::MissingRunEntry {
            package: Box::new(root.clone()),
        })?
        .entry
        .clone();
    let entry = ModuleIdentity::new(root, ModuleRoot::Source, entry_path);
    let (graph, modules) = {
        let mut state = SourceDiscovery::new(&packages)?;
        state.discover(entry.clone())?;
        let graph = state.finish()?;
        (graph, state.modules)
    };
    Ok(LoadedLocalProject::new(packages, entry, graph, modules))
}

struct SourceDiscovery<'a> {
    packages: &'a LocalPackageGraph,
    source_roots: BTreeMap<PackageIdentity, PathBuf>,
    pending: BTreeSet<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
    edges: BTreeMap<ModuleIdentity, BTreeMap<String, ModuleIdentity>>,
    physical_owners: BTreeMap<PathBuf, ModuleIdentity>,
}

impl<'a> SourceDiscovery<'a> {
    fn new(packages: &'a LocalPackageGraph) -> Result<Self, LocalProjectLoadError> {
        let mut source_roots = BTreeMap::new();
        for (identity, package) in packages.packages() {
            let source_root =
                filesystem::resolve_source_root(package.root(), &package.manifest().layout.source)
                    .map_err(|error| LocalProjectLoadError::Filesystem {
                        package: Box::new(identity.clone()),
                        error: Box::new(error),
                    })?;
            audit::audit_source_root(&source_root).map_err(|error| {
                LocalProjectLoadError::Filesystem {
                    package: Box::new(identity.clone()),
                    error: Box::new(error),
                }
            })?;
            source_roots.insert(identity.clone(), source_root);
        }
        Ok(Self {
            packages,
            source_roots,
            pending: BTreeSet::new(),
            modules: BTreeMap::new(),
            edges: BTreeMap::new(),
            physical_owners: BTreeMap::new(),
        })
    }

    fn discover(&mut self, entry: ModuleIdentity) -> Result<(), LocalProjectLoadError> {
        self.pending.insert(entry);
        while let Some(module) = self.pending.pop_first() {
            if self.modules.contains_key(&module) {
                continue;
            }
            self.discover_module(module)?;
        }
        Ok(())
    }

    fn discover_module(&mut self, module: ModuleIdentity) -> Result<(), LocalProjectLoadError> {
        let source_root = self
            .source_roots
            .get(module.package())
            .expect("package graph has a source root for every package");
        let source_path =
            filesystem::resolve_module_file(source_root, module.path()).map_err(|error| {
                LocalProjectLoadError::Filesystem {
                    package: Box::new(module.package().clone()),
                    error: Box::new(error),
                }
            })?;
        let canonical_path =
            fs::canonicalize(&source_path).map_err(|source| LocalProjectLoadError::Filesystem {
                package: Box::new(module.package().clone()),
                error: Box::new(PackageLoadError::io(
                    "canonicalize module",
                    source_path.clone(),
                    source,
                )),
            })?;
        if !canonical_path.starts_with(source_root) {
            return Err(LocalProjectLoadError::Filesystem {
                package: Box::new(module.package().clone()),
                error: Box::new(PackageLoadError::RootEscape {
                    path: source_path,
                    canonical_path,
                }),
            });
        }
        if let Some(first) = self
            .physical_owners
            .insert(canonical_path.clone(), module.clone())
        {
            if first != module {
                return Err(LocalProjectLoadError::DuplicatePhysicalModule {
                    first: Box::new(first),
                    second: Box::new(module),
                    canonical_path,
                });
            }
        }
        let source = fs::read_to_string(&canonical_path).map_err(|error| {
            LocalProjectLoadError::Filesystem {
                package: Box::new(module.package().clone()),
                error: Box::new(PackageLoadError::io(
                    "read module",
                    canonical_path.clone(),
                    error,
                )),
            }
        })?;
        let unlinked = parse_unlinked_module_interface(
            canonical_path.to_string_lossy(),
            module_label(&module),
            &source,
        );
        let mut edges = BTreeMap::new();
        for import in unlinked.imports {
            if is_standard_module(&import.specifier) {
                continue;
            }
            let dependency =
                resolve_import(self.packages, &module, &import.specifier).map_err(|failure| {
                    LocalProjectLoadError::Import {
                        module: Box::new(module.clone()),
                        specifier: import.specifier.clone(),
                        origin: import.span,
                        code: failure.code,
                        reason: failure.reason,
                    }
                })?;
            edges.insert(import.specifier, dependency.clone());
            if !self.modules.contains_key(&dependency) {
                self.pending.insert(dependency);
            }
        }
        self.edges.insert(module.clone(), edges);
        self.modules.insert(
            module.clone(),
            LoadedModule::new(module, canonical_path, source),
        );
        Ok(())
    }

    fn finish(&self) -> Result<ModuleGraph<ModuleIdentity>, LocalProjectLoadError> {
        let mut graph = ModuleGraph::new();
        for (module, dependencies) in &self.edges {
            graph
                .add_module(module.clone(), dependencies.clone())
                .map_err(|error| LocalProjectLoadError::Graph(Box::new(error)))?;
        }
        graph
            .topological_order()
            .map_err(|error| LocalProjectLoadError::Graph(Box::new(error)))?;
        Ok(graph)
    }
}

fn module_label(module: &ModuleIdentity) -> String {
    format!(
        "{}::{}",
        module.package().name().as_str(),
        module.path().as_str()
    )
}
