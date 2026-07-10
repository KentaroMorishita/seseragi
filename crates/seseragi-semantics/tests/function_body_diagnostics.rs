use seseragi_semantics::semantic_diagnostics;

#[test]
fn accepts_body_matching_declared_return_type() {
    let diagnostics =
        semantic_diagnostics("main.ssrg", "pub fn identity value: Int -> Int = value\n");

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn reports_function_body_return_type_mismatch() {
    let diagnostics =
        semantic_diagnostics("main.ssrg", "pub fn wrong unit: Unit -> Int = \"wrong\"\n");

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "function.return-type-mismatch"
    );
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "declared Int, body produces String"
    );
}

#[test]
fn does_not_cascade_return_mismatch_from_invalid_conditional() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn invalid unit: Unit -> Int = if True then 1 else \"no\"\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "if.branch-type-mismatch"
    );
}
