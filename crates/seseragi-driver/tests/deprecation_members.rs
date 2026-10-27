use seseragi_driver::{
    analyze_module, analyze_project, compile_module, BuildProfile, CompileInput, ProjectModuleInput,
};
use seseragi_project::ModuleGraph;
use seseragi_syntax::{Diagnostic, DiagnosticSeverity};

fn warnings(document: &seseragi_driver::AnalysisDocument) -> Vec<Diagnostic> {
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
fn trait_method_implementation_is_not_a_use_and_call_keeps_the_original_notice() {
    let source = concat!(
        "// 元の宣言 🙂\n",
        "pub trait Render<A> {\n",
        "  deprecated \"Use current\" since \"0.61.0\"\n",
        "  fn old value: A -> String\n",
        "}\n",
        "instance Render<Int> { fn old value: Int -> String = \"old\" }\n",
        "pub fn answer -> String = old 41\n",
    );
    let document = analyze_module(CompileInput::new(
        "λ/trait.ssrg",
        "deprecation/trait",
        source,
    ));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 1, "{facts:#?}");
    assert_eq!(facts[0].primary.start, source.rfind("old 41").unwrap());
    assert_eq!(&source[facts[0].primary.start..facts[0].primary.end], "old");
    assert_eq!(facts[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(facts[0].related[0].source.as_deref(), Some("λ/trait.ssrg"));
    assert_eq!(
        &source[facts[0].related[0].primary.start..facts[0].related[0].primary.end],
        "deprecated \"Use current\" since \"0.61.0\""
    );
    let method = document.symbol_at(facts[0].primary.start).unwrap();
    assert_eq!(method.identity, "deprecation/trait::trait(Render)::old");
    assert_eq!(
        method.member_owner.as_deref(),
        Some("deprecation/trait::trait(Render)")
    );
    assert_eq!(method.deprecation.as_ref().unwrap().message, "Use current");
    let owner = document
        .symbols
        .iter()
        .find(|symbol| symbol.name == "Render")
        .unwrap();
    assert!(owner.deprecation.is_none());
    assert_eq!(warnings(&document).len(), 1);
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let plain = source.replace("  deprecated \"Use current\" since \"0.61.0\"\n", "");
        let before = compile_module(
            CompileInput::new("λ/trait.ssrg", "deprecation/trait", &plain).with_profile(profile),
        )
        .unwrap();
        let after = compile_module(
            CompileInput::new("λ/trait.ssrg", "deprecation/trait", source).with_profile(profile),
        )
        .unwrap();
        assert_eq!(after.generated.typescript, before.generated.typescript);
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
        assert_eq!(contracts(&after), contracts(&before));
        let owner = after
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "Render")
            .unwrap();
        assert_eq!(
            owner.methods[0].deprecation.as_ref().unwrap().message,
            "Use current"
        );
    }
}

#[test]
fn same_named_trait_methods_warn_only_for_the_selected_canonical_owner() {
    let source = concat!(
        "type Mark = | Mark Int\n",
        "type Word = | Word String\n",
        "trait Render<A> { deprecated \"Use current\" fn old value: A -> String }\n",
        "trait Encode<A> { fn old value: A -> String }\n",
        "instance Render<Mark> { fn old value: Mark -> String = \"mark\" }\n",
        "instance Encode<Word> { fn old value: Word -> String = \"word\" }\n",
        "pub fn legacy -> String = old (Mark 41)\n",
        "pub fn ordinary -> String = old (Word \"ok\")\n",
    );
    let document = analyze_module(CompileInput::new(
        "owners.ssrg",
        "deprecation/owners",
        source,
    ));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 1, "{facts:#?}");
    assert_eq!(facts[0].primary.start, source.find("old (Mark").unwrap());
    assert_eq!(
        document.symbol_at(facts[0].primary.start).unwrap().identity,
        "deprecation/owners::trait(Render)::old"
    );
    let ordinary = document
        .symbol_at(source.find("old (Word").unwrap())
        .unwrap();
    assert_eq!(ordinary.identity, "deprecation/owners::trait(Encode)::old");
    assert!(ordinary.deprecation.is_none());
}

#[test]
fn inherent_member_uses_are_navigable_without_becoming_lexical_functions() {
    let source = concat!(
        "pub struct Box { value: Int }\n",
        "impl Box {\n",
        " pub fn read self: Box -> Int = self.value\n",
        " pub deprecated \"Use read\" since \"0.61.0\"\n",
        " fn old self: Box -> Int = if self.value < 0 then self.old else self.value\n",
        "}\n",
        "pub struct Other { old: Int }\n",
        "pub fn field other: Other -> Int = other.old\n",
        "pub fn answer box: Box -> Int = box.old + box.old\n",
    );
    let document = analyze_module(CompileInput::new("λ/box.ssrg", "deprecation/box", source));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 2, "{facts:#?}");
    for fact in &facts {
        assert_eq!(&source[fact.primary.start..fact.primary.end], "old");
        assert!(fact.primary.start > source.find("pub fn answer").unwrap());
        let method = document.symbol_at(fact.primary.start).unwrap();
        assert_eq!(method.identity, "deprecation/box::Box::old");
        assert_eq!(method.member_owner.as_deref(), Some("deprecation/box::Box"));
        assert_eq!(method.kind, "inherent-method");
        assert_eq!(
            &source[method.definition.start..method.definition.end],
            "old"
        );
    }
    assert!(document
        .workspace_symbols()
        .iter()
        .any(|symbol| symbol.identity == "deprecation/box::Box::old"));
    assert!(!document
        .visible_symbols(source.rfind("box.old").unwrap())
        .iter()
        .any(|symbol| symbol.identity == "deprecation/box::Box::old"));
    assert!(document
        .symbol_at(source.find("other.old").unwrap() + "other.".len())
        .is_none());
}

#[test]
fn imported_inherent_member_keeps_provider_source_through_a_type_alias() {
    let provider = "// provider 🙂\npub struct Box { value: Int }\nimpl Box { pub deprecated \"Use read\" since \"0.61.0\" fn old self: Box -> Int = self.value }\n";
    let consumer = "import { Box as Container } from \"./provider\"\npub fn answer box: Container -> Int = box.old\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "api/provider".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "app/main".to_owned(),
            [("./provider".to_owned(), "api/provider".to_owned())],
        )
        .unwrap();
    let analyzed = analyze_project(
        graph,
        [
            ProjectModuleInput::new(
                "/api/src/model.ssrg",
                "api/provider",
                provider,
                "api/provider.js",
            ),
            ProjectModuleInput::new("/app/src/model.ssrg", "app/main", consumer, "app/main.js"),
        ],
    )
    .unwrap();
    let document = &analyzed.documents["app/main"];
    let facts = warnings(document);
    assert_eq!(facts.len(), 1, "{facts:#?}");
    assert_eq!(
        &consumer[facts[0].primary.start..facts[0].primary.end],
        "old"
    );
    assert_eq!(
        facts[0].related[0].source.as_deref(),
        Some("/api/src/model.ssrg")
    );
    assert!(
        provider[facts[0].related[0].primary.start..facts[0].related[0].primary.end]
            .starts_with("deprecated")
    );
    let symbol = document.symbol_at(facts[0].primary.start).unwrap();
    assert_eq!(symbol.identity, "api/provider::Box::old");
    assert_eq!(symbol.member_owner.as_deref(), Some("api/provider::Box"));
}

#[test]
fn foreign_function_and_opaque_type_keep_metadata_without_host_name_inference() {
    let source = concat!(
        "pub foreign \"typescript\" from \"./host.mjs\" {\n",
        " deprecated \"Use NewHandle\" opaque type Handle\n",
        " deprecated \"Use current\" since \"0.61.0\" pure fn old value: Int -> Int = \"old_host\"\n",
        " pure fn current value: Int -> Int\n",
        " namespace nested { deprecated \"Use modern\" pure fn old value: Int -> Int }\n",
        "}\n",
        "pub fn retain handle: Handle -> Handle = handle\n",
        "pub fn answer -> Int = old 41 + nested.old 1 + current 0\n",
    );
    let document = analyze_module(CompileInput::new(
        "λ/foreign.ssrg",
        "deprecation/foreign",
        source,
    ));
    let facts = warnings(&document);
    assert_eq!(facts.len(), 4, "{facts:#?}");
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.message().contains("Use NewHandle"))
            .count(),
        2
    );
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.message().contains("Use current"))
            .count(),
        1
    );
    assert_eq!(
        facts
            .iter()
            .filter(|fact| fact.message().contains("Use modern"))
            .count(),
        1
    );
    assert!(document
        .symbols
        .iter()
        .filter(|symbol| symbol.name == "current")
        .all(|symbol| symbol.deprecation.is_none()));
    for fact in &facts {
        assert_eq!(fact.related[0].source.as_deref(), Some("λ/foreign.ssrg"));
    }
}

#[test]
fn canonical_path_package_member_program_compiles_in_both_typescript_profiles() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/deprecation-members"
    ))
    .unwrap();
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let project =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        assert_eq!(project.compiled.modules.len(), 2);
        let owners = project
            .compiled
            .modules
            .values()
            .flat_map(|module| &module.typed_interface.exports)
            .filter(|export| export.name == "Box")
            .collect::<Vec<_>>();
        assert_eq!(owners.len(), 1);
        assert_eq!(
            owners[0]
                .methods
                .iter()
                .find(|method| method.name == "old")
                .unwrap()
                .deprecation
                .as_ref()
                .unwrap()
                .message,
            "Use read"
        );
    }
}
