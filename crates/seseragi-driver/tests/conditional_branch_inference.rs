use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn conditional_callback_branches_infer_the_same_checked_maybe_payload_in_both_orders() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/conditional-maybe-callback.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for source in [
            original.to_owned(),
            original.replace(
                "Just (value + 10) else Nothing",
                "Nothing else Just (value + 10)",
            ),
        ] {
            compile_module(
                CompileInput::new("branch-雪🙂.ssrg", "fixture/branch", &source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{error:#?}"));
        }
    }
}

#[test]
fn conditional_inference_preserves_lexical_generic_payload_and_nominal_owners() {
    let source="type Item = | Item Int\npub fn generic<A> value:A -> Maybe<A> = if True then Nothing else Just value\npub fn first value:Int -> Maybe<Item> = if value == 0 then Just (Item value) else Nothing\npub fn second value:Int -> Maybe<Item> = if value == 0 then Nothing else Just (Item value)\n";
    compile_module(CompileInput::new(
        "branch-owner.ssrg",
        "fixture/branch-owner",
        source,
    ))
    .unwrap();
    for branches in ["Just (One 1) else Just (Two 1)", "Just 1 else Just True"] {
        let source=format!("type One = | One Int\ntype Two = | Two Int\npub fn probe value:Int -> Unit = {{let invalid = if value == 0 then {branches};()}}\n");
        let diagnostics = compile_module(CompileInput::new(
            "branch-wrong.ssrg",
            "fixture/branch-wrong",
            &source,
        ))
        .unwrap_err();
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.message_key == "if.branch-type-mismatch"),
            "{diagnostics:?}"
        );
    }
}
