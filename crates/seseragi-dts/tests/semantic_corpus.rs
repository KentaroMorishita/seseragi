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
        let snapshot = serde_json::json!({ "semantic": graph, "interpretation": report, "compatibility": converted.compatibility, "projection": projection });
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
        assert_eq!(snapshot, golden, "semantic/projection drift in {name}; inspect relationships/reasons before explicitly updating the corpus");
    }
}
