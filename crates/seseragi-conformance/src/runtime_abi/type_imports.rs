use super::{read_core_runtime_abi, RuntimeImport};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn runtime_type_imports(root: &Path) -> Result<BTreeMap<String, RuntimeImport>, String> {
    runtime_type_imports_from_abi(&read_core_runtime_abi(root)?)
}

fn runtime_type_imports_from_abi(
    abi: &serde_json::Value,
) -> Result<BTreeMap<String, RuntimeImport>, String> {
    let features = abi
        .get("features")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "runtime ABI features must be an array".to_owned())?;
    let mut imports = BTreeMap::new();
    for feature in features {
        if feature.get("kind").and_then(|value| value.as_str()) != Some("value-representation") {
            continue;
        }
        let Some(type_import) = feature.get("typeImport") else {
            continue;
        };
        let id = feature
            .get("id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "runtime type import feature id must be a string".to_owned())?;
        let module = type_import
            .get("module")
            .and_then(nonempty_string)
            .ok_or_else(|| {
                format!("runtime type import {id} typeImport.module must be a non-empty string")
            })?;
        let export_name = type_import
            .get("export")
            .and_then(nonempty_string)
            .ok_or_else(|| {
                format!("runtime type import {id} typeImport.export must be a non-empty string")
            })?;
        imports.insert(
            id.to_owned(),
            RuntimeImport {
                module: module.to_owned(),
                export_name: export_name.to_owned(),
            },
        );
    }
    Ok(imports)
}

pub(super) fn check_feature_type_metadata(
    id: &str,
    kind: &str,
    feature: &serde_json::Value,
) -> Result<(), String> {
    let type_identity = feature.get("typeIdentity");
    let type_import = feature.get("typeImport");

    match (kind, type_identity, type_import) {
        ("value-representation", None, None) => Ok(()),
        ("value-representation", Some(type_identity), Some(type_import)) => {
            nonempty_string(type_identity).ok_or_else(|| {
                format!("runtime ABI feature {id} typeIdentity must be a non-empty string")
            })?;
            let module = type_import.get("module").ok_or_else(|| {
                format!("runtime ABI feature {id} typeImport.module is missing")
            })?;
            nonempty_string(module).ok_or_else(|| {
                format!("runtime ABI feature {id} typeImport.module must be a non-empty string")
            })?;
            let export = type_import.get("export").ok_or_else(|| {
                format!("runtime ABI feature {id} typeImport.export is missing")
            })?;
            nonempty_string(export).ok_or_else(|| {
                format!("runtime ABI feature {id} typeImport.export must be a non-empty string")
            })?;
            Ok(())
        }
        ("value-representation", _, _) => Err(format!(
            "runtime ABI feature {id} typeIdentity and typeImport must be specified together"
        )),
        (_, None, None) => Ok(()),
        _ => Err(format!(
            "runtime ABI feature {id} typeIdentity and typeImport are only valid for value-representation features"
        )),
    }
}

fn nonempty_string(value: &serde_json::Value) -> Option<&str> {
    value
        .as_str()
        .and_then(|value| (!value.is_empty()).then_some(value))
}

#[cfg(test)]
mod tests {
    use super::runtime_type_imports_from_abi;
    use crate::runtime_abi::{check_runtime_abi_feature, RuntimeImport};
    use std::collections::{BTreeMap, BTreeSet};

    fn feature_with_type_metadata() -> serde_json::Value {
        serde_json::json!({
            "id": "effect.console.service",
            "kind": "value-representation",
            "typescript": "type Console",
            "boundary": "Console service interface",
            "typeIdentity": "std/prelude::Console",
            "typeImport": {
                "module": "@seseragi/runtime/console",
                "export": "Console"
            },
            "import": null
        })
    }

    fn validate(feature: &serde_json::Value) -> Result<(), String> {
        check_runtime_abi_feature(feature, &mut BTreeSet::new())
    }

    #[test]
    fn accepts_paired_value_representation_type_metadata() {
        assert_eq!(validate(&feature_with_type_metadata()), Ok(()));
    }

    #[test]
    fn rejects_unpaired_value_representation_type_metadata() {
        let mut feature = feature_with_type_metadata();
        feature.as_object_mut().unwrap().remove("typeImport");

        assert_eq!(
            validate(&feature),
            Err("runtime ABI feature effect.console.service typeIdentity and typeImport must be specified together".to_owned())
        );
    }

    #[test]
    fn rejects_type_metadata_on_runtime_helpers() {
        let mut feature = feature_with_type_metadata();
        feature["kind"] = serde_json::Value::String("runtime-helper".to_owned());
        feature["import"] = serde_json::json!({
            "module": "@seseragi/runtime/console",
            "export": "print"
        });

        assert_eq!(
            validate(&feature),
            Err("runtime ABI feature effect.console.service typeIdentity and typeImport are only valid for value-representation features".to_owned())
        );
    }

    #[test]
    fn rejects_malformed_type_import_shape() {
        let mut feature = feature_with_type_metadata();
        feature["typeImport"]["export"] = serde_json::Value::String(String::new());

        assert_eq!(
            validate(&feature),
            Err("runtime ABI feature effect.console.service typeImport.export must be a non-empty string".to_owned())
        );
    }

    #[test]
    fn indexes_type_imports_by_runtime_feature() {
        let abi = serde_json::json!({
            "features": [
                feature_with_type_metadata(),
                {
                    "id": "core.string",
                    "kind": "value-representation",
                    "typescript": "string",
                    "boundary": "string",
                    "import": null
                }
            ]
        });

        assert_eq!(
            runtime_type_imports_from_abi(&abi),
            Ok(BTreeMap::from([(
                "effect.console.service".to_owned(),
                RuntimeImport {
                    module: "@seseragi/runtime/console".to_owned(),
                    export_name: "Console".to_owned(),
                },
            )]))
        );
    }
}
