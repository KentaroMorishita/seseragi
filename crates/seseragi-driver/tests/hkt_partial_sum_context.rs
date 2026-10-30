use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn partially_inferred_sum_context_does_not_fix_a_nested_constructor_to_an_unresolved_scheme_key() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/hkt-partial-sum-context.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("partial-sum-雪🙂.ssrg", "fixture/partial-sum", original)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
    }
}

#[test]
fn nested_partial_context_still_rejects_a_different_nominal_owner_and_wrong_sum_payload() {
    let prefix="type One = | One Int\ntype Two = | Two Int\ntype Box<A> = | Box A\nfn extract<A> value:Box<A> -> A = match value {Box item -> item}\nfn require value:One -> Int = match value {One n -> n}\n";
    for invalid in ["Two 42", "True"] {
        let source = format!("{prefix}pub fn probe -> Int = require (extract (Box ({invalid})))\n");
        let error = compile_module(CompileInput::new(
            "wrong-owner.ssrg",
            "fixture/wrong-owner",
            &source,
        ))
        .unwrap_err();
        assert!(
            error.diagnostics.iter().any(|d| d.code == "SES-T0101"),
            "{error:?}"
        );
    }
}
