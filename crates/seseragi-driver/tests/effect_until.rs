use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn effect_until_uses_generic_iterable_evidence_and_canonical_control() {
    let source = r#"
import * as effects from "std/effect"
import * as iterator from "std/iterator"
fn step n: Int -> Effect<{}, Never, effects.LoopControl> =
  effects.succeed $ if n == 3 then effects.Break else effects.Continue
fn visit<C, R, E, A> action: (A -> Effect<R, E, effects.LoopControl>)
  -> values: C -> Effect<R, E, Unit> where Iterable<C, A> =
  effects.forEachUntil action values
fn classify control: effects.LoopControl -> Bool = match control {
  effects.Continue -> False
  effects.Break -> True
}
fn advance n: Int -> Maybe<(Int, Int)> = Just (n, n + 1)
pub let direct = effects.forEachUntil step [1, 2]
pub let array = visit step [1, 2, 3, 4]
pub let list = visit step `[1, 2, 3, 4]
pub let range = visit step (1..=4)
pub let infinite = visit step (iterator.unfold advance 1)
"#;
    let analysis = analyze_module(CompileInput::new("until.ssrg", "fixture/until", source));
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for identity in [
        "std/effect::LoopControl",
        "std/effect::Continue",
        "std/effect::Break",
        "std/effect::forEachUntil",
    ] {
        assert!(
            analysis
                .standard_library
                .iter()
                .any(|item| item.identity == identity),
            "missing {identity}"
        );
    }
    let compiled =
        compile_module(CompileInput::new("until.ssrg", "fixture/until", source)).unwrap();
    let ts = compiled.generated.typescript;
    assert!(ts.contains("_ssrg_effect_forEachUntil"), "{ts}");
    assert!(ts.contains("iteratorIterable"), "{ts}");
    assert!(
        ts.contains("_ssrg_effect_forEachUntil(step, [1, 2], _ssrg_array_iterable)"),
        "{ts}"
    );
}

#[test]
fn effect_until_crosses_imported_iterable_and_partial_application_boundaries() {
    use seseragi_driver::{analyze_project, compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/until::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/until::main".to_owned(),
            [("./domain".to_owned(), "fixture/until::domain".to_owned())],
        )
        .unwrap();
    let inputs = [
        ProjectModuleInput::new(
            "domain.ssrg",
            "fixture/until::domain",
            include_str!("../../../examples/spec/fixtures/projects/effect-until/src/domain.ssrg"),
            "dist/domain.js",
        ),
        ProjectModuleInput::new(
            "main.ssrg",
            "fixture/until::main",
            include_str!("../../../examples/spec/fixtures/projects/effect-until/src/main.ssrg"),
            "dist/main.js",
        ),
    ];
    let analysis = analyze_project(graph.clone(), inputs.clone()).unwrap();
    for document in analysis.documents.values() {
        assert!(
            document.diagnostics.diagnostics.is_empty(),
            "{:?}",
            document.diagnostics
        );
    }
    let compiled =
        compile_project(graph, inputs).expect("imported Iterable and generic Effect evidence");
    let ts = &compiled.modules["fixture/until::main"].generated.typescript;
    assert!(
        !ts.contains("observe()"),
        "callback reference must remain cold: {ts}"
    );
    assert!(
        ts.contains(
            "_ssrg_effect_forEachUntil(observe, __ssrg$effect$partial$0, _ssrg_array_iterable)"
        ),
        "partial application must put the collection before evidence: {ts}"
    );
}

#[test]
fn canonical_short_circuit_traversal_contract_compiles() {
    compile_module(CompileInput::new(
        "short-circuit.ssrg",
        "fixture/short-circuit",
        include_str!("../../../examples/spec/fixtures/compile/short-circuit-traversal.ssrg"),
    ))
    .unwrap();
}

#[test]
fn pure_standard_effect_calls_preserve_existing_reducible_evidence() {
    let source = r#"
import * as effects from "std/effect"
fn action n: Int -> Effect<{}, Never, Unit> = effects.succeed ()
pub let pending = effects.forEachParallel (effects.unboundedParallelism ()) action [1, 2]
"#;
    let compiled = compile_module(CompileInput::new(
        "parallel.ssrg",
        "fixture/parallel",
        source,
    ))
    .unwrap();
    assert!(
        compiled
            .generated
            .typescript
            .contains("action, [1, 2], _ssrg_array_reducible)"),
        "{}",
        compiled.generated.typescript
    );
}
