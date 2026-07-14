use seseragi_driver::{compile_local_project, render_terminal_diagnostics, ProjectCompileError};
use std::path::Path;

pub(super) fn run_package(path: &Path) -> Result<i32, String> {
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let compiled = match compile_local_project(&project) {
        Ok(compiled) => compiled,
        Err(error) => {
            if let (Some(module_path), ProjectCompileError::Diagnostics { diagnostics, .. }) =
                (error.module(), error.error())
            {
                let module = project
                    .module(module_path)
                    .expect("compiler diagnostic module came from the loaded project");
                eprint!(
                    "{}",
                    render_terminal_diagnostics(diagnostics, module.source())
                );
                return Ok(2);
            }
            return Err(format!(
                "project compiler rejected package: {:?}",
                error.error()
            ));
        }
    };
    seseragi_runtime::run_local_project(&compiled)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}
