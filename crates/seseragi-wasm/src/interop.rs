//! Workspace transport for the portable declaration converter.
use serde::{Deserialize, Serialize};
use seseragi_dts::{
    resolve_host_module_identity, Converter, DeclarationInput, DiagnosticSeverity, GeneratedBinding,
};
use seseragi_project::{parse_manifest, ModulePath};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
const MAX_FILES: usize = 512;
const MAX_FILE_BYTES: usize = 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    schema: u32,
    revision: String,
    manifest: String,
    files: Vec<File>,
    #[serde(default)]
    entry: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    path: String,
    source: String,
}

#[derive(Serialize)]
struct Output {
    declaration: String,
    #[serde(flatten)]
    generated: GeneratedBinding,
}

#[derive(Serialize)]
struct Diagnostic {
    #[serde(skip_serializing_if = "Option::is_none")]
    entry: Option<String>,
    code: String,
    severity: &'static str,
    message: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
}

impl Diagnostic {
    fn error(path: &str, message: impl Into<String>) -> Self {
        Self {
            entry: None,
            code: "SES-K0001".to_owned(),
            severity: "error",
            message: message.into(),
            path: path.to_owned(),
            start: None,
            end: None,
            symbol: None,
        }
    }
}

#[derive(Serialize)]
struct Response {
    schema: u32,
    revision: String,
    status: &'static str,
    generated: Vec<Output>,
    diagnostics: Vec<Diagnostic>,
}

/// Conversion is atomic at this boundary: errors return diagnostics and no
/// generated set. Callers apply successful output only to the echoed revision.
#[wasm_bindgen]
pub fn convert_bindings(request: &str) -> String {
    let parsed = if request.len() > MAX_REQUEST_BYTES {
        Err("binding conversion request exceeds 8 MiB".to_owned())
    } else {
        serde_json::from_str::<Request>(request).map_err(|error| error.to_string())
    };
    let mut response = Response {
        schema: 1,
        revision: String::new(),
        status: "failure",
        generated: Vec::new(),
        diagnostics: Vec::new(),
    };
    match parsed {
        Ok(request) => {
            response.revision.clone_from(&request.revision);
            match convert_workspace(request) {
                Ok((generated, diagnostics)) => {
                    response.diagnostics = diagnostics;
                    if !response
                        .diagnostics
                        .iter()
                        .any(|value| value.severity == "error")
                    {
                        response.status = "success";
                        response.generated = generated;
                    }
                }
                Err(diagnostic) => response.diagnostics.push(diagnostic),
            }
        }
        Err(message) => response
            .diagnostics
            .push(Diagnostic::error("seseragi.toml", message)),
    }
    serde_json::to_string(&response).expect("binding conversion response serializes")
}

fn convert_workspace(request: Request) -> Result<(Vec<Output>, Vec<Diagnostic>), Diagnostic> {
    if request.schema != 1 {
        return Err(Diagnostic::error(
            "seseragi.toml",
            "unsupported binding conversion request schema",
        ));
    }
    if request.files.len() > MAX_FILES {
        return Err(Diagnostic::error(
            "seseragi.toml",
            "binding conversion workspace exceeds 512 files",
        ));
    }
    let mut files = BTreeMap::new();
    for file in request.files {
        let path = ModulePath::parse(file.path.strip_suffix(".ssrg").unwrap_or(&file.path))
            .map_err(|error| Diagnostic::error(&file.path, error.to_string()))?;
        let canonical = if file.path.ends_with(".ssrg") {
            format!("{}.ssrg", path.as_str())
        } else {
            path.as_str().to_owned()
        };
        if canonical != file.path || file.path.contains('\0') {
            return Err(Diagnostic::error(
                &file.path,
                "workspace artifact path must be canonical",
            ));
        }
        if file.source.len() > MAX_FILE_BYTES {
            return Err(Diagnostic::error(
                &file.path,
                "workspace artifact exceeds 1 MiB",
            ));
        }
        if files.insert(file.path.clone(), file.source).is_some() {
            return Err(Diagnostic::error(
                &file.path,
                "workspace contains a duplicate artifact path",
            ));
        }
    }
    let read = |path: &str| {
        files
            .get(path)
            .map(String::as_str)
            .ok_or_else(|| Diagnostic::error(path, "required workspace artifact is missing"))
    };
    let manifest = parse_manifest(&request.manifest)
        .map_err(|error| Diagnostic::error("seseragi.toml", error.to_string()))?;
    let foreign = manifest.foreign_typescript.ok_or_else(|| {
        Diagnostic::error(
            "seseragi.toml",
            "package has no [foreign.typescript] configuration for dts conversion",
        )
    })?;
    let bindings = foreign.bindings.ok_or_else(|| {
        Diagnostic::error(
            "seseragi.toml",
            "package has no foreign.typescript.bindings settings file",
        )
    })?;
    let converter = Converter::new(read(bindings.as_str())?)
        .map_err(|error| Diagnostic::error(bindings.as_str(), error.to_string()))?;
    if let Some(entry) = &request.entry {
        if !converter.entries().contains_key(entry) {
            return Err(Diagnostic::error(
                bindings.as_str(),
                format!("binding entry `{entry}` does not exist"),
            ));
        }
    }
    let host_source = read(foreign.manifest.as_str())?;
    let mut generated = Vec::new();
    let mut diagnostics = Vec::new();
    for (id, entry) in converter.entries() {
        if request
            .entry
            .as_deref()
            .is_some_and(|selected| selected != id)
        {
            continue;
        }
        let source = read(&entry.declaration)?;
        let host_module = resolve_host_module_identity(
            foreign.manifest.as_str(),
            host_source,
            &entry.specifier,
            |path| {
                read(path)
                    .map(str::to_owned)
                    .map_err(|error| format!("{}: {}", error.path, error.message))
            },
        )
        .map_err(|error| Diagnostic::error(foreign.manifest.as_str(), error.to_string()))?;
        let previous_path = format!("gen/{}.binding.json", entry.output);
        let result = converter
            .convert(DeclarationInput {
                entry: id,
                source,
                host_module,
                previous_metadata: files.get(&previous_path).map(String::as_str),
            })
            .map_err(|error| Diagnostic::error(&entry.declaration, error.to_string()))?;
        diagnostics.extend(result.diagnostics.into_iter().map(|diagnostic| Diagnostic {
            entry: Some(id.clone()),
            code: diagnostic.code,
            severity: match diagnostic.severity {
                DiagnosticSeverity::Error => "error",
                DiagnosticSeverity::Warning => "warning",
            },
            message: diagnostic.message,
            path: diagnostic.file,
            start: Some(diagnostic.start),
            end: Some(diagnostic.end),
            symbol: diagnostic.symbol,
        }));
        if let Some(binding) = result.generated {
            generated.push(Output {
                declaration: entry.declaration.clone(),
                generated: binding,
            });
        }
    }
    Ok((generated, diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn request() -> Value {
        json!({
            "schema": 1, "revision": "opaque revision: 0123/日本語",
            "manifest": include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/seseragi.toml"),
            "files": [
                {"path": "seseragi.bindings.toml", "source": include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/seseragi.bindings.toml")},
                {"path": "host/index.d.ts", "source": include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/host/index.d.ts")},
                {"path": "host/package.json", "source": include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/host/package.json")}
            ]
        })
    }

    fn convert(request: &Value) -> Value {
        serde_json::from_str(&convert_bindings(&request.to_string())).unwrap()
    }

    #[test]
    fn converts_workspace_artifacts_using_cli_settings_and_preserves_revision() {
        let request = request();
        let result = convert(&request);
        assert_eq!(result["status"], "success", "{result}");
        assert_eq!(result["revision"], request["revision"]);
        let generated = &result["generated"][0];
        assert_eq!(generated["entry"], "api");
        assert_eq!(generated["declaration"], "host/index.d.ts");
        assert_eq!(generated["source"], include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/expected/fixture-api.ssrg"));
        let metadata: Value =
            serde_json::from_str(generated["metadata"].as_str().unwrap()).unwrap();
        assert_eq!(metadata["hostModule"]["exactIdentity"], "fixture-api@1.0.0");
        let project = json!({
            "schema":1, "entry":"main.ssrg", "files":[
                {"path":"main.ssrg", "source":"import { Config } from \"gen/fixture-api\"\npub fn accepts config: Config -> Unit = ()\npub effect fn main -> Unit = succeed ()\n"},
                {"path":"gen/fixture-api.ssrg", "root":"generated", "source":generated["source"]}
            ]
        });
        for operation in [crate::compile_project, crate::analyze_project] {
            let response: Value = serde_json::from_str(&operation(&project.to_string())).unwrap();
            assert_eq!(response["status"], "success", "{response}");
        }
    }

    #[test]
    fn conversion_errors_keep_entry_and_declaration_range_and_never_partial_output() {
        let mut request = request();
        request["files"][0]["source"] = json!(format!("{}\n[entries.bad]\ndeclaration = \"host/bad.d.ts\"\nspecifier = \"fixture-api/bad\"\noutput = \"bad-api\"\n", request["files"][0]["source"].as_str().unwrap()));
        let source = "export declare function bad(value: any): unknown;";
        request["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path":"host/bad.d.ts", "source":source}));
        let result = convert(&request);
        assert_eq!(result["status"], "failure", "{result}");
        assert_eq!(result["generated"], json!([]));
        assert_eq!(result["revision"], request["revision"]);
        let diagnostic = result["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .find(|value| value["code"] == "SES-F0101")
            .unwrap();
        assert_eq!(diagnostic["entry"], "bad");
        assert_eq!(diagnostic["path"], "host/bad.d.ts");
        let start = diagnostic["start"].as_u64().unwrap() as usize;
        let end = diagnostic["end"].as_u64().unwrap() as usize;
        assert_eq!(&source[start..end], "any");
    }

    #[test]
    fn artifact_renames_deletes_duplicates_and_invalid_paths_do_not_reuse_old_inputs() {
        for path in [
            "host/moved.d.ts",
            "../index.d.ts",
            "/index.d.ts",
            "host//index.d.ts",
            "host/./index.d.ts",
            "host\\index.d.ts",
            "host/index\0.d.ts",
        ] {
            let mut request = request();
            request["files"][1]["path"] = json!(path);
            let result = convert(&request);
            assert_eq!(result["status"], "failure", "{path}: {result}");
            assert_eq!(result["generated"], json!([]));
        }
        let mut request = request();
        request["files"].as_array_mut().unwrap().remove(1);
        assert_eq!(
            convert(&request)["diagnostics"][0]["path"],
            "host/index.d.ts"
        );
        let duplicate = request["files"][0].clone();
        request["files"].as_array_mut().unwrap().push(duplicate);
        assert_eq!(convert(&request)["status"], "failure");
    }

    #[test]
    fn resolved_dependency_metadata_and_previous_reports_are_workspace_artifacts() {
        let mut request = request();
        request["files"][2]["source"] = json!("{\"name\":\"host\",\"version\":\"0.0.0\"}");
        assert_eq!(convert(&request)["status"], "failure");
        request["files"].as_array_mut().unwrap().push(json!({
            "path":"host/node_modules/fixture-api/package.json", "source":"{\"name\":\"fixture-api\",\"version\":\"2.3.4\"}"
        }));
        let result = convert(&request);
        assert_eq!(result["status"], "success", "{result}");
        let metadata = result["generated"][0]["metadata"].as_str().unwrap();
        assert!(metadata.contains("fixture-api@2.3.4"));
        request["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path":"gen/fixture-api.binding.json", "source":metadata}));
        let repeated = convert(&request);
        let report: Value =
            serde_json::from_str(repeated["generated"][0]["report"].as_str().unwrap()).unwrap();
        assert_eq!(report["added"], json!([]));
        assert_eq!(report["changed"], json!([]));
        assert_eq!(report["removed"], json!([]));
    }

    #[test]
    fn enforces_transport_schema_and_size_limits_without_panics() {
        for value in ["not json".to_owned(), " ".repeat(MAX_REQUEST_BYTES + 1)] {
            let result: Value = serde_json::from_str(&convert_bindings(&value)).unwrap();
            assert_eq!(result["status"], "failure");
        }
        let mut request = request();
        request["schema"] = json!(2);
        assert_eq!(convert(&request)["status"], "failure");
        request["schema"] = json!(1);
        request["files"][1]["source"] = json!(" ".repeat(MAX_FILE_BYTES + 1));
        assert_eq!(
            convert(&request)["diagnostics"][0]["path"],
            "host/index.d.ts"
        );
    }
}
