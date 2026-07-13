mod error;
mod model;

#[cfg(test)]
mod tests;

pub use error::LocalPackageGraphError;
pub use model::{LocalPackageGraph, LocalPackageManifest};

use crate::{
    parse_manifest, ManifestDependency, ModuleGraph, PackageIdentity, PackageName,
    PackageSourceIdentity, IMPLEMENTED_LANGUAGE_VERSION,
};
use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_local_package_graph(
    root: impl AsRef<Path>,
) -> Result<LocalPackageGraph, LocalPackageGraphError> {
    let root = canonical_directory(root.as_ref())?;
    let mut state = DiscoveryState::default();
    let root_identity = state.discover(root)?;
    let mut graph = ModuleGraph::new();
    for (package, dependencies) in state.edges {
        graph
            .add_module(package, dependencies)
            .map_err(|error| LocalPackageGraphError::Graph(Box::new(error)))?;
    }
    graph
        .topological_order()
        .map_err(|error| LocalPackageGraphError::Graph(Box::new(error)))?;
    Ok(LocalPackageGraph::new(root_identity, graph, state.packages))
}

#[derive(Default)]
struct DiscoveryState {
    roots: BTreeMap<PathBuf, PackageIdentity>,
    sources: BTreeMap<(PackageName, Version), PackageIdentity>,
    packages: BTreeMap<PackageIdentity, LocalPackageManifest>,
    edges: BTreeMap<PackageIdentity, BTreeMap<String, PackageIdentity>>,
}

impl DiscoveryState {
    fn discover(&mut self, root: PathBuf) -> Result<PackageIdentity, LocalPackageGraphError> {
        if let Some(identity) = self.roots.get(&root) {
            return Ok(identity.clone());
        }
        let manifest_path = root.join("seseragi.toml");
        let source = fs::read_to_string(&manifest_path).map_err(|source| {
            LocalPackageGraphError::io("read manifest", manifest_path.clone(), source)
        })?;
        let manifest =
            parse_manifest(&source).map_err(|error| LocalPackageGraphError::Manifest {
                path: manifest_path,
                error,
            })?;
        let implemented = Version::parse(IMPLEMENTED_LANGUAGE_VERSION)
            .expect("implemented language version is valid SemVer");
        if !manifest.package.language.matches(&implemented) {
            return Err(LocalPackageGraphError::UnsupportedLanguageVersion {
                package: manifest.package.name.clone(),
                requirement: manifest.package.language.as_str().to_owned(),
                implemented: IMPLEMENTED_LANGUAGE_VERSION.to_owned(),
            });
        }
        let identity = PackageIdentity::new(
            manifest.package.name.clone(),
            manifest.package.version.clone(),
            PackageSourceIdentity::path(root.clone())
                .expect("canonical package roots are absolute"),
        );
        let source_key = (
            manifest.package.name.clone(),
            manifest.package.version.clone(),
        );
        if let Some(first) = self.sources.get(&source_key) {
            if first.source() != identity.source() {
                return Err(LocalPackageGraphError::DependencyConfusion {
                    first: Box::new(first.clone()),
                    second: Box::new(identity),
                });
            }
        } else {
            self.sources.insert(source_key, identity.clone());
        }

        let dependencies = manifest
            .dependencies
            .iter()
            .map(|(key, dependency)| (key.as_str().to_owned(), dependency.clone()))
            .collect::<Vec<_>>();
        self.roots.insert(root.clone(), identity.clone());
        self.packages.insert(
            identity.clone(),
            LocalPackageManifest::new(root.clone(), identity.clone(), manifest),
        );

        let mut edges = BTreeMap::new();
        for (key, dependency) in dependencies {
            let (expected, dependency_root) = match dependency {
                ManifestDependency::Path { package, path } => {
                    (package, canonical_directory(&root.join(path.as_str()))?)
                }
                ManifestDependency::Registry { .. } => {
                    return Err(LocalPackageGraphError::RegistryDependencyUnsupported {
                        package: Box::new(identity.clone()),
                        key,
                    });
                }
            };
            let target = self.discover(dependency_root.clone())?;
            if target.name() != &expected {
                return Err(LocalPackageGraphError::DependencyNameMismatch {
                    package: Box::new(identity.clone()),
                    key,
                    expected,
                    actual: target.name().clone(),
                    dependency_root,
                });
            }
            edges.insert(key, target);
        }
        self.edges.insert(identity.clone(), edges);
        Ok(identity)
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, LocalPackageGraphError> {
    let canonical = fs::canonicalize(path)
        .map_err(|source| LocalPackageGraphError::io("canonicalize package root", path, source))?;
    if !canonical.is_dir() {
        return Err(LocalPackageGraphError::NotDirectory(canonical));
    }
    Ok(canonical)
}
