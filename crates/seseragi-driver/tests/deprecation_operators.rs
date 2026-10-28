use seseragi_driver::{
    analyze_module, analyze_project, compile_module, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_syntax::Diagnostic;

const BOX: &str = concat!(
    "// 演算 🙂\n",
    "pub struct Box { pub value: Int }\n",
    "impl Box {\n",
    " deprecated \"Use combine\" since \"0.61.0\" operator + self -> other: Box -> Box = Box { value: self.value + other.value }\n",
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
fn only_the_selected_implementation_warns_for_operator_and_method_uses() {
    let source = format!("{BOX}pub struct Other {{ pub value: Int }}\nimpl Other {{ operator + self -> other: Other -> Other = other }}\npub fn binary left: Box -> right: Box -> Box = left + right\npub fn method left: Box -> right: Box -> Box = add left right\npub fn section left: Box -> right: Box -> Box = {{ let selected: Box -> Box -> Box = (+); selected left right }}\npub fn ordinary left: Other -> right: Other -> Other = left + right\npub fn scalar -> Int = 1 + 2\n");
    let document = analyze_module(CompileInput::new("λ/box.ssrg", "operators/local", &source));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 3, "{facts:#?}");
    for fact in &facts {
        assert!(matches!(
            &source[fact.primary.start..fact.primary.end],
            "+" | "add"
        ));
        assert!(fact.primary.start > source.find("pub fn binary").unwrap());
        let symbol = document.symbol_at(fact.primary.start).unwrap();
        assert_eq!(symbol.kind, "operator-implementation");
        assert!(symbol
            .identity
            .starts_with("operators/local::implementation("));
        assert!(symbol
            .member_owner
            .as_ref()
            .unwrap()
            .contains("operators/local::Box"));
        assert_eq!(fact.related[0].source.as_deref(), Some("λ/box.ssrg"));
        assert_eq!(
            &source[fact.related[0].primary.start..fact.related[0].primary.end],
            "deprecated \"Use combine\" since \"0.61.0\""
        );
    }
    assert!(document
        .symbols
        .iter()
        .filter(|symbol| symbol.identity.starts_with("std/prelude::"))
        .all(|symbol| symbol.deprecation.is_none()));
    assert!(document
        .visible_symbols(source.len() - 2)
        .iter()
        .all(|symbol| symbol.kind != "operator-implementation"));
    let ordinary = document
        .symbol_at(source.rfind("1 + 2").unwrap() + 2)
        .unwrap();
    assert!(ordinary.deprecation.is_none());
}

#[test]
fn notices_follow_imported_dictionary_identity_through_a_public_facade() {
    let facade = "pub import { Box } from \"./model\"\n";
    let consumer = "import { Box as Container } from \"./facade\"\npub fn answer left: Container -> right: Container -> Container = left + right\npub fn ordinary -> Int = 1 + 2\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module("api/model".into(), std::iter::empty::<(String, String)>())
        .unwrap();
    graph
        .add_module(
            "api/facade".into(),
            [("./model".into(), "api/model".into())],
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
            ProjectModuleInput::new("/λ/model.ssrg", "api/model", BOX, "model.js"),
            ProjectModuleInput::new("/api/facade.ssrg", "api/facade", facade, "facade.js"),
            ProjectModuleInput::new("/app/main.ssrg", "app/main", consumer, "main.js"),
        ],
    )
    .unwrap();
    assert!(warnings(&analyzed.documents["api/model"]).is_empty());
    assert!(warnings(&analyzed.documents["api/facade"]).is_empty());
    let document = &analyzed.documents["app/main"];
    let facts = warnings(document);
    assert_eq!(facts.len(), 1, "{facts:#?}");
    assert_eq!(
        facts[0].primary.start,
        consumer.find("left + right").unwrap() + 5
    );
    assert_eq!(facts[0].related[0].source.as_deref(), Some("/λ/model.ssrg"));
    let symbol = document.symbol_at(facts[0].primary.start).unwrap();
    assert!(symbol.identity.starts_with("api/model::implementation("));
    assert_eq!(symbol.deprecation.as_ref().unwrap().message, "Use combine");
}

#[test]
fn metadata_keeps_instance_contracts_and_generated_code_unchanged() {
    let source = format!("{BOX}pub fn answer left: Box -> right: Box -> Box = left + right\n");
    let plain = source.replace("deprecated \"Use combine\" since \"0.61.0\" ", "");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("box.ssrg", "operators/metadata", &source).with_profile(profile),
        )
        .unwrap();
        let ordinary = compile_module(
            CompileInput::new("box.ssrg", "operators/metadata", &plain).with_profile(profile),
        )
        .unwrap();
        assert_eq!(compiled.generated.typescript, ordinary.generated.typescript);
        let notices = &compiled.typed_interface.operator_deprecations;
        assert_eq!(notices.len(), 1, "{notices:#?}");
        let notice = &notices[0];
        assert!(compiled
            .typed_interface
            .instances
            .iter()
            .any(
                |instance| instance.identity.as_deref() == Some(notice.instance.as_str())
                    && instance.provider_module.as_deref() == Some(notice.provider_module.as_str())
                    && instance.trait_identity.as_deref() == Some(notice.trait_identity.as_str())
            ));
        let generated = &compiled.generated.metadata.deprecations;
        assert_eq!(generated.len(), 1);
        assert_eq!(generated[0].identity, notice.symbol());
        assert_eq!(
            generated[0].owner.as_deref(),
            Some(notice.instance.as_str())
        );
        assert_eq!(generated, &compiled.generated.source_map.deprecations);
        let contracts = |module: &seseragi_driver::CompiledModule| {
            module
                .core_ir
                .instances
                .iter()
                .map(|instance| {
                    (
                        instance.identity.clone(),
                        instance.trait_name.clone(),
                        instance.type_parameters.clone(),
                        instance.arguments.clone(),
                        instance.type_identity.clone(),
                        instance.constraints.clone(),
                        instance.supertrait_count,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(contracts(&compiled), contracts(&ordinary));
        assert!(serde_json::to_value(ordinary.typed_interface)
            .unwrap()
            .get("operatorDeprecations")
            .is_none());
    }
}

#[test]
fn scoped_dictionary_parameters_and_error_recovery_do_not_guess_an_implementation() {
    let source = format!("{BOX}pub fn generic<A> left: A -> right: A -> A where Add<A, A, A> = left + right\npub fn ordinary -> Int = 1 + 2\n");
    let document = analyze_module(CompileInput::new(
        "generic.ssrg",
        "operators/generic",
        &source,
    ));
    assert!(warnings(&document).is_empty());
    let invalid = format!("{BOX}pub fn invalid -> Int = missing + 1\n");
    let document = analyze_module(CompileInput::new(
        "invalid.ssrg",
        "operators/invalid",
        &invalid,
    ));
    assert!(!document.diagnostics.diagnostics.is_empty());
    assert!(document
        .authoring_diagnostics()
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code != "SES-L0201"));
}
