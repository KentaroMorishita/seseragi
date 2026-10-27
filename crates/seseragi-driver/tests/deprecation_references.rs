use seseragi_driver::{analyze_module, analyze_project, CompileInput, ProjectModuleInput};
use seseragi_project::ModuleGraph;
use seseragi_syntax::{Diagnostic, DiagnosticSeverity};

fn deprecations(document: &seseragi_driver::AnalysisDocument) -> Vec<Diagnostic> {
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:?}",
        document.diagnostics
    );
    document
        .authoring_diagnostics()
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.code == "SES-L0201")
        .collect()
}

#[test]
fn deprecation_references_exclude_own_body_and_keep_each_external_use() {
    let source = concat!(
        "// 移行 🙂\n",
        "pub deprecated \"Use modern 関数\" since \"0.61.0\"\n",
        "fn historical value: Int -> Int = if value < 0 then historical 0 else value\n",
        "pub fn answer -> Int = historical 41 + historical 1\n",
        "pub fn shadow historical: Int -> Int = historical\n",
    );
    let document = analyze_module(CompileInput::new(
        "unicode.ssrg",
        "deprecation/local",
        source,
    ));
    let facts = deprecations(&document);
    assert_eq!(facts.len(), 2, "{facts:?}");
    for diagnostic in facts {
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            "historical"
        );
        assert!(diagnostic.primary.start >= source.find("pub fn answer").unwrap());
        assert_eq!(diagnostic.related.len(), 1);
        let related = &diagnostic.related[0];
        assert_eq!(related.source.as_deref(), Some("unicode.ssrg"));
        assert_eq!(
            &source[related.primary.start..related.primary.end],
            "deprecated \"Use modern 関数\" since \"0.61.0\""
        );
        assert!(diagnostic.message().contains("Use modern 関数"));
        assert!(diagnostic.message().contains("0.61.0"));
        assert!(diagnostic.fixes.is_empty());
    }
    let deprecated = document
        .symbols
        .iter()
        .find(|symbol| {
            symbol.name == "historical" && symbol.identity == "deprecation/local::historical"
        })
        .unwrap();
    assert_eq!(
        deprecated.deprecation.as_ref().unwrap().since.as_deref(),
        Some("0.61.0")
    );
    assert!(document
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == "parameter")
        .all(|symbol| symbol.deprecation.is_none()));
}

#[test]
fn private_values_and_types_keep_namespace_without_tagging_constructors_or_fields() {
    let source = concat!(
        "deprecated \"Use modern\"\nfn historical value: Int -> Int = value\n",
        "pub deprecated \"Use Ticket\"\ntype Old = | Old Int\n",
        "pub fn construct -> Int = { let _old = Old 1\n historical 41 }\n",
        "pub fn retain value: Old -> Old = value\n",
        "pub struct Holder { historical: Int }\n",
        "pub fn read value: Holder -> Int = value.historical\n",
    );
    let document = analyze_module(CompileInput::new("types.ssrg", "deprecation/types", source));
    let facts = deprecations(&document);
    assert_eq!(facts.len(), 3, "{facts:?}");
    assert_eq!(
        facts
            .iter()
            .filter(|fact| &source[fact.primary.start..fact.primary.end] == "Old")
            .count(),
        2
    );
    assert!(document
        .symbols
        .iter()
        .filter(|symbol| symbol.kind == "constructor" || symbol.namespace == "field")
        .all(|symbol| symbol.deprecation.is_none()));
    assert!(document.symbols.iter().any(|symbol| symbol.name == "Old"
        && symbol.namespace == "type"
        && symbol.deprecation.is_some()));
}

#[test]
fn imports_do_not_warn_until_used_and_reexports_keep_defining_provenance() {
    let model = "// 元のAPI 🙂\npub deprecated \"Use modern\" since \"0.61.0\"\nfn old value: Int -> Int = value\npub fn modern value: Int -> Int = value\n";
    let facade = "pub import { old } from \"./model\"\n";
    let consumer = "import { old as historical } from \"./facade\"\nimport * as api from \"./model\"\npub fn answer -> Int = historical 41 + api.old 1\n";
    let unused = "import { old } from \"./model\"\npub fn answer -> Int = 42\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module("model".to_owned(), std::iter::empty::<(String, String)>())
        .unwrap();
    graph
        .add_module(
            "facade".to_owned(),
            [("./model".to_owned(), "model".to_owned())],
        )
        .unwrap();
    graph
        .add_module(
            "main".to_owned(),
            [
                ("./model".to_owned(), "model".to_owned()),
                ("./facade".to_owned(), "facade".to_owned()),
            ],
        )
        .unwrap();
    graph
        .add_module(
            "unused".to_owned(),
            [("./model".to_owned(), "model".to_owned())],
        )
        .unwrap();
    let project = analyze_project(
        graph,
        [
            ProjectModuleInput::new("model.ssrg", "model", model, "model.js"),
            ProjectModuleInput::new("facade.ssrg", "facade", facade, "facade.js"),
            ProjectModuleInput::new("main.ssrg", "main", consumer, "main.js"),
            ProjectModuleInput::new("unused.ssrg", "unused", unused, "unused.js"),
        ],
    )
    .unwrap();
    assert!(deprecations(&project.documents["model"]).is_empty());
    assert!(deprecations(&project.documents["unused"]).is_empty());
    let exported = deprecations(&project.documents["facade"]);
    assert_eq!(exported.len(), 1, "{exported:?}");
    assert!(facade[exported[0].primary.start..exported[0].primary.end].contains("old"));
    let used = deprecations(&project.documents["main"]);
    assert_eq!(used.len(), 2, "{used:?}");
    for fact in used.iter().chain(&exported) {
        let related = &fact.related[0];
        assert_eq!(related.source.as_deref(), Some("model.ssrg"));
        assert_eq!(
            &model[related.primary.start..related.primary.end],
            "deprecated \"Use modern\" since \"0.61.0\""
        );
    }
    let imported = project.documents["main"]
        .symbols
        .iter()
        .find(|symbol| symbol.identity == "model::old")
        .unwrap();
    assert_eq!(imported.deprecation.as_ref().unwrap().source, "model.ssrg");
}

#[test]
fn invalid_program_does_not_add_speculative_deprecation_warnings() {
    let source = "pub deprecated \"Use modern\"\nfn old value: Int -> Int = value\npub fn answer -> Int = old missing\n";
    let document = analyze_module(CompileInput::new(
        "broken.ssrg",
        "deprecation/broken",
        source,
    ));
    assert!(document
        .diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error));
    assert!(!document
        .authoring_diagnostics()
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-L0201"));
}

#[test]
fn same_named_modules_keep_distinct_deprecation_sources() {
    let left = "pub deprecated \"Use leftModern\" fn old value: Int -> Int = value\n";
    let right = "pub deprecated \"Use rightModern\" fn old value: Int -> Int = value\n";
    let consumer = "import { old as leftOld } from \"./left\"\nimport { old as rightOld } from \"./right\"\npub let answer: Int = leftOld 40 + rightOld 2\n";
    let mut graph = ModuleGraph::new();
    for module in ["left", "right"] {
        graph
            .add_module(module.to_owned(), std::iter::empty::<(String, String)>())
            .unwrap();
    }
    graph
        .add_module(
            "main".to_owned(),
            [
                ("./left".to_owned(), "left".to_owned()),
                ("./right".to_owned(), "right".to_owned()),
            ],
        )
        .unwrap();
    let project = analyze_project(
        graph,
        [
            ProjectModuleInput::new("/left/src/lib.ssrg", "left", left, "left.js"),
            ProjectModuleInput::new("/right/src/lib.ssrg", "right", right, "right.js"),
            ProjectModuleInput::new("/app/main.ssrg", "main", consumer, "main.js"),
        ],
    )
    .unwrap();
    let facts = deprecations(&project.documents["main"]);
    assert_eq!(facts.len(), 2, "{facts:?}");
    assert_eq!(
        facts[0].related[0].source.as_deref(),
        Some("/left/src/lib.ssrg")
    );
    assert_eq!(
        facts[1].related[0].source.as_deref(),
        Some("/right/src/lib.ssrg")
    );
    assert!(facts[0].message().contains("leftModern"));
    assert!(facts[1].message().contains("rightModern"));
}

#[test]
fn path_package_warning_keeps_original_package_symbol_and_source() {
    use seseragi_project::{logical_module_id, logical_package_scope};
    let project = seseragi_project::load_local_documents(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/deprecation"
    ))
    .unwrap();
    let mut graph = ModuleGraph::new();
    for (identity, _) in project.modules() {
        graph
            .add_module(
                logical_module_id(identity),
                project
                    .graph()
                    .dependencies_for(identity)
                    .unwrap()
                    .into_iter()
                    .map(|(specifier, dependency)| (specifier, logical_module_id(&dependency))),
            )
            .unwrap();
    }
    let analyzed = analyze_project(
        graph,
        project.modules().map(|(identity, module)| {
            let logical = logical_module_id(identity);
            ProjectModuleInput::new(
                module.source_path().to_string_lossy(),
                &logical,
                module.source(),
                format!("lint/{logical}.js"),
            )
            .with_package_scope(logical_package_scope(identity.package()))
        }),
    )
    .unwrap();
    let (main_id, main_source) = project
        .modules()
        .find(|(_, module)| module.source_path().ends_with("src/main.ssrg"))
        .map(|(identity, module)| (logical_module_id(identity), module.source()))
        .unwrap();
    let main = &analyzed.documents[&main_id];
    let facts = deprecations(main);
    let original = main
        .symbols
        .iter()
        .find(|symbol| symbol.identity.ends_with("::historical"))
        .unwrap();
    assert!(original.identity.contains("deprecation-model"));
    let metadata = original.deprecation.as_ref().unwrap();
    assert!(
        std::path::Path::new(&metadata.source).ends_with("vendor/model/src/lib.ssrg"),
        "{metadata:?}"
    );
    assert_eq!(metadata.message, "Use modern");
    assert!(facts
        .iter()
        .any(
            |fact| main_source[fact.primary.start..fact.primary.end].contains("older")
                && fact.related[0].source.as_deref() == Some(metadata.source.as_str())
        ));
    assert!(!facts
        .iter()
        .any(|fact| &main_source[fact.primary.start..fact.primary.end] == "Label"));
}
