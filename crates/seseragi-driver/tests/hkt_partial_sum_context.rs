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
fn partially_inferred_validation_function_context_keeps_checked_source_argument_keys() {
    let original =
        include_str!("../../../examples/spec/artifacts/schema-1/validation-apis/main.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("validation-apis.ssrg", "fixture/validation-apis", original)
                .with_profile(profile),
        )
        .unwrap_or_else(|error| panic!("{error:#?}"));
    }
}

#[test]
fn partial_validation_context_still_rejects_wrong_function_inputs_and_error_owners() {
    for source in [
        "import * as validation from \"std/validation\"\npub let wrong:validation.Validation<String,Int> = pure (\\value:String -> 42) <*> validation.valid 7\n",
        "import * as validation from \"std/validation\"\ntype One = | One\ntype Two = | Two\npub let wrong:validation.Validation<One,Int> = pure (\\value:Int -> value) <*> validation.invalid Two\n",
    ] {
        let error = compile_module(CompileInput::new(
            "wrong-validation-context.ssrg",
            "fixture/wrong-validation-context",
            source,
        ))
        .unwrap_err();
        assert!(
            error.diagnostics.iter().any(|d| d.code == "SES-T0101"),
            "{error:?}"
        );
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
