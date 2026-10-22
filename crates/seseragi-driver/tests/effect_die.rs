use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn cold_die_keeps_never_channels_and_contract_widening_in_both_profiles() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/effect-die.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("effect-die.ssrg", "fixture/effect-die", source)
                .with_profile(profile),
        )
        .expect("die supports a saved function and checked Never contract widening");
        assert!(compiled.diagnostics.diagnostics.is_empty());
        let ts = compiled.generated.typescript;
        assert!(ts.contains("_ssrg_effect_die"), "{ts}");
        assert!(ts.contains("@seseragi/runtime/effect"), "{ts}");
    }
}

#[test]
fn die_rejects_non_string_messages_before_lowering() {
    for source in [
        "import * as effects from \"std/effect\"\npub effect fn probe = effects.die 1\n",
        "import { die as abort } from \"std/effect\"\npub effect fn probe = abort 1\n",
        "import * as effects from \"std/effect\"\ntype String = | Message\npub effect fn probe = effects.die Message\n",
    ] {
        let error = compile_module(CompileInput::new("invalid-die.ssrg", "fixture/invalid-die", source))
            .expect_err("only the canonical String message is accepted");
        assert!(!error.diagnostics.is_empty());
    }
}
