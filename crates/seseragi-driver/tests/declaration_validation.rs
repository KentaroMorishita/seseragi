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
