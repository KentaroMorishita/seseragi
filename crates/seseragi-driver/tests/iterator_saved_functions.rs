use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn saved_iterator_functions_keep_runtime_imports_and_typed_source_currying_in_both_profiles() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/iterator-saved-runtime.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("iterator-saved-雪🙂.ssrg", "fixture/iterator-saved", source)
                .with_profile(profile),
        )
        .unwrap();
        let ts = &compiled.generated.typescript;
        assert!(ts.contains("unfold as _ssrg_iterator_unfold"), "{ts}");
        assert!(ts.contains("next as _ssrg_iterator_next"), "{ts}");
        assert!(ts.contains("__ssrg$iterator$saved0"), "{ts}");
        assert!(ts.contains("__ssrg$iterator$partial$0"), "{ts}");
        for feature in ["core.iterator.unfold", "core.iterator.next"] {
            assert!(compiled
                .typescript_ir
                .runtime_requirements
                .iter()
                .any(|value| value == feature));
        }
    }
}

#[test]
fn user_next_and_unfold_function_values_do_not_request_iterator_runtime_exports() {
    let source="fn unfold value:Int -> Int = value + 1\nfn next value:Int -> Int = value + 2\npub fn probe -> Int = {let saved:Int -> Int = next;saved (unfold 39)}\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("shadow-iterator.ssrg", "fixture/shadow-iterator", source)
                .with_profile(profile),
        )
        .unwrap();
        assert!(!compiled
            .typescript_ir
            .runtime_requirements
            .iter()
            .any(|value| value.starts_with("core.iterator")));
        assert!(!compiled
            .generated
            .typescript
            .contains("@seseragi/runtime/iterator"));
    }
}
