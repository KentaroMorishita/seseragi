use seseragi_driver::{
    compile_local_project, compile_local_project_with_providers, render_terminal_diagnostics,
    LinkedCompileError, ProjectCompileError,
};
use seseragi_project::{select_project_target, ProjectCommand, ProjectTarget};
use seseragi_runtime::{project_main_contract, validate_target, ExecutionTarget};
use std::path::Path;

pub(crate) enum LocalProjectCompilation {
    Compiled(ResolvedLocalProject),
    Diagnostics,
}

pub(crate) struct ResolvedLocalProject {
    pub compiled: seseragi_driver::CompiledLocalProject,
    pub target: ProjectTarget,
}

pub(crate) fn compile_path(
    path: &Path,
    command: ProjectCommand,
    invocation_target: Option<ProjectTarget>,
) -> Result<LocalProjectCompilation, String> {
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let baseline = match render_compile_result(&project, compile_local_project(&project))? {
        Some(compiled) => compiled,
        None => return Ok(LocalProjectCompilation::Diagnostics),
    };
    let manifest_target = project
        .packages()
        .package(project.packages().root())
        .expect("local project graph contains its root manifest")
        .manifest()
        .run
        .as_ref()
        .and_then(|run| run.target.as_ref());
    let contract = project_main_contract(&baseline.compiled, &baseline.entry_module)
        .map_err(|error| format!("invalid entry point: {error}"))?;
    let compatible_targets = compatible_targets(&contract);
    let infer_from_capabilities = command == ProjectCommand::Build
        && invocation_target.is_none()
        && manifest_target.is_none();
    let selection = select_project_target(
        command,
        invocation_target,
        manifest_target,
        infer_from_capabilities.then_some(compatible_targets.as_slice()),
    )
    .map_err(|error| error.to_string())?;
    let result = match selection.target {
        ProjectTarget::Process => {
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
                Ok(compiled) => Ok(compiled),
                other => other,
            }
        }
        ProjectTarget::Web => compile_local_project_with_providers(
            &project,
            seseragi_runtime::browser_provider_configuration()?,
        ),
    };
    Ok(match render_compile_result(&project, result)? {
        Some(compiled) => LocalProjectCompilation::Compiled(ResolvedLocalProject {
            compiled,
            target: selection.target,
        }),
        None => LocalProjectCompilation::Diagnostics,
    })
}

pub(crate) fn compatible_targets(contract: &seseragi_runtime::MainContract) -> Vec<ProjectTarget> {
    ProjectTarget::ALL
        .into_iter()
        .filter(|target| {
            let target = match target {
                ProjectTarget::Process => ExecutionTarget::Process,
                ProjectTarget::Web => ExecutionTarget::Browser,
            };
            validate_target(contract, target).is_ok()
        })
        .collect()
}

fn render_compile_result(
    project: &seseragi_project::LoadedLocalProject,
    result: Result<
        seseragi_driver::CompiledLocalProject,
        seseragi_driver::LocalProjectCompileError,
    >,
) -> Result<Option<seseragi_driver::CompiledLocalProject>, String> {
    match result {
        Ok(compiled) => Ok(Some(compiled)),
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
                return Ok(None);
            }
            Err(format!(
                "project compiler rejected package: {:?}",
                error.error()
            ))
        }
    }
}
