use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn shared_analysis_and_compilation_reject_invalid_used_and_unused_declarations() {
    for (header, usage, key) in [
        (
            "fn add left -> Int = left",
            "add 1",
            "parser.function-annotations-required",
        ),
        (
            "fn add left: Int = left",
            "add 1",
            "parser.function-annotations-required",
        ),
        (
            "pub let count = 3",
            "count",
            "parser.public-let-annotation-required",
        ),
        (
            "let 1value = 1",
            "1value",
            "parser.invalid-declaration-name",
        ),
        ("let match = 1", "match", "parser.invalid-declaration-name"),
        (
            "println \"unexpected\"",
            "1",
            "parser.module-expression-statement",
        ),
    ] {
        for expression in ["1", usage] {
            let source = format!("{header}\npub effect fn main = println (show ({expression}))\n");
            let input = CompileInput::new("main.ssrg", "regression/declarations", &source);
            let analysis = analyze_module(input);
            let compilation = compile_module(input).expect_err(&source);
            assert_eq!(analysis.diagnostics, compilation);
            let diagnostic = compilation
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message_key == key)
                .unwrap_or_else(|| panic!("missing {key}: {compilation:?}"));
            assert!(diagnostic.primary.start < header.len());
            assert!(diagnostic.primary.end <= header.len());
        }
    }
}

#[test]
fn accepts_private_inference_compact_effects_and_cold_initializers() {
    let source = r#"import * as effects from "std/effect"
pub let count: Int = 3
let 次の値 = count
let account' = 次の値
fn add value: Int -> Int = { let local = 1; value + local }
let pending = effects.succeed (add account')
pub effect fn main = do {
  value <- pending
  println (show value)
}
"#;
    let result = compile_module(CompileInput::new("main.ssrg", "regression/valid", source));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn parser_invalid_declarations_do_not_cascade_but_valid_editor_queries_survive() {
    let source = "fn broken value -> Int = value\npub fn double value: Int -> Int = value + value\nlet answer = double 21\npub effect fn main = println (show (broken 1))\n";
    let input = CompileInput::new("editing.ssrg", "regression/editing", source);
    let analysis = analyze_module(input);
    let parser = seseragi_syntax::parse_diagnostics("editing.ssrg", source);
    assert_eq!(analysis.diagnostics, parser);
    assert_eq!(compile_module(input).expect_err(source), parser);
    let reference = source.find("double 21").unwrap();
    assert_eq!(analysis.symbol_at(reference).unwrap().name, "double");
    assert_eq!(analysis.type_at(reference).unwrap().type_name, "Int -> Int");
    assert!(analysis.lint_diagnostics().diagnostics.is_empty());
}

#[test]
fn linked_parser_invalid_declarations_keep_imported_queries_without_semantic_cascades() {
    use seseragi_driver::{analyze_project_recovering, compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;

    let mut graph = ModuleGraph::new();
    graph.add_module("editing/domain".to_owned(), []).unwrap();
    graph
        .add_module("editing/semantic-error".to_owned(), [])
        .unwrap();
    graph
        .add_module(
            "editing/main".to_owned(),
            [("./domain".to_owned(), "editing/domain".to_owned())],
        )
        .unwrap();
    let source = "import { double } from \"./domain\"\nfn broken value -> Int = value\nlet answer = double 21\npub effect fn main = println (show (broken 1))\n";
    let inputs = vec![
        ProjectModuleInput::new(
            "semantic-error.ssrg",
            "editing/semantic-error",
            "pub fn invalid -> Int = absent\n",
            "semantic-error.js",
        ),
        ProjectModuleInput::new(
            "domain.ssrg",
            "editing/domain",
            "pub fn double value: Int -> Int = value + value\n",
            "domain.js",
        ),
        ProjectModuleInput::new("main.ssrg", "editing/main", source, "main.js"),
    ];
    assert!(compile_project(graph.clone(), inputs.clone()).is_err());
    let analysis = analyze_project_recovering(graph, inputs).unwrap();
    let semantic_error = &analysis.documents["editing/semantic-error"];
    assert!(!semantic_error.diagnostics.diagnostics.is_empty());
    assert!(semantic_error
        .diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message_key.starts_with("parser.")));
    let document = &analysis.documents["editing/main"];
    assert_eq!(
        document.diagnostics,
        seseragi_syntax::parse_diagnostics("main.ssrg", source)
    );
    let reference = source.find("double 21").unwrap();
    assert_eq!(
        document.symbol_at(reference).unwrap().module,
        "editing/domain"
    );
    assert_eq!(document.type_at(reference).unwrap().type_name, "Int -> Int");
    assert!(document.lint_diagnostics().diagnostics.is_empty());
}
