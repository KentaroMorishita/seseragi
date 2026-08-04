//! Reachable source discovery for an editor workspace without a package
//! manifest.
//!
//! A workspace project deliberately follows only the source modules reachable
//! from the document being analyzed. This lets editor clients share the
//! package layer's path and specifier rules without treating every `.ssrg`
//! file in an opened folder as part of one implicit program.

use crate::{
    classify_specifier, is_standard_module, resolve_relative_specifier, ImportSpecifier,
    ModuleGraph, ModuleGraphError, ModulePath,
};
use seseragi_syntax::{parse_unlinked_module_interface, ByteSpan};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// An in-memory source buffer that shadows one workspace file while a client
/// is editing it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceOverlay {
    path: PathBuf,
    source: String,
}

impl SourceOverlay {
    pub fn new(path: impl Into<PathBuf>, source: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            source: source.into(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceModule {
    path: ModulePath,
    source_path: PathBuf,
    source: String,
}

impl WorkspaceModule {
    pub const fn path(&self) -> &ModulePath {
        &self.path
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

/// A closed, reachable graph rooted at one file inside a workspace folder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedWorkspaceProject {
    root: PathBuf,
    entry: ModulePath,
    graph: ModuleGraph<ModulePath>,
    modules: BTreeMap<ModulePath, WorkspaceModule>,
}

impl LoadedWorkspaceProject {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn entry(&self) -> &ModulePath {
        &self.entry
    }

    pub const fn graph(&self) -> &ModuleGraph<ModulePath> {
        &self.graph
    }

    pub fn modules(&self) -> impl Iterator<Item = (&ModulePath, &WorkspaceModule)> {
        self.modules.iter()
    }

    pub fn module(&self, path: &ModulePath) -> Option<&WorkspaceModule> {
        self.modules.get(path)
    }
}

/// Discovers source modules reachable through relative and `self/` imports.
///
/// Standard modules remain compiler-owned link targets. Package imports need
/// a `seseragi.toml` project and are therefore rejected here instead of being
/// guessed from a workspace folder.
pub fn load_workspace_project(
    root: impl AsRef<Path>,
    entry_path: impl AsRef<Path>,
    overlays: impl IntoIterator<Item = SourceOverlay>,
) -> Result<LoadedWorkspaceProject, WorkspaceProjectLoadError> {
    let root = canonical_directory(root.as_ref(), "workspace root")?;
    let overlays = normalize_overlays(&root, overlays)?;
    let entry_path = normalize_path(&root, entry_path.as_ref())?;
    if !entry_path.starts_with(&root) {
        return Err(WorkspaceProjectLoadError::RootEscape {
            path: entry_path.clone(),
            canonical_path: entry_path,
        });
    }
    let entry = module_path_for(&root, &entry_path)?;
    let mut pending = BTreeSet::from([entry.clone()]);
    let mut modules = BTreeMap::new();
    let mut dependencies = BTreeMap::<ModulePath, BTreeMap<String, ModulePath>>::new();

    while let Some(path) = pending.pop_first() {
        if modules.contains_key(&path) {
            continue;
        }
        let source_path = module_file_path(&root, &path);
        let source_path = normalize_path(&root, &source_path)?;
        if !source_path.starts_with(&root) {
            return Err(WorkspaceProjectLoadError::RootEscape {
                path: module_file_path(&root, &path),
                canonical_path: source_path,
            });
        }
        let actual_path = module_path_for(&root, &source_path)?;
        if actual_path != path {
            return Err(WorkspaceProjectLoadError::PathMismatch {
                expected: path,
                actual: actual_path,
                path: source_path,
            });
        }
        let source = match overlays.get(&source_path) {
            Some(source) => source.clone(),
            None => fs::read_to_string(&source_path).map_err(|source| {
                WorkspaceProjectLoadError::io("read module", source_path.clone(), source)
            })?,
        };
        discover_module(
            &root,
            path,
            source_path,
            source,
            &overlays,
            &mut pending,
            &mut modules,
            &mut dependencies,
        )?;
    }

    let mut graph = ModuleGraph::new();
    for (module, edges) in dependencies {
        graph
            .add_module(module, edges)
            .map_err(WorkspaceProjectLoadError::Graph)?;
    }
    graph
        .topological_order()
        .map_err(WorkspaceProjectLoadError::Graph)?;
    Ok(LoadedWorkspaceProject {
        root,
        entry,
        graph,
        modules,
    })
}

fn discover_module(
    root: &Path,
    path: ModulePath,
    source_path: PathBuf,
    source: String,
    overlays: &BTreeMap<PathBuf, String>,
    pending: &mut BTreeSet<ModulePath>,
    modules: &mut BTreeMap<ModulePath, WorkspaceModule>,
    dependencies: &mut BTreeMap<ModulePath, BTreeMap<String, ModulePath>>,
) -> Result<(), WorkspaceProjectLoadError> {
    let unlinked = parse_unlinked_module_interface(
        source_path.to_string_lossy(),
        format!("workspace/{}", path.as_str()),
        &source,
    );
    let mut edges = BTreeMap::new();
    for import in unlinked.imports {
        if is_standard_module(&import.specifier) {
            continue;
        }
        let dependency = resolve_import(&path, &import.specifier).map_err(|reason| {
            WorkspaceProjectLoadError::Import {
                module: path.clone(),
                specifier: import.specifier.clone(),
                origin: import.span,
                reason,
            }
        })?;
        let candidate = module_file_path(root, &dependency);
        if !candidate.exists() && !overlays.contains_key(&candidate) {
            return Err(WorkspaceProjectLoadError::MissingModule {
                module: path.clone(),
                specifier: import.specifier.clone(),
                origin: import.span,
                path: candidate,
            });
        }
        edges.insert(import.specifier, dependency.clone());
        if !modules.contains_key(&dependency) {
            pending.insert(dependency);
        }
    }
    dependencies.insert(path.clone(), edges);
    modules.insert(
        path.clone(),
        WorkspaceModule {
            path,
            source_path,
            source,
        },
    );
    Ok(())
}

fn resolve_import(current: &ModulePath, specifier: &str) -> Result<ModulePath, String> {
    match classify_specifier(specifier).map_err(|error| error.to_string())? {
        ImportSpecifier::Relative(value) => {
            resolve_relative_specifier(current, &value).map_err(|error| error.to_string())
        }
        ImportSpecifier::SelfPackage(value) => {
            ModulePath::parse(&value).map_err(|error| error.to_string())
        }
        unsupported => Err(format!(
            "workspace source import {unsupported:?} requires seseragi.toml"
        )),
    }
}

fn canonical_directory(
    path: &Path,
    label: &'static str,
) -> Result<PathBuf, WorkspaceProjectLoadError> {
    let canonical = fs::canonicalize(path).map_err(|source| {
        WorkspaceProjectLoadError::io("canonicalize directory", path.into(), source)
    })?;
    if !canonical.is_dir() {
        return Err(WorkspaceProjectLoadError::io(
            label,
            canonical,
            io::Error::new(io::ErrorKind::NotADirectory, "not a directory"),
        ));
    }
    Ok(canonical)
}

fn normalize_overlays(
    root: &Path,
    overlays: impl IntoIterator<Item = SourceOverlay>,
) -> Result<BTreeMap<PathBuf, String>, WorkspaceProjectLoadError> {
    let mut normalized = BTreeMap::new();
    for overlay in overlays {
        let path = normalize_path(root, overlay.path())?;
        if !path.starts_with(root) {
            return Err(WorkspaceProjectLoadError::RootEscape {
                path: overlay.path().to_owned(),
                canonical_path: path,
            });
        }
        normalized.insert(path, overlay.source().to_owned());
    }
    Ok(normalized)
}

fn normalize_path(root: &Path, path: &Path) -> Result<PathBuf, WorkspaceProjectLoadError> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    };
    match fs::canonicalize(&path) {
        Ok(path) => Ok(path),
        Err(source) if source.kind() == io::ErrorKind::NotFound && path.starts_with(root) => {
            Ok(path)
        }
        Err(source) => Err(WorkspaceProjectLoadError::io(
            "canonicalize module",
            path,
            source,
        )),
    }
}

fn module_file_path(root: &Path, module: &ModulePath) -> PathBuf {
    let mut path = root.to_owned();
    for segment in module.as_str().split('/') {
        path.push(segment);
    }
    path.set_extension("ssrg");
    path
}

fn module_path_for(root: &Path, path: &Path) -> Result<ModulePath, WorkspaceProjectLoadError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| WorkspaceProjectLoadError::RootEscape {
            path: path.to_owned(),
            canonical_path: path.to_owned(),
        })?;
    if relative
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("ssrg")
    {
        return Err(WorkspaceProjectLoadError::InvalidModulePath {
            path: path.to_owned(),
            reason: "workspace entry must have a .ssrg extension".to_owned(),
        });
    }
    let mut segments = Vec::new();
    for component in relative.components() {
        let Component::Normal(segment) = component else {
            return Err(WorkspaceProjectLoadError::InvalidModulePath {
                path: path.to_owned(),
                reason: "workspace module path must be a normal relative path".to_owned(),
            });
        };
        let segment =
            segment
                .to_str()
                .ok_or_else(|| WorkspaceProjectLoadError::InvalidModulePath {
                    path: path.to_owned(),
                    reason: "workspace module path must be valid UTF-8".to_owned(),
                })?;
        segments.push(segment.to_owned());
    }
    let Some(last) = segments.last_mut() else {
        return Err(WorkspaceProjectLoadError::InvalidModulePath {
            path: path.to_owned(),
            reason: "workspace module path must not be empty".to_owned(),
        });
    };
    *last = last
        .strip_suffix(".ssrg")
        .expect("extension was checked above")
        .to_owned();
    ModulePath::parse(&segments.join("/")).map_err(|error| {
        WorkspaceProjectLoadError::InvalidModulePath {
            path: path.to_owned(),
            reason: error.to_string(),
        }
    })
}

#[derive(Debug)]
pub enum WorkspaceProjectLoadError {
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    RootEscape {
        path: PathBuf,
        canonical_path: PathBuf,
    },
    InvalidModulePath {
        path: PathBuf,
        reason: String,
    },
    PathMismatch {
        expected: ModulePath,
        actual: ModulePath,
        path: PathBuf,
    },
    Import {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        reason: String,
    },
    MissingModule {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        path: PathBuf,
    },
    Graph(ModuleGraphError<ModulePath>),
}

impl WorkspaceProjectLoadError {
    fn io(action: &'static str, path: PathBuf, source: io::Error) -> Self {
        Self::Io {
            action,
            path,
            source,
        }
    }
}

impl fmt::Display for WorkspaceProjectLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                action,
                path,
                source,
            } => write!(
                formatter,
                "failed to {action} `{}`: {source}",
                path.display()
            ),
            Self::RootEscape {
                path,
                canonical_path,
            } => write!(
                formatter,
                "workspace module `{}` resolves outside the workspace root to `{}`",
                path.display(),
                canonical_path.display()
            ),
            Self::InvalidModulePath { path, reason } => {
                write!(
                    formatter,
                    "workspace module `{}` is invalid: {reason}",
                    path.display()
                )
            }
            Self::PathMismatch {
                expected,
                actual,
                path,
            } => write!(
                formatter,
                "workspace module `{}` is spelled `{}` but resolves to `{}`",
                path.display(),
                expected.as_str(),
                actual.as_str()
            ),
            Self::Import {
                module,
                specifier,
                reason,
                ..
            } => write!(
                formatter,
                "module `{}` cannot resolve import `{specifier}`: {reason}",
                module.as_str()
            ),
            Self::MissingModule {
                module,
                specifier,
                path,
                ..
            } => write!(
                formatter,
                "module `{}` cannot resolve import `{specifier}` at `{}`",
                module.as_str(),
                path.display()
            ),
            Self::Graph(error) => write!(formatter, "invalid workspace source graph: {error:?}"),
        }
    }
}

impl std::error::Error for WorkspaceProjectLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discovers_only_reachable_relative_modules_and_applies_overlays() {
        let project = TempWorkspace::new();
        project.write(
            "main.ssrg",
            "import { increment } from \"./domain\"\npub fn main value: Int -> Int = increment value\n",
        );
        project.write(
            "domain.ssrg",
            "pub fn increment value: Int -> Int = value + 1\n",
        );
        project.write("unrelated.ssrg", "pub let ignored = missing\n");

        let loaded = load_workspace_project(
            project.path(),
            project.path().join("main.ssrg"),
            [SourceOverlay::new(
                project.path().join("domain.ssrg"),
                "pub fn increment value: Int -> Int = value + 2\n",
            )],
        )
        .unwrap();

        assert_eq!(loaded.modules().count(), 2);
        assert_eq!(
            loaded.graph().topological_order().unwrap(),
            [
                ModulePath::parse("domain").unwrap(),
                ModulePath::parse("main").unwrap()
            ]
        );
        assert!(loaded
            .module(&ModulePath::parse("domain").unwrap())
            .unwrap()
            .source()
            .contains("+ 2"));
    }

    struct TempWorkspace {
        path: PathBuf,
    }

    impl TempWorkspace {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "seseragi-workspace-project-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write(&self, relative: &str, source: &str) {
            fs::write(self.path.join(relative), source).unwrap();
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
