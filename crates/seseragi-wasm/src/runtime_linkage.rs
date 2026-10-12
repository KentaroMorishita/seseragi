use serde::{Deserialize, Serialize};
use seseragi_project::{
    resolve_browser_runtime_snapshot, RuntimeLinkageDiagnostic, RuntimeTarget,
    VirtualRuntimeBundle, VirtualRuntimeImport,
};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    schema: u32,
    files: Vec<File>,
    imports: Vec<VirtualRuntimeImport>,
    #[serde(default)]
    manifest: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    path: String,
    source: String,
}
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum Response {
    Success {
        schema: u32,
        bundle: VirtualRuntimeBundle,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<RuntimeLinkageDiagnostic>,
    },
}
#[wasm_bindgen]
pub fn resolve_browser_runtime_modules(request_json: &str) -> String {
    let resolved = serde_json::from_str::<Request>(request_json)
        .map_err(|error| vec![problem(error.to_string())])
        .and_then(resolve);
    let response = match resolved {
        Ok(bundle) => Response::Success { schema: 1, bundle },
        Err(diagnostics) => Response::Failure {
            schema: 1,
            diagnostics,
        },
    };
    serde_json::to_string(&response)
        .expect("runtime linkage response contains JSON-compatible values")
}
fn problem(reason: String) -> RuntimeLinkageDiagnostic {
    RuntimeLinkageDiagnostic::new("SES-F0203", Some(RuntimeTarget::Browser), reason)
}
fn resolve(request: Request) -> Result<VirtualRuntimeBundle, Vec<RuntimeLinkageDiagnostic>> {
    if request.schema != 1 {
        return Err(vec![problem(
            "unsupported browser runtime request schema".to_owned(),
        )]);
    }
    let mut files = BTreeMap::new();
    for file in request.files {
        if files.insert(file.path.clone(), file.source).is_some() {
            return Err(vec![problem(format!(
                "duplicate runtime snapshot path `{}`",
                file.path
            ))]);
        }
    }
    let foreign = match request.manifest {
        None => None,
        Some(manifest) => {
            seseragi_project::parse_manifest(&manifest)
                .map_err(|e| vec![problem(e.to_string())])?
                .foreign_typescript
        }
    };
    let settings = foreign
        .as_ref()
        .and_then(|foreign| foreign.bindings.as_ref())
        .and_then(|path| files.get(path.as_str()))
        .map(String::as_str);
    let host_root = foreign
        .as_ref()
        .map(|foreign| foreign.manifest.as_str())
        .and_then(|path| path.rsplit_once('/').map(|(parent, _)| parent))
        .unwrap_or_default();
    let bundle = resolve_browser_runtime_snapshot(&files, &request.imports, host_root)?;
    for import in &request.imports {
        let stem = import
            .importer
            .rsplit_once('.')
            .map_or(import.importer.as_str(), |(stem, _)| stem);
        if let Some(metadata) = files.get(&format!("{stem}.binding.json")) {
            if foreign
                .as_ref()
                .is_some_and(|foreign| foreign.bindings.is_some())
                && settings.is_none()
                && serde_json::from_str::<serde_json::Value>(metadata)
                    .is_ok_and(|metadata| metadata.get("runtimeLinkage").is_some())
            {
                return Err(vec![RuntimeLinkageDiagnostic::new(
                    "SES-F0201",
                    Some(RuntimeTarget::Browser),
                    "runtime linkage binding settings are missing from snapshot",
                )]);
            }
            seseragi_project::validate_binding_runtime_linkage_inputs(
                metadata,
                &files,
                settings,
                RuntimeTarget::Browser,
            )?;
            seseragi_project::validate_binding_runtime_linkage(
                metadata,
                RuntimeTarget::Browser,
                false,
                |specifier| {
                    seseragi_project::resolve_runtime_snapshot_artifact(
                        &files,
                        &import.importer,
                        specifier,
                        host_root,
                        RuntimeTarget::Browser,
                    )
                },
            )?;
        }
    }
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_npm_conversion_contract_rejects_stale_runtime_declaration_and_settings() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/playground/tests/fixtures/interop-base64-js-1.5.1");
        let mapping: Vec<serde_json::Value> = serde_json::from_str(
            &std::fs::read_to_string(root.join("workspace-files.json")).unwrap(),
        )
        .unwrap();
        let mut files: Vec<serde_json::Value> = mapping.iter().map(|file| serde_json::json!({"path":file["path"],"source":std::fs::read_to_string(root.join(file["fixture"].as_str().unwrap())).unwrap()})).collect();
        let manifest = std::fs::read_to_string(root.join("seseragi.toml")).unwrap();
        let conversion: serde_json::Value =
            serde_json::from_str(&crate::bindings::convert_workspace_bindings(
                &serde_json::json!({"schema":1,"revision":"real-package-1","files":files})
                    .to_string(),
            ))
            .unwrap();
        assert_eq!(
            conversion["status"], "success",
            "{}",
            conversion["diagnostics"]
        );
        let generated = &conversion["generated"][0];
        let importer = format!(
            "{}/base64.js",
            conversion["generatedRoot"].as_str().unwrap()
        );
        files.push(serde_json::json!({"path":format!("{}/base64.binding.json",conversion["generatedRoot"].as_str().unwrap()),"source":generated["metadata"]}));
        let request = serde_json::json!({"schema":1,"files":files,"manifest":manifest,"imports":[{"importer":importer,"specifier":"base64-js"}]});
        let answer = |request: &serde_json::Value| {
            serde_json::from_str::<serde_json::Value>(&resolve_browser_runtime_modules(
                &request.to_string(),
            ))
            .unwrap()
        };
        assert_eq!(answer(&request)["status"], "success");
        for path in [
            "host/node_modules/base64-js/index.js",
            "host/node_modules/base64-js/index.d.ts",
            "seseragi.bindings.toml",
        ] {
            let mut changed = request.clone();
            let file = changed["files"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|file| file["path"] == path)
                .unwrap();
            file["source"] = format!("{}\n", file["source"].as_str().unwrap()).into();
            let response = answer(&changed);
            assert_eq!(response["status"], "failure", "{path}: {response}");
            assert_eq!(
                response["diagnostics"][0]["code"], "SES-F0201",
                "{path}: {response}"
            );
        }
        let mut node_only = request;
        let file = node_only["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|file| file["path"] == "host/node_modules/base64-js/package.json")
            .unwrap();
        let mut package: serde_json::Value =
            serde_json::from_str(file["source"].as_str().unwrap()).unwrap();
        package["exports"] = serde_json::json!({"node":"./index.js"});
        file["source"] = package.to_string().into();
        assert_eq!(answer(&node_only)["diagnostics"][0]["code"], "SES-F0203");
        let conversion: serde_json::Value =
            serde_json::from_str(&crate::bindings::convert_workspace_bindings(
                &serde_json::json!({"schema":1,"revision":"node-only","files":node_only["files"]})
                    .to_string(),
            ))
            .unwrap();
        assert_eq!(conversion["status"], "success");
        let metadata: serde_json::Value =
            serde_json::from_str(conversion["generated"][0]["metadata"].as_str().unwrap()).unwrap();
        assert!(metadata["runtimeLinkage"]["contract"]["exports"]
            .as_object()
            .unwrap()
            .values()
            .all(|export| export["targets"].get("browser").is_none()));
    }

    #[test]
    fn browser_adapter_resolves_manifest_selected_exports_and_rejects_host_mismatch() {
        let request = serde_json::json!({"schema":1,"imports":[{"importer":"main.js","specifier":"sample"}],"files":[
            {"path":"node_modules/sample/package.json","source":r#"{"name":"sample","version":"1.2.3","exports":{"browser":"./web.js","node":"./server.js"}}"#},
            {"path":"node_modules/sample/web.js","source":"export const value = 42;"},
            {"path":"node_modules/sample/server.js","source":"import fs from 'node:fs'; export const value = fs;"}
        ]});
        let success: serde_json::Value =
            serde_json::from_str(&resolve_browser_runtime_modules(&request.to_string())).unwrap();
        assert_eq!(success["status"], "success");
        assert_eq!(
            success["bundle"]["modules"]["node_modules/sample/web.js"]["exactIdentity"],
            "npm:sample@1.2.3/web.js"
        );
        let mut unsupported = request;
        unsupported["files"][1]["source"] =
            "import fs from 'node:fs'; export const value = fs;".into();
        let failure: serde_json::Value =
            serde_json::from_str(&resolve_browser_runtime_modules(&unsupported.to_string()))
                .unwrap();
        assert_eq!(failure["status"], "failure");
        assert_eq!(failure["diagnostics"][0]["code"], "SES-F0203");
        assert_eq!(failure["diagnostics"][0]["target"], "browser");
    }
}
