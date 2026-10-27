use seseragi_driver::{analyze_module, analyze_project, CompileInput, ProjectModuleInput};
use seseragi_project::ModuleGraph;
use seseragi_syntax::ByteSpan;

const OWNERS: &str = concat!(
    "pub struct Box { value: Int }\n",
    "impl Box {\n",
    " pub deprecated \"Use read\" since \"0.61.0\" fn old self: Box -> Int = self.value\n",
    " pub fn read self: Box -> Int = self.value\n",
    "}\n",
    "pub struct Plain { value: Int }\n",
    "impl Plain { pub fn old self: Plain -> Int = self.value }\n",
);

fn receiver(source: &str, marker: &str) -> ByteSpan {
    let start = source.rfind(marker).unwrap();
    ByteSpan {
        start,
        end: start + marker.split('.').next().unwrap().len(),
    }
}

#[test]
fn nominal_parameters_and_inferred_local_aliases_keep_canonical_completion_owners() {
    let source = format!("{OWNERS}\npub fn parameter box: Box -> Int = box.old\npub fn inferred -> Int = {{ let original = Box {{ value: 41 }}; let alias = original; alias.old }}\npub fn plain value: Plain -> Int = value.old\n");
    let document = analyze_module(CompileInput::new(
        "owners.ssrg",
        "completion/owners",
        &source,
    ));
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:#?}",
        document.diagnostics
    );
    for marker in ["box.old", "alias.old"] {
        let methods = document.members_of_receiver(
            marker.split('.').next().unwrap(),
            receiver(&source, marker).start,
        );
        assert_eq!(
            methods
                .iter()
                .map(|method| method.name.as_str())
                .collect::<Vec<_>>(),
            ["old", "read"]
        );
        assert!(methods
            .iter()
            .all(|method| method.member_owner.as_deref() == Some("completion/owners::Box")));
        assert_eq!(methods[0].deprecation.as_ref().unwrap().message, "Use read");
        assert!(methods[1].deprecation.is_none());
    }
    let plain = document.members_of_receiver("value", receiver(&source, "value.old").start);
    assert_eq!(plain.len(), 1);
    assert_eq!(
        plain[0].member_owner.as_deref(),
        Some("completion/owners::Plain")
    );
    assert!(plain[0].deprecation.is_none());
    for profile in [
        seseragi_driver::BuildProfile::Development,
        seseragi_driver::BuildProfile::Release,
    ] {
        let compiled = seseragi_driver::compile_module(
            CompileInput::new("owners.ssrg", "completion/owners", &source).with_profile(profile),
        )
        .unwrap();
        assert!(serde_json::to_value(&compiled.typed_hir)
            .unwrap()
            .get("nominalBindings")
            .is_none());
        assert!(serde_json::to_value(&compiled.core_ir)
            .unwrap()
            .get("nominalBindings")
            .is_none());
    }
}

#[test]
fn a_scoped_binder_with_the_same_spelling_never_gains_nominal_methods() {
    let source = format!("{OWNERS}\npub fn generic<Box> value: Box -> Box = value\n");
    let document = analyze_module(CompileInput::new(
        "generic.ssrg",
        "completion/generic",
        &source,
    ));
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:#?}",
        document.diagnostics
    );
    let start = source.rfind("value").unwrap();
    assert!(document.members_of_receiver("value", start).is_empty());
    let binding = document.symbol_at(start).unwrap().id;
    assert!(!document
        .nominal_bindings
        .iter()
        .any(|fact| fact.binding == binding));
}

#[test]
fn an_unfinished_dot_keeps_valid_binding_facts_and_original_parser_diagnostics() {
    for (tail, name) in [
        ("pub fn editing box: Box -> Int = box.\n", "box"),
        (
            "pub fn editing box: Box -> Int = { let alias = box; alias. }\n",
            "alias",
        ),
    ] {
        let source = format!("{OWNERS}\n{tail}");
        let document = analyze_module(CompileInput::new(
            "editing.ssrg",
            "completion/editing",
            &source,
        ));
        let parser = seseragi_syntax::parse_diagnostics("editing.ssrg", &source);
        assert!(!parser.diagnostics.is_empty());
        for diagnostic in &parser.diagnostics {
            assert!(document
                .diagnostics
                .diagnostics
                .iter()
                .any(|original| original.code == diagnostic.code
                    && original.primary == diagnostic.primary));
        }
        let methods =
            document.members_of_receiver(name, receiver(&source, &format!("{name}.")).start);
        assert_eq!(
            methods
                .iter()
                .map(|method| method.name.as_str())
                .collect::<Vec<_>>(),
            ["old", "read"]
        );
        assert!(seseragi_driver::compile_module(CompileInput::new(
            "editing.ssrg",
            "completion/editing",
            &source
        ))
        .is_err());
    }
}

#[test]
fn lexical_shadowing_selects_the_actual_binding_before_and_after_the_local_declaration() {
    let source = format!("{OWNERS}\npub fn shadow box: Box -> Int = {{ let before = box.old; let box = Plain {{ value: 1 }}; before + box.old }}\n");
    let document = analyze_module(CompileInput::new(
        "shadow.ssrg",
        "completion/shadow",
        &source,
    ));
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:#?}",
        document.diagnostics
    );
    let before = document.members_of_receiver("box", source.find("box.old").unwrap());
    assert_eq!(
        before
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>(),
        ["old", "read"]
    );
    let after = document.members_of_receiver("box", source.rfind("box.old").unwrap());
    assert_eq!(after.len(), 1);
    assert_eq!(
        after[0].member_owner.as_deref(),
        Some("completion/shadow::Plain")
    );
    assert!(after[0].deprecation.is_none());
}

#[test]
fn imported_aliases_keep_the_provider_owner_without_display_name_matching() {
    let provider = "pub struct Box { value: Int }\nimpl Box {\n pub deprecated \"Use read\" fn old self: Box -> Int = self.value\n pub fn read self: Box -> Int = self.value\n}\n";
    let consumer = "import { Box as Container } from \"./provider\"\npub fn answer value: Container -> Int = value.old\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "api/model".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "app/main".to_owned(),
            [("./provider".to_owned(), "api/model".to_owned())],
        )
        .unwrap();
    let analyzed = analyze_project(
        graph,
        [
            ProjectModuleInput::new("/api/src/model.ssrg", "api/model", provider, "api/model.js"),
            ProjectModuleInput::new("/app/src/main.ssrg", "app/main", consumer, "app/main.js"),
        ],
    )
    .unwrap();
    let document = &analyzed.documents["app/main"];
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:#?}",
        document.diagnostics
    );
    let methods = document.members_of_receiver("value", receiver(consumer, "value.old").start);
    assert_eq!(
        methods
            .iter()
            .map(|method| method.name.as_str())
            .collect::<Vec<_>>(),
        ["old", "read"],
        "bindings={:#?}, scopes={:#?}, symbols={:#?}",
        document.nominal_bindings,
        document.scopes,
        document.symbols
    );
    assert!(methods
        .iter()
        .all(|method| method.member_owner.as_deref() == Some("api/model::Box")));
    assert_eq!(
        methods[0].deprecation.as_ref().unwrap().source,
        "/api/src/model.ssrg"
    );
}
