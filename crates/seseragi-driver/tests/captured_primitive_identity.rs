use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn captured_primitive_and_scoped_effects_use_the_shared_standard_module_loader() {
    let original = include_str!(
        "../../../examples/spec/fixtures/rust-backend/captured-primitive-identity-minimal.ssrg"
    );
    let fixture = include_str!(
        "../../../examples/spec/fixtures/rust-backend/captured-primitive-identity.ssrg"
    );
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    for source in [
        original.to_owned(),
        fixture.to_owned(),
        fixture.replace("body \"ready\"", "body \"fail\""),
        fixture.replace("body \"ready\"", "body \"defect\""),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new("captured.ssrg", "fixture/captured", &source)
                    .with_profile(profile),
            )
            .unwrap();
            let checked = format!("{:#?}", compiled.core_ir);
            assert!(!checked.contains("Hole"), "{checked}");
        }
    }
}
