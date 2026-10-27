use seseragi_driver::{
    analyze_module, analyze_project, compile_module, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_syntax::Diagnostic;

const FOREIGN: &str = concat!(
    "// 宣言 🙂\n",
    "pub foreign \"typescript\" from \"./host.mjs\" {\n",
    " deprecated \"Use current\" since \"0.61.0\" namespace legacy {\n",
    "  opaque type Handle\n",
    "  pure fn read value: Int -> Int\n",
    "  deprecated \"Use flat read\" namespace nested { pure fn read value: Int -> Int }\n",
    " }\n",
    " namespace current { pure fn read value: Int -> Int }\n",
    "}\n",
);

fn warnings(document: &seseragi_driver::AnalysisDocument) -> Vec<Diagnostic> {
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:#?}",
        document.diagnostics
    );
    document
        .authoring_diagnostics()
        .diagnostics
        .into_iter()
        .filter(|d| d.code == "SES-L0201")
        .collect()
}

#[test]
fn source_namespaces_warn_at_the_actual_owner_without_marking_their_children() {
    let source = format!("{FOREIGN}pub fn retain value: legacy.Handle -> legacy.Handle = value\npub fn answer -> Int = legacy . read 41 + legacy.nested.read 1 + current.read 0\n");
    let document = analyze_module(CompileInput::new(
        "λ/source.ssrg",
        "namespace/source",
        &source,
    ));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 5, "{facts:#?}");
    for fact in &facts {
        let name = &source[fact.primary.start..fact.primary.end];
        assert!(matches!(name, "legacy" | "nested"));
        let symbol = document.symbol_at(fact.primary.start).unwrap();
        assert_eq!(symbol.namespace, "module");
        assert_eq!(
            symbol.identity,
            if name == "legacy" {
                "namespace/source::legacy"
            } else {
                "namespace/source::legacy.nested"
            }
        );
        assert_eq!(fact.related[0].source.as_deref(), Some("λ/source.ssrg"));
        let notice = &source[fact.related[0].primary.start..fact.related[0].primary.end];
        assert!(notice.starts_with("deprecated \"Use "));
    }
    assert!(document
        .symbols
        .iter()
        .filter(|s| s.name.ends_with("read") || s.name.ends_with("Handle"))
        .all(|s| s.deprecation.is_none()));
    let root = document
        .workspace_symbols()
        .into_iter()
        .find(|s| s.identity == "namespace/source::legacy")
        .unwrap();
    assert_eq!(
        root.deprecation.as_ref().unwrap().since.as_deref(),
        Some("0.61.0")
    );
    assert!(document
        .visible_symbols(source.len() - 2)
        .iter()
        .all(|s| s.name != "legacy.nested"));
}

#[test]
fn public_namespace_metadata_preserves_generated_code_and_standard_source_maps() {
    let plain = FOREIGN
        .replace("deprecated \"Use current\" since \"0.61.0\" ", "")
        .replace("deprecated \"Use flat read\" ", "");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("λ/source.ssrg", "namespace/source", FOREIGN).with_profile(profile),
        )
        .unwrap();
        let ordinary = compile_module(
            CompileInput::new("λ/source.ssrg", "namespace/source", &plain).with_profile(profile),
        )
        .unwrap();
        assert_eq!(compiled.generated.typescript, ordinary.generated.typescript);
        assert_eq!(compiled.typed_interface.namespace_deprecations.len(), 2);
        assert!(compiled
            .typed_interface
            .exports
            .iter()
            .all(|export| export.deprecation.is_none()));
        let entries = &compiled.generated.metadata.deprecations;
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|entry| entry.namespace == "namespace"));
        assert_eq!(entries, &compiled.generated.source_map.deprecations);
        let raw =
            seseragi_lowering::emit_typescript_module(compiled.typescript_ir.clone(), FOREIGN);
        let mut map = serde_json::to_value(compiled.generated.source_map).unwrap();
        map.as_object_mut()
            .unwrap()
            .remove("x_seseragi_deprecations")
            .unwrap();
        assert_eq!(map, serde_json::to_value(raw.source_map).unwrap());
        for value in [
            serde_json::to_value(&ordinary.typed_interface).unwrap(),
            serde_json::to_value(ordinary.typed_interface.into_link_interface()).unwrap(),
        ] {
            assert!(value.get("namespaceDeprecations").is_none());
        }
    }
}

#[test]
fn imported_namespace_aliases_and_reexports_keep_the_provider_source_once_per_owner() {
    let provider = "// 元 🙂\npub foreign \"typescript\" from \"./host.mjs\" { deprecated \"Use current\" namespace legacy { pure fn read value: Int -> Int } }\n";
    let facade = "pub import * as api from \"./provider\"\n";
    let consumer =
        "import * as lib from \"./facade\"\npub fn answer -> Int = lib.api.legacy.read 41\n";
    let unused = "import * as lib from \"./facade\"\npub fn answer -> Int = 41\n";
    for (source, count) in [(consumer, 1), (unused, 0)] {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("api/model".into(), std::iter::empty::<(String, String)>())
            .unwrap();
        graph
            .add_module(
                "api/facade".into(),
                [("./provider".into(), "api/model".into())],
            )
            .unwrap();
        graph
            .add_module(
                "app/main".into(),
                [("./facade".into(), "api/facade".into())],
            )
            .unwrap();
        let analyzed = analyze_project(
            graph,
            [
                ProjectModuleInput::new("/λ/model.ssrg", "api/model", provider, "model.js"),
                ProjectModuleInput::new("/api/facade.ssrg", "api/facade", facade, "facade.js"),
                ProjectModuleInput::new("/app/main.ssrg", "app/main", source, "main.js"),
            ],
        )
        .unwrap();
        let publication = warnings(&analyzed.documents["api/facade"]);
        assert_eq!(publication.len(), 1, "{publication:#?}");
        assert_eq!(
            &facade[publication[0].primary.start..publication[0].primary.end],
            "api"
        );
        let facts = warnings(&analyzed.documents["app/main"]);
        assert_eq!(facts.len(), count, "{facts:#?}");
        for fact in publication.iter().chain(&facts) {
            assert_eq!(fact.related[0].source.as_deref(), Some("/λ/model.ssrg"));
            assert_eq!(
                &provider[fact.related[0].primary.start..fact.related[0].primary.end],
                "deprecated \"Use current\""
            );
        }
        if let Some(fact) = facts.first() {
            assert_eq!(&source[fact.primary.start..fact.primary.end], "legacy");
            assert_eq!(
                analyzed.documents["app/main"]
                    .symbol_at(fact.primary.start)
                    .unwrap()
                    .identity,
                "api/model::legacy"
            );
        }
    }
}

#[test]
fn private_owner_notices_stay_out_of_public_artifacts() {
    let source =
        FOREIGN.replacen("pub foreign", "foreign", 1) + "pub fn answer -> Int = legacy.read 41\n";
    let document = analyze_module(CompileInput::new(
        "private.ssrg",
        "namespace/private",
        &source,
    ));
    assert_eq!(warnings(&document).len(), 1);
    let compiled = compile_module(CompileInput::new(
        "private.ssrg",
        "namespace/private",
        &source,
    ))
    .unwrap();
    assert!(compiled.typed_interface.namespace_deprecations.is_empty());
    assert!(compiled.generated.metadata.deprecations.is_empty());
}
