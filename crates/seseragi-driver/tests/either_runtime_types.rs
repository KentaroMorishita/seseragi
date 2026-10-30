use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn either_runtime_calls_keep_distinct_error_and_success_types_across_every_saved_source_stage() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for source in [
            include_str!("../../../examples/spec/fixtures/rust-backend-regressions/either-saved-runtime-types.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/basic-either-operations.ssrg"),
            include_str!("../../../examples/spec/fixtures/rust-backend/basic-either-operations-effects.ssrg"),
        ] {
            let compiled=compile_module(CompileInput::new("sum-types-雪🙂.ssrg","fixture/sum-types",source).with_profile(profile)).unwrap();
            assert!(compiled.generated.typescript.contains("_ssrg_either_traverse<"),"{}",compiled.generated.typescript);
        }
    }
}

#[test]
fn match_and_derived_guards_resolve_host_error_without_capturing_source_error_or_global_this() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        for source in [
            include_str!("../../../examples/spec/fixtures/rust-backend-regressions/host-error-shadow.ssrg").to_owned(),
            include_str!("../../../examples/spec/fixtures/rust-backend-regressions/host-global-this-binding.ssrg").to_owned(),
            "type Error deriving Eq,Ord = | First | Second\npub fn compareErrors -> Bool = First < Second\n".into(),
        ] {
            let compiled=compile_module(CompileInput::new("host-雪🙂.ssrg","fixture/host",&source).with_profile(profile)).unwrap();
            assert!(!compiled.generated.typescript.contains("new Error("));
            if compiled.generated.typescript.contains("throw new ") {
                assert!(compiled.generated.typescript.contains("new globalThis.Error("));
            }
        }
    }
}
