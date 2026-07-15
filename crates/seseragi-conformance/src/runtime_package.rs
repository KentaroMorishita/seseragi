use std::fs;
use std::path::Path;
use std::process::Command;

mod effect;
mod imports;
mod range;
mod service;
mod services;
mod sum;

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
    imports::package_export_source(&package, ".")
        .ok_or_else(|| "TypeScript runtime package root export is missing".to_owned())?;

    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    for feature in features {
        match feature.get("kind").and_then(|value| value.as_str()) {
            Some("runtime-helper" | "runtime-binding") => {
                imports::check_runtime_import(root, &package, feature)?;
            }
            Some("value-representation") if feature.get("typeImport").is_some() => {
                imports::check_runtime_type_import(root, &package, feature)?;
            }
            _ => {}
        }
    }
    check_typescript_runtime_package_typecheck(root)?;
    service::check_typed_service_boundary(root)?;
    sum::check_tagged_standard_sums(root)?;
    effect::check_from_either_boundary(root)?;
    if runtime_helper_is_declared(abi, "effect.stdin.readLine") {
        services::check_typescript_runtime_read_line(root)?;
    }
    if runtime_helper_is_declared(abi, "effect.console.println") {
        services::check_typescript_runtime_console_services(root)?;
    }
    if runtime_helper_is_declared(abi, "effect.core.fail")
        || runtime_helper_is_declared(abi, "effect.core.mapError")
    {
        effect::check_typed_failure_boundary(root)?;
    }
    if runtime_helper_is_declared(abi, "core.int64.add") {
        check_typescript_runtime_int64(root)?;
    }
    if runtime_helper_is_declared(abi, "core.range.reduce") {
        range::check_typescript_runtime_range(root)?;
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

fn check_typescript_runtime_int64(root: &Path) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("--eval")
        .arg(
            "import { add, divide, multiply, power, remainder, subtract } from \"./src/int64.ts\";\n\
             const defects = [];\n\
             for (const operation of [() => add(9223372036854775807n, 1n), () => subtract(-9223372036854775808n, 1n), () => multiply(9223372036854775807n, 2n), () => divide(1n, 0n), () => remainder(1n, 0n), () => divide(-9223372036854775808n, -1n), () => remainder(-9223372036854775808n, -1n), () => power(2n, 63n), () => power(2n, -1n)]) {\n\
               try { operation(); defects.push(false); } catch (error) { defects.push(error instanceof RangeError); }\n\
             }\n\
             process.stdout.write(JSON.stringify({ defects, values: [add(2n, 3n), subtract(2n, 3n), multiply(-2n, 3n), divide(-5n, 2n), remainder(-5n, 2n), power(0n, 0n)].map(String) }));\n",
        )
        .current_dir(root.join("runtime/ts"))
        .output()
        .map_err(|error| format!("failed to run TypeScript Int64 runtime probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "TypeScript Int64 runtime probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout
        != b"{\"defects\":[true,true,true,true,true,true,true,true,true],\"values\":[\"5\",\"-1\",\"-6\",\"-2\",\"-1\",\"1\"]}"
    {
        return Err(format!(
            "TypeScript Int64 runtime probe returned unexpected values: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}
