use seseragi_dts::{convert_entry, ConversionInput, DiagnosticSeverity, HostMetadata};

const SETTINGS: &str = include_str!(
    "../../../examples/spec/fixtures/projects/dts-basic-conversion/seseragi.bindings.toml"
);
const DECLARATION: &str =
    include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/host/index.d.ts");
const HOST: &str =
    include_str!("../../../examples/spec/fixtures/projects/dts-basic-conversion/host/package.json");
const EXPECTED: &str = include_str!(
    "../../../examples/spec/fixtures/projects/dts-basic-conversion/expected/fixture-api.ssrg"
);

fn input() -> ConversionInput<'static> {
    ConversionInput {
        settings: SETTINGS,
        entry: "api",
        declaration: DECLARATION,
        host: HostMetadata {
            manifest: HOST,
            resolved_package: None,
        },
        previous_metadata: None,
    }
}

#[test]
fn converts_text_without_files_and_preserves_metadata_and_report_semantics() {
    let first = convert_entry(&input()).unwrap();
    assert!(first.diagnostics.is_empty());
    let generated = first.generated.unwrap();
    assert_eq!(generated.source, EXPECTED);
    assert_eq!(generated.id, "api");
    assert_eq!(generated.output, "fixture-api");
    let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
    assert_eq!(metadata["hostModule"]["exactIdentity"], "fixture-api@1.0.0");
    assert_eq!(metadata["declaration"], "host/index.d.ts");
    let report: serde_json::Value = serde_json::from_str(&generated.report).unwrap();
    assert_eq!(report["added"].as_array().unwrap().len(), 4);
    let second = convert_entry(&ConversionInput {
        previous_metadata: Some(&generated.metadata),
        ..input()
    })
    .unwrap()
    .generated
    .unwrap();
    assert_eq!(generated.source, second.source);
    assert_eq!(generated.metadata, second.metadata);
    let report: serde_json::Value = serde_json::from_str(&second.report).unwrap();
    for key in ["added", "changed", "removed"] {
        assert_eq!(report[key], serde_json::json!([]));
    }
    let invalid_previous = convert_entry(&ConversionInput {
        previous_metadata: Some("invalid JSON"),
        ..input()
    })
    .unwrap()
    .generated
    .unwrap();
    assert_eq!(generated, invalid_previous);
}

#[test]
fn compares_previous_symbols_and_signatures() {
    let first = convert_entry(&ConversionInput {
        declaration:
            "export declare function old(): string; export declare function change(): string;",
        ..input()
    })
    .unwrap()
    .generated
    .unwrap();
    let next = convert_entry(&ConversionInput {
        declaration:
            "export declare function newOne(): string; export declare function change(): boolean;",
        previous_metadata: Some(&first.metadata),
        ..input()
    })
    .unwrap()
    .generated
    .unwrap();
    let report: serde_json::Value = serde_json::from_str(&next.report).unwrap();
    for key in ["added", "changed", "removed"] {
        assert_eq!(report[key].as_array().unwrap().len(), 1, "{report}");
    }
}

#[test]
fn resolves_explicit_dependency_metadata_and_scoped_subpaths() {
    let settings = SETTINGS.replace(
        "specifier = \"fixture-api\"",
        "specifier = \"@scope/api/subpath\"",
    );
    let request = ConversionInput {
        settings: &settings,
        host: HostMetadata {
            manifest: r#"{"name":"app"}"#,
            resolved_package: Some(r#"{"name":"@scope/api","version":"2.3.4"}"#),
        },
        ..input()
    };
    let generated = convert_entry(&request).unwrap().generated.unwrap();
    let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
    assert_eq!(
        metadata["hostModule"]["exactIdentity"],
        "@scope/api@2.3.4/subpath"
    );
    for (manifest, expected) in [
        (None, "missing resolved foreign package manifest"),
        (
            Some("not JSON"),
            "invalid resolved foreign package manifest",
        ),
        (
            Some(r#"{"name":"wrong","version":"1"}"#),
            "instead of `@scope/api`",
        ),
        (Some(r#"{"name":"@scope/api"}"#), "no string version"),
    ] {
        let error = convert_entry(&ConversionInput {
            host: HostMetadata {
                resolved_package: manifest,
                ..request.host
            },
            ..request
        })
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn relative_imports_need_no_package_discovery() {
    let settings = SETTINGS.replace("specifier = \"fixture-api\"", "specifier = \"./api.js\"");
    let generated = convert_entry(&ConversionInput {
        settings: &settings,
        host: HostMetadata {
            manifest: "",
            resolved_package: None,
        },
        ..input()
    })
    .unwrap()
    .generated
    .unwrap();
    let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
    assert_eq!(
        metadata["hostModule"]["exactIdentity"],
        "workspace:./api.js"
    );
}

#[test]
fn errors_keep_logical_paths_and_spans_without_artifacts_or_host_resolution() {
    let request = ConversionInput {
        declaration: "export declare function unsafe(value: any): any;",
        host: HostMetadata {
            manifest: "invalid",
            resolved_package: None,
        },
        ..input()
    };
    let result = convert_entry(&request).unwrap();
    assert!(result.generated.is_none());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|d| d.code == "SES-F0101")
        .unwrap();
    assert_eq!(diagnostic.file, "host/index.d.ts");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(
        &request.declaration[diagnostic.start..diagnostic.end],
        "any"
    );
    let result = convert_entry(&ConversionInput {
        declaration: "export declare function (",
        ..request
    })
    .unwrap();
    assert!(result.generated.is_none());
    assert_eq!(result.diagnostics[0].code, "SES-F0100");
}

#[test]
fn explicit_unsafe_fallback_keeps_warnings_in_the_result_and_report() {
    let settings = format!("{SETTINGS}\n[entries.api.symbols.unsafe]\nunsafe_any = true\n");
    let result = convert_entry(&ConversionInput {
        settings: &settings,
        declaration: "export declare function unsafe(value: any): any;",
        ..input()
    })
    .unwrap();
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|d| d.severity == DiagnosticSeverity::Warning));
    let generated = result.generated.unwrap();
    let report: serde_json::Value = serde_json::from_str(&generated.report).unwrap();
    assert_eq!(
        report["warnings"],
        serde_json::to_value(result.diagnostics).unwrap()
    );
}

#[test]
fn rejects_invalid_settings_and_missing_entry() {
    for (settings, entry, expected) in [
        ("bad TOML", "api", "invalid binding settings"),
        (
            "schema = 2\n[entries]",
            "api",
            "unsupported binding settings schema",
        ),
        (
            SETTINGS,
            "missing",
            "binding entry `missing` does not exist",
        ),
    ] {
        let error = convert_entry(&ConversionInput {
            settings,
            entry,
            ..input()
        })
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[cfg(feature = "filesystem")]
#[test]
fn portable_and_filesystem_artifacts_match_for_every_fixture_and_regeneration() {
    use seseragi_dts::{convert_package, BindingsConfig, ConvertRequest};
    use std::{
        fs,
        path::{Path, PathBuf},
    };
    for fixture in [
        "dts-basic-conversion",
        "dts-callback-during-call",
        "dts-declaration-merge",
        "dts-generated-name",
        "dts-namespace-runtime",
        "dts-overload-selection",
        "dts-callback-missing-release",
        "dts-unsupported-any",
    ] {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(fixture);
        let generated = tempfile::tempdir().unwrap();
        let request = ConvertRequest {
            package_root: root.clone(),
            generated_root: generated.path().to_owned(),
            bindings: PathBuf::from("seseragi.bindings.toml"),
            host_manifest: PathBuf::from("host/package.json"),
            entry: None,
        };
        let settings = fs::read_to_string(root.join(&request.bindings)).unwrap();
        let config: BindingsConfig = toml::from_str(&settings).unwrap();
        let host = fs::read_to_string(root.join(&request.host_manifest)).unwrap();
        for _ in 0..2 {
            let mut memory = Vec::new();
            for (id, entry) in &config.entries {
                let declaration = fs::read_to_string(root.join(&entry.declaration)).unwrap();
                let dependency = fs::read_to_string(
                    root.join("host/node_modules")
                        .join(&entry.specifier)
                        .join("package.json"),
                )
                .ok();
                let previous = fs::read_to_string(
                    generated
                        .path()
                        .join(format!("{}.binding.json", entry.output)),
                )
                .ok();
                memory.push(
                    convert_entry(&ConversionInput {
                        settings: &settings,
                        entry: id,
                        declaration: &declaration,
                        host: HostMetadata {
                            manifest: &host,
                            resolved_package: dependency.as_deref(),
                        },
                        previous_metadata: previous.as_deref(),
                    })
                    .unwrap(),
                );
            }
            let disk = convert_package(&request).unwrap();
            assert_eq!(
                disk.diagnostics,
                memory
                    .iter()
                    .flat_map(|m| m.diagnostics.clone())
                    .collect::<Vec<_>>(),
                "{fixture}"
            );
            assert_eq!(
                disk.converted.len(),
                memory.iter().filter(|m| m.generated.is_some()).count()
            );
            for result in memory {
                if let Some(binding) = result.generated {
                    for (suffix, text) in [
                        ("ssrg", binding.source),
                        ("binding.json", binding.metadata),
                        ("report.json", binding.report),
                    ] {
                        assert_eq!(
                            fs::read_to_string(
                                generated
                                    .path()
                                    .join(format!("{}.{suffix}", binding.output))
                            )
                            .unwrap(),
                            text,
                            "{fixture}: {suffix}"
                        );
                    }
                }
            }
        }
    }
}
