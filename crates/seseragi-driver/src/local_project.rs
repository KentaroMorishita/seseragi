use crate::{compile_project, CompiledProject, ProjectCompileError, ProjectModuleInput};
use seseragi_project::{LoadedLocalProject, ModuleGraph, ModuleIdentity, PackageIdentity};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledLocalProject {
    pub compiled: CompiledProject,
    pub entry_module: String,
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
    let mut graph = ModuleGraph::new();
    let mut identities_by_id = BTreeMap::new();
    for (identity, _) in project.modules() {
        let module = module_id(identity);
        identities_by_id.insert(module.clone(), identity.clone());
        let dependencies = project
            .graph()
            .dependencies_for(identity)
            .expect("loaded local project graph contains every source module")
            .into_iter()
            .map(|(specifier, dependency)| (specifier, module_id(&dependency)));
        graph
            .add_module(module, dependencies)
            .expect("loaded local project graph was already validated");
    }
    let inputs = project.modules().map(|(identity, module)| {
        ProjectModuleInput::new(
            module.source_path().to_string_lossy(),
            module_id(identity),
            module.source(),
            output_path(identity),
        )
        .with_package_scope(package_scope(identity.package()))
    });
    let compiled = compile_project(graph, inputs).map_err(|error| LocalProjectCompileError {
        module: error_module(&error)
            .and_then(|module| identities_by_id.get(module).cloned())
            .map(Box::new),
        error: Box::new(error),
    })?;
    Ok(CompiledLocalProject {
        entry_module: module_id(project.entry()),
        compiled,
    })
}

fn module_id(identity: &ModuleIdentity) -> String {
    format!(
        "{}::{}",
        package_scope(identity.package()),
        identity.path().as_str()
    )
}

fn package_scope(package: &PackageIdentity) -> String {
    format!("{}@{}", package.name().as_str(), package.version())
}

fn output_path(identity: &ModuleIdentity) -> String {
    format!(
        "dist/packages/{}/{}/{}.js",
        identity.package().name().as_str(),
        identity.package().version(),
        identity.path().as_str()
    )
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
}
