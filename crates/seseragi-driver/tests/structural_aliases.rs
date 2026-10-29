use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_semantics::TypedType;

#[test]
fn structural_alias_calls_keep_checked_bodies_in_both_profiles() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/structural-alias-functions.ssrg"
    );
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("structural-雫.ssrg", "fixture/structural-alias", source)
                .with_profile(profile),
        )
        .unwrap_or_else(|e| panic!("{e:#?}"));
        assert_eq!(compiled.typed_hir.instances.len(), 3);
        for instance in &compiled.typed_hir.instances {
            assert!(
                matches!(instance.arguments.as_slice(), [TypedType::TypeFunction { parameter_kinds, .. }] if parameter_kinds == &[0])
            );
        }
        assert!(!compiled.generated.typescript.contains("unknown as"));
    }
}
