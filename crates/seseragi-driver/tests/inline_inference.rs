use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn inline_comprehension_instantiates_effect_payload() {
    let source = r#"
import * as random from "std/random"
import * as arrays from "std/array"
pub effect fn main -> Unit
with Console, Random
fails ConsoleError = do {
  shuffled <- random.shuffle [n | n <- 1..=9]
  println $ show shuffled
  println $ show (arrays.sort shuffled)
}
"#;
    let analysis = analyze_module(CompileInput::new("inline.ssrg", "fixture/inline", source));
    assert!(
        analysis.diagnostics.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    compile_module(CompileInput::new("inline.ssrg", "fixture/inline", source))
        .expect("inline effect payload is concrete before evidence and lowering");
}

#[test]
fn inline_collection_matrix_matches_let_bound_pure_effect_and_partial_calls() {
    for (container, prefix) in [("Array", ""), ("List", "`")] {
        let literal = format!("{prefix}[1, 2, 3]");
        let comprehension = format!("{prefix}[n | n <- 1..=3]");
        let expressions = [
            literal.clone(),
            comprehension.clone(),
            format!("{prefix}[n | n <- [1, 2, 3]]"),
            format!("(if True then {comprehension} else {literal})"),
            format!("(match True {{ True -> {comprehension}\n False -> {literal} }})"),
            format!("{{ let values = {comprehension}\n values }}"),
        ];
        for expression in expressions {
            let source = format!(
                r#"
fn keep<A> values: {container}<A> -> {container}<A> = values
fn keepFirst<A> values: {container}<A> -> ignored: {container}<A> -> {container}<A> = values
effect fn echo<A> values: {container}<A> -> {container}<A> = succeed values
let standalone = {expression}
pub let inlineValue = keep {expression}
pub let boundValue = keep standalone
let partial = keepFirst {expression}
pub let partialValue = partial standalone
pub effect fn main = do {{
  effectValue <- echo {expression}
  boundEffectValue <- echo standalone
  println $ show inlineValue
  println $ show partialValue
  println $ show effectValue
  println $ show boundEffectValue
}}
"#
            );
            let analysis =
                analyze_module(CompileInput::new("matrix.ssrg", "fixture/matrix", &source));
            assert!(
                analysis.diagnostics.diagnostics.is_empty(),
                "{container} {expression}: {:?}",
                analysis.diagnostics
            );
            for name in [
                "standalone",
                "inlineValue",
                "boundValue",
                "partialValue",
                "effectValue",
                "boundEffectValue",
            ] {
                let symbol = analysis
                    .symbols
                    .iter()
                    .find(|symbol| symbol.name == name)
                    .unwrap();
                assert_eq!(
                    symbol.type_name.as_deref(),
                    Some(format!("{container}<Int>").as_str()),
                    "{expression}: {name}"
                );
            }
            compile_module(CompileInput::new("matrix.ssrg", "fixture/matrix", &source))
                .unwrap_or_else(|diagnostics| panic!("{container} {expression}: {diagnostics:?}"));
        }
    }
}

#[test]
fn nested_expected_collection_elements_preserve_inferred_fields() {
    for (element_type, expression, concrete_type) in [
        ("(Int, A)", "[(n, n + 1) | n <- 1..=3]", "(Int, Int)"),
        (
            "{ value: A }",
            "[{ value: n } | n <- 1..=3]",
            "{ value: Int }",
        ),
        ("{ value: A }", "[{ value: 1 }]", "{ value: Int }"),
        ("(Int, A)", "[(1, 2)]", "(Int, Int)"),
    ] {
        let source = format!(
            r#"
fn keep<A> values: Array<{element_type}> -> Array<{element_type}> = values
let result = keep {expression}
pub let checked: Array<{concrete_type}> = result
pub let rendered = show result
"#
        );
        compile_module(CompileInput::new("nested.ssrg", "fixture/nested", &source))
            .unwrap_or_else(|diagnostics| panic!("{expression}: {diagnostics:?}"));
    }
}

#[test]
fn imported_generic_calls_keep_concrete_analysis_and_compile_types() {
    use seseragi_driver::{analyze_project, compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/inline::domain".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/inline::main".to_owned(),
            [("./domain".to_owned(), "fixture/inline::domain".to_owned())],
        )
        .unwrap();
    let inputs = [
        ProjectModuleInput::new("domain.ssrg", "fixture/inline::domain", include_str!("../../../examples/spec/fixtures/projects/inline-polymorphic-inference/src/domain.ssrg"), "dist/domain.js"),
        ProjectModuleInput::new("main.ssrg", "fixture/inline::main", include_str!("../../../examples/spec/fixtures/projects/inline-polymorphic-inference/src/main.ssrg"), "dist/main.js"),
    ];
    let analysis = analyze_project(graph.clone(), inputs.clone()).unwrap();
    let document = &analysis.documents["fixture/inline::main"];
    assert!(
        document.diagnostics.diagnostics.is_empty(),
        "{:?}",
        document.diagnostics
    );
    for (name, expected) in [
        ("inlineArray", "Array<Int>"),
        ("inlineList", "List<Int>"),
        ("arrayValue", "Array<Int>"),
        ("listValue", "List<Int>"),
        ("shuffled", "Array<Int>"),
    ] {
        let symbol = document
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .unwrap();
        assert_eq!(symbol.type_name.as_deref(), Some(expected), "{name}");
    }
    compile_project(graph, inputs)
        .expect("imported inferred payloads materialize evidence and lower");
}

#[test]
fn refining_inference_holes_preserves_concrete_element_requirements() {
    for (element_type, expression) in [
        ("(Bool, A)", "[(n, n) | n <- 1..=3]"),
        (
            "{ fixed: Bool, value: A }",
            "[{ fixed: n, value: n } | n <- 1..=3]",
        ),
    ] {
        let source = format!("fn keep<A> values: Array<{element_type}> -> Array<{element_type}> = values\npub let invalid = keep {expression}\n");
        let diagnostics = compile_module(CompileInput::new(
            "invalid.ssrg",
            "fixture/invalid",
            &source,
        ))
        .expect_err("filling unknown generic fields must not erase concrete type mismatches");
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "SES-T0101"),
            "{diagnostics:?}"
        );
        assert!(diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "SES-P0001"));
    }
}
