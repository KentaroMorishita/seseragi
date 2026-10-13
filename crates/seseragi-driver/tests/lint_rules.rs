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

fn readonly_signal_facts(source: &str) -> Vec<seseragi_syntax::Diagnostic> {
    diagnostics(source)
        .into_iter()
        .filter(|fact| fact.code == "SES-L0307")
        .collect()
}

#[test]
fn public_signal_input_proves_direct_reads_by_identity_and_same_payload() {
    for body in ["signals.read source", "*source"] {
        let source = format!("import * as signals from \"std/signal\"\npub fn snapshot source: signals.MutableSignal<Int> -> Task<Int> = {body}\n");
        let facts = readonly_signal_facts(&source);
        assert_eq!(facts.len(), 1, "{body}: {facts:?}");
        assert_eq!(
            &source[facts[0].primary.start..facts[0].primary.end],
            "source"
        );
        assert!(facts[0].related[0].message.contains("All 1 resolved uses"));
        assert_eq!(
            &source[facts[0].related[2].primary.start..facts[0].related[2].primary.end],
            body
        );
        assert!(facts[0].fixes.is_empty());
        let wire = serde_json::to_value(&facts[0]).unwrap();
        assert_eq!(wire["lint"]["ruleId"], "SES-L0307");
        assert_eq!(wire["lint"]["confidence"], "proven");
        assert_eq!(wire["lint"]["fixSafety"], "non-fixable");
        assert!(wire["lint"]["nonFixableReason"]
            .as_str()
            .unwrap()
            .contains("public callable"));
    }
    let renamed = "import { MutableSignal as Writable, read as snapshotRead } from \"std/signal\"\npub fn snapshot<A> source: Writable<A> -> Task<A> = snapshotRead source\n";
    assert_eq!(readonly_signal_facts(renamed).len(), 1);
    let alias = "import * as signals from \"std/signal\"\nalias Writable = signals.MutableSignal<Int>\npub fn snapshot source: Writable -> Task<Int> = signals.read source\n";
    assert_eq!(readonly_signal_facts(alias).len(), 1);
    let branches = "import * as signals from \"std/signal\"\npub fn snapshot enabled: Bool -> source: signals.MutableSignal<Int> -> Task<Int> = if enabled then signals.read source else *source\n";
    assert!(readonly_signal_facts(branches)[0].related[0]
        .message
        .contains("All 2 resolved uses"));
}

#[test]
fn public_effect_reads_count_all_uses_and_distinguish_parameters() {
    let source = "import * as signals from \"std/signal\"\nimport * as eff from \"std/effect\"\npub effect fn snapshot first: signals.MutableSignal<Int> -> second: signals.MutableSignal<Int> -> Int = do {\n a <- signals.read first\n b <- *first\n c <- signals.read second\n eff.succeed (a + b + c)\n}\n";
    let facts = readonly_signal_facts(source);
    assert_eq!(facts.len(), 2, "{facts:?}");
    assert_eq!(
        &source[facts[0].primary.start..facts[0].primary.end],
        "first"
    );
    assert_eq!(
        &source[facts[1].primary.start..facts[1].primary.end],
        "second"
    );
    assert!(facts[0].related[0].message.contains("All 2 resolved uses"));
    assert_eq!(facts[0].related.len(), 4);
    assert_eq!(facts[1].related.len(), 3);
}

#[test]
fn public_signal_input_rejects_alias_escape_write_and_captured_reads() {
    let prefix = "import * as signals from \"std/signal\"\n";
    for declaration in [
        "pub fn retain source: signals.MutableSignal<Int> -> signals.MutableSignal<Int> = source",
        "pub fn aliased source: signals.MutableSignal<Int> -> Task<Int> = {\n let retained = source\n signals.read retained\n}",
        "pub fn store source: signals.MutableSignal<Int> -> { value: signals.MutableSignal<Int> } = { value: source }",
        "pub fn callback source: signals.MutableSignal<Int> -> (Unit -> Task<Int>) = \\_ -> signals.read source",
        "pub fn nested source: signals.MutableSignal<Int> -> Task<Int> = {\n fn later -> Task<Int> = signals.read source\n later ()\n}",
        "pub fn mixedLambda source: signals.MutableSignal<Int> -> Task<Int> = {\n let _callback: (Unit -> Task<Int>) = \\_ -> signals.read source\n signals.read source\n}",
        "pub fn mixedLocal source: signals.MutableSignal<Int> -> Task<Int> = {\n fn later -> Task<Int> = signals.read source\n let _callback = later\n signals.read source\n}",
        "pub fn writes source: signals.MutableSignal<Int> -> Task<Unit> = signals.set 1 source",
        "pub fn updates source: signals.MutableSignal<Int> -> Task<Unit> = signals.update (\\value -> value + 1) source",
        "pub fn plan source: signals.MutableSignal<Int> -> signals.SignalChange = signals.planSet 1 source",
        "pub fn pass source: signals.MutableSignal<Int> -> Task<Int> = helper source\nfn helper source: signals.MutableSignal<Int> -> Task<Int> = signals.read source",
        "pub fn reader source: signals.Signal<Int> -> Task<Int> = signals.read source",
        "fn privateRead source: signals.MutableSignal<Int> -> Task<Int> = signals.read source",
        "pub fn unused source: signals.MutableSignal<Int> -> Int = 1",
        "pub fn ignored _source: signals.MutableSignal<Int> -> Task<Int> = signals.read _source",
        "pub fn shadow source: signals.MutableSignal<Int> -> Task<Int> = {\n let source = signals.constant 1\n signals.read source\n}",
        "pub fn dynamicRead source: signals.MutableSignal<Int> -> Task<Int> = {\n let reader = signals.read\n reader source\n}",
    ] {
        let source = format!("{prefix}{declaration}\n");
        assert!(readonly_signal_facts(&source).is_empty(), "{source}");
    }
    let mixed = "import * as signals from \"std/signal\"\nimport * as eff from \"std/effect\"\npub effect fn mixed source: signals.MutableSignal<Int> -> Int = do {\n value <- signals.read source\n signals.set value source\n eff.succeed value\n}\n";
    assert!(readonly_signal_facts(mixed).is_empty());
    let fake = "struct MutableSignal { value: Int }\nfn read source: MutableSignal -> Int = source.value\npub fn snapshot source: MutableSignal -> Int = read source\n";
    assert!(readonly_signal_facts(fake).is_empty());
    let host = "import * as signals from \"std/signal\"\nforeign \"typescript\" from \"./host.mjs\" {\n pure fn inspect source: signals.MutableSignal<Int> -> Int\n}\npub fn handToHost source: signals.MutableSignal<Int> -> Task<Int> = {\n let _host = inspect source\n signals.read source\n}\n";
    assert!(readonly_signal_facts(host).is_empty());
}

#[test]
fn signal_parameter_proof_does_not_confuse_shadowed_symbol_or_error_recovery() {
    let source = "import * as signals from \"std/signal\"\npub fn snapshot source: signals.MutableSignal<Int> -> Task<Int> = {\n fn shadow source: signals.MutableSignal<Int> -> Task<Int> = signals.read source\n let _ignored = shadow\n signals.read source\n}\n";
    assert_eq!(readonly_signal_facts(source).len(), 1);
    let broken = format!("{source}\npub let invalid: Int = missing\n");
    let analyzed = analyze_module(CompileInput::new("lint.ssrg", "lint/rules", &broken));
    assert!(!analyzed.diagnostics.diagnostics.is_empty());
    assert!(analyzed.lint_diagnostics().diagnostics.is_empty());
}
