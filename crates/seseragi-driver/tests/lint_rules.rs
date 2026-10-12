use seseragi_driver::{analyze_module, CompileInput};

fn diagnostics(source: &str) -> Vec<seseragi_syntax::Diagnostic> {
    let analysis = analyze_module(CompileInput::new("lint.ssrg", "lint/rules", source));
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.lint_diagnostics().diagnostics
}

#[test]
fn private_unused_and_self_recursive_declarations_are_proven_without_dce() {
    let source = "fn unused -> Int = 1\nfn recursive x: Int -> Int = if x == 0 then 0 else recursive (x - 1)\nfn helper -> Int = 2\npub fn value -> Int = helper ()\n";
    let facts = diagnostics(source);
    assert_eq!(facts.len(), 2, "{facts:?}");
    assert!(facts
        .iter()
        .all(|d| d.code == "SES-L0303" && d.fixes.is_empty()));
    assert_eq!(
        &source[facts[0].primary.start..facts[0].primary.end],
        "unused"
    );
    assert_eq!(
        &source[facts[1].primary.start..facts[1].primary.end],
        "recursive"
    );
}

#[test]
fn public_entrypoint_underscore_and_used_type_are_not_unused() {
    assert!(diagnostics(
        "fn main -> Int = 1\nfn _intentional -> Int = 2\npub fn exported -> Int = 3\n"
    )
    .is_empty());
    assert!(diagnostics(
        "struct Item { value: Int }\npub fn read item: Item -> Int = item.value\n"
    )
    .is_empty());
}

#[test]
fn discarded_cold_effect_and_constant_bool_have_ranges_and_fix_safety() {
    let source = "import * as eff from \"std/effect\"\npub fn value -> Int = {\n  let lost = eff.succeed 1\n  if True then 1 else 2\n}\n";
    let facts = diagnostics(source);
    assert_eq!(
        facts.iter().map(|d| d.code.as_str()).collect::<Vec<_>>(),
        vec!["SES-L0304", "SES-L0306"],
        "{facts:?}"
    );
    let wire = serde_json::to_value(&facts).unwrap();
    for fact in wire.as_array().unwrap() {
        assert_eq!(fact["lint"]["confidence"], "proven");
        assert_eq!(fact["lint"]["fixSafety"], "non-fixable");
        assert!(!fact["lint"]["nonFixableReason"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(!fact["related"].as_array().unwrap().is_empty());
    }
    assert_eq!(
        &source[facts[1].primary.start..facts[1].primary.end],
        "True"
    );
}

#[test]
fn used_cold_effect_and_nonliteral_condition_are_not_discarded_or_constant() {
    let source = "import * as eff from \"std/effect\"\npub fn keep -> Effect<{}, Never, Int> = {\n let work = eff.succeed 1\n work\n}\npub fn choose yes: Bool -> Int = if yes then 1 else 2\n";
    assert!(diagnostics(source).is_empty());
}

#[test]
fn unused_import_resolves_by_identity_and_type_uses_are_retained() {
    let source = "import { abs, sign } from \"std/int\"\npub fn value x: Int -> Int = abs x\n";
    let facts = diagnostics(source);
    assert_eq!(facts.len(), 1, "{facts:?}");
    assert_eq!(facts[0].code, "SES-L0302");
    assert!(facts[0].related[0].message.contains("sign"));
}

#[test]
fn canonical_subscription_is_owned_but_same_named_user_type_is_not() {
    let source = "import * as signal from \"std/signal\"\nimport * as eff from \"std/effect\"\npub effect fn discard source: signal.Signal<Int> -> Unit =\n do {\n signal.subscribe (\\value -> eff.succeed ()) source\n eff.succeed ()\n }\n";
    let facts = diagnostics(source);
    assert!(facts.iter().any(|d| d.code == "SES-L0305"), "{facts:?}");
    let other = "struct Subscription { value: Int }\npub fn value -> Int = {\n let _ignored = Subscription { value: 1 }\n 1\n}\n";
    assert!(!diagnostics(other).iter().any(|d| d.code == "SES-L0305"));
}

#[test]
fn advisory_counterexamples_do_not_become_architecture_warnings() {
    let source = "pub fn predicate value: Int -> Bool = value > 0\npub fn protocol status: String -> String = status\npub fn join a: String -> b: String -> c: String -> d: String -> String = a + b + c + d\nimport * as signals from \"std/signal\"\npub fn retain target: signals.MutableSignal<Int> -> signals.MutableSignal<Int> = target\n";
    assert!(diagnostics(source).is_empty());
}

#[test]
fn acquired_subscription_binding_has_specific_warning_and_used_handle_does_not() {
    let source = "import * as signal from \"std/signal\"\nimport * as eff from \"std/effect\"\npub effect fn discard source: signal.Signal<Int> -> Unit =\n do {\n lost <- signal.subscribe (\\value -> eff.succeed ()) source\n eff.succeed ()\n }\n";
    let facts = diagnostics(source);
    assert_eq!(facts.len(), 1, "{facts:?}");
    assert_eq!(facts[0].code, "SES-L0305");
    let retained = source.replace("eff.succeed ()\n }", "signal.unsubscribe lost\n }");
    assert!(diagnostics(&retained).is_empty());
}

#[test]
fn reexport_and_used_type_import_do_not_warn() {
    assert!(diagnostics("pub import { abs } from \"std/int\"\n").is_empty());
    assert!(diagnostics("import { Signal } from \"std/signal\"\npub fn retain value: Signal<Int> -> Signal<Int> = value\n").is_empty());
}

#[test]
fn private_module_effect_is_discarded_but_public_effect_is_exported() {
    let source = "import * as eff from \"std/effect\"\nlet lost = eff.succeed 1\npub let exposed: Effect<{}, Never, Int> = eff.succeed 2\n";
    let facts = diagnostics(source);
    assert_eq!(facts.len(), 1, "{facts:?}");
    assert_eq!(facts[0].code, "SES-L0304");
}

#[test]
fn imported_adt_type_retains_shared_constructor_import() {
    use seseragi_driver::{analyze_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module("domain".to_owned(), std::iter::empty::<(String, String)>())
        .unwrap();
    graph
        .add_module(
            "main".to_owned(),
            [("./domain".to_owned(), "domain".to_owned())],
        )
        .unwrap();
    let project = analyze_project(graph, [
        ProjectModuleInput::new("domain.ssrg", "domain", "pub type Ticket = | Ticket Int\n", "domain.js"),
        ProjectModuleInput::new("main.ssrg", "main", "import { Ticket } from \"./domain\"\npub fn retain value: Ticket -> Ticket = value\n", "main.js"),
    ]).unwrap();
    assert!(project.documents["main"]
        .authoring_diagnostics()
        .diagnostics
        .is_empty());
}

#[test]
fn unused_adt_import_is_one_binding_warning_across_namespaces() {
    use seseragi_driver::{analyze_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module("domain".to_owned(), std::iter::empty::<(String, String)>())
        .unwrap();
    graph
        .add_module(
            "main".to_owned(),
            [("./domain".to_owned(), "domain".to_owned())],
        )
        .unwrap();
    let project = analyze_project(
        graph,
        [
            ProjectModuleInput::new(
                "domain.ssrg",
                "domain",
                "pub type Ticket = | Ticket Int\n",
                "domain.js",
            ),
            ProjectModuleInput::new(
                "main.ssrg",
                "main",
                "import { Ticket } from \"./domain\"\npub let answer: Int = 1\n",
                "main.js",
            ),
        ],
    )
    .unwrap();
    let facts = project.documents["main"].authoring_diagnostics();
    assert_eq!(facts.diagnostics.len(), 1, "{facts:?}");
    assert_eq!(facts.diagnostics[0].code, "SES-L0302");
}
