use seseragi_driver::{compile_module, CompileInput};

#[test]
fn types_effect_match_payloads_through_nested_control_flow() {
    let source = r#"
pub effect fn main = do {
  let value: Maybe<Int> = Just 1
  match value {
    Just item -> println $ show item
    Nothing -> println "nothing"
  }
}
"#;
    let compiled = compile_module(CompileInput::new("match.ssrg", "fixture/match", source))
        .expect("effect match arms retain payload typing and effect-call elaboration");
    assert!(compiled.generated.typescript.contains("intShow"));
}

#[test]
fn joins_effect_branches_and_retains_generic_evidence() {
    let source = r#"
import * as effects from "std/effect"
pub type Box<A> = | Box A
pub effect fn display<A> value: Box<A> -> Unit with Console fails ConsoleError
where Show<A> = match value {
  Box item -> if True then { let text = show item
    println text } else println "unused"
}
pub effect fn choose value: Maybe<Int> = match value {
  Just item -> effects.succeed (item + 1)
  Nothing -> effects.fail "missing"
}
pub effect fn main = do {
  display (Box 1)
  value <- choose (Just 2) |> effects.recover (\_ -> effects.succeed 0)
  println $ show value
}
"#;
    compile_module(CompileInput::new(
        "branches.ssrg",
        "fixture/branches",
        source,
    ))
    .expect("branch success and failure Never widen while evidence remains in scope");
}

#[test]
fn rejects_invalid_effect_match_branches_with_precise_diagnostics() {
    for (body, expected) in [
        (
            "match True { True -> effects.fail 1\n False -> effects.fail \"bad\" }",
            "match.branch-type-mismatch",
        ),
        (
            "match (Just 1) { Just item -> println (show item) }",
            "match.non-exhaustive",
        ),
        (
            "match True { True -> effects.succeed 1\n False -> effects.succeed \"bad\" }",
            "match.branch-type-mismatch",
        ),
        (
            "match True { True -> effects.succeed 1\n False -> 2 }",
            "match.branch-type-mismatch",
        ),
        (
            "if 1 then effects.succeed 1 else effects.succeed 2",
            "if.condition-not-bool",
        ),
        (
            "if True then effects.succeed 1 else effects.succeed \"bad\"",
            "if.branch-type-mismatch",
        ),
    ] {
        let source =
            format!("import * as effects from \"std/effect\"\npub effect fn main = {body}\n");
        let diagnostics = compile_module(CompileInput::new(
            "negative.ssrg",
            "fixture/negative",
            &source,
        ))
        .expect_err("invalid branch must remain rejected");
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.message_key == expected),
            "{expected}: {diagnostics:?}"
        );
    }
}

#[test]
fn imported_effect_match_analysis_agrees_with_compilation() {
    use seseragi_driver::{analyze_project, compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/match::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/match::main".to_owned(),
            [("./domain".to_owned(), "fixture/match::domain".to_owned())],
        )
        .unwrap();
    let inputs = [
        ProjectModuleInput::new(
            "domain.ssrg",
            "fixture/match::domain",
            include_str!("../../../examples/spec/fixtures/projects/effect-match/src/domain.ssrg"),
            "dist/domain.js",
        ),
        ProjectModuleInput::new(
            "main.ssrg",
            "fixture/match::main",
            include_str!("../../../examples/spec/fixtures/projects/effect-match/src/main.ssrg"),
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
    let domain = &analysis.documents["fixture/match::domain"];
    let choose = domain
        .symbols
        .iter()
        .find(|symbol| symbol.name == "choose")
        .unwrap();
    assert!(
        choose
            .type_name
            .as_ref()
            .unwrap()
            .contains("Effect<{}, String, Int>"),
        "{choose:?}"
    );
    compile_project(graph, inputs).expect("Analysis and Compile share effect branch typing");
}

#[test]
fn effect_match_does_not_synthesize_missing_payload_evidence() {
    let source = r#"
pub effect fn display<A> value: Maybe<A> -> Unit with Console fails ConsoleError =
  match value {
    Just item -> println $ show item
    Nothing -> println "nothing"
  }
"#;
    let diagnostics = compile_module(CompileInput::new(
        "evidence.ssrg",
        "fixture/evidence",
        source,
    ))
    .expect_err("generic payload requires Show evidence");
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "instance.missing"),
        "{diagnostics:?}"
    );
}
