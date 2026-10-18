use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn equality_stages_use_the_same_checked_core_in_both_profiles() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partials.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-defect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/equality-partial-children.ssrg"),
    ] {
        let outputs = [BuildProfile::Development, BuildProfile::Release].map(|profile| {
            compile_module(
                CompileInput::new("equality-λ.ssrg", "fixture/equality", source)
                    .with_profile(profile),
            )
            .unwrap()
        });
        assert_eq!(outputs[0].core_ir, outputs[1].core_ir);
        assert_eq!(
            outputs[0].generated.typescript,
            outputs[1].generated.typescript
        );
    }
}

#[test]
fn equality_imports_keep_the_checked_owner_and_child_dictionary() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/equality-partials"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    assert_eq!(core.compiled.modules.len(), 3);
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let result =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        for (id, module) in &result.compiled.modules {
            assert_eq!(module.core_ir, core.compiled.modules[id].core_ir);
        }
    }
}
