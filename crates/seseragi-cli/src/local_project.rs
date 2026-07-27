use seseragi_driver::{
    compile_local_project, render_terminal_diagnostics, LinkedCompileError, ProjectCompileError,
};
use std::path::Path;

pub(crate) enum LocalProjectCompilation {
    Compiled(seseragi_driver::CompiledLocalProject),
    Diagnostics,
}

pub(crate) fn compile_path(path: &Path) -> Result<LocalProjectCompilation, String> {
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    match compile_local_project(&project) {
        Ok(compiled) => Ok(LocalProjectCompilation::Compiled(compiled)),
        Err(error) => {
            let diagnostics = match error.error() {
                ProjectCompileError::Diagnostics { diagnostics, .. }
                | ProjectCompileError::Compile {
                    error: LinkedCompileError::Diagnostics(diagnostics),
                    ..
                } => Some(diagnostics),
                _ => None,
            };
            if let (Some(module_path), Some(diagnostics)) = (error.module(), diagnostics) {
                let module = project
                    .module(module_path)
                    .expect("compiler diagnostic module came from the loaded project");
                eprint!(
                    "{}",
                    render_terminal_diagnostics(diagnostics, module.source())
                );
                return Ok(LocalProjectCompilation::Diagnostics);
            }
            Err(format!(
                "project compiler rejected package: {:?}",
                error.error()
            ))
        }
    }
}
