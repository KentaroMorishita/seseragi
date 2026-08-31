use crate::{
    compile_project, compile_project_with_providers, CompiledProject, ProjectCompileError,
    ProjectModuleInput, ProjectProviderConfiguration,
};
use seseragi_project::{
    logical_module_id, logical_package_scope, LoadedLocalProject, LoadedLocalTests, ModuleGraph,
    ModuleIdentity, ModuleRoot,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledLocalProject {
    pub compiled: CompiledProject,
    pub entry_module: String,
    pub foreign_host_directories: Vec<ForeignHostDirectory>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForeignHostDirectory {
    pub package_root: PathBuf,
    pub source: PathBuf,
    pub target: PathBuf,
    pub required_files: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledLocalTests {
    pub compiled: CompiledProject,
    pub test_modules: Vec<CompiledTestModule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledTestModule {
    pub name: String,
    pub module_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalTestCompileError {
    Compile(LocalProjectCompileError),
    Discovery { module: String, reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalProjectCompileError {
    module: Option<Box<ModuleIdentity>>,
    error: Box<ProjectCompileError>,
}

impl LocalProjectCompileError {
    pub fn module(&self) -> Option<&ModuleIdentity> {
        match &self.module {
            Some(module) => Some(module.as_ref()),
            None => None,
        }
    }

    pub fn error(&self) -> &ProjectCompileError {
        self.error.as_ref()
    }
}

/// Compiles an already-discovered multi-package local project through the
/// shared linked project pipeline. Package identity and filesystem resolution
/// remain owned by `seseragi-project`.
pub fn compile_local_project(
    project: &LoadedLocalProject,
) -> Result<CompiledLocalProject, LocalProjectCompileError> {
    compile_local_project_inner(project, None)
}

/// Compiles a local project and resolves the entry point's provider
/// requirements against the target toolchain catalog.
pub fn compile_local_project_with_providers(
    project: &LoadedLocalProject,
    mut configuration: ProjectProviderConfiguration,
) -> Result<CompiledLocalProject, LocalProjectCompileError> {
    configuration.entry_module = logical_module_id(project.entry());
    compile_local_project_inner(project, Some(configuration))
}

/// Compiles all test source through the ordinary linked project pipeline, then
/// selects modules with the exact `pub let tests: std/test::Test` export.
pub fn compile_local_tests(
    project: &LoadedLocalTests,
) -> Result<CompiledLocalTests, LocalTestCompileError> {
    let mut graph = ModuleGraph::new();
    let mut identities_by_id = BTreeMap::new();
    for (identity, _) in project.modules() {
        let module = test_aware_module_id(identity);
        identities_by_id.insert(module.clone(), identity.clone());
        let dependencies = project
            .graph()
            .dependencies_for(identity)
            .expect("loaded local test graph contains every module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, test_aware_module_id(&dependency)));
        graph
            .add_module(module, dependencies)
            .expect("loaded local test graph was already validated");
    }
    let inputs = project.modules().map(|(identity, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            test_aware_module_id(identity),
            module.source(),
            test_output_path(identity),
        )
        .with_package_scope(logical_package_scope(identity.package()))
    });
    let compiled = compile_project(graph, inputs).map_err(|error| {
        LocalTestCompileError::Compile(LocalProjectCompileError {
            module: error_module(&error)
                .and_then(|module| identities_by_id.get(module).cloned())
                .map(Box::new),
            error: Box::new(error),
        })
    })?;
    let mut test_modules = Vec::new();
    for identity in project.roots() {
        let module_id = test_aware_module_id(identity);
        let module = compiled
            .modules
            .get(&module_id)
            .expect("compiled test project contains every root");
        let matching = module
            .typed_interface
            .exports
            .iter()
            .filter(|export| export.namespace == "value" && export.name == "tests")
            .collect::<Vec<_>>();
        if matching.is_empty() {
            continue;
        }
        let [export] = matching.as_slice() else {
            return Err(LocalTestCompileError::Discovery {
                module: identity.path().as_str().to_owned(),
                reason: "test module must export exactly one `pub let tests: test.Test`".to_owned(),
            });
        };
        let exact = export.scheme.type_parameters.is_empty()
            && export.scheme.constraints.is_empty()
            && matches!(
                &export.scheme.type_ref,
                seseragi_syntax::InterfaceType::ExternalNamed {
                    canonical,
                    arguments,
                    ..
                } if canonical == "std/test::Test" && arguments.is_empty()
            );
        if !exact {
            return Err(LocalTestCompileError::Discovery {
                module: identity.path().as_str().to_owned(),
                reason: "`tests` must have the exact type `std/test::Test`".to_owned(),
            });
        }
        test_modules.push(CompiledTestModule {
            name: identity.path().as_str().to_owned(),
            module_id,
        });
    }
    Ok(CompiledLocalTests {
        compiled,
        test_modules,
    })
}

fn compile_local_project_inner(
    project: &LoadedLocalProject,
    configuration: Option<ProjectProviderConfiguration>,
) -> Result<CompiledLocalProject, LocalProjectCompileError> {
    let foreign_host_directories = collect_foreign_host_directories(project);
    let mut graph = ModuleGraph::new();
    let mut identities_by_id = BTreeMap::new();
    for (identity, _) in project.modules() {
        let module = logical_module_id(identity);
        identities_by_id.insert(module.clone(), identity.clone());
        let dependencies = project
            .graph()
            .dependencies_for(identity)
            .expect("loaded local project graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, logical_module_id(&dependency)));
        graph
            .add_module(module, dependencies)
            .expect("loaded local project graph was already validated");
    }
    let inputs = project.modules().map(|(identity, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            logical_module_id(identity),
            module.source(),
            output_path(identity),
        )
        .with_package_scope(logical_package_scope(identity.package()))
    });
    let compiled = match configuration {
        Some(configuration) => compile_project_with_providers(graph, inputs, configuration),
        None => compile_project(graph, inputs),
    }
    .map_err(|error| LocalProjectCompileError {
        module: error_module(&error)
            .and_then(|module| identities_by_id.get(module).cloned())
            .map(Box::new),
        error: Box::new(error),
    })?;
    Ok(CompiledLocalProject {
        entry_module: logical_module_id(project.entry()),
        compiled,
        foreign_host_directories,
    })
}

fn collect_foreign_host_directories(project: &LoadedLocalProject) -> Vec<ForeignHostDirectory> {
    let mut directories = BTreeMap::<(PathBuf, PathBuf, PathBuf), BTreeSet<PathBuf>>::new();
    for (identity, module) in project.modules() {
        let Some(package) = project.packages().package(identity.package()) else {
            continue;
        };
        let surface = seseragi_syntax::parse_surface_ast(
            module.source_path().to_string_lossy(),
            module.source(),
        );
        let output = output_path(identity);
        let Some(output_parent) = Path::new(&output).parent() else {
            continue;
        };
        for foreign in surface.foreign_modules {
            if !foreign.specifier.starts_with('.') {
                if foreign.specifier.starts_with("node:") {
                    continue;
                }
                let Some(configuration) = package.manifest().foreign_typescript.as_ref() else {
                    continue;
                };
                let manifest = package.root().join(configuration.manifest.as_str());
                let Some(host_root) = manifest.parent() else {
                    continue;
                };
                let node_modules = host_root.join("node_modules");
                directories
                    .entry((
                        package.root().to_owned(),
                        node_modules,
                        PathBuf::from("dist/packages")
                            .join(identity.package().name().as_str())
                            .join(identity.package().version().to_string())
                            .join("node_modules"),
                    ))
                    .or_default()
                    .extend(foreign_host_input_paths(package));
                continue;
            }
            let source_file = module
                .source_path()
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(&foreign.specifier);
            let Some(source_directory) = source_file.parent() else {
                continue;
            };
            let target_file = normalize_relative_path(&output_parent.join(&foreign.specifier));
            let Some(target_directory) = target_file.parent() else {
                continue;
            };
            let package_relative = normalize_relative_path(
                source_file
                    .strip_prefix(package.root())
                    .unwrap_or(source_file.as_path()),
            );
            let host_root = package_relative.components().next().and_then(|component| {
                let Component::Normal(component) = component else {
                    return None;
                };
                let component = component.to_str()?;
                (!matches!(
                    component,
                    "src" | "test" | "tests" | "benchmarks" | "generated"
                ))
                .then_some(component)
            });
            let (source_directory, target_directory) = if let Some(host_root) = host_root {
                (
                    package.root().join(host_root),
                    PathBuf::from("dist/packages")
                        .join(identity.package().name().as_str())
                        .join(host_root),
                )
            } else {
                (source_directory.to_owned(), target_directory.to_owned())
            };
            directories
                .entry((
                    package.root().to_owned(),
                    source_directory,
                    target_directory,
                ))
                .or_default()
                .extend(foreign_host_input_paths(package));
        }
    }
    directories
        .into_iter()
        .map(
            |((package_root, source, target), required_files)| ForeignHostDirectory {
                package_root,
                source,
                target,
                required_files: required_files.into_iter().collect(),
            },
        )
        .collect()
}

fn foreign_host_input_paths(package: &seseragi_project::LocalPackageManifest) -> Vec<PathBuf> {
    let Some(configuration) = package.manifest().foreign_typescript.as_ref() else {
        return Vec::new();
    };
    let mut inputs = vec![
        package.root().join(configuration.manifest.as_str()),
        package.root().join(configuration.lockfile.as_str()),
    ];
    if let Some(bindings) = &configuration.bindings {
        inputs.push(package.root().join(bindings.as_str()));
    }
    inputs
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir => return PathBuf::new(),
        }
    }
    normalized
}

fn output_path(identity: &ModuleIdentity) -> String {
    format!(
        "dist/packages/{}/{}/{}.js",
        identity.package().name().as_str(),
        identity.package().version(),
        identity.path().as_str()
    )
}

fn test_aware_module_id(identity: &ModuleIdentity) -> String {
    match identity.root() {
        ModuleRoot::Test => format!(
            "{}::test/{}",
            logical_package_scope(identity.package()),
            identity.path().as_str()
        ),
        _ => logical_module_id(identity),
    }
}

fn test_output_path(identity: &ModuleIdentity) -> String {
    let root = match identity.root() {
        ModuleRoot::Test => "tests",
        ModuleRoot::Source => "src",
        ModuleRoot::Benchmark => "benchmarks",
        ModuleRoot::Generated => "generated",
    };
    format!(
        "dist/packages/{}/{}/{}/{}.js",
        identity.package().name().as_str(),
        identity.package().version(),
        root,
        identity.path().as_str()
    )
}

fn error_module(error: &ProjectCompileError) -> Option<&str> {
    match error {
        ProjectCompileError::DuplicateInput { module }
        | ProjectCompileError::UnexpectedInput { module }
        | ProjectCompileError::MissingInput { module }
        | ProjectCompileError::GraphImportMismatch { module, .. }
        | ProjectCompileError::Link { module, .. }
        | ProjectCompileError::LinkTarget { module, .. }
        | ProjectCompileError::OutputPlan { module, .. }
        | ProjectCompileError::Compile { module, .. } => Some(module),
        ProjectCompileError::Provider { diagnostic } => {
            diagnostic.trace.as_ref().map(|trace| trace.module.as_str())
        }
        ProjectCompileError::Diagnostics { modules } => modules
            .first()
            .map(|diagnostics| diagnostics.module.as_str()),
        ProjectCompileError::DuplicateOutputPath { first_module, .. } => Some(first_module),
        ProjectCompileError::Graph(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn compiles_a_path_dependency_through_the_shared_pipeline() {
        let root =
            repository_root().join("examples/spec/fixtures/projects/package-path-dependency-basic");
        let project = seseragi_project::load_local_project(root).unwrap();
        let compiled = compile_local_project(&project).unwrap();

        assert_eq!(
            compiled.compiled.order,
            [
                "fixture/math-basic@1.0.0::lib",
                "fixture/package-path-dependency-basic@0.0.0::main",
            ]
        );
        let main = compiled
            .compiled
            .modules
            .get(&compiled.entry_module)
            .unwrap();
        assert!(main.generated.typescript.contains("export const main"));
        assert!(main
            .generated
            .typescript
            .contains("math-basic/1.0.0/lib.js"));
    }

    #[test]
    fn compiles_the_canonical_std_test_fixture() {
        let root = repository_root().join("examples/spec/fixtures/projects/test-discovery");
        let project = seseragi_project::load_local_tests(root).unwrap();
        let compiled = compile_local_tests(&project).unwrap();

        assert_eq!(compiled.test_modules.len(), 1);
        assert_eq!(compiled.test_modules[0].name, "basic");
        let module = compiled
            .compiled
            .modules
            .get(&compiled.test_modules[0].module_id)
            .unwrap();
        assert!(module.generated.typescript.contains("_ssrg_test_suite"));
        assert!(!module
            .generated
            .typescript
            .contains("export const tests: unknown = _;"));
    }
}
