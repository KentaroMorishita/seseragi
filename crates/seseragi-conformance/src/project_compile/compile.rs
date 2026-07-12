use super::model::{load_project_compile_case, ProjectCompileCase};
use seseragi_driver::{compile_project, CompiledProject, ProjectModuleInput};
use seseragi_project::ModuleGraph;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompiledProjectCompileCase {
    pub(crate) descriptor: ProjectCompileCase,
    pub(crate) compiled: CompiledProject,
}

/// Runs the normal multi-module compiler pipeline described by `project.json`.
pub(crate) fn compile_project_compile_case(
    case: &Path,
) -> Result<CompiledProjectCompileCase, String> {
    let descriptor = load_project_compile_case(case)?;
    let mut graph = ModuleGraph::new();
    for module in &descriptor.modules {
        graph
            .add_module(
                module.id.clone(),
                module
                    .imports
                    .iter()
                    .map(|import| (import.specifier.clone(), import.module.clone())),
            )
            .map_err(|error| format!("invalid project graph: {error:?}"))?;
    }
    let inputs = descriptor
        .modules
        .iter()
        .map(|module| {
            let source = fs::read_to_string(case.join(&module.source)).map_err(|error| {
                format!("failed to read project source {}: {error}", module.source)
            })?;
            let source_name = stable_source_name(case, &module.source)?;
            Ok(ProjectModuleInput::new(
                source_name,
                module.id.clone(),
                source,
                module.output.clone(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let compiled = compile_project(graph, inputs)
        .map_err(|error| format!("project compiler rejected fixture: {error:?}"))?;
    Ok(CompiledProjectCompileCase {
        descriptor,
        compiled,
    })
}

fn stable_source_name(case: &Path, source: &str) -> Result<String, String> {
    let case_name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "project fixture has no UTF-8 directory name".to_owned())?;
    Ok(format!("project/{case_name}/{source}"))
}
