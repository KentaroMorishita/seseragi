use std::fs;
use std::path::Path;

pub(super) fn check_runtime_import(
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

    let source = runtime_export_source(root, package, id, module, "runtime import")?;
    if !source_exports_name(&source, export_name) {
        return Err(format!(
            "runtime import {id} module {module} does not export {export_name}"
        ));
    }
    Ok(())
}

pub(super) fn check_runtime_type_import(
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

    let source = runtime_export_source(root, package, id, module, "runtime type import")?;
    if !source_exports_type_name(&source, export_name) {
        return Err(format!(
            "runtime type import {id} module {module} does not export type {export_name}"
        ));
    }
    Ok(())
}

pub(super) fn package_export_source(package: &serde_json::Value, subpath: &str) -> Option<String> {
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

fn runtime_export_source(
    root: &Path,
    package: &serde_json::Value,
    id: &str,
    module: &str,
    label: &str,
) -> Result<String, String> {
    let subpath = runtime_package_subpath(module)
        .ok_or_else(|| format!("{label} {id} uses unexpected module {module}"))?;
    let source_path = package_export_source(package, &subpath)
        .ok_or_else(|| format!("{label} {id} module {module} is not exported"))?;
    fs::read_to_string(root.join("runtime/ts").join(source_path))
        .map_err(|error| format!("failed to read TypeScript {label} {module}: {error}"))
}

fn runtime_package_subpath(module: &str) -> Option<String> {
    module
        .strip_prefix("@seseragi/runtime")
        .map(|subpath| match subpath {
            "" => ".".to_owned(),
            subpath => format!("./{}", subpath.strip_prefix('/').unwrap_or(subpath)),
        })
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
        || source
            .match_indices("export type")
            .filter_map(|(start, _)| {
                source[start + "export type".len()..]
                    .trim_start()
                    .strip_prefix('{')
            })
            .filter_map(|block| block.split_once('}').map(|(bindings, _)| bindings))
            .any(|bindings| {
                bindings
                    .split(|character: char| !(character.is_alphanumeric() || character == '_'))
                    .any(|binding| binding == name)
            })
}

#[cfg(test)]
mod tests {
    use super::{check_runtime_type_import, source_exports_name, source_exports_type_name};
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
        assert!(source_exports_type_name(
            "export type {\n  Console,\n  ConsoleError,\n} from \"./console\";",
            "ConsoleError"
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

        assert_eq!(check_runtime_type_import(&root, &package, &feature), Ok(()));

        fs::write(&source_path, "export const Console = {};").unwrap();
        assert_eq!(
            check_runtime_type_import(&root, &package, &feature),
            Err("runtime type import effect.console.service module @seseragi/runtime/console does not export type Console".to_owned())
        );
        fs::remove_dir_all(root).unwrap();
    }
}
