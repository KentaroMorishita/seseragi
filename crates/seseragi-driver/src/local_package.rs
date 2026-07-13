use crate::{compile_project, CompiledProject, ProjectCompileError, ProjectModuleInput};
use seseragi_project::{LoadedPackage, ModuleGraph, ModulePath};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledLocalPackage {
    pub compiled: CompiledProject,
    pub entry_module: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalPackageCompileError {
    module: Option<ModulePath>,
    error: Box<ProjectCompileError>,
}

impl LocalPackageCompileError {
    pub const fn module(&self) -> Option<&ModulePath> {
        self.module.as_ref()
    }

    pub fn error(&self) -> &ProjectCompileError {
        self.error.as_ref()
    }
}

/// Compiles one already-discovered local package through the shared linked
/// project pipeline. Filesystem discovery remains in `seseragi-project`; this
/// adapter only assigns opaque compiler IDs and transient ESM output paths.
pub fn compile_local_package(
    package: &LoadedPackage,
) -> Result<CompiledLocalPackage, LocalPackageCompileError> {
    let mut graph = ModuleGraph::new();
    let mut paths_by_id = BTreeMap::new();
    for (path, _) in package.modules() {
        let module = module_id(package, path);
        paths_by_id.insert(module.clone(), path.clone());
        let dependencies = package
            .graph()
            .dependencies_for(path)
            .expect("loaded package graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, module_id(package, &dependency)));
        graph
            .add_module(module, dependencies)
            .expect("loaded package graph was already validated");
    }
    let inputs = package.modules().map(|(path, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            module_id(package, path),
            module.source(),
            format!("dist/{}.js", path.as_str()),
        )
    });
    let compiled = compile_project(graph, inputs).map_err(|error| LocalPackageCompileError {
        module: error_module(&error).and_then(|module| paths_by_id.get(module).cloned()),
        error: Box::new(error),
    })?;
    Ok(CompiledLocalPackage {
        entry_module: module_id(package, package.entry()),
        compiled,
    })
}

fn module_id(package: &LoadedPackage, path: &ModulePath) -> String {
    format!("{}::{}", package.identity().name().as_str(), path.as_str())
}

fn error_module(error: &ProjectCompileError) -> Option<&str> {
    match error {
        ProjectCompileError::DuplicateInput { module }
        | ProjectCompileError::UnexpectedInput { module }
        | ProjectCompileError::MissingInput { module }
        | ProjectCompileError::GraphImportMismatch { module, .. }
        | ProjectCompileError::Diagnostics { module, .. }
        | ProjectCompileError::Link { module, .. }
        | ProjectCompileError::LinkTarget { module, .. }
        | ProjectCompileError::OutputPlan { module, .. }
        | ProjectCompileError::Compile { module, .. } => Some(module),
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
    fn compiles_the_manifest_discovered_split_rps_package() {
        let root = repository_root()
            .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split");
        let package = seseragi_project::load_package(root).unwrap();
        let compiled = compile_local_package(&package).unwrap();

        assert_eq!(
            compiled.compiled.order,
            [
                "fixture/rps-cli-split::domain",
                "fixture/rps-cli-split::input",
                "fixture/rps-cli-split::main",
            ]
        );
        let main = compiled
            .compiled
            .modules
            .get(&compiled.entry_module)
            .unwrap();
        assert!(main.generated.typescript.contains("export const main"));
        assert!(main.generated.typescript.contains("./domain.js"));
        assert!(main.generated.typescript.contains("./input.js"));
    }
}
