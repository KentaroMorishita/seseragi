use seseragi_driver::{compile_module, CompileInput};

#[test]
fn recursive_group_supports_forward_calls_capture_and_generic_members() {
    for source in [
        "pub fn result -> Int = { rec { fn first<A> x: A -> n: Int -> A = if n == 0 then x else second x (n - 1); fn second<B> x: B -> n: Int -> B = first x n }; first 7 3 }",
        "pub fn result -> Bool = { rec { fn even n: Int -> Bool = if n == 0 then True else odd (n - 1); fn odd n: Int -> Bool = if n == 0 then False else even (n - 1) }; even 4 }",
        "pub fn result -> Int = { let base = 7; rec { fn first n: Int -> Int = if n == 0 then base else second (n - 1); fn second n: Int -> Int = third n; fn third n: Int -> Int = first n; fn identity<A> x: A -> A = x }; identity (first 3) }",
        "pub fn result -> Int = { fn first -> Int = 1; rec { fn first -> Int = second (); fn second -> Int = 2 }; first () }",
        "pub fn result -> Effect<{}, Never, Int> = { rec { effect fn first n: Int -> Int = if n == 0 then pure 7 else second (n - 1); effect fn second n: Int -> Int = first n }; first 3 }",
    ] {
        let compiled = compile_module(CompileInput::new("rec.ssrg", "fixture/rec", source));
        assert!(compiled.is_ok(), "{source}\n{compiled:#?}");
    }
}

#[test]
fn recursive_group_rejects_invalid_members_and_scope_leaks() {
    for source in [
        "pub fn result -> Int = { rec { fn specialized<A> x: A -> Int = specialized 1 }; specialized True }",
        "pub fn result -> Int = { rec { fn grow<A> x: A -> Int = grow [x] }; grow 1 }",
        "pub fn result -> Int = { rec { fn first<A> x: A -> Int = second [x]; fn second<B> x: B -> Int = first x }; first 1 }",
        "pub fn result -> Int = { rec { effect fn missing = pure 1 }; 1 }",
        "pub fn result -> Int = { rec { fn first -> Int = later }; let later = 1; first () }",
        "pub fn result -> Int = { rec { fn first -> Int = 1; fn first -> Int = 2 }; first () }",
        "pub fn result -> Int = { rec { fn first -> Int = second (); fn second -> Bool = True }; first () }",
        "pub fn result -> Int = { fn first -> Int = second (); fn second -> Int = 1; first () }",
        "pub fn result -> Int = { let value = first (); rec { fn first -> Int = 1 }; value }",
        "pub fn result -> Int = { rec { let value = 1 }; 1 }",
        "pub fn result -> Int = { rec {}; 1 }",
    ] {
        let compiled = compile_module(CompileInput::new("rec.ssrg", "fixture/rec", source));
        assert!(compiled.is_err(), "{source}\n{compiled:#?}");
    }
}

#[test]
fn local_rec_formatter_preserves_execution_and_reference_identity() {
    let source = include_str!("../../../examples/spec/fixtures/projects/local-rec/src/main.ssrg");
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
        .find(|symbol| symbol.name == "second")
        .unwrap();
    let references: Vec<_> = analysis
        .symbol_occurrences
        .iter()
        .filter(|occurrence| occurrence.symbol == count.id)
        .collect();
    assert!(references.len() >= 2, "{references:#?}");
}

#[test]
fn polymorphic_recursion_has_a_specific_diagnostic() {
    for declaration in [
        "fn grow<A> x: A -> Int = grow [x]",
        "fn change<A> x: A -> Int = change 1",
        "fn first<A> x: A -> Int = second [x]; fn second<B> x: B -> Int = first x",
    ] {
        let source = format!("pub fn result -> Int = {{ rec {{ {declaration} }}; 1 }}");
        let failure =
            compile_module(CompileInput::new("rec.ssrg", "fixture/rec", &source)).unwrap_err();
        assert!(
            failure
                .diagnostics
                .iter()
                .any(|d| d.message_key == "rec.polymorphic-call"),
            "{failure:#?}"
        );
    }
}

#[test]
fn polymorphic_recursive_callback_is_rejected() {
    let source = "fn apply<A> callback: (A -> Int) -> x: A -> Int = callback x\npub fn result -> Int = { rec { fn grow<A> x: A -> Int = apply grow [x] }; 1 }";
    let failure = compile_module(CompileInput::new("rec.ssrg", "fixture/rec", source)).unwrap_err();
    assert!(
        failure
            .diagnostics
            .iter()
            .any(|d| d.message_key == "rec.polymorphic-call"),
        "{failure:#?}"
    );
}
