use seseragi_driver::{compile_module, CompileInput};

#[test]
fn local_effect_functions_compile_with_capture_and_contracts() {
    for source in [
        "pub fn task -> Effect<{}, Never, Int> = { fn get -> Int = 1; effect fn get = pure 2; get () }",
        "pub fn task -> Int = { effect fn get = pure 1; fn get -> Int = 2; get () }",
        "pub fn task -> Effect<{}, Never, Unit> = { effect fn noop = do { pure () }; noop () }",
        "pub fn task n: Int -> Effect<{}, Never, Int> = { effect fn get = do { pure n }; get () }",
        "pub fn task n: Int -> Effect<{}, Never, Int> = { effect fn get -> Int = do { pure n }; get () }",
        "pub fn task -> Effect<{}, Never, Int> = { effect fn loop n: Int -> Int = if n == 0 then pure 1 else loop (n - 1); loop 3 }",
    ] {
        let result = compile_module(CompileInput::new("local.ssrg", "fixture/local", source));
        assert!(result.is_ok(), "{source}\n{result:#?}");
    }
}

#[test]
fn local_effect_functions_reject_invalid_contracts_and_later_capture() {
    for source in [
        "pub fn task -> Effect<{}, Never, Int> = { effect fn get = do { pure later }; let later = 1; get () }",
        "pub fn task -> Effect<{}, Never, Int> = { effect fn get -> Int = pure True; get () }",
        "pub fn task -> Effect<{}, Never, Unit> = { effect fn get -> Unit = println 1; get () }",
        "pub fn task -> Effect<{}, Never, Unit> = { effect fn get = 1; get () }",
    ] {
        let result = compile_module(CompileInput::new("local.ssrg", "fixture/local", source));
        assert!(result.is_err(), "{source}\n{result:#?}");
    }
}

#[test]
fn local_effect_project_supports_newline_separated_declarations() {
    let source =
        include_str!("../../../examples/spec/fixtures/projects/local-effect-fn/src/main.ssrg");
    let result = compile_module(CompileInput::new("main.ssrg", "fixture/local", source));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn declaration_only_local_effect_block_returns_unit_and_exposes_effect_hover() {
    let source = "pub fn silent -> Unit = { effect fn ignored = println 1 }";
    let result = compile_module(CompileInput::new("local.ssrg", "fixture/local", source));
    assert!(result.is_ok(), "{result:#?}");
    let analysis =
        seseragi_driver::analyze_module(CompileInput::new("local.ssrg", "fixture/local", source));
    let symbol = analysis
        .symbols
        .iter()
        .find(|symbol| symbol.name == "ignored")
        .unwrap();
    assert!(
        symbol
            .type_name
            .as_ref()
            .is_some_and(|name| name.contains("Effect") && name.contains("ConsoleError")),
        "{symbol:#?}"
    );
}

#[test]
fn local_contract_diagnostics_match_module_level_effect_diagnostics() {
    for (declaration, key) in [
        (
            "effect fn get -> Unit = println 1",
            "effect.explicit-environment-mismatch",
        ),
        (
            "effect fn get -> Unit with console: Console = println 1",
            "effect.explicit-failure-mismatch",
        ),
        (
            "effect fn get with console: Console = println 1",
            "effect.compact-contract-clause",
        ),
    ] {
        let source = format!("pub fn task -> Unit = {{ {declaration}; () }}");
        for source in [&source, declaration] {
            let result = compile_module(CompileInput::new("local.ssrg", "fixture/local", source))
                .unwrap_err();
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.message_key == key),
                "{result:#?}"
            );
        }
    }
}

#[test]
fn local_effect_formatter_preserves_execution_and_reference_identity() {
    let source =
        include_str!("../../../examples/spec/fixtures/projects/local-effect-fn/src/main.ssrg");
    let original = compile_module(CompileInput::new("main.ssrg", "fixture/local", source)).unwrap();
    for width in [40, 88] {
        let tokens = seseragi_syntax::lex("main.ssrg", source);
        let cst = seseragi_syntax::parse_cst_from_tokens(tokens.clone());
        let formatted = seseragi_formatter::format_cst_with_options(
            &tokens,
            &cst,
            seseragi_formatter::FormatOptions::new(width),
        );
        let compiled = compile_module(CompileInput::new(
            "main.ssrg",
            "fixture/local",
            &formatted.text,
        ))
        .unwrap();
        assert_eq!(original.generated.typescript, compiled.generated.typescript);
    }
    let analysis =
        seseragi_driver::analyze_module(CompileInput::new("main.ssrg", "fixture/local", source));
    let count = analysis
        .symbols
        .iter()
        .find(|symbol| symbol.name == "count")
        .unwrap();
    let references: Vec<_> = analysis
        .symbol_occurrences
        .iter()
        .filter(|occurrence| occurrence.symbol == count.id)
        .collect();
    assert!(references.len() >= 2, "{references:#?}");
}
