use seseragi_semantics::semantic_diagnostics;

#[test]
fn accepts_refutable_constructor_and_tuple_comprehension_patterns() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        r#"pub fn presentValues values: Array<Maybe<Int>> -> Array<Int> =
  [value | Just value <- values, value > 1]

pub fn matchingValues values: Array<(Int, Int)> -> Array<Int> =
  [value | (1, value) <- values]
"#,
    );

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}
