use crate::{
    logical_module_id, parse_manifest, resolve_module_import, Manifest, ModuleGraph,
    ModuleGraphError, ModuleIdentity, ModuleImportResolution, ModulePath, ModuleRoot,
    PackageIdentity, PackageSourceIdentity, SourceImportError, IMPLEMENTED_LANGUAGE_VERSION,
};
use semver::Version;
use seseragi_syntax::{parse_unlinked_module_interface, ByteSpan};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualSourceFile {
    root: ModuleRoot,
    path: String,
    source: String,
}

impl VirtualSourceFile {
    pub fn new(path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            root: ModuleRoot::Source,
            path: path.into(),
            source: source.into(),
        }
    }

    /// A path relative to the generated module root, displayed under `gen/`.
    pub fn generated(path: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            root: ModuleRoot::Generated,
            path: path.into(),
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VirtualPackageModule {
    identity: ModuleIdentity,
    source_path: String,
    source: String,
}

impl VirtualPackageModule {
    pub const fn identity(&self) -> &ModuleIdentity {
        &self.identity
    }

    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedVirtualPackage {
    manifest: Manifest,
    identity: PackageIdentity,
    entry: ModuleIdentity,
    graph: ModuleGraph<ModuleIdentity>,
    modules: BTreeMap<ModuleIdentity, VirtualPackageModule>,
}

impl LoadedVirtualPackage {
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub const fn identity(&self) -> &PackageIdentity {
        &self.identity
    }

    pub const fn entry(&self) -> &ModuleIdentity {
        &self.entry
    }

    pub const fn graph(&self) -> &ModuleGraph<ModuleIdentity> {
        &self.graph
    }

    pub fn modules(&self) -> impl Iterator<Item = (&ModuleIdentity, &VirtualPackageModule)> {
        self.modules.iter()
    }

    pub fn module(&self, identity: &ModuleIdentity) -> Option<&VirtualPackageModule> {
        self.modules.get(identity)
    }
}

pub fn load_virtual_package(
    workspace: impl Into<String>,
    manifest_source: &str,
    files: impl IntoIterator<Item = VirtualSourceFile>,
) -> Result<LoadedVirtualPackage, VirtualPackageLoadError> {
    let manifest = parse_manifest(manifest_source)
        .map_err(|error| VirtualPackageLoadError::Manifest(error.to_string()))?;
    let implemented = Version::parse(IMPLEMENTED_LANGUAGE_VERSION)
        .expect("implemented language version is valid SemVer");
    if !manifest.package.language.matches(&implemented) {
        return Err(VirtualPackageLoadError::UnsupportedLanguageVersion {
            requirement: manifest.package.language.as_str().to_owned(),
            implemented: IMPLEMENTED_LANGUAGE_VERSION.to_owned(),
        });
    }
    let entry_path = manifest
        .run
        .as_ref()
        .ok_or(VirtualPackageLoadError::MissingRunEntry)?
        .entry
        .clone();
    let identity = PackageIdentity::new(
        manifest.package.name.clone(),
        manifest.package.version.clone(),
        PackageSourceIdentity::virtual_workspace(workspace)
            .expect("virtual workspace identity was checked by its constructor"),
    );

    let mut sources = BTreeMap::<(ModuleRoot, ModulePath), (String, String)>::new();
    let mut displayed_paths = BTreeSet::new();
    let mut logical_paths = BTreeSet::new();
    for file in files {
        let module = source_module_path(&file.path)?;
        let canonical_path = format!("{}.ssrg", module.as_str());
        if canonical_path != file.path {
            return Err(VirtualPackageLoadError::NonCanonicalSourcePath {
                path: file.path,
                canonical: canonical_path,
            });
        }
        let displayed_path = if file.root == ModuleRoot::Generated {
            format!("gen/{canonical_path}")
        } else {
            canonical_path
        };
        // Logical IDs currently share one package namespace. Diagnose both ID
        // and editor-path collisions before either map can overwrite a module.
        if !logical_paths.insert(module.clone()) || !displayed_paths.insert(displayed_path.clone())
        {
            return Err(VirtualPackageLoadError::DuplicateModule {
                module: source_module_path(&displayed_path)?,
            });
        }
        sources.insert((file.root, module), (displayed_path, file.source));
    }
    if sources.is_empty() {
        return Err(VirtualPackageLoadError::EmptySourceRoot);
    }
    if !sources.contains_key(&(ModuleRoot::Source, entry_path.clone())) {
        return Err(VirtualPackageLoadError::MissingEntry {
            entry: entry_path.clone(),
        });
    }

    let available = sources.keys().cloned().collect::<BTreeSet<_>>();
    let mut graph = ModuleGraph::new();
    let mut modules = BTreeMap::new();
    for ((root, path), (source_path, source)) in sources {
        let module = ModuleIdentity::new(identity.clone(), root, path.clone());
        let diagnostic_module = source_module_path(&source_path)?;
        let unlinked =
            parse_unlinked_module_interface(&source_path, logical_module_id(&module), &source);
        let mut dependencies = BTreeMap::new();
        for import in unlinked.imports {
            match resolve_module_import(root, &path, &import.specifier).map_err(|error| {
                VirtualPackageLoadError::Import {
                    module: diagnostic_module.clone(),
                    specifier: import.specifier.clone(),
                    origin: import.span,
                    error,
                }
            })? {
                ModuleImportResolution::Standard => {}
                ModuleImportResolution::Local {
                    root,
                    path: dependency,
                } => {
                    if !available.contains(&(root, dependency.clone())) {
                        return Err(VirtualPackageLoadError::MissingModule {
                            module: diagnostic_module.clone(),
                            specifier: import.specifier,
                            origin: import.span,
                            dependency,
                        });
                    }
                    dependencies.insert(
                        import.specifier,
                        ModuleIdentity::new(identity.clone(), root, dependency),
                    );
                }
            }
        }
        graph
            .add_module(module.clone(), dependencies)
            .map_err(VirtualPackageLoadError::Graph)?;
        modules.insert(
            module.clone(),
            VirtualPackageModule {
                identity: module,
                source_path,
                source,
            },
        );
    }
    graph
        .topological_order()
        .map_err(VirtualPackageLoadError::Graph)?;
    let entry = ModuleIdentity::new(identity.clone(), ModuleRoot::Source, entry_path);
    Ok(LoadedVirtualPackage {
        manifest,
        identity,
        entry,
        graph,
        modules,
    })
}

fn source_module_path(path: &str) -> Result<ModulePath, VirtualPackageLoadError> {
    let Some(module) = path.strip_suffix(".ssrg") else {
        return Err(VirtualPackageLoadError::InvalidSourcePath {
            path: path.to_owned(),
            reason: "source path must end in `.ssrg`".to_owned(),
        });
    };
    ModulePath::parse(module).map_err(|error| VirtualPackageLoadError::InvalidSourcePath {
        path: path.to_owned(),
        reason: error.to_string(),
    })
}

#[derive(Debug)]
pub enum VirtualPackageLoadError {
    Manifest(String),
    UnsupportedLanguageVersion {
        requirement: String,
        implemented: String,
    },
    MissingRunEntry,
    EmptySourceRoot,
    InvalidSourcePath {
        path: String,
        reason: String,
    },
    NonCanonicalSourcePath {
        path: String,
        canonical: String,
    },
    DuplicateModule {
        module: ModulePath,
    },
    MissingEntry {
        entry: ModulePath,
    },
    Import {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        error: SourceImportError,
    },
    MissingModule {
        module: ModulePath,
        specifier: String,
        origin: ByteSpan,
        dependency: ModulePath,
    },
    Graph(ModuleGraphError<ModuleIdentity>),
}

impl fmt::Display for VirtualPackageLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(error) => write!(formatter, "invalid virtual package manifest: {error}"),
            Self::UnsupportedLanguageVersion {
                requirement,
                implemented,
            } => write!(
                formatter,
                "package language requirement `{requirement}` does not include `{implemented}`"
            ),
            Self::MissingRunEntry => formatter.write_str("virtual package requires a run entry"),
            Self::EmptySourceRoot => formatter.write_str("virtual package source root is empty"),
            Self::InvalidSourcePath { path, reason } => {
                write!(formatter, "virtual source `{path}` is invalid: {reason}")
            }
            Self::NonCanonicalSourcePath { path, canonical } => write!(
                formatter,
                "virtual source `{path}` must use canonical path `{canonical}`"
            ),
            Self::DuplicateModule { module } => {
                write!(
                    formatter,
                    "virtual package contains duplicate module `{}`",
                    module.as_str()
                )
            }
            Self::MissingEntry { entry } => write!(
                formatter,
                "virtual package entry `{}` is not present in the source root",
                entry.as_str()
            ),
            Self::Import {
                module,
                specifier,
                error,
                ..
            } => write!(
                formatter,
                "module `{}` cannot resolve import `{specifier}`: {error}",
                module.as_str()
            ),
            Self::MissingModule {
                module,
                specifier,
                dependency,
                ..
            } => write!(
                formatter,
                "module `{}` import `{specifier}` resolves to missing module `{}`",
                module.as_str(),
                dependency.as_str()
            ),
            Self::Graph(error) => write!(formatter, "invalid virtual package graph: {error:?}"),
        }
    }
}

impl std::error::Error for VirtualPackageLoadError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn manifest() -> &'static str {
        concat!(
            "[package]\n",
            "name = \"sample/project\"\n",
            "version = \"1.0.0\"\n",
            "language = \"^0.1.0\"\n\n",
            "[run]\n",
            "entry = \"main\"\n",
        )
    }

    #[test]
    fn loads_a_manifest_backed_in_memory_package() {
        let project = load_virtual_package(
            "playground",
            manifest(),
            [
                VirtualSourceFile::new("feature/value.ssrg", "pub let value: Int = 42\n"),
                VirtualSourceFile::new(
                    "main.ssrg",
                    "import { value } from \"./feature/value\"\npub let answer = value\n",
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            logical_module_id(project.entry()),
            "sample/project@1.0.0::main"
        );
        assert_eq!(project.modules().count(), 2);
        let order = project.graph().topological_order().unwrap();
        assert_eq!(order[0].path().as_str(), "feature/value");
        assert_eq!(order[1], *project.entry());
    }

    #[test]
    fn resolves_generated_modules_and_relative_generated_dependencies() {
        let project = load_virtual_package(
            "playground",
            manifest(),
            [
                VirtualSourceFile::new(
                    "main.ssrg",
                    "import { value } from \"gen/api\"\npub let answer = value\n",
                ),
                VirtualSourceFile::generated(
                    "api.ssrg",
                    "import { value } from \"./model\"\npub let answer = value\n",
                ),
                VirtualSourceFile::generated("model.ssrg", "pub let value = 42\n"),
            ],
        )
        .unwrap();
        let dependencies = project
            .graph()
            .dependencies_for(project.entry())
            .unwrap()
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        let api = &dependencies["gen/api"];
        assert_eq!(api.root(), ModuleRoot::Generated);
        assert_eq!(project.module(api).unwrap().source_path(), "gen/api.ssrg");
        let dependencies = project
            .graph()
            .dependencies_for(api)
            .unwrap()
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        assert_eq!(dependencies["./model"].root(), ModuleRoot::Generated);
    }

    #[test]
    fn rejects_generated_namespace_and_display_path_collisions() {
        for handwritten in ["api.ssrg", "gen/api.ssrg"] {
            let error = load_virtual_package(
                "playground",
                manifest(),
                [
                    VirtualSourceFile::new("main.ssrg", "pub let main = 0\n"),
                    VirtualSourceFile::new(handwritten, "pub let value = 1\n"),
                    VirtualSourceFile::generated("api.ssrg", "pub let value = 2\n"),
                ],
            )
            .unwrap_err();
            assert!(
                matches!(error, VirtualPackageLoadError::DuplicateModule { .. }),
                "{error}"
            );
        }
    }

    #[test]
    fn preserves_generated_import_boundaries_and_diagnostic_paths() {
        for specifier in ["self/main", "gen/model", "../model"] {
            let error = load_virtual_package(
                "playground",
                manifest(),
                [
                    VirtualSourceFile::new("main.ssrg", "pub let main = 0\n"),
                    VirtualSourceFile::generated(
                        "api.ssrg",
                        format!("import {{ value }} from \"{specifier}\"\n"),
                    ),
                ],
            )
            .unwrap_err();
            match error {
                VirtualPackageLoadError::Import { module, .. } => {
                    assert_eq!(module.as_str(), "gen/api")
                }
                other => panic!("unexpected error: {other}"),
            }
        }
    }

    #[test]
    fn rejects_noncanonical_paths_and_missing_local_modules() {
        assert!(matches!(
            load_virtual_package(
                "playground",
                manifest(),
                [VirtualSourceFile::new(
                    "cafe\u{301}.ssrg",
                    "pub let value = 1\n"
                )],
            ),
            Err(VirtualPackageLoadError::NonCanonicalSourcePath { .. })
                | Err(VirtualPackageLoadError::MissingEntry { .. })
        ));

        let error = load_virtual_package(
            "playground",
            manifest(),
            [VirtualSourceFile::new(
                "main.ssrg",
                "import { value } from \"./missing\"\n",
            )],
        )
        .unwrap_err();
        assert!(matches!(
            error,
            VirtualPackageLoadError::MissingModule { .. }
        ));
    }

    #[test]
    fn matches_filesystem_module_identity_and_import_edges() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "seseragi-logical-project-parity-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("src/feature")).unwrap();
        fs::write(root.join("seseragi.toml"), manifest()).unwrap();
        fs::write(root.join("src/feature/value.ssrg"), "pub let value = 42\n").unwrap();
        fs::write(
            root.join("src/main.ssrg"),
            "import { value } from \"./feature/value\"\npub let answer = value\n",
        )
        .unwrap();

        let filesystem = crate::load_local_project(&root).unwrap();
        let virtualized = load_virtual_package(
            "playground",
            manifest(),
            [
                VirtualSourceFile::new("feature/value.ssrg", "pub let value = 42\n"),
                VirtualSourceFile::new(
                    "main.ssrg",
                    "import { value } from \"./feature/value\"\npub let answer = value\n",
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            logical_module_id(filesystem.entry()),
            logical_module_id(virtualized.entry())
        );
        let filesystem_modules = filesystem
            .modules()
            .map(|(module, _)| logical_module_id(module))
            .collect::<Vec<_>>();
        let virtual_modules = virtualized
            .modules()
            .map(|(module, _)| logical_module_id(module))
            .collect::<Vec<_>>();
        assert_eq!(filesystem_modules, virtual_modules);
        let filesystem_edges = filesystem
            .graph()
            .dependencies_for(filesystem.entry())
            .unwrap()
            .into_iter()
            .map(|(specifier, module)| (specifier, logical_module_id(&module)))
            .collect::<Vec<_>>();
        let virtual_edges = virtualized
            .graph()
            .dependencies_for(virtualized.entry())
            .unwrap()
            .into_iter()
            .map(|(specifier, module)| (specifier, logical_module_id(&module)))
            .collect::<Vec<_>>();
        assert_eq!(filesystem_edges, virtual_edges);

        fs::remove_dir_all(root).unwrap();
    }
}
