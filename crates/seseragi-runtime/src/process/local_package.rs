use super::{
    entry_source, finish_run, prepare_directory, run_target, ProcessRunOptions, RunError,
    RunOutcome,
};
use crate::{project_main_contract, validate_target, ExecutionTarget};
use seseragi_driver::{CompiledLocalPackage, CompiledLocalProject, CompiledProject};
use std::fs;
use std::path::{Path, PathBuf};

/// Runs the manifest-selected entry from a compiled local package with the
/// same process Console and Stdin adapters as single-file execution.
pub fn run_local_package(package: &CompiledLocalPackage) -> Result<RunOutcome, RunError> {
    run_local_package_with_options(package, ProcessRunOptions::default())
}

pub fn run_local_package_with_options(
    package: &CompiledLocalPackage,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    run_compiled_project(&package.compiled, &package.entry_module, options, None)
}

/// Runs a manifest-selected entry from a compiled multi-package local project.
pub fn run_local_project(project: &CompiledLocalProject) -> Result<RunOutcome, RunError> {
    run_local_project_with_options(project, ProcessRunOptions::default())
}

pub fn run_local_project_with_options(
    project: &CompiledLocalProject,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    run_compiled_project(&project.compiled, &project.entry_module, options, None)
}

pub fn run_local_project_in_directory_with_options(
    project: &CompiledLocalProject,
    application_directory: &Path,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    run_compiled_project(
        &project.compiled,
        &project.entry_module,
        options,
        Some(application_directory),
    )
}

fn run_compiled_project(
    compiled: &CompiledProject,
    entry_module: &str,
    options: ProcessRunOptions,
    application_directory: Option<&Path>,
) -> Result<RunOutcome, RunError> {
    let contract = project_main_contract(compiled, entry_module).map_err(RunError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(RunError::TargetMismatch)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = run_in_directory(
        compiled,
        entry_module,
        &contract,
        &directory,
        options,
        application_directory,
    );
    finish_run(result, &directory)
}

fn run_in_directory(
    compiled: &CompiledProject,
    entry_module: &str,
    contract: &crate::MainContract,
    directory: &Path,
    options: ProcessRunOptions,
    application_directory: Option<&Path>,
) -> Result<RunOutcome, RunError> {
    stage_project_modules(compiled, directory).map_err(RunError::Host)?;
    crate::stage_typescript_package(directory).map_err(RunError::Host)?;
    let entry = compiled
        .modules
        .get(entry_module)
        .expect("entry was validated");
    let entry_path = canonical_output_path(&entry.generated.metadata.outputs.typescript)
        .map_err(RunError::Host)?;
    let entry_specifier = format!("./{}", entry_path.to_string_lossy());
    fs::write(
        directory.join("entry.ts"),
        entry_source(
            contract,
            &entry_specifier,
            compiled.provider_resolution.as_ref(),
            options,
        ),
    )
    .map_err(|error| RunError::Host(format!("failed to stage runtime entry: {error}")))?;
    run_target(directory, application_directory)
}

pub(super) fn stage_project_modules(
    compiled: &CompiledProject,
    directory: &Path,
) -> Result<(), String> {
    for module_id in &compiled.order {
        let module = compiled
            .modules
            .get(module_id)
            .ok_or_else(|| format!("compiled package omitted {module_id}"))?;
        let relative = canonical_output_path(&module.generated.metadata.outputs.typescript)?;
        let target = directory.join(&relative);
        let parent = target
            .parent()
            .ok_or_else(|| format!("generated output has no parent: {}", relative.display()))?;
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create generated module directory {}: {error}",
                parent.display()
            )
        })?;
        fs::write(&target, &module.generated.typescript).map_err(|error| {
            format!(
                "failed to stage generated module {}: {error}",
                target.display()
            )
        })?;
    }
    Ok(())
}

pub(super) fn canonical_output_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
        || !value.ends_with(".ts")
    {
        return Err(format!(
            "generated package output must be a canonical relative TypeScript path: {value}"
        ));
    }
    Ok(path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::canonical_output_path;
    use std::path::PathBuf;

    #[test]
    fn validates_staged_project_output_paths() {
        assert_eq!(
            canonical_output_path("dist/domain.ts"),
            Ok(PathBuf::from("dist/domain.ts"))
        );
        for invalid in [
            "",
            "../main.ts",
            "/tmp/main.ts",
            "dist\\main.ts",
            "dist/main.js",
        ] {
            assert!(canonical_output_path(invalid).is_err(), "{invalid}");
        }
    }
}
