use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};

#[test]
fn imported_standard_parse_error_show_keeps_its_owner_under_local_shadowing() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/int-parse-error-show.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let output = compile_module(
            CompileInput::new("parse-error-雫.ssrg", "proof/error-show", source)
                .with_profile(profile),
        )
        .unwrap();
        assert!(output.generated.typescript.contains("intParseErrorShow"));
        assert!(output
            .generated
            .typescript
            .contains("type IntParseError as __ssrg$IntParseError"));
        assert!(output.generated.typescript.contains("type IntParseError ="));
        assert!(output
            .generated
            .metadata
            .runtime
            .requirements
            .iter()
            .any(|feature| feature == "core.int.parse-error.show"));
        let core = compile_core_module(CompileInput::new(
            "parse-error-雫.ssrg",
            "proof/error-show",
            source,
        ))
        .unwrap();
        let json = serde_json::to_string(&core.core_ir).unwrap();
        assert!(json.contains("Show<std/int::IntParseError>"));
        assert!(json.contains("proof/error-show::IntParseError"));
    }
}
