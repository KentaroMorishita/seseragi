use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn rejects_unresolved_callable_storage_before_lowering() {
    for declaration in [
        "fn identity<A> value: A -> A = value",
        "let identity = \\value -> value",
    ] {
        for initializer in [
            "[identity]",
            "{apply: identity}",
            "(identity, 42)",
            "{nested: [identity]}",
        ] {
            let source = format!("{declaration}\nlet stored = {initializer}\n");
            let input = CompileInput::new("main.ssrg", "regression/storage", &source);
            let diagnostics = compile_module(input).expect_err(&source);
            assert_eq!(analyze_module(input).diagnostics, diagnostics);
            let diagnostic = diagnostics
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.message_key == "call.value-type-unresolved")
                .unwrap_or_else(|| panic!("{source}\n{diagnostics:?}"));
            assert_eq!(diagnostic.code, "SES-T0101");
            assert_eq!(
                &source[diagnostic.primary.start..diagnostic.primary.end],
                "identity"
            );
        }
    }
}

#[test]
fn resolves_storage_from_annotations_explicit_arguments_and_lexical_types() {
    let source = r#"
fn identity<A> value: A -> A = value
fn increment value: Int -> Int = value + 1
let inferred = \value -> value
let aliasValue = inferred
let a: Array<Int -> Int> = [identity, inferred]
let b: {apply: String -> String} = {apply: inferred}
let c: (Int -> Int, String -> String) = (identity, inferred)
let d = [identity<Int>]
let e = [increment, identity, inferred]
fn keep<A, B> first: A -> second: B -> A = first
let partial: {apply: String -> Int} = {apply: keep 42}
fn contextual<A> value: A -> A = {
  let stored: {apply: A -> A} = {apply: identity}
  let apply = stored.apply
  apply value
}

fn callback<A> f: (A -> A) -> value: A -> A = f value
let result: Int = callback inferred 42
let text: String = aliasValue "hello"
pub effect fn main = do { let apply = b.apply; println (apply text) }
"#;
    let result = compile_module(CompileInput::new("main.ssrg", "regression/storage", source));
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn explicit_callable_values_check_arity_and_constraints() {
    for (source, key) in [
        (
            "fn identity<A> value: A -> A = value\nlet stored = [identity<Int, String>]\n",
            "call.type-argument-arity-mismatch",
        ),
        (
            "fn render<A> value: A -> String where Show<A> = show value\nlet stored = [render<Int -> Int>]\n",
            "instance.missing",
        ),
    ] {
        let diagnostics = compile_module(CompileInput::new("main.ssrg", "regression/storage", source))
            .expect_err(source);
        assert!(diagnostics.diagnostics.iter().any(|diagnostic| diagnostic.message_key == key), "{source}\n{diagnostics:?}");
    }
}

#[test]
fn rejects_unresolved_callable_storage_from_partial_application() {
    let source =
        "fn keep<A, B> first: A -> second: B -> A = first\nlet stored = {apply: keep 42}\n";
    let diagnostics = compile_module(CompileInput::new("main.ssrg", "regression/storage", source))
        .expect_err(source);
    let diagnostic = diagnostics
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message_key == "call.value-type-unresolved")
        .unwrap();
    assert_eq!(
        &source[diagnostic.primary.start..diagnostic.primary.end],
        "keep 42"
    );
}

#[test]
fn invalid_arguments_do_not_cascade_into_unresolved_callable_results() {
    let source = r#"
import * as json from "std/json"
let stringDecoder: json.Decoder<String> = decodeJson
let intDecoder: json.Decoder<Int> = decodeJson
let mixed = json.record [("name", stringDecoder), ("retries", intDecoder)]
"#;
    let input = CompileInput::new("main.ssrg", "regression/storage", source);
    let diagnostics = compile_module(input).expect_err(source);
    assert_eq!(analyze_module(input).diagnostics, diagnostics);
    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "array.element-type-mismatch"
    );
    assert_eq!(
        &source[diagnostics.diagnostics[0].primary.start..diagnostics.diagnostics[0].primary.end],
        "(\"retries\", intDecoder)"
    );
}

#[test]
fn constrained_operator_aliases_keep_rank_one_and_required_instances() {
    let source =
        include_str!("../../../examples/spec/fixtures/compile/constrained-operator-alias.ssrg");
    for (suffix, key) in [
        ("let stored = [aliasValue]\n", "call.value-type-unresolved"),
        ("let invalid = aliasValue True False\n", "instance.missing"),
    ] {
        let source = format!("{source}\n{suffix}");
        let input = CompileInput::new("main.ssrg", "regression/operator-alias", &source);
        let diagnostics = compile_module(input).expect_err(&source);
        assert_eq!(analyze_module(input).diagnostics, diagnostics);
        assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(
            diagnostics.diagnostics[0].message_key, key,
            "{diagnostics:?}"
        );
    }
}
