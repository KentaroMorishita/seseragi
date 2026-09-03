use super::{
    entry_source, finish_run, prepare_directory, run_target, CapturedRunOutcome, ProcessRunOptions,
    RunError, RunOutcome,
};
use crate::{project_main_contract, validate_target, ExecutionTarget, HostService};
use seseragi_driver::{
    CompiledLocalPackage, CompiledLocalProject, CompiledLocalTests, CompiledProject,
    ForeignHostDirectory,
};
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
    run_compiled_project(&package.compiled, &package.entry_module, &[], options, None)
}

/// Runs a manifest-selected entry from a compiled multi-package local project.
pub fn run_local_project(project: &CompiledLocalProject) -> Result<RunOutcome, RunError> {
    run_local_project_with_options(project, ProcessRunOptions::default())
}

pub fn run_local_project_with_options(
    project: &CompiledLocalProject,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    run_compiled_project(
        &project.compiled,
        &project.entry_module,
        &project.foreign_host_directories,
        options,
        None,
    )
}

pub fn run_local_project_in_directory_with_options(
    project: &CompiledLocalProject,
    application_directory: &Path,
    options: ProcessRunOptions,
) -> Result<RunOutcome, RunError> {
    let application_directory = absolute_application_directory(application_directory)?;
    run_compiled_project(
        &project.compiled,
        &project.entry_module,
        &project.foreign_host_directories,
        options,
        Some(&application_directory),
    )
}

/// Runs one synthetic documentation entry with captured output and only the
/// deterministic built-in Console/Logger services.
pub fn run_document_entry_in_directory(
    compiled: &CompiledProject,
    entry_module: &str,
    application_directory: &Path,
) -> Result<CapturedRunOutcome, RunError> {
    let contract = project_main_contract(compiled, entry_module).map_err(RunError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(RunError::TargetMismatch)?;
    if let Some(binding) = contract
        .environment
        .iter()
        .find(|binding| !matches!(binding.service, HostService::Console | HostService::Logger))
    {
        return Err(RunError::InvalidEntry(format!(
            "documentation tests do not provide the {:?} service",
            binding.service
        )));
    }
    let application_directory = absolute_application_directory(application_directory)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = (|| {
        stage_project_modules(compiled, &directory).map_err(RunError::Host)?;
        crate::stage_typescript_package(&directory).map_err(RunError::Host)?;
        let entry = compiled
            .modules
            .get(entry_module)
            .expect("entry was validated");
        let entry_path = canonical_output_path(&entry.generated.metadata.outputs.typescript)
            .map_err(RunError::Host)?;
        fs::write(
            directory.join("entry.ts"),
            entry_source(
                &contract,
                &format!("./{}", entry_path.to_string_lossy()),
                None,
                ProcessRunOptions {
                    random_seed: super::RandomSeed::Fixed(0),
                    ..ProcessRunOptions::default()
                },
            ),
        )
        .map_err(|error| RunError::Host(format!("failed to stage documentation entry: {error}")))?;
        super::run_target_captured(&directory, &application_directory)
    })();
    let cleanup = fs::remove_dir_all(&directory)
        .map_err(|error| RunError::Host(format!("failed to clean execution directory: {error}")));
    match (result, cleanup) {
        (Ok(outcome), Ok(())) => Ok(outcome),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestRunOptions {
    pub filter: Option<String>,
    pub exact: Option<String>,
    pub jobs: usize,
    pub timeout_ms: u64,
    pub cleanup_grace_ms: u64,
    pub seed: i64,
}

/// Runs compiled test modules through the embedded deterministic test host.
pub fn run_local_tests_in_directory(
    project: &CompiledLocalTests,
    application_directory: &Path,
    options: &TestRunOptions,
) -> Result<RunOutcome, RunError> {
    let application_directory = absolute_application_directory(application_directory)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = (|| {
        stage_project_modules(&project.compiled, &directory).map_err(RunError::Host)?;
        crate::stage_typescript_package(&directory).map_err(RunError::Host)?;
        fs::write(
            directory.join("entry.ts"),
            test_entry_source(project, options)?,
        )
        .map_err(|error| RunError::Host(format!("failed to stage test entry: {error}")))?;
        run_target(&directory, Some(&application_directory))
    })();
    finish_run(result, &directory)
}

fn test_entry_source(
    project: &CompiledLocalTests,
    options: &TestRunOptions,
) -> Result<String, RunError> {
    let mut source = String::from("import { runTestModules } from \"@seseragi/runtime/test\";\nimport { processHashSeed } from \"@seseragi/runtime/hash\";\nprocessHashSeed();\n");
    let mut modules = Vec::new();
    for (index, test) in project.test_modules.iter().enumerate() {
        let module = project
            .compiled
            .modules
            .get(&test.module_id)
            .ok_or_else(|| RunError::Host(format!("compiled tests omitted {}", test.module_id)))?;
        let path = canonical_output_path(&module.generated.metadata.outputs.typescript)
            .map_err(RunError::Host)?;
        source.push_str(&format!(
            "const {{ tests: tests{index} }} = await import(\"./{}\");\n",
            path.to_string_lossy()
        ));
        modules.push(serde_json::json!({
            "name": test.name,
            "binding": format!("tests{index}"),
        }));
    }
    let module_source = modules
        .iter()
        .map(|module| {
            let name =
                serde_json::to_string(&module["name"]).expect("test module name is JSON encodable");
            let binding = module["binding"].as_str().expect("binding is a string");
            format!("{{ name: {name}, tests: {binding} }}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let filter = serde_json::to_string(&options.filter)
        .map_err(|error| RunError::Host(format!("failed to encode test filter: {error}")))?;
    let exact = serde_json::to_string(&options.exact)
        .map_err(|error| RunError::Host(format!("failed to encode exact test name: {error}")))?;
    source.push_str(&format!(
        "const exitCode = await runTestModules([{module_source}], {{ filter: {filter} ?? undefined, exact: {exact} ?? undefined, jobs: {}, timeoutMs: {}, cleanupGraceMs: {}, seed: {} }});\nprocess.exitCode = exitCode;\n",
        options.jobs,
        options.timeout_ms,
        options.cleanup_grace_ms,
        options.seed,
    ));
    Ok(source)
}

fn absolute_application_directory(directory: &Path) -> Result<PathBuf, RunError> {
    if directory.is_absolute() {
        return Ok(directory.to_owned());
    }
    std::env::current_dir()
        .map(|current| current.join(directory))
        .map_err(|error| {
            RunError::Host(format!(
                "failed to resolve application working directory: {error}"
            ))
        })
}

fn run_compiled_project(
    compiled: &CompiledProject,
    entry_module: &str,
    foreign_host_directories: &[ForeignHostDirectory],
    options: ProcessRunOptions,
    application_directory: Option<&Path>,
) -> Result<RunOutcome, RunError> {
    let contract = project_main_contract(compiled, entry_module).map_err(RunError::InvalidEntry)?;
    validate_target(&contract, ExecutionTarget::Process).map_err(RunError::TargetMismatch)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = run_in_directory(
        compiled,
        entry_module,
        foreign_host_directories,
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
    foreign_host_directories: &[ForeignHostDirectory],
    contract: &crate::MainContract,
    directory: &Path,
    options: ProcessRunOptions,
    application_directory: Option<&Path>,
) -> Result<RunOutcome, RunError> {
    stage_project_modules(compiled, directory).map_err(RunError::Host)?;
    stage_foreign_host_directories(foreign_host_directories, directory).map_err(RunError::Host)?;
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

pub(super) fn stage_foreign_host_directories(
    directories: &[ForeignHostDirectory],
    staging: &Path,
) -> Result<(), String> {
    let mut targets = std::collections::BTreeMap::<PathBuf, PathBuf>::new();
    for directory in directories {
        let package_root = fs::canonicalize(&directory.package_root).map_err(|error| {
            format!(
                "failed to resolve foreign host package root {}: {error}",
                directory.package_root.display()
            )
        })?;
        let source = fs::canonicalize(&directory.source).map_err(|error| {
            format!(
                "failed to resolve foreign host directory {}: {error}",
                directory.source.display()
            )
        })?;
        if !source.starts_with(&package_root) {
            return Err(format!(
                "foreign host source must stay inside package root: {}",
                directory.source.display()
            ));
        }
        for required in &directory.required_files {
            let resolved = fs::canonicalize(required).map_err(|error| {
                format!(
                    "failed to resolve declared foreign host input {}: {error}",
                    required.display()
                )
            })?;
            if !resolved.starts_with(&package_root) || !resolved.is_file() {
                return Err(format!(
                    "declared foreign host input must be a file inside package root: {}",
                    required.display()
                ));
            }
        }
        if directory.target.as_os_str().is_empty()
            || directory.target.is_absolute()
            || directory
                .target
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(format!(
                "foreign host target must be a canonical relative path: {}",
                directory.target.display()
            ));
        }
        if let Some(existing) = targets.insert(directory.target.clone(), source.clone()) {
            if existing != source {
                return Err(format!(
                    "foreign host target {} resolves from multiple source directories",
                    directory.target.display()
                ));
            }
        }
        copy_host_directory(&source, &staging.join(&directory.target))?;
    }
    Ok(())
}

fn copy_host_directory(source: &Path, target: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(source).map_err(|error| {
        format!(
            "failed to inspect foreign host directory {}: {error}",
            source.display()
        )
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(format!(
            "foreign host source must be a real directory: {}",
            source.display()
        ));
    }
    fs::create_dir_all(target).map_err(|error| {
        format!(
            "failed to create foreign host directory {}: {error}",
            target.display()
        )
    })?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| format!("failed to read foreign host directory: {error}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read foreign host entry: {error}"))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect foreign host entry: {error}"))?;
        if file_type.is_symlink() {
            return Err(format!(
                "foreign host package must not contain symlinks: {}",
                entry.path().display()
            ));
        }
        let destination = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_host_directory(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            if destination.exists() {
                return Err(format!(
                    "foreign host file would overwrite staged output: {}",
                    destination.display()
                ));
            }
            fs::copy(entry.path(), &destination).map_err(|error| {
                format!(
                    "failed to stage foreign host file {}: {error}",
                    entry.path().display()
                )
            })?;
        }
    }
    Ok(())
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
