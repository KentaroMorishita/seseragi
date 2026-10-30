use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};

#[test]
fn original_array_shadow_keeps_its_literal_owner_through_generic_call_inference() {
    let original = include_str!("../../../examples/spec/fixtures/rust-backend-regressions/inferred-builtin-array-owner.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for source in [
            original.to_owned(),
            original.replace("show [Box [1, 2]]", "show [Box [[], [1, 2]]]"),
        ] {
            let input = || {
                CompileInput::new("owner-雪🙂.ssrg", "proof/owner", &source).with_profile(profile)
            };
            compile_module(input()).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
            assert!(analyze_module(input()).diagnostics.diagnostics.is_empty());
        }
    }
}

#[test]
fn inferred_builtin_literals_do_not_match_same_named_nominal_parameters() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for expression in ["keep [1, 2]", "keep (LocalArray 1)"] {
            let source = format!("type Array<A> = | LocalArray A\nfn keep<A> value: Array<A> -> Array<A> = value\npub fn probe -> Array<Int> = {expression}\n");
            let input = CompileInput::new("nominal-雪🙂.ssrg", "proof/owner", &source)
                .with_profile(profile);
            if expression.contains('[') {
                assert!(
                    compile_module(input).is_err(),
                    "builtin literal matched nominal Array"
                );
            } else {
                compile_module(input).unwrap();
            }
        }
    }
}
