use seseragi_driver::{compile_local_package, render_terminal_diagnostics, ProjectCompileError};
use std::path::Path;

pub(super) fn run_package(path: &Path) -> Result<i32, String> {
    let package = seseragi_project::load_package(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let compiled = match compile_local_package(&package) {
        Ok(compiled) => compiled,
        Err(error) => {
            if let (Some(module_path), ProjectCompileError::Diagnostics { diagnostics, .. }) =
                (error.module(), error.error())
            {
                let module = package
                    .module(module_path)
                    .expect("compiler diagnostic module came from the loaded package");
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
    seseragi_runtime::run_local_package(&compiled)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}
