//! Filesystem discovery for one local package.
//!
//! This layer resolves physical files to logical module paths. It deliberately
//! stops before compiler orchestration and generated-output planning so CLI,
//! LSP, and browser adapters can share the same project model.

pub(crate) mod audit;
mod error;
pub(crate) mod filesystem;
mod model;

#[cfg(test)]
mod tests;

pub use error::PackageLoadError;
pub use model::{LoadedModule, LoadedPackage};

use crate::{
    classify_specifier, parse_manifest, resolve_relative_specifier, ImportSpecifier, ModuleGraph,
    ModuleIdentity, ModulePath, ModuleRoot, PackageIdentity, PackageSourceIdentity,
};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub const IMPLEMENTED_LANGUAGE_VERSION: &str = "0.1.0";

pub fn load_package(root: impl AsRef<Path>) -> Result<LoadedPackage, PackageLoadError> {
    let root = filesystem::canonical_directory(root.as_ref(), "package root")?;
    let manifest_path = root.join("seseragi.toml");
    let manifest_source = fs::read_to_string(&manifest_path)
        .map_err(|source| PackageLoadError::io("read manifest", manifest_path.clone(), source))?;
    let manifest =
        parse_manifest(&manifest_source).map_err(|error| PackageLoadError::Manifest {
            path: manifest_path,
            error,
        })?;
    let implemented = semver::Version::parse(IMPLEMENTED_LANGUAGE_VERSION)
        .expect("implemented language version is valid SemVer");
    if !manifest.package.language.matches(&implemented) {
        return Err(PackageLoadError::UnsupportedLanguageVersion {
            requirement: manifest.package.language.as_str().to_owned(),
            implemented: IMPLEMENTED_LANGUAGE_VERSION.to_owned(),
        });
    }
    let entry = manifest
        .run
        .as_ref()
        .ok_or(PackageLoadError::MissingRunEntry)?
        .entry
        .clone();
    let source_root = filesystem::resolve_source_root(&root, &manifest.layout.source)?;
    audit::audit_source_root(&source_root)?;
    let package_identity = PackageIdentity::new(
        manifest.package.name.clone(),
        manifest.package.version.clone(),
        PackageSourceIdentity::path(root.clone()).expect("a canonical package root is absolute"),
    );

    let mut pending = BTreeSet::from([entry.clone()]);
    let mut modules = BTreeMap::new();
    let mut dependencies = BTreeMap::<ModulePath, BTreeMap<String, ModulePath>>::new();
    let mut physical_owners = BTreeMap::new();

    while let Some(path) = pending.pop_first() {
        if modules.contains_key(&path) {
            continue;
        }
        let source_path = filesystem::resolve_module_file(&source_root, &path)?;
        let canonical_path = fs::canonicalize(&source_path).map_err(|source| {
            PackageLoadError::io("canonicalize module", source_path.clone(), source)
        })?;
        if !canonical_path.starts_with(&source_root) {
            return Err(PackageLoadError::RootEscape {
                path: source_path,
                canonical_path,
            });
        }
        if let Some(first) = physical_owners.insert(canonical_path.clone(), path.clone()) {
            return Err(PackageLoadError::DuplicatePhysicalModule {
                first,
                second: path,
                canonical_path,
            });
        }
        let source = fs::read_to_string(&canonical_path)
            .map_err(|error| PackageLoadError::io("read module", canonical_path.clone(), error))?;
        let unlinked = parse_unlinked_module_interface(
            canonical_path.to_string_lossy(),
            path.as_str(),
            &source,
        );
        let mut edges = BTreeMap::new();
        for import in unlinked.imports {
            if crate::is_standard_module(&import.specifier) {
                continue;
            }
            let dependency =
                resolve_import(&path, &import.specifier).map_err(|error| match error {
                    ResolveImportError::Invalid(reason) => PackageLoadError::InvalidImport {
                        module: path.clone(),
                        specifier: import.specifier.clone(),
                        origin: import.span,
                        reason,
                    },
                    ResolveImportError::Unsupported(kind) => PackageLoadError::UnsupportedImport {
                        module: path.clone(),
                        specifier: import.specifier.clone(),
                        origin: import.span,
                        kind,
                    },
                })?;
            edges.insert(import.specifier, dependency.clone());
            if !modules.contains_key(&dependency) {
                pending.insert(dependency);
            }
        }
        dependencies.insert(path.clone(), edges);
        modules.insert(
            path.clone(),
            LoadedModule::new(
                ModuleIdentity::new(package_identity.clone(), ModuleRoot::Source, path),
                canonical_path,
                source,
            ),
        );
    }

    let mut graph = ModuleGraph::new();
    for (module, edges) in dependencies {
        graph
            .add_module(module, edges)
            .map_err(PackageLoadError::Graph)?;
    }
    graph.topological_order().map_err(PackageLoadError::Graph)?;

    Ok(LoadedPackage::new(
        root,
        source_root,
        manifest,
        package_identity,
        entry,
        graph,
        modules,
    ))
}

fn resolve_import(current: &ModulePath, specifier: &str) -> Result<ModulePath, ResolveImportError> {
    let kind = classify_specifier(specifier)
        .map_err(|error| ResolveImportError::Invalid(error.to_string()))?;
    match &kind {
        ImportSpecifier::Relative(value) => resolve_relative_specifier(current, value)
            .map_err(|error| ResolveImportError::Invalid(error.to_string())),
        ImportSpecifier::SelfPackage(value) => {
            ModulePath::parse(value).map_err(|error| ResolveImportError::Invalid(error.to_string()))
        }
        _ => Err(ResolveImportError::Unsupported(kind)),
    }
}

enum ResolveImportError {
    Invalid(String),
    Unsupported(ImportSpecifier),
}
