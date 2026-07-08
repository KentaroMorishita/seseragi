use crate::runtime_package::check_typescript_runtime_package;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_runtime_abi_case(root: &Path, case: &Path) -> Result<(), String> {
    let abi_path = case.join("abi.json");
    let raw = fs::read_to_string(&abi_path)
        .map_err(|error| format!("failed to read runtime ABI: {error}"))?;
    let abi: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse runtime ABI: {error}"))?;

    check_runtime_abi_envelope(&abi)?;
    check_runtime_abi_features(&abi)?;
    check_typescript_runtime_package(root, &abi)
}

fn check_runtime_abi_envelope(abi: &serde_json::Value) -> Result<(), String> {
    if abi.get("schema") != Some(&serde_json::Value::from(1)) {
        return Err("runtime ABI must use schema 1".to_owned());
    }
    if abi.pointer("/identity").and_then(|value| value.as_str()) != Some("@seseragi/runtime") {
        return Err("runtime ABI identity must be @seseragi/runtime".to_owned());
    }
    if abi.pointer("/abiMajor").and_then(|value| value.as_u64()) != Some(1) {
        return Err("runtime ABI major must be 1".to_owned());
    }
    if abi
        .pointer("/targetFamily")
        .and_then(|value| value.as_str())
        != Some("typescript")
    {
        return Err("runtime ABI targetFamily must be typescript".to_owned());
    }
    Ok(())
}

fn check_runtime_abi_features(abi: &serde_json::Value) -> Result<(), String> {
    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    let mut seen = BTreeSet::new();
    for feature in features {
        check_runtime_abi_feature(feature, &mut seen)?;
    }
    Ok(())
}

fn check_runtime_abi_feature(
    feature: &serde_json::Value,
    seen: &mut BTreeSet<String>,
) -> Result<(), String> {
    let id = feature
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "runtime ABI feature id must be a string".to_owned())?;
    if !seen.insert(id.to_owned()) {
        return Err(format!("runtime ABI feature id is duplicated: {id}"));
    }
    let kind = feature
        .get("kind")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime ABI feature {id} kind must be a string"))?;
    if !matches!(kind, "value-representation" | "runtime-helper") {
        return Err(format!("runtime ABI feature {id} kind is not supported"));
    }
    feature
        .get("typescript")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime ABI feature {id} typescript must be a string"))?;
    feature
        .get("boundary")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime ABI feature {id} boundary must be a string"))?;

    check_runtime_abi_feature_import(id, kind, feature.get("import"))
}

fn check_runtime_abi_feature_import(
    id: &str,
    kind: &str,
    import: Option<&serde_json::Value>,
) -> Result<(), String> {
    match (kind, import) {
        ("runtime-helper", Some(import)) => {
            import
                .get("module")
                .and_then(|value| value.as_str())
                .ok_or_else(|| format!("runtime helper {id} import.module must be a string"))?;
            import
                .get("export")
                .and_then(|value| value.as_str())
                .ok_or_else(|| format!("runtime helper {id} import.export must be a string"))?;
        }
        (_, Some(serde_json::Value::Null)) => {}
        (_, None) => return Err(format!("runtime ABI feature {id} import is missing")),
        _ => {
            return Err(format!(
                "runtime ABI feature {id} may only have structured import for runtime-helper"
            ));
        }
    }
    Ok(())
}
