use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};

#[test]
fn operator_staging_preserves_checked_core_in_both_typescript_profiles() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partials.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-defect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/operator-partial-effects.ssrg"),
    ] {
        let input = CompileInput::new("operator-λ.ssrg", "fixture/operator", source);
        let core = compile_core_module(input).unwrap().core_ir;
        let mut outputs = vec![];
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(input.with_profile(profile)).unwrap();
            assert_eq!(compiled.core_ir, core);
            outputs.push(compiled.generated.typescript);
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}
