use seseragi_driver::{
    compile_local_project, compile_local_project_with_providers, render_terminal_diagnostics,
    LinkedCompileError, ProjectCompileError,
};
use seseragi_project::{select_project_target, ProjectCommand, ProjectTarget};
use seseragi_runtime::{
    project_main_contract, validate_target, ExecutionTarget, ProcessRunOptions, ProcessSignalMode,
};
use std::path::Path;

pub(crate) fn containing_package(path: &Path) -> Option<std::path::PathBuf> {
    let mut directory = if path.is_dir() { path } else { path.parent()? };
    loop {
        if directory.join("seseragi.toml").is_file() {
            return Some(directory.to_owned());
        }
        directory = directory.parent()?;
    }
}

pub(crate) enum LocalProjectCompilation {
    Compiled(ResolvedLocalProject),
    Diagnostics,
}

pub(crate) struct ResolvedLocalProject {
    pub compiled: seseragi_driver::CompiledLocalProject,
    pub target: ProjectTarget,
    pub process_run_options: ProcessRunOptions,
}

pub(crate) fn compile_path(
    path: &Path,
    command: ProjectCommand,
    invocation_target: Option<ProjectTarget>,
) -> Result<LocalProjectCompilation, String> {
    compile_path_inner(path, command, invocation_target, true)
}

pub(crate) fn compile_path_unlocked(
    path: &Path,
    command: ProjectCommand,
    invocation_target: Option<ProjectTarget>,
) -> Result<LocalProjectCompilation, String> {
    compile_path_inner(path, command, invocation_target, false)
}

fn compile_path_inner(
    path: &Path,
    command: ProjectCommand,
    invocation_target: Option<ProjectTarget>,
    validate_lock: bool,
) -> Result<LocalProjectCompilation, String> {
    let lockfile = if !validate_lock {
        Ok(None)
    } else if command == ProjectCommand::Dev {
        seseragi_project::read_and_validate_development_lockfile(path).map(Some)
    } else {
        seseragi_project::read_and_validate_lockfile(path).map(Some)
    };
    let lockfile = lockfile.map_err(|error| format!("{}: {error}", error.code()))?;
    let project = seseragi_project::load_local_project(path)
        .map_err(|error| format!("{}: {error}", error.code()))?;
    let process_run_options = project_run_options(&project);
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
    let (result, provider_target) = match selection.target {
        ProjectTarget::Process => {
            let configuration = seseragi_runtime::bun_process_provider_configuration()?;
            let provider_target = configuration.context.target.clone();
            let resolved = compile_local_project_with_providers(&project, configuration);
            let result = match resolved {
                Err(error)
                    if matches!(
                        error.error(),
                        ProjectCompileError::Provider { diagnostic }
                            if diagnostic.code == "SES-K0203"
                                && !diagnostic
                                    .details
                                    .reasons
                                    .iter()
                                    .any(|reason| reason == "standard-module-target")
                    ) =>
                {
                    compile_local_project(&project)
                }
                Ok(compiled) => Ok(compiled),
                other => other,
            };
            (result, provider_target)
        }
        ProjectTarget::Web => {
            let configuration = seseragi_runtime::browser_provider_configuration()?;
            let provider_target = configuration.context.target.clone();
            (
                compile_local_project_with_providers(&project, configuration),
                provider_target,
            )
        }
    };
    Ok(match render_compile_result(&project, result)? {
        Some(compiled) => {
            if let Some(lockfile) = lockfile {
                let expected = lockfile
                    .providers
                    .into_iter()
                    .filter(|provider| provider.target == provider_target)
                    .collect::<Vec<_>>();
                let actual = compiled
                    .compiled
                    .provider_resolution
                    .as_ref()
                    .map(|resolution| resolution.lock.project_lock_selections())
                    .unwrap_or_default();
                if expected != actual {
                    return Err(
                        "SES-K0102: seseragi.lock is stale: provider selection metadata changed; run `seseragi lock update` explicitly"
                            .to_owned(),
                    );
                }
            }
            LocalProjectCompilation::Compiled(ResolvedLocalProject {
                compiled,
                target: selection.target,
                process_run_options,
            })
        }
        None => LocalProjectCompilation::Diagnostics,
    })
}

fn project_run_options(project: &seseragi_project::LoadedLocalProject) -> ProcessRunOptions {
    let run = project
        .packages()
        .package(project.packages().root())
        .and_then(|package| package.manifest().run.as_ref());
    let Some(run) = run else {
        return ProcessRunOptions::default();
    };
    ProcessRunOptions {
        signal_mode: match run.signal_mode {
            seseragi_project::SignalMode::Cancel => ProcessSignalMode::Cancel,
            seseragi_project::SignalMode::Forward => ProcessSignalMode::Forward,
        },
        shutdown_grace_ms: run.shutdown_grace_ms.unwrap_or(10_000),
    }
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
            if let ProjectCompileError::Provider { diagnostic } = error.error() {
                let origin = diagnostic.trace.as_ref().map_or_else(String::new, |trace| {
                    format!("\n  at {}:{}..{}", trace.source, trace.start, trace.end)
                });
                let compatible = if diagnostic.details.compatible_targets.is_empty() {
                    String::new()
                } else {
                    format!(
                        "\n  compatible targets: {}",
                        diagnostic.details.compatible_targets.join(", ")
                    )
                };
                let target = diagnostic
                    .details
                    .target
                    .as_ref()
                    .map_or_else(String::new, |target| format!("\n  target: {target}"));
                return Err(format!(
                    "{} {}: {}{}{}{}",
                    diagnostic.code,
                    diagnostic.label,
                    diagnostic.message,
                    origin,
                    target,
                    compatible
                ));
            }
            Err(format!(
                "project compiler rejected package: {:?}",
                error.error()
            ))
        }
    }
}
