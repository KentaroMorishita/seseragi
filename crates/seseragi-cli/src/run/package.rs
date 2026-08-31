use crate::local_project::{compile_path, LocalProjectCompilation};
use crate::run::Invocation;
use seseragi_project::ProjectCommand;
use std::path::Path;

pub(super) fn run_package(path: &Path, invocation: &Invocation) -> Result<i32, String> {
    let compiled = match compile_path(
        path,
        ProjectCommand::Run,
        invocation.target(),
        invocation.diagnostic_format(),
    )? {
        LocalProjectCompilation::Compiled(compiled) => compiled,
        LocalProjectCompilation::Diagnostics => return Ok(2),
    };
    let options = invocation.apply(compiled.process_run_options)?;
    seseragi_runtime::run_local_project_in_directory_with_options(&compiled.compiled, path, options)
        .map(|outcome| outcome.exit_code)
        .map_err(|error| error.to_string())
}
