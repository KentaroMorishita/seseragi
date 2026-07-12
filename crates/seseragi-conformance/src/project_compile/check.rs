use super::compile::compile_project_compile_case;
use super::typecheck::check_project_typescript;
use std::fs;
use std::path::Path;

/// Compares all generated per-module artifacts for one project fixture.
pub(crate) fn check_project_compile_case(root: &Path, case: &Path) -> Result<(), String> {
    let compiled_case = compile_project_compile_case(case)?;
    if compiled_case.compiled.order != compiled_case.descriptor.expected_order {
        return Err(format!(
            "project compile order mismatch: expected {:?}, got {:?}",
            compiled_case.descriptor.expected_order, compiled_case.compiled.order
        ));
    }
    for descriptor in &compiled_case.descriptor.modules {
        let compiled = compiled_case
            .compiled
            .modules
            .get(&descriptor.id)
            .ok_or_else(|| format!("project compiler did not produce module {}", descriptor.id))?;
        let artifacts = case.join(&descriptor.artifacts);
        compare_json(&artifacts.join("typed-hir.json"), &compiled.typed_hir)?;
        compare_json(
            &artifacts.join("typed-interface.json"),
            &compiled.typed_interface,
        )?;
        compare_json(&artifacts.join("core-ir.json"), &compiled.core_ir)?;
        compare_json(
            &artifacts.join("typescript-ir.json"),
            &compiled.typescript_ir,
        )?;
        compare_json(
            &artifacts.join("generated-module.json"),
            &compiled.generated.metadata,
        )?;
        compare_text(&artifacts.join("main.ts"), &compiled.generated.typescript)?;
        compare_json(
            &artifacts.join("main.ts.map"),
            &compiled.generated.source_map,
        )?;
    }
    check_project_typescript(root, case, &compiled_case)?;
    Ok(())
}

fn compare_json<T: serde::Serialize>(path: &Path, actual: &T) -> Result<(), String> {
    let expected_raw = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read project artifact {}: {error}",
            path.display()
        )
    })?;
    let expected: serde_json::Value = serde_json::from_str(&expected_raw).map_err(|error| {
        format!(
            "failed to parse project artifact {}: {error}",
            path.display()
        )
    })?;
    let actual = serde_json::to_value(actual).map_err(|error| {
        format!(
            "failed to encode project artifact {}: {error}",
            path.display()
        )
    })?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("project artifact mismatch: {}", path.display()))
    }
}

fn compare_text(path: &Path, actual: &str) -> Result<(), String> {
    let expected = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read project artifact {}: {error}",
            path.display()
        )
    })?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("project artifact mismatch: {}", path.display()))
    }
}
