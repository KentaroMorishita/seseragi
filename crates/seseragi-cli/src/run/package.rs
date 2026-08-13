use crate::local_project::{compile_path, LocalProjectCompilation};
use seseragi_project::ProjectCommand;
use std::path::Path;

pub(super) fn run_package(path: &Path) -> Result<i32, String> {
    let compiled = match compile_path(path, ProjectCommand::Run, None)? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => return Ok(2),
    };
    seseragi_runtime::run_local_project(&compiled.compiled)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}
