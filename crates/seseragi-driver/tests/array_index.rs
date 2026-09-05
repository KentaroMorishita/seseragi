use seseragi_driver::{compile_module, CompileInput};

#[test]
fn array_index_uses_safe_get_and_preserves_application_precedence() {
    let source = r#"
fn pick<A> values: Array<A> -> index: Int -> Maybe<A> = values[index]
fn first values: Array<Int> -> Maybe<Int> = values[0]
let values = [10, 20, 30]
let grouped = (values)[1]
let applied = first [4, 5]
let passed = pick values 1
let empty: Maybe<Int> = [][0]
let nested: Maybe<Int> = [[7]][0] >>= first
pub let output = (values[0], values[-1], values[3], grouped, applied, passed, empty, nested)
"#;
    let result = compile_module(CompileInput::new("index.ssrg", "fixture/index", source));
    assert!(result.is_ok(), "{result:#?}");
    let generated = result.unwrap().generated.typescript;
    assert!(generated.contains("_ssrg_array_index"), "{generated}");
}

#[test]
fn invalid_index_operands_have_specific_diagnostics() {
    for (source, key) in [
        ("pub let value = 1[0]", "array.index-receiver-not-array"),
        ("pub let value = `[1][0]", "array.index-receiver-not-array"),
        (
            "pub let value = (1, 2)[0]",
            "array.index-receiver-not-array",
        ),
        ("pub let value = [1][True]", "array.index-not-int"),
    ] {
        let diagnostics = seseragi_semantics::semantic_diagnostics("bad-index.ssrg", source);
        assert!(
            diagnostics.diagnostics.iter().any(|d| d.message_key == key),
            "{diagnostics:#?}"
        );
    }
}
