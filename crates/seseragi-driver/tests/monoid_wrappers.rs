use seseragi_driver::{analyze_module, compile_module, CompileInput};

#[test]
fn numeric_wrappers_use_conditional_canonical_evidence() {
    let source = r#"
fn total<C, A> values: C -> A where Reducible<C, A>, Monoid<A> = combine values
fn unwrapSum<A> value: Sum<A> -> A = match value { Sum n -> n }
fn unwrapProduct<A> value: Product<A> -> A = match value { Product n -> n }
pub let added = unwrapSum $ total [Sum 1, Sum 2, Sum 3]
pub let multiplied = unwrapProduct $ combine [Product 2, Product 3, Product 4]
pub let none: Sum<Int> = combine ([]: Array<Sum<Int>>)
"#;
    let analysis = analyze_module(CompileInput::new("wrapper.ssrg", "fixture/wrapper", source));
    for name in ["Sum", "Product"] {
        for kind in ["type", "constructor"] {
            assert!(
                analysis.standard_library.iter().any(|item| item.identity
                    == format!("std/prelude::{name}")
                    && item.kind == kind),
                "missing {name} {kind}"
            );
        }
    }
    let compiled =
        compile_module(CompileInput::new("wrapper.ssrg", "fixture/wrapper", source)).unwrap();
    assert!(compiled.generated.typescript.contains("sumMonoid"));
    assert!(compiled.generated.typescript.contains("productMonoid"));
}

#[test]
fn numeric_values_have_no_ambiguous_monoid() {
    let analysis = analyze_module(CompileInput::new(
        "negative.ssrg",
        "fixture/negative",
        "pub let invalid: Int = combine [1, 2]",
    ));
    assert!(!analysis.diagnostics.diagnostics.is_empty());
}

#[test]
fn local_wrappers_do_not_inherit_the_prelude_instance() {
    let analysis = analyze_module(CompileInput::new(
        "shadow.ssrg",
        "fixture/shadow",
        r#"
type Sum<A> = | Sum A
pub let invalid = combine [Sum 1, Sum 2]
"#,
    ));
    assert!(
        analysis
            .diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0201"),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn collection_result_refinement_preserves_existing_nominal_arguments() {
    let source = r#"
fn total<C, A> values: C -> A where Reducible<C, A>, Monoid<A> = combine values
fn unwrap value: Maybe<String> -> String = match value { Just n -> n; Nothing -> "" }
pub let result: String = unwrap $ total [Just "a", Just "b"]
"#;
    compile_module(CompileInput::new(
        "existing.ssrg",
        "fixture/existing",
        source,
    ))
    .unwrap();
}
