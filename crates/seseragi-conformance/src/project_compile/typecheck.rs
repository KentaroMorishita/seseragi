use super::compile::CompiledProjectCompileCase;
use super::stage::stage_project_typescript;
use crate::runtime_stage::stage_runtime;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Type-checks all generated modules after staging them at their planned output
/// paths. This keeps emitted `.js` ESM specifiers intact while TypeScript
/// resolves the corresponding staged `.ts` sources.
pub(super) fn check_project_typescript(
    root: &Path,
    case: &Path,
    compiled_case: &CompiledProjectCompileCase,
) -> Result<(), String> {
    let typecheck_dir = prepare_typecheck_dir(root, case)?;
    let sources = stage_project_typescript(&typecheck_dir, compiled_case)?;
    stage_runtime(root, &typecheck_dir)?;

    let output = Command::new("bunx")
        .arg("tsc")
        .arg("--noEmit")
        .arg("--strict")
        .arg("--target")
        .arg("ES2022")
        .arg("--module")
        .arg("ESNext")
        .arg("--moduleResolution")
        .arg("bundler")
        .arg("--types")
        .arg("node")
        .arg("--allowImportingTsExtensions")
        .args(&sources)
        .current_dir(&typecheck_dir)
        .output()
        .map_err(|error| format!("failed to type-check generated project: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "generated project TypeScript type-check failed for {}\nstdout:\n{}\nstderr:\n{}",
        case.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn prepare_typecheck_dir(root: &Path, case: &Path) -> Result<PathBuf, String> {
    let case_name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "project fixture has no UTF-8 directory name".to_owned())?;
    let typecheck_dir = root
        .join("target/seseragi-conformance/project-generated-typecheck")
        .join(case_name);
    if typecheck_dir.exists() {
        fs::remove_dir_all(&typecheck_dir).map_err(|error| {
            format!(
                "failed to reset generated project type-check directory {}: {error}",
                typecheck_dir.display()
            )
        })?;
    }
    fs::create_dir_all(&typecheck_dir).map_err(|error| {
        format!(
            "failed to create generated project type-check directory {}: {error}",
            typecheck_dir.display()
        )
    })?;
    Ok(typecheck_dir)
}
