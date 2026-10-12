//! Public semantic relationships and unsupported reasons, rather than parse counts.
use seseragi_dts::{interpret_declaration, project_graph};
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
        let projection = project_graph(&graph, &settings, "api").unwrap();
        let snapshot = serde_json::json!({ "semantic": graph, "compatibility": report, "projection": projection });
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
