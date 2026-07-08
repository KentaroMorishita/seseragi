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

fn check_typescript_runtime_package(root: &Path, abi: &serde_json::Value) -> Result<(), String> {
    if abi
        .pointer("/targetFamily")
        .and_then(|value| value.as_str())
        != Some("typescript")
    {
        return Ok(());
    }

    let package_path = root.join("runtime/ts/package.json");
    let package_raw = fs::read_to_string(&package_path)
        .map_err(|error| format!("failed to read TypeScript runtime package: {error}"))?;
    let package: serde_json::Value = serde_json::from_str(&package_raw)
        .map_err(|error| format!("failed to parse TypeScript runtime package: {error}"))?;

    if package.get("name").and_then(|value| value.as_str()) != Some("@seseragi/runtime") {
        return Err("TypeScript runtime package name must be @seseragi/runtime".to_owned());
    }

    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    for feature in features {
        if feature.get("kind").and_then(|value| value.as_str()) == Some("runtime-helper") {
            check_typescript_runtime_helper(root, &package, feature)?;
        }
    }
    Ok(())
}

fn check_typescript_runtime_helper(
    root: &Path,
    package: &serde_json::Value,
    feature: &serde_json::Value,
) -> Result<(), String> {
    let id = feature
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "runtime helper feature id must be a string".to_owned())?;
    let import = feature
        .get("import")
        .ok_or_else(|| format!("runtime helper {id} import is missing"))?;
    let module = import
        .get("module")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime helper {id} import.module must be a string"))?;
    let export_name = import
        .get("export")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime helper {id} import.export must be a string"))?;

    let subpath = runtime_package_subpath(module)
        .ok_or_else(|| format!("runtime helper {id} imports unexpected module {module}"))?;
    let source_path = package_export_source(package, &subpath)
        .ok_or_else(|| format!("runtime helper {id} import {module} is not exported"))?;
    let source = fs::read_to_string(root.join("runtime/ts").join(source_path))
        .map_err(|error| format!("failed to read TypeScript runtime helper {module}: {error}"))?;

    if !source_exports_name(&source, export_name) {
        return Err(format!(
            "runtime helper {id} import {module} does not export {export_name}"
        ));
    }
    Ok(())
}

fn runtime_package_subpath(module: &str) -> Option<String> {
    module
        .strip_prefix("@seseragi/runtime")
        .map(|subpath| match subpath {
            "" => ".".to_owned(),
            subpath => format!("./{}", subpath.strip_prefix('/').unwrap_or(subpath)),
        })
}

fn package_export_source(package: &serde_json::Value, subpath: &str) -> Option<String> {
    let export = package.get("exports")?.get(subpath)?;
    match export {
        serde_json::Value::String(path) => Some(path.clone()),
        serde_json::Value::Object(map) => map
            .get("default")
            .or_else(|| map.get("import"))
            .or_else(|| map.get("types"))
            .and_then(|value| value.as_str())
            .map(str::to_owned),
        _ => None,
    }
}

fn source_exports_name(source: &str, name: &str) -> bool {
    source.contains(&format!("export function {name}"))
        || source.contains(&format!("export const {name}"))
        || source.contains(&format!("export {{ {name}"))
        || source.contains(&format!(", {name}"))
}
