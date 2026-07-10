use seseragi_semantics::semantic_diagnostics;

#[test]
fn accepts_bool_condition_with_matching_branch_types() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn choose ready: Bool -> Int = if ready then 1 else 2\n",
    );

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn reports_non_bool_if_condition() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn invalid unit: Unit -> Int = if 1 then 1 else 2\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "if.condition-not-bool"
    );
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "expected Bool, received Int"
    );
}

#[test]
fn reports_if_branch_type_mismatch() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn invalid unit: Unit -> Int = if True then 1 else \"no\"\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "if.branch-type-mismatch"
    );
    assert_eq!(diagnostics.diagnostics[0].related.len(), 2);
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "then branch has type Int"
    );
    assert_eq!(
        diagnostics.diagnostics[0].related[1].message,
        "else branch has type String"
    );
}
