use seseragi_semantics::semantic_diagnostics;

#[test]
fn accepts_exhaustive_adt_match() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match value { One -> 1; Two -> 2 }\n",
    );

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn reports_missing_standard_sum_constructor() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn valueOrZero result: Either<String, Int> -> Int = match result { Right value -> value }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0301");
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "missing patterns: Left _"
    );
}

#[test]
fn reports_missing_adt_constructor() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match value { One -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0301");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "match.non-exhaustive"
    );
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "missing patterns: Two"
    );
}

#[test]
fn reports_arm_after_catchall_as_unreachable() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match value { _ -> 0; One -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0302");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "match.unreachable-arm"
    );
}

#[test]
fn guarded_arm_does_not_make_a_match_exhaustive() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose ready: Bool -> value: Choice -> Int = match value { One when ready -> 1; Two -> 2 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0301");
    assert!(diagnostics.diagnostics[0].related[0]
        .message
        .contains("One"));
}

#[test]
fn guarded_duplicates_remain_reachable_until_an_unguarded_arm_covers() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose first: Bool -> second: Bool -> value: Choice -> Int = match value { One when first -> 1; One when second -> 2; One -> 3; Two -> 4 }\n",
    );

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn reports_guard_and_branch_type_mismatches_as_type_errors() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn choose value: Choice -> Int = match value { One when 1 -> 1; One -> 1; Two -> \"two\" }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 2);
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code == "SES-T0101"));
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message_key == "match.guard-not-bool"));
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message_key == "match.branch-type-mismatch"));
}

#[test]
fn bare_function_reference_is_not_typed_as_its_result_adt() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn pass value: Choice -> Choice = value\nfn invalid unit: Unit -> Int = match pass { One -> 1; _ -> 0 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "match.pattern-type-mismatch"
    );
}

#[test]
fn invalid_call_scrutinee_does_not_cascade_match_errors() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn make value: Int -> Choice = One\nfn invalid unit: Unit -> Int = match make \"bad\" { One -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
}

#[test]
fn unresolved_scrutinee_does_not_cascade_match_errors() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "type Choice = | One | Two\nfn invalid unit: Unit -> Int = match missing { One -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
}

#[test]
fn unresolved_scrutinee_suppresses_tuple_pattern_mismatch() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn invalid unit: Unit -> Int = match missing { (left, right) -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-N0001");
}

#[test]
fn accepts_string_literals_followed_by_a_catchall() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn parse input: String -> Int = match input { \"rock\" -> 1; \"paper\" -> 2; _ -> 0 }\n",
    );

    assert!(diagnostics.diagnostics.is_empty());
}

#[test]
fn reports_open_literal_match_without_a_catchall() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn parse input: String -> Int = match input { \"rock\" -> 1 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0301");
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "missing patterns: _"
    );
}

#[test]
fn reports_duplicate_literal_arm_as_unreachable() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn parse input: String -> Int = match input { \"rock\" -> 1; \"rock\" -> 2; _ -> 0 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0302");
}

#[test]
fn reports_literal_pattern_type_mismatch() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn invalid input: Int -> Int = match input { \"one\" -> 1; _ -> 0 }\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "match.pattern-type-mismatch"
    );
}

#[test]
fn keeps_tuple_literal_catchall_reachable() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "fn classify values: (Int, Int) -> Bool = match values { (45, 55) -> True; _ -> False }\n",
    );

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}
