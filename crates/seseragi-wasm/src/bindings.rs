//! Resolves converter inputs from an immutable virtual filesystem snapshot.
use serde::{Deserialize, Serialize};
use seseragi_dts::{
    convert_entry, parse_bindings, ConversionDiagnostic, ConversionInput, DiagnosticSeverity,
    GeneratedBinding, HostMetadata,
};
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;
use wasm_bindgen::prelude::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: u32,
    revision: String,
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
#[serde(rename_all = "camelCase")]
struct Response {
    schema: u32,
    revision: String,
    status: &'static str,
    generated: Vec<GeneratedBinding>,
    generated_root: Option<String>,
    diagnostics: Vec<ConversionDiagnostic>,
}

#[wasm_bindgen]
pub fn convert_workspace_bindings(json: &str) -> String {
    let mut response = Response {
        schema: 1,
        revision: String::new(),
        status: "success",
        generated: vec![],
        generated_root: None,
        diagnostics: vec![],
    };
    let result = serde_json::from_str::<Request>(json)
        .map_err(|error| {
            (
                "seseragi.toml".to_owned(),
                format!("invalid conversion request: {error}"),
            )
        })
        .and_then(|request| {
            response.revision = request.revision.clone();
            convert(&request, &mut response)
        });
    if let Err((file, message)) = result {
        response.diagnostics.push(ConversionDiagnostic {
            code: "SES-F0100".to_owned(),
            severity: DiagnosticSeverity::Error,
            message,
            file,
            start: 0,
            end: 0,
            symbol: None,
        });
    }
    if response
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        response.status = "failure";
    }
    serde_json::to_string(&response).expect("conversion response must serialize")
}

type Problem = (String, String);
fn path(value: &str) -> Result<String, Problem> {
    if value.is_empty()
        || value.contains(['\\', '\0'])
        || value.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err((
            value.to_owned(),
            format!("invalid workspace artifact path `{value}`"),
        ));
    }
    Ok(value.nfc().collect())
}
fn read<'a>(files: &BTreeMap<String, &'a str>, name: &str) -> Result<&'a str, Problem> {
    let name = path(name)?;
    files.get(&name).copied().ok_or_else(|| {
        (
            name.clone(),
            format!("workspace artifact `{name}` does not exist"),
        )
    })
}
fn convert(request: &Request, response: &mut Response) -> Result<(), Problem> {
    if request.schema != 1 {
        return Err((
            "seseragi.toml".to_owned(),
            "unsupported conversion request schema; expected 1".to_owned(),
        ));
    }
    let mut files = BTreeMap::new();
    for file in &request.files {
        let normalized = path(&file.path)?;
        if files
            .insert(normalized.clone(), file.source.as_str())
            .is_some()
        {
            return Err((normalized, "duplicate workspace artifact path".to_owned()));
        }
    }
    let manifest_path = "seseragi.toml";
    let manifest = seseragi_project::parse_manifest(read(&files, manifest_path)?)
        .map_err(|error| (manifest_path.to_owned(), error.to_string()))?;
    response.generated_root = Some(manifest.layout.generated.as_str().to_owned());
    let foreign = manifest.foreign_typescript.ok_or_else(|| {
        (
            manifest_path.to_owned(),
            "package has no [foreign.typescript] configuration for dts conversion".to_owned(),
        )
    })?;
    let bindings = foreign.bindings.ok_or_else(|| {
        (
            manifest_path.to_owned(),
            "package has no foreign.typescript.bindings settings file".to_owned(),
        )
    })?;
    let settings = read(&files, bindings.as_str())?;
    let config = parse_bindings(settings)
        .map_err(|error| (bindings.as_str().to_owned(), error.to_string()))?;
    if let Some(id) = &request.entry {
        if !config.entries.contains_key(id) {
            return Err((
                bindings.as_str().to_owned(),
                format!(
                    "binding entry `{id}` does not exist in {}",
                    bindings.as_str()
                ),
            ));
        }
    }
    let host = read(&files, foreign.manifest.as_str())?;
    for (id, entry) in &config.entries {
        if request
            .entry
            .as_ref()
            .is_some_and(|selected| selected != id)
        {
            continue;
        }
        let declaration = read(&files, &entry.declaration)?;
        let package = if entry.specifier.starts_with('@') {
            entry
                .specifier
                .split('/')
                .take(2)
                .collect::<Vec<_>>()
                .join("/")
        } else {
            entry
                .specifier
                .split('/')
                .next()
                .unwrap_or_default()
                .to_owned()
        };
        let parent = foreign
            .manifest
            .as_str()
            .rsplit_once('/')
            .map(|(parent, _)| format!("{parent}/"))
            .unwrap_or_default();
        let resolved_path = format!("{parent}node_modules/{package}/package.json");
        // The core selects the root manifest when it names the imported package;
        // otherwise identity errors belong to the resolved dependency artifact.
        // Invalid root JSON is still diagnosed at the root manifest itself.
        let dependency_metadata = !entry.specifier.starts_with("./")
            && !entry.specifier.starts_with("../")
            && serde_json::from_str::<serde_json::Value>(host).is_ok_and(|manifest| {
                manifest.get("name").and_then(|name| name.as_str()) != Some(package.as_str())
            });
        let host_diagnostic_path = if dependency_metadata {
            resolved_path.as_str()
        } else {
            foreign.manifest.as_str()
        };

        let resolved_package = path(&resolved_path)
            .ok()
            .and_then(|name| files.get(&name).copied());
        let previous_path = format!(
            "{}/{}.binding.json",
            manifest.layout.generated.as_str(),
            entry.output
        );
        let previous_metadata = path(&previous_path)
            .ok()
            .and_then(|name| files.get(&name).copied());
        let converted = convert_entry(&ConversionInput {
            settings,
            entry: id,
            declaration,
            host: HostMetadata {
                manifest: host,
                resolved_package,
            },
            previous_metadata,
        })
        .map_err(|error| (host_diagnostic_path.to_owned(), error.to_string()))?;
        response.diagnostics.extend(converted.diagnostics);
        if let Some(generated) = converted.generated {
            response.generated.push(generated);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::{fs, path::Path};

    fn fixture(name: &str) -> Value {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(name);
        let mut files = vec![];
        fn collect(root: &Path, directory: &Path, files: &mut Vec<Value>) {
            for item in fs::read_dir(directory).unwrap() {
                let item = item.unwrap();
                if item.file_type().unwrap().is_dir() {
                    collect(root, &item.path(), files);
                } else if let Ok(source) = fs::read_to_string(item.path()) {
                    files.push(json!({"path": item.path().strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"), "source": source}));
                }
            }
        }
        collect(&root, &root, &mut files);
        json!({"schema": 1, "revision": "snapshot-1", "files": files})
    }
    fn response(request: &Value) -> Value {
        serde_json::from_str(&convert_workspace_bindings(&request.to_string())).unwrap()
    }
    fn source<'a>(request: &'a Value, name: &str) -> &'a str {
        request["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == name)
            .unwrap()["source"]
            .as_str()
            .unwrap()
    }
    fn replace(request: &mut Value, name: &str, value: &str) {
        request["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|file| file["path"] == name)
            .unwrap()["source"] = json!(value);
    }

    #[test]
    fn virtual_artifacts_match_portable_core_for_every_fixture() {
        for name in [
            "dts-basic-conversion",
            "dts-callback-during-call",
            "dts-declaration-merge",
            "dts-generated-name",
            "dts-namespace-runtime",
            "dts-overload-selection",
            "dts-callback-missing-release",
            "dts-unsupported-any",
        ] {
            let request = fixture(name);
            let settings = source(&request, "seseragi.bindings.toml");
            let config = parse_bindings(settings).unwrap();
            let mut generated = vec![];
            let mut diagnostics = vec![];
            for (id, entry) in config.entries {
                let converted = convert_entry(&ConversionInput {
                    settings,
                    entry: &id,
                    declaration: source(&request, &entry.declaration),
                    host: HostMetadata {
                        manifest: source(&request, "host/package.json"),
                        resolved_package: request["files"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .find(|file| {
                                file["path"]
                                    == format!("host/node_modules/{}/package.json", entry.specifier)
                            })
                            .and_then(|file| file["source"].as_str()),
                    },
                    previous_metadata: None,
                })
                .unwrap();
                generated.extend(converted.generated);
                diagnostics.extend(converted.diagnostics);
            }
            let actual = response(&request);
            assert_eq!(actual["revision"], "snapshot-1");
            assert_eq!(actual["generated"], json!(generated), "{name}");
            assert_eq!(actual["diagnostics"], json!(diagnostics), "{name}");
        }
    }

    #[test]
    fn missing_and_renamed_inputs_are_diagnosed_at_the_workspace_path() {
        let mut request = fixture("dts-basic-conversion");
        request["files"]
            .as_array_mut()
            .unwrap()
            .retain(|file| file["path"] != "host/index.d.ts");
        let result = response(&request);
        assert_eq!(result["status"], "failure");
        assert_eq!(result["diagnostics"][0]["file"], "host/index.d.ts");
        assert_eq!(result["generated"], json!([]));
        request["entry"] = json!("unknown");
        assert!(response(&request)["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("unknown"));
    }

    #[test]
    fn custom_paths_scoped_dependencies_and_previous_metadata_keep_cli_semantics() {
        let mut request = fixture("dts-basic-conversion");
        let settings = source(&request, "seseragi.bindings.toml")
            .replace("fixture-api\"", "@scope/api/subpath\"")
            .replace(
                "output = \"@scope/api/subpath\"",
                "output = \"fixture-api\"",
            );
        replace(&mut request, "seseragi.bindings.toml", &settings);
        replace(&mut request, "host/package.json", r#"{"name":"app"}"#);
        request["files"].as_array_mut().unwrap().push(json!({"path":"host/node_modules/@scope/api/package.json","source":r#"{"name":"@scope/api","version":"2.3.4"}"#}));
        let manifest =
            source(&request, "seseragi.toml").replace("seseragi.bindings.toml", "custom.settings");
        replace(&mut request, "seseragi.toml", &manifest);
        request["files"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|file| file["path"] == "seseragi.bindings.toml")
            .unwrap()["path"] = json!("custom.settings");
        let first = response(&request);
        assert_eq!(first["status"], "success", "{first}");
        let metadata: Value =
            serde_json::from_str(first["generated"][0]["metadata"].as_str().unwrap()).unwrap();
        assert_eq!(
            metadata["hostModule"]["exactIdentity"],
            "@scope/api@2.3.4/subpath"
        );
        request["files"].as_array_mut().unwrap().push(json!({"path":".seseragi/generated/fixture-api.binding.json","source":first["generated"][0]["metadata"]}));
        let second = response(&request);
        let report: Value =
            serde_json::from_str(second["generated"][0]["report"].as_str().unwrap()).unwrap();
        assert_eq!(report["added"], json!([]));
    }

    #[test]
    fn malformed_requests_configs_and_ambiguous_paths_fail_closed() {
        assert!(convert_workspace_bindings("invalid").contains("failure"));
        let mut request = fixture("dts-basic-conversion");
        request["schema"] = json!(2);
        assert_eq!(response(&request)["status"], "failure");
        request["schema"] = json!(1);
        replace(&mut request, "seseragi.bindings.toml", "schema = 2");
        assert_eq!(
            response(&request)["diagnostics"][0]["file"],
            "seseragi.bindings.toml"
        );
        request["files"]
            .as_array_mut()
            .unwrap()
            .push(json!({"path":"../outside","source":""}));
        assert!(response(&request)["diagnostics"][0]["message"]
            .as_str()
            .unwrap()
            .contains("invalid workspace"));
    }
}
