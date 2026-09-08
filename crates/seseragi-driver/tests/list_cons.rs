use seseragi_driver::{compile_module, CompileInput};

#[test]
fn cons_uses_canonical_persistent_list_and_right_associativity() {
    let source = r#"
fn prepend<A> head: A -> tail: List<A> -> List<A> = head : tail
let values: List<Int> = 1 : 2 : 3 : `[]
pub let result = (values, prepend 4 values, 1 + 2 : `[4])
"#;
    let result = compile_module(CompileInput::new("cons.ssrg", "fixture/cons", source));
    assert!(result.is_ok(), "{result:#?}");
    let generated = result.unwrap().generated.typescript;
    assert!(
        generated.contains(
            "_ssrg_list_cons<number>(1, _ssrg_list_cons<number>(2, _ssrg_list_cons<number>(3,"
        ),
        "{generated}"
    );
    assert!(generated.contains("Cons as _ssrg_list_cons"), "{generated}");
}

#[test]
fn cons_rejects_invalid_tail_and_nominal_shadowing() {
    for source in [
        "pub let bad = 1 : 2",
        "pub let bad = 1 : [2]",
        "pub let bad = True : `[1]",
        "struct List<A> { value: A }\nfn bad head: Int -> tail: List<Int> -> List<Int> = head : tail",
        "fn bad<List> head: Int -> tail: List<Int> -> List<Int> = head : tail",
        "operator infixr 4 : head: Int -> tail: Int -> Int = head",
    ] {
        let result = compile_module(CompileInput::new("bad.ssrg", "fixture/cons", source));
        assert!(result.is_err(), "{source}\n{result:#?}");
    }
}

#[test]
fn cons_sections_preserve_generic_currying() {
    let source = "let prepend: Bool -> List<Bool> -> List<Bool> = (:)\nlet withOne = (:) 1\npub let result = (prepend True `[], withOne `[2], (:) 3 `[4])\n";
    let result = compile_module(CompileInput::new("section.ssrg", "fixture/cons", source));
    assert!(result.is_ok(), "{result:#?}");
    let generated = result.unwrap().generated.typescript;
    assert!(generated.contains("_ssrg_list_cons"), "{generated}");
}

#[test]
fn cons_continues_across_lines_and_preserves_formatted_execution_shape() {
    let source = "pub fn values unit: Unit -> List<Int> = {\n  1\n  : 2 :\n  3 : `[]\n}\n";
    let result = compile_module(CompileInput::new("lines.ssrg", "fixture/cons", source));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn formatted_cons_keeps_the_same_generated_program() {
    let source = "pub let values: List<Int> = 1:2:3:`[]\n";
    let original = compile_module(CompileInput::new("format.ssrg", "fixture/cons", source))
        .unwrap()
        .generated
        .typescript;
    for width in [20, 88] {
        let tokens = seseragi_syntax::lex("format.ssrg", source);
        let cst = seseragi_syntax::parse_cst_from_tokens(tokens.clone());
        let formatted = seseragi_formatter::format_cst_with_options(
            &tokens,
            &cst,
            seseragi_formatter::FormatOptions::new(width),
        );
        let generated = compile_module(CompileInput::new(
            "format.ssrg",
            "fixture/cons",
            &formatted.text,
        ))
        .unwrap()
        .generated
        .typescript;
        assert_eq!(original, generated);
    }
}

#[test]
fn cons_infers_nullary_head_and_empty_collection_from_tail() {
    let source =
        "pub let values = (Nothing : `[Just 1], [] : `[[1]], Nothing : Nothing : `[Just 2], (:) Nothing `[Just 3])\n";
    let result = compile_module(CompileInput::new("infer.ssrg", "fixture/cons", source));
    assert!(result.is_ok(), "{result:#?}");
}
