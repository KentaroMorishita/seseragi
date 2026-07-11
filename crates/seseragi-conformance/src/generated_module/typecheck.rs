use crate::runtime_stage::stage_runtime;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_generated_typescript(
    root: &Path,
    case: &Path,
    typescript_ir: &Value,
    typescript: &str,
) -> Result<(), String> {
    if !has_type_imports(typescript_ir)? {
        return Ok(());
    }

    let case_name = case
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "generated module case has no directory name".to_owned())?;
    let typecheck_dir = root
        .join("target/seseragi-conformance/generated-typecheck")
        .join(case_name);
    reset_dir(&typecheck_dir)?;
    fs::write(typecheck_dir.join("main.ts"), typescript)
        .map_err(|error| format!("failed to stage generated main.ts for type-check: {error}"))?;
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
        .arg("main.ts")
        .current_dir(&typecheck_dir)
        .output()
        .map_err(|error| format!("failed to type-check generated module {case_name}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "generated TypeScript type-check failed for {case_name}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn has_type_imports(typescript_ir: &Value) -> Result<bool, String> {
    match typescript_ir.get("typeImports") {
        None => Ok(false),
        Some(imports) => imports
            .as_array()
            .map(|imports| !imports.is_empty())
            .ok_or_else(|| "TypeScriptIr typeImports must be an array".to_owned()),
    }
}

fn reset_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("failed to reset generated type-check directory: {error}"))?;
    }
    fs::create_dir_all(path)
        .map_err(|error| format!("failed to create generated type-check directory: {error}"))
}

#[cfg(test)]
mod tests {
    use super::has_type_imports;

    #[test]
    fn skips_modules_without_type_imports() {
        assert_eq!(has_type_imports(&serde_json::json!({})), Ok(false));
        assert_eq!(
            has_type_imports(&serde_json::json!({ "typeImports": [] })),
            Ok(false)
        );
    }

    #[test]
    fn selects_modules_with_type_imports() {
        assert_eq!(
            has_type_imports(&serde_json::json!({
                "typeImports": [{
                    "feature": "effect.stdin.error",
                    "local": "StdinError"
                }]
            })),
            Ok(true)
        );
    }

    #[test]
    fn rejects_malformed_type_imports() {
        assert_eq!(
            has_type_imports(&serde_json::json!({ "typeImports": {} })),
            Err("TypeScriptIr typeImports must be an array".to_owned())
        );
    }
}
