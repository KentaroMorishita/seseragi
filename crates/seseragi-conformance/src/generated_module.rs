use crate::pipeline::{emit_generated_module, interface_source_name};
use std::fs;
use std::path::Path;

pub(crate) fn check_generated_module(root: &Path, case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_metadata_path = case.join("generated-module.json");
    let expected_typescript_path = case.join("main.ts");
    let expected_source_map_path = case.join("main.ts.map");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected_metadata = fs::read_to_string(&expected_metadata_path)
        .map_err(|error| format!("failed to read expected GeneratedModule: {error}"))?;
    let expected_typescript = fs::read_to_string(&expected_typescript_path)
        .map_err(|error| format!("failed to read expected main.ts: {error}"))?;
    let expected_source_map = fs::read_to_string(&expected_source_map_path)
        .map_err(|error| format!("failed to read expected main.ts.map: {error}"))?;
    let bundle = emit_generated_module(interface_source_name(case)?, &source)?;

    let actual_metadata_value = serde_json::to_value(&bundle.metadata)
        .map_err(|error| format!("failed to encode GeneratedModule: {error}"))?;
    let expected_metadata_value: serde_json::Value = serde_json::from_str(&expected_metadata)
        .map_err(|error| format!("failed to parse expected GeneratedModule: {error}"))?;
    if actual_metadata_value != expected_metadata_value {
        return Err("GeneratedModule artifact mismatch".to_owned());
    }
    check_generated_runtime_requirements(root, &actual_metadata_value)?;
    if bundle.typescript != expected_typescript {
        return Err("main.ts artifact mismatch".to_owned());
    }

    let actual_source_map_value = serde_json::to_value(&bundle.source_map)
        .map_err(|error| format!("failed to encode SourceMap: {error}"))?;
    let expected_source_map_value: serde_json::Value =
        serde_json::from_str(&expected_source_map)
            .map_err(|error| format!("failed to parse expected main.ts.map: {error}"))?;
    if actual_source_map_value != expected_source_map_value {
        return Err("main.ts.map artifact mismatch".to_owned());
    }
    Ok(())
}

fn check_generated_runtime_requirements(
    root: &Path,
    generated_module: &serde_json::Value,
) -> Result<(), String> {
    let abi_path = root.join("examples/spec/artifacts/runtime-schema-1/core/abi.json");
    let abi_raw = fs::read_to_string(&abi_path)
        .map_err(|error| format!("failed to read runtime ABI for generated module: {error}"))?;
    let abi: serde_json::Value = serde_json::from_str(&abi_raw)
        .map_err(|error| format!("failed to parse runtime ABI for generated module: {error}"))?;
    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    let available = features
        .iter()
        .filter_map(|feature| feature.get("id").and_then(|value| value.as_str()))
        .collect::<std::collections::BTreeSet<_>>();
    let requirements = generated_module
        .pointer("/runtime/requirements")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated module runtime.requirements must be an array".to_owned())?;

    for requirement in requirements {
        let requirement = requirement
            .as_str()
            .ok_or_else(|| "generated module runtime requirement must be a string".to_owned())?;
        if !available.contains(requirement) {
            return Err(format!(
                "generated module requires unknown runtime feature {requirement}"
            ));
        }
    }
    Ok(())
}
