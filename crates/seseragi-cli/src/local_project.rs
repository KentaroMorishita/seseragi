use seseragi_driver::{
    compile_local_project, compile_local_project_with_providers, render_terminal_diagnostics,
    LinkedCompileError, ProjectCompileError,
};
use std::path::Path;

pub(crate) enum LocalProjectCompilation {
    Compiled(seseragi_driver::CompiledLocalProject),
    Diagnostics,
}

pub(crate) enum LocalProjectTarget {
    BunProcess,
    Web,
}

pub(crate) fn compile_path(
    path: &Path,
    target: LocalProjectTarget,
) -> Result<LocalProjectCompilation, String> {
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let result = match target {
        LocalProjectTarget::BunProcess => {
            let resolved = compile_local_project_with_providers(
                &project,
                seseragi_runtime::bun_process_provider_configuration()?,
            );
            match resolved {
                Err(error)
                    if matches!(
                        error.error(),
                        ProjectCompileError::Provider { diagnostic }
                            if diagnostic.code == "SES-K0203"
                    ) =>
                {
                    compile_local_project(&project)
                }
                other => other,
            }
        }
        LocalProjectTarget::Web => compile_local_project(&project),
    };
    match result {
        Ok(compiled) => Ok(LocalProjectCompilation::Compiled(compiled)),
        Err(error) => {
            let diagnostics = match error.error() {
                ProjectCompileError::Diagnostics { modules } => {
                    modules.first().map(|diagnostics| &diagnostics.diagnostics)
                }
                ProjectCompileError::Compile {
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
