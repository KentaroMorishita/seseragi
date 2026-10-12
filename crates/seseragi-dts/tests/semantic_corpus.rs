//! Public semantic relationships and unsupported reasons, rather than parse counts.
use seseragi_dts::{
    convert_entry, interpret_declaration, project_graph, ConversionInput, HostMetadata,
};
use seseragi_interop::SemanticCompatibilityReport;
use std::path::PathBuf;

#[test]
fn semantic_corpus_relationships_classification_and_projection_are_stable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in ["domains", "relations", "structures", "callables"] {
        let source = std::fs::read_to_string(root.join(format!(
            "examples/spec/fixtures/semantic-interop-schema-1/{name}.d.ts"
        )))
        .unwrap();
        let path = format!("{name}.d.ts");
        let settings = format!("schema = 1\n[entries.api]\ndeclaration = '{path}'\noutput = 'api'\nspecifier = './api.js'\nevaluation = 'pure'\n");
        let graph = interpret_declaration(&source, &path, &settings).unwrap();
        assert!(!graph.symbols.is_empty(), "{name} lost its public surface");
        let report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
        if name == "domains" {
            use seseragi_interop::{LiteralValue, SemanticClassification, SemanticType};
            for (name, value) in [
                ("RoundedPositive", 9_007_199_254_740_992),
                ("RoundedNegative", -9_007_199_254_740_992),
            ] {
                assert_eq!(
                    graph.types[&graph.symbols[name].type_ref],
                    SemanticType::Literal {
                        value: LiteralValue::Integer(value)
                    }
                );
                assert_eq!(
                    report.symbols[name].classification,
                    SemanticClassification::Normalized
                );
            }
            for name in ["Fraction", "OutsideInt64"] {
                assert_eq!(
                    report.symbols[name].classification,
                    SemanticClassification::PolicyRequired
                );
                assert!(report.symbols[name]
                    .reasons
                    .iter()
                    .any(|reason| reason.rule == "typescript-binary64-literal"));
            }
        }
        let projection = project_graph(&graph, &settings, "api").unwrap();
        let converted = convert_entry(&ConversionInput {
            settings: &settings,
            entry: "api",
            declaration: &source,
            host: HostMetadata {
                manifest: "",
                resolved_package: None,
            },
            previous_metadata: None,
        })
        .unwrap();
        assert_eq!(converted.semantic.as_ref(), Some(&graph));
        assert_eq!(
            graph.identity.as_ref().unwrap().frontend.version,
            env!("CARGO_PKG_VERSION"),
            "production artifacts must retain the actual producer version"
        );
        let snapshot = serde_json::json!({ "semantic": graph, "interpretation": report, "compatibility": converted.compatibility, "projection": projection });
        let snapshot = release_neutral_snapshot(snapshot).unwrap();
        let expected = root.join(format!(
            "examples/spec/artifacts/semantic-interop-schema-1/{name}.json"
        ));
        if std::env::var_os("SESERAGI_UPDATE_SEMANTIC_CORPUS").is_some() {
            std::fs::write(
                &expected,
                format!("{}\n", serde_json::to_string_pretty(&snapshot).unwrap()),
            )
            .unwrap();
        }
        let golden: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&expected).unwrap()).unwrap();
        let golden = release_neutral_snapshot(golden).unwrap();
        assert_eq!(snapshot, golden, "semantic/projection drift in {name}; inspect relationships/reasons before explicitly updating the corpus");
    }
}

// Only this semantic fixture comparison is release-neutral. Production graph and
// binding identity/freshness tests still compare the actual producer version.
// Verify the original report digest/identity before recalculating the digest;
// deleting graphDigest would hide stale reports and real provenance changes.
fn release_neutral_snapshot(mut snapshot: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut graph: seseragi_interop::SemanticGraph =
        serde_json::from_value(snapshot["semantic"].clone()).map_err(|e| e.to_string())?;
    graph.validate().map_err(|e| e.to_string())?;
    let original_digest = graph.digest().map_err(|e| e.to_string())?;
    let original_identity = serde_json::to_value(&graph.identity).map_err(|e| e.to_string())?;
    let frontend = &mut graph
        .identity
        .as_mut()
        .ok_or("missing graph identity")?
        .frontend;
    if frontend.name != "seseragi-dts" || frontend.version.is_empty() {
        return Err("unexpected semantic corpus producer identity".to_owned());
    }
    frontend.version = "semantic-corpus-release-neutral".to_owned();
    let normalized_digest = graph.digest().map_err(|e| e.to_string())?;
    let normalized_identity = serde_json::to_value(&graph.identity).map_err(|e| e.to_string())?;
    for field in ["interpretation", "compatibility"] {
        let report = snapshot.get_mut(field).ok_or("missing semantic report")?;
        if report["graphDigest"] != original_digest || report["identity"] != original_identity {
            return Err(format!(
                "{field} does not describe the original complete graph"
            ));
        }
        report["graphDigest"] = normalized_digest.clone().into();
        report["identity"] = normalized_identity.clone();
    }
    snapshot["semantic"] = serde_json::to_value(graph).map_err(|e| e.to_string())?;
    Ok(snapshot)
}

fn small_snapshot(source: &str) -> serde_json::Value {
    let settings = "schema = 1\n[entries.api]\ndeclaration = 'api.d.ts'\noutput = 'api'\nspecifier = './api.js'\nevaluation = 'pure'\n";
    let graph = interpret_declaration(source, "api.d.ts", settings).unwrap();
    let report = SemanticCompatibilityReport::from_graph(&graph).unwrap();
    let projection = project_graph(&graph, settings, "api").unwrap();
    serde_json::json!({"semantic": graph, "interpretation": report, "compatibility": report, "projection": projection})
}

fn refresh_report_identity(snapshot: &mut serde_json::Value) {
    let graph: seseragi_interop::SemanticGraph =
        serde_json::from_value(snapshot["semantic"].clone()).unwrap();
    let digest = graph.digest().unwrap();
    for field in ["interpretation", "compatibility"] {
        snapshot[field]["identity"] = snapshot["semantic"]["identity"].clone();
        snapshot[field]["graphDigest"] = digest.clone().into();
    }
}

#[test]
fn release_only_changes_match_without_changing_production_identity() {
    let original = small_snapshot("export type Value = string;");
    let mut next_release = original.clone();
    next_release["semantic"]["identity"]["frontend"]["version"] = "999.0.1".into();
    refresh_report_identity(&mut next_release);
    assert_ne!(original, next_release);
    assert_eq!(
        release_neutral_snapshot(original.clone()).unwrap(),
        release_neutral_snapshot(next_release).unwrap()
    );
    assert_eq!(
        original["semantic"]["identity"]["frontend"]["version"],
        env!("CARGO_PKG_VERSION")
    );
}

#[test]
fn release_neutral_comparison_rejects_stale_digests_and_producer_mismatches() {
    let original = small_snapshot("export type Value = string;");
    for field in ["interpretation", "compatibility"] {
        let mut bad = original.clone();
        bad[field]["graphDigest"] = "unrelated-graph-digest".into();
        assert!(release_neutral_snapshot(bad).is_err());
        let mut bad = original.clone();
        bad[field]["identity"]["frontend"]["version"] = "another-release".into();
        assert!(release_neutral_snapshot(bad).is_err());
    }
    let mut bad = original;
    bad["semantic"]["identity"]["frontend"]["name"] = "another-frontend".into();
    refresh_report_identity(&mut bad);
    assert!(release_neutral_snapshot(bad).is_err());
}

#[test]
fn release_neutral_comparison_retains_semantics_provenance_policies_and_projection() {
    let original = small_snapshot("export type Value<A> = {readonly value: A};");
    let baseline = release_neutral_snapshot(original.clone()).unwrap();
    // Different sources exercise real graph type/relationship and digest changes.
    for source in [
        "export type Value<A> = {readonly value: string};",
        "export type Value<A> = {value: A};",
        "\nexport type Value<A> = {readonly value: A};",
    ] {
        assert_ne!(
            baseline,
            release_neutral_snapshot(small_snapshot(source)).unwrap()
        );
    }
    for pointer in [
        "/semantic/identity/settingsDigest",
        "/semantic/identity/runtimeContractDigest",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = "f".repeat(64).into();
        refresh_report_identity(&mut changed);
        assert_ne!(baseline, release_neutral_snapshot(changed).unwrap());
    }
    for pointer in [
        "/interpretation/symbols/Value/semanticDigest",
        "/compatibility/symbols/Value/classification",
        "/projection/source",
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = "different-evidence".into();
        assert_ne!(baseline, release_neutral_snapshot(changed).unwrap());
    }
    let mut changed = original;
    changed["compatibility"]["symbols"]["Value"]["reasons"].as_array_mut().unwrap().push(
        serde_json::json!({"rule": "explicit-policy", "reason": "meaning changed", "policy": {"name": "policy", "version": "new-policy"}})
    );
    assert_ne!(baseline, release_neutral_snapshot(changed).unwrap());
}
