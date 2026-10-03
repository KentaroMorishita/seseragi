use seseragi_dts::{Converter, DeclarationInput, HostModuleIdentity};

const SETTINGS: &str = "schema = 1\n[entries.api]\ndeclaration = \"host/index.d.ts\"\nspecifier = \"fixture-api\"\noutput = \"fixture-api\"\n";

fn host(specifier: &str) -> HostModuleIdentity {
    HostModuleIdentity {
        specifier: specifier.to_owned(),
        exact_identity: format!("{specifier}@1.0.0"),
    }
}

macro_rules! fixture {
    ($name:literal, $id:literal, $output:literal) => {
        (
            $name,
            $id,
            $output,
            include_str!(concat!(
                "../../../examples/spec/fixtures/projects/",
                $name,
                "/seseragi.bindings.toml"
            )),
            include_str!(concat!(
                "../../../examples/spec/fixtures/projects/",
                $name,
                "/host/index.d.ts"
            )),
            include_str!(concat!(
                "../../../examples/spec/fixtures/projects/",
                $name,
                "/expected/",
                $output,
                ".ssrg"
            )),
        )
    };
}

const FIXTURES: &[(&str, &str, &str, &str, &str, &str)] = &[
    fixture!("dts-basic-conversion", "api", "fixture-api"),
    fixture!("dts-callback-during-call", "api", "callback-api"),
    fixture!("dts-declaration-merge", "merge", "merge-api"),
    fixture!("dts-generated-name", "naming", "naming-api"),
    fixture!("dts-namespace-runtime", "analytics", "analytics"),
    fixture!("dts-overload-selection", "parser", "parser-api"),
];

#[test]
fn converts_existing_declaration_text_without_filesystem_inputs() {
    for &(name, id, output, settings, source, expected) in FIXTURES {
        let result = Converter::new(settings)
            .unwrap()
            .convert(DeclarationInput {
                entry: id,
                source,
                host_module: host(output),
                previous_metadata: None,
            })
            .unwrap();
        assert!(!result.has_errors(), "{name}: {:?}", result.diagnostics);
        let generated = result.generated.unwrap();
        assert_eq!(generated.source, expected, "{name}");
        assert_eq!(generated.entry, id);
        assert_eq!(generated.output, output);
        assert!(generated.metadata.ends_with('\n'));
        assert!(generated.report.ends_with('\n'));
        let metadata: serde_json::Value = serde_json::from_str(&generated.metadata).unwrap();
        assert_eq!(
            metadata["hostModule"]["exactIdentity"],
            format!("{output}@1.0.0")
        );
    }
}

#[test]
fn retains_precise_conversion_errors_and_returns_no_generated_artifacts() {
    let source = "export declare function unsafe(value: any): unknown;\n";
    let result = Converter::new(SETTINGS)
        .unwrap()
        .convert(DeclarationInput {
            entry: "api",
            source,
            host_module: host("fixture-api"),
            previous_metadata: Some("invalid JSON"),
        })
        .unwrap();
    assert!(result.has_errors());
    assert!(result.generated.is_none());
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|value| value.code == "SES-F0101")
        .unwrap();
    assert_eq!(&source[diagnostic.start..diagnostic.end], "any");
    assert_eq!(diagnostic.file, "host/index.d.ts");
}

#[test]
fn uses_caller_supplied_previous_metadata_for_report_deltas() {
    let converter = Converter::new(SETTINGS).unwrap();
    let before = converter.convert(DeclarationInput {
        entry: "api",
        source: "export declare function changed(value: number): string;\nexport declare function removed(): string;\n",
        host_module: host("fixture-api"), previous_metadata: None,
    }).unwrap().generated.unwrap();
    let after = converter.convert(DeclarationInput {
        entry: "api",
        source: "export declare function changed(value: string): string;\nexport declare function added(): string;\n",
        host_module: host("fixture-api"), previous_metadata: Some(&before.metadata),
    }).unwrap().generated.unwrap();
    let report: serde_json::Value = serde_json::from_str(&after.report).unwrap();
    for (field, symbol) in [
        ("added", "added"),
        ("changed", "changed"),
        ("removed", "removed"),
    ] {
        let values = report[field].as_array().unwrap();
        assert_eq!(values.len(), 1, "{field}");
        assert!(values[0].as_str().unwrap().contains(symbol));
    }
}

#[test]
fn validates_settings_entry_selection_and_caller_host_identity() {
    assert!(Converter::new("schema = 99\n").is_err());
    let converter = Converter::new(SETTINGS).unwrap();
    for (entry, identity) in [("missing", host("fixture-api")), ("api", host("other-api"))] {
        assert!(converter
            .convert(DeclarationInput {
                entry,
                source: "export declare function format(value: number): string;",
                host_module: identity,
                previous_metadata: None,
            })
            .is_err());
    }
}

#[cfg(feature = "filesystem")]
#[test]
fn filesystem_adapter_writes_the_exact_in_memory_source_metadata_and_report() {
    use seseragi_dts::{convert_package, ConvertRequest};
    use std::path::{Path, PathBuf};

    for &(name, id, output, settings, source, _) in FIXTURES {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects")
            .join(name);
        let temporary = tempfile::TempDir::new().unwrap();
        let request = ConvertRequest {
            package_root: root,
            generated_root: temporary.path().to_path_buf(),
            bindings: PathBuf::from("seseragi.bindings.toml"),
            host_manifest: PathBuf::from("host/package.json"),
            entry: Some(id.to_owned()),
        };
        let converter = Converter::new(settings).unwrap();
        let mut previous = None;
        for _ in 0..2 {
            let memory = converter
                .convert(DeclarationInput {
                    entry: id,
                    source,
                    host_module: host(output),
                    previous_metadata: previous.as_deref(),
                })
                .unwrap();
            let disk = convert_package(&request).unwrap();
            assert_eq!(disk.diagnostics, memory.diagnostics, "{name}");
            assert_eq!(disk.converted.len(), 1, "{name}");
            let generated = memory.generated.unwrap();
            for (suffix, contents) in [
                ("ssrg", &generated.source),
                ("binding.json", &generated.metadata),
                ("report.json", &generated.report),
            ] {
                assert_eq!(
                    std::fs::read_to_string(temporary.path().join(format!("{output}.{suffix}")))
                        .unwrap(),
                    *contents,
                    "{name}: {suffix}"
                );
            }
            previous = Some(generated.metadata);
        }
    }
}
