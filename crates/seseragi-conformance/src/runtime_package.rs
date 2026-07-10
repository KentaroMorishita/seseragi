use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

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
    package_export_source(&package, ".")
        .ok_or_else(|| "TypeScript runtime package root export is missing".to_owned())?;

    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    for feature in features {
        if feature.get("kind").and_then(|value| value.as_str()) == Some("runtime-helper") {
            check_typescript_runtime_helper(root, &package, feature)?;
        }
    }
    check_typescript_runtime_package_typecheck(root)?;
    if runtime_helper_is_declared(abi, "effect.stdin.readLine") {
        check_typescript_runtime_read_line(root)?;
    }
    if runtime_helper_is_declared(abi, "effect.console.println") {
        check_typescript_runtime_console_is_cold(root)?;
    }
    Ok(())
}

fn runtime_helper_is_declared(abi: &serde_json::Value, id: &str) -> bool {
    abi.get("features")
        .and_then(|value| value.as_array())
        .is_some_and(|features| {
            features.iter().any(|feature| {
                feature.get("kind").and_then(|value| value.as_str()) == Some("runtime-helper")
                    && feature.get("id").and_then(|value| value.as_str()) == Some(id)
            })
        })
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
        || source.contains(&format!("export async function {name}"))
        || source.contains(&format!("export const {name}"))
        || source.contains(&format!("export {{ {name}"))
        || source.contains(&format!(", {name}"))
}

fn check_typescript_runtime_package_typecheck(root: &Path) -> Result<(), String> {
    let output = Command::new("bunx")
        .arg("tsc")
        .arg("-p")
        .arg("runtime/ts/tsconfig.json")
        .arg("--noEmit")
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to type-check TypeScript runtime package: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "TypeScript runtime package type-check failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn check_typescript_runtime_read_line(root: &Path) -> Result<(), String> {
    let mut child = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { readLine } from \"./src/stdin.ts\";\n\
             const effects = [readLine(), readLine(), readLine()];\n\
             const results = [];\n\
             for (const effect of effects) results.push(await run(effect, {}));\n\
             const values = results.map((result) => result.kind === \"success\" ? result.value : null);\n\
             process.stdout.write(JSON.stringify(values));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run TypeScript stdin runtime probe: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to open TypeScript stdin runtime probe input".to_owned())?;
    stdin.write_all(b"first\nsecond\n").map_err(|error| {
        format!("failed to write TypeScript stdin runtime probe input: {error}")
    })?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed to wait for TypeScript stdin runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript stdin runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"[\"first\",\"second\",null]" {
        return Err(format!(
            "TypeScript stdin runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn check_typescript_runtime_console_is_cold(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { run } from \"./src/effect.ts\";\n\
             import { println } from \"./src/console.ts\";\n\
             const effect = println(\"after\");\n\
             process.stdout.write(\"before\\n\");\n\
             await run(effect, {});\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript cold console probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript cold console probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"before\nafter\n" {
        return Err(format!(
            "TypeScript console effect ran before the runner boundary: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::source_exports_name;

    #[test]
    fn recognizes_async_function_exports() {
        assert!(source_exports_name(
            "export async function readLine(): Promise<string | undefined> { return undefined; }",
            "readLine"
        ));
    }
}
