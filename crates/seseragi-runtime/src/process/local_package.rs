use super::{entry_source, finish_run, prepare_directory, run_target, RunError, RunOutcome};
use crate::main_contract;
use seseragi_driver::CompiledLocalPackage;
use std::fs;
use std::path::{Path, PathBuf};

/// Runs the manifest-selected entry from a compiled local package with the
/// same process Console and Stdin adapters as single-file execution.
pub fn run_local_package(package: &CompiledLocalPackage) -> Result<RunOutcome, RunError> {
    let entry = package
        .compiled
        .modules
        .get(&package.entry_module)
        .ok_or_else(|| RunError::InvalidEntry("compiled package omitted its entry".to_owned()))?;
    let contract = main_contract(entry).map_err(RunError::InvalidEntry)?;
    let directory = prepare_directory().map_err(RunError::Host)?;
    let result = run_in_directory(package, &contract, &directory);
    finish_run(result, &directory)
}

fn run_in_directory(
    package: &CompiledLocalPackage,
    contract: &crate::MainContract,
    directory: &Path,
) -> Result<RunOutcome, RunError> {
    for module_id in &package.compiled.order {
        let module = package
            .compiled
            .modules
            .get(module_id)
            .ok_or_else(|| RunError::Host(format!("compiled package omitted {module_id}")))?;
        let relative = canonical_output_path(&module.generated.metadata.outputs.typescript)
            .map_err(RunError::Host)?;
        let target = directory.join(&relative);
        let parent = target.parent().ok_or_else(|| {
            RunError::Host(format!(
                "generated output has no parent: {}",
                relative.display()
            ))
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            RunError::Host(format!(
                "failed to create generated module directory {}: {error}",
                parent.display()
            ))
        })?;
        fs::write(&target, &module.generated.typescript).map_err(|error| {
            RunError::Host(format!(
                "failed to stage generated module {}: {error}",
                target.display()
            ))
        })?;
    }
    crate::stage_typescript_package(directory).map_err(RunError::Host)?;
    let entry = package
        .compiled
        .modules
        .get(&package.entry_module)
        .expect("entry was validated");
    let entry_path = canonical_output_path(&entry.generated.metadata.outputs.typescript)
        .map_err(RunError::Host)?;
    let entry_specifier = format!("./{}", entry_path.to_string_lossy());
    fs::write(
        directory.join("entry.ts"),
        entry_source(contract, &entry_specifier),
    )
    .map_err(|error| RunError::Host(format!("failed to stage runtime entry: {error}")))?;
    run_target(directory)
}

fn canonical_output_path(value: &str) -> Result<PathBuf, String> {
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
