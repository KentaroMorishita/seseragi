mod error;
mod import;
mod model;

#[cfg(test)]
mod tests;

pub use error::LocalProjectLoadError;
pub use model::{LoadedLocalDocuments, LoadedLocalProject, LoadedLocalTests};

use crate::loader::audit;
use crate::loader::filesystem;
use crate::{
    discover_local_package_graph, LoadedModule, LocalPackageGraph, ModuleGraph, ModuleIdentity,
    ModuleRoot, PackageIdentity, PackageLoadError, SourceOverlay,
};
use import::{resolve_import, ResolvedImport};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_local_project(
    root: impl AsRef<Path>,
) -> Result<LoadedLocalProject, LocalProjectLoadError> {
    load_local_project_with_overlays(root, std::iter::empty())
}

/// Discovers every canonical test module under the root package's configured
/// test directory and links its normal source dependencies through the same
/// package graph as production compilation.
pub fn load_local_tests(root: impl AsRef<Path>) -> Result<LoadedLocalTests, LocalProjectLoadError> {
    let packages = discover_local_package_graph(root).map_err(LocalProjectLoadError::Packages)?;
    let root = packages.root().clone();
    let package = packages
        .package(&root)
        .expect("discovered package graph contains its root");
    let candidate = package
        .root()
        .join(package.manifest().layout.tests.as_str());
    let roots = if candidate.exists() {
        let test_root =
            filesystem::resolve_source_root(package.root(), &package.manifest().layout.tests)
                .map_err(|error| LocalProjectLoadError::Filesystem {
                    package: Box::new(root.clone()),
                    error: Box::new(error),
                })?;
        discover_test_modules(&root, &test_root)?
    } else {
        Vec::new()
    };
    let (graph, modules) = {
        let mut state = SourceDiscovery::new_with_tests(&packages, BTreeMap::new())?;
        state.discover_all(roots.iter().cloned())?;
        let graph = state.finish()?;
        (graph, state.modules)
    };
    Ok(LoadedLocalTests::new(packages, roots, graph, modules))
}

/// Discovers every source module in the root package for API documentation,
/// plus only the dependency modules reachable from those roots.
pub fn load_local_documents(
    root: impl AsRef<Path>,
) -> Result<LoadedLocalDocuments, LocalProjectLoadError> {
    let packages = discover_local_package_graph(root).map_err(LocalProjectLoadError::Packages)?;
    let root = packages.root().clone();
    let package = packages
        .package(&root)
        .expect("discovered package graph contains its root");
    let source_root =
        filesystem::resolve_source_root(package.root(), &package.manifest().layout.source)
            .map_err(|error| LocalProjectLoadError::Filesystem {
                package: Box::new(root.clone()),
                error: Box::new(error),
            })?;
    let roots = discover_modules(&root, ModuleRoot::Source, &source_root)?;
    let (graph, modules) = {
        let mut state = SourceDiscovery::new(&packages, BTreeMap::new())?;
        state.discover_all(roots.iter().cloned())?;
        let graph = state.finish()?;
        (graph, state.modules)
    };
    Ok(LoadedLocalDocuments::new(packages, roots, graph, modules))
}

fn discover_test_modules(
    package: &PackageIdentity,
    test_root: &Path,
) -> Result<Vec<ModuleIdentity>, LocalProjectLoadError> {
    discover_modules(package, ModuleRoot::Test, test_root)
}

fn discover_modules(
    package: &PackageIdentity,
    module_root: ModuleRoot,
    source_root: &Path,
) -> Result<Vec<ModuleIdentity>, LocalProjectLoadError> {
    fn visit(
        package: &PackageIdentity,
        module_root: ModuleRoot,
        root: &Path,
        directory: &Path,
        modules: &mut Vec<ModuleIdentity>,
    ) -> Result<(), LocalProjectLoadError> {
        let mut entries = fs::read_dir(directory)
            .map_err(|error| LocalProjectLoadError::Filesystem {
                package: Box::new(package.clone()),
                error: Box::new(PackageLoadError::io(
                    "read test directory",
                    directory.to_owned(),
                    error,
                )),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| LocalProjectLoadError::Filesystem {
                package: Box::new(package.clone()),
                error: Box::new(PackageLoadError::io(
                    "read test entry",
                    directory.to_owned(),
                    error,
                )),
            })?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type =
                entry
                    .file_type()
                    .map_err(|error| LocalProjectLoadError::Filesystem {
                        package: Box::new(package.clone()),
                        error: Box::new(PackageLoadError::io(
                            "inspect test entry",
                            path.clone(),
                            error,
                        )),
                    })?;
            if file_type.is_dir() {
                visit(package, module_root, root, &path, modules)?;
            } else if file_type.is_file()
                && path.extension().and_then(|value| value.to_str()) == Some("ssrg")
            {
                let relative = path.strip_prefix(root).expect("walk stays under test root");
                let mut module = relative.to_string_lossy().replace('\\', "/");
                module.truncate(module.len() - ".ssrg".len());
                let module = crate::ModulePath::parse(&module).map_err(|error| {
                    LocalProjectLoadError::Import {
                        module: Box::new(ModuleIdentity::new(
                            package.clone(),
                            module_root,
                            crate::ModulePath::parse("invalid").expect("literal module path"),
                        )),
                        specifier: module,
                        origin: seseragi_syntax::ByteSpan { start: 0, end: 0 },
                        code: "SES-N0104",
                        reason: error.to_string(),
                    }
                })?;
                modules.push(ModuleIdentity::new(package.clone(), module_root, module));
            }
        }
        Ok(())
    }

    let mut modules = Vec::new();
    visit(package, module_root, source_root, source_root, &mut modules)?;
    modules.sort();
    Ok(modules)
}

/// Loads a local package project while allowing editor buffers to shadow the
/// corresponding files on disk.
pub fn load_local_project_with_overlays(
    root: impl AsRef<Path>,
    overlays: impl IntoIterator<Item = SourceOverlay>,
) -> Result<LoadedLocalProject, LocalProjectLoadError> {
    let packages = discover_local_package_graph(root).map_err(LocalProjectLoadError::Packages)?;
    let overlays = normalize_overlays(overlays)?;
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
        let mut state = SourceDiscovery::new(&packages, overlays)?;
        state.discover(entry.clone())?;
        let graph = state.finish()?;
        (graph, state.modules)
    };
    Ok(LoadedLocalProject::new(packages, entry, graph, modules))
}

fn normalize_overlays(
    overlays: impl IntoIterator<Item = SourceOverlay>,
) -> Result<BTreeMap<PathBuf, String>, LocalProjectLoadError> {
    let mut normalized = BTreeMap::new();
    for overlay in overlays {
        let path =
            fs::canonicalize(overlay.path()).map_err(|source| LocalProjectLoadError::Overlay {
                path: overlay.path().to_owned(),
                source,
            })?;
        normalized.insert(path, overlay.source().to_owned());
    }
    Ok(normalized)
}

struct SourceDiscovery<'a> {
    packages: &'a LocalPackageGraph,
    source_roots: BTreeMap<(PackageIdentity, ModuleRoot), PathBuf>,
    pending: BTreeSet<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, LoadedModule>,
    edges: BTreeMap<ModuleIdentity, BTreeMap<String, ModuleIdentity>>,
    physical_owners: BTreeMap<PathBuf, ModuleIdentity>,
    overlays: BTreeMap<PathBuf, String>,
}

impl<'a> SourceDiscovery<'a> {
    fn new(
        packages: &'a LocalPackageGraph,
        overlays: BTreeMap<PathBuf, String>,
    ) -> Result<Self, LocalProjectLoadError> {
        Self::new_inner(packages, overlays, false)
    }

    fn new_with_tests(
        packages: &'a LocalPackageGraph,
        overlays: BTreeMap<PathBuf, String>,
    ) -> Result<Self, LocalProjectLoadError> {
        Self::new_inner(packages, overlays, true)
    }

    fn new_inner(
        packages: &'a LocalPackageGraph,
        overlays: BTreeMap<PathBuf, String>,
        include_tests: bool,
    ) -> Result<Self, LocalProjectLoadError> {
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
            source_roots.insert((identity.clone(), ModuleRoot::Source), source_root);
            if include_tests && identity == packages.root() {
                let candidate = package
                    .root()
                    .join(package.manifest().layout.tests.as_str());
                if candidate.exists() {
                    let test_root = filesystem::resolve_source_root(
                        package.root(),
                        &package.manifest().layout.tests,
                    )
                    .map_err(|error| LocalProjectLoadError::Filesystem {
                        package: Box::new(identity.clone()),
                        error: Box::new(error),
                    })?;
                    audit::audit_source_root(&test_root).map_err(|error| {
                        LocalProjectLoadError::Filesystem {
                            package: Box::new(identity.clone()),
                            error: Box::new(error),
                        }
                    })?;
                    source_roots.insert((identity.clone(), ModuleRoot::Test), test_root);
                }
            }
        }
        Ok(Self {
            packages,
            source_roots,
            pending: BTreeSet::new(),
            modules: BTreeMap::new(),
            edges: BTreeMap::new(),
            physical_owners: BTreeMap::new(),
            overlays,
        })
    }

    fn discover(&mut self, entry: ModuleIdentity) -> Result<(), LocalProjectLoadError> {
        self.discover_all([entry])
    }

    fn discover_all(
        &mut self,
        entries: impl IntoIterator<Item = ModuleIdentity>,
    ) -> Result<(), LocalProjectLoadError> {
        self.pending.extend(entries);
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
            .get(&(module.package().clone(), module.root()))
            .expect("package graph has a filesystem root for every discovered module");
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
        let source = match self.overlays.get(&canonical_path) {
            Some(source) => source.clone(),
            None => fs::read_to_string(&canonical_path).map_err(|error| {
                LocalProjectLoadError::Filesystem {
                    package: Box::new(module.package().clone()),
                    error: Box::new(PackageLoadError::io(
                        "read module",
                        canonical_path.clone(),
                        error,
                    )),
                }
            })?,
        };
        let unlinked = parse_unlinked_module_interface(
            canonical_path.to_string_lossy(),
            module_label(&module),
            &source,
        );
        let mut edges = BTreeMap::new();
        for import in unlinked.imports {
            let dependency = match resolve_import(self.packages, &module, &import.specifier)
                .map_err(|failure| LocalProjectLoadError::Import {
                    module: Box::new(module.clone()),
                    specifier: import.specifier.clone(),
                    origin: import.span,
                    code: failure.code,
                    reason: failure.reason,
                })? {
                ResolvedImport::Standard => continue,
                ResolvedImport::Module(dependency) => dependency,
            };
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
