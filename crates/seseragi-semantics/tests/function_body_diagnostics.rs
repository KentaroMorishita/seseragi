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

#[test]
fn accepts_a_trait_call_backed_by_a_function_constraint() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub trait Ready<A> { fn ready value: A -> String }\n\
         pub fn describe<T> value: T -> String\n\
         where Ready<T> =\n\
           ready value\n",
    );

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn reports_a_missing_instance_for_a_constrained_function_call() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub type Badge = | Active\n\
         pub trait Ready<A> { fn ready value: A -> String }\n\
         pub fn describe<T> value: T -> String\n\
         where Ready<T> =\n\
           \"unknown\"\n\
         pub fn label value: Badge -> String = describe value\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics.diagnostics[0].message_key, "instance.missing");
}

#[test]
fn does_not_accept_an_unmaterialized_standard_function_dictionary() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn duplicate<T> value: T -> T\n\
         where Add<T, T, T> =\n\
           value\n\
         pub fn duplicateInt value: Int -> Int = duplicate value\n",
    );

    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics.diagnostics[0].message_key, "instance.missing");
}

#[test]
fn accepts_a_materializable_standard_show_dictionary() {
    let diagnostics = semantic_diagnostics(
        "main.ssrg",
        "pub fn acknowledge<T> value: T -> String\n\
         where Show<T> =\n\
           \"shown\"\n\
         pub fn acknowledgeString value: String -> String = acknowledge value\n",
    );

    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}
