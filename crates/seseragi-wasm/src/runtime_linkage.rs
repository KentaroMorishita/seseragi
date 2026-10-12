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
    let host_manifest = match request.manifest {
        None => None,
        Some(manifest) => seseragi_project::parse_manifest(&manifest)
            .map_err(|e| vec![problem(e.to_string())])?
            .foreign_typescript
            .map(|foreign| foreign.manifest.as_str().to_owned()),
    };
    let host_root = host_manifest
        .as_deref()
        .and_then(|path| path.rsplit_once('/').map(|(parent, _)| parent))
        .unwrap_or_default();
    resolve_browser_runtime_snapshot(&files, &request.imports, host_root)
}

#[cfg(test)]
mod tests {
    use super::*;
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
