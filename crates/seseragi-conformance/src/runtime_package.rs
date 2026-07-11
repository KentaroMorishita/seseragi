use std::fs;
use std::path::Path;
use std::process::Command;

mod effect;
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
    package_export_source(&package, ".")
        .ok_or_else(|| "TypeScript runtime package root export is missing".to_owned())?;

    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    for feature in features {
        match feature.get("kind").and_then(|value| value.as_str()) {
            Some("runtime-helper" | "runtime-binding") => {
                check_typescript_runtime_import(root, &package, feature)?;
            }
            Some("value-representation") if feature.get("typeImport").is_some() => {
                check_typescript_runtime_type_import(root, &package, feature)?;
            }
            _ => {}
        }
    }
    check_typescript_runtime_package_typecheck(root)?;
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

fn check_typescript_runtime_import(
    root: &Path,
    package: &serde_json::Value,
    feature: &serde_json::Value,
) -> Result<(), String> {
    let id = feature
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "runtime import feature id must be a string".to_owned())?;
    let import = feature
        .get("import")
        .ok_or_else(|| format!("runtime import {id} import is missing"))?;
    let module = import
        .get("module")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime import {id} import.module must be a string"))?;
    let export_name = import
        .get("export")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime import {id} import.export must be a string"))?;

    let subpath = runtime_package_subpath(module)
        .ok_or_else(|| format!("runtime import {id} uses unexpected module {module}"))?;
    let source_path = package_export_source(package, &subpath)
        .ok_or_else(|| format!("runtime import {id} module {module} is not exported"))?;
    let source = fs::read_to_string(root.join("runtime/ts").join(source_path))
        .map_err(|error| format!("failed to read TypeScript runtime import {module}: {error}"))?;

    if !source_exports_name(&source, export_name) {
        return Err(format!(
            "runtime import {id} module {module} does not export {export_name}"
        ));
    }
    Ok(())
}

fn check_typescript_runtime_type_import(
    root: &Path,
    package: &serde_json::Value,
    feature: &serde_json::Value,
) -> Result<(), String> {
    let id = feature
        .get("id")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "runtime type import feature id must be a string".to_owned())?;
    let import = feature
        .get("typeImport")
        .ok_or_else(|| format!("runtime type import {id} typeImport is missing"))?;
    let module = import
        .get("module")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime type import {id} typeImport.module must be a string"))?;
    let export_name = import
        .get("export")
        .and_then(|value| value.as_str())
        .ok_or_else(|| format!("runtime type import {id} typeImport.export must be a string"))?;

    let subpath = runtime_package_subpath(module)
        .ok_or_else(|| format!("runtime type import {id} uses unexpected module {module}"))?;
    let source_path = package_export_source(package, &subpath)
        .ok_or_else(|| format!("runtime type import {id} module {module} is not exported"))?;
    let source =
        fs::read_to_string(root.join("runtime/ts").join(source_path)).map_err(|error| {
            format!("failed to read TypeScript runtime type import {module}: {error}")
        })?;

    if !source_exports_type_name(&source, export_name) {
        return Err(format!(
            "runtime type import {id} module {module} does not export type {export_name}"
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

fn source_exports_type_name(source: &str, name: &str) -> bool {
    let tokens = source
        .split(|character: char| !(character.is_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens
        .windows(3)
        .any(|tokens| tokens[0] == "export" && tokens[1] == "type" && tokens[2] == name)
        || tokens
            .windows(3)
            .any(|tokens| tokens[0] == "export" && tokens[1] == "interface" && tokens[2] == name)
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

#[cfg(test)]
mod tests {
    use super::{
        check_typescript_runtime_type_import, source_exports_name, source_exports_type_name,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn recognizes_async_function_exports() {
        assert!(source_exports_name(
            "export function readLine(): Effect<StdinEnvironment, StdinError, Maybe<string>> { return effect; }",
            "readLine"
        ));
    }

    #[test]
    fn recognizes_exported_type_aliases_and_interfaces() {
        assert!(source_exports_type_name(
            "export type Console = { readonly print: (value: string) => void };",
            "Console"
        ));
        assert!(source_exports_type_name(
            "export interface Stdin { readLine(): Promise<string>; }",
            "Stdin"
        ));
    }

    #[test]
    fn recognizes_multiline_type_reexports() {
        assert!(source_exports_type_name(
            "export type {\n  Console,\n  ConsoleError,\n} from \"./console\";",
            "Console"
        ));
    }

    #[test]
    fn does_not_treat_value_exports_as_type_exports() {
        assert!(!source_exports_type_name(
            "export const Console = {};",
            "Console"
        ));
    }

    #[test]
    fn validates_referenced_runtime_type_exports() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "seseragi-runtime-type-import-{}-{suffix}",
            std::process::id()
        ));
        let source_dir = root.join("runtime/ts/src");
        fs::create_dir_all(&source_dir).unwrap();
        let source_path = source_dir.join("console.ts");
        fs::write(&source_path, "export interface Console { print(): void; }").unwrap();
        let package = serde_json::json!({
            "exports": {
                "./console": {
                    "default": "./src/console.ts"
                }
            }
        });
        let feature = serde_json::json!({
            "id": "effect.console.service",
            "typeImport": {
                "module": "@seseragi/runtime/console",
                "export": "Console"
            }
        });

        assert_eq!(
            check_typescript_runtime_type_import(&root, &package, &feature),
            Ok(())
        );

        fs::write(&source_path, "export const Console = {};").unwrap();
        assert_eq!(
            check_typescript_runtime_type_import(&root, &package, &feature),
            Err("runtime type import effect.console.service module @seseragi/runtime/console does not export type Console".to_owned())
        );
        fs::remove_dir_all(root).unwrap();
    }
}
