use std::fs;
use std::path::Path;

pub(crate) fn check_typescript_runtime_package(
    root: &Path,
    abi: &serde_json::Value,
) -> Result<(), String> {
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
