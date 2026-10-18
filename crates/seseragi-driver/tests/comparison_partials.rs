use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn specialized_prefixes_and_comparison_methods_are_staged_in_both_profiles() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/comparison-partials.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/comparison-partial-effects.ssrg"),
        "import * as ints from \"std/int\"\npub fn probe -> Unit = { let saved = ints.clamp (9007199254740991 + 1); () }",
        "import * as json from \"std/json\"\npub fn probe -> Unit = { let saved = json.index<Int> (9007199254740991 + 1); () }",
    ] {
        let mut outputs = vec![];
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(CompileInput::new("comparison-λ.ssrg", "fixture/comparison", source).with_profile(profile)).unwrap();
            outputs.push(compiled.generated.typescript);
        }
        assert_eq!(outputs[0], outputs[1]);
        assert!(outputs[0].contains("$saved") || outputs[0].contains("$comparison$stage"));
    }
}

#[test]
fn imported_generic_ord_keeps_its_canonical_owner_and_partial_capture() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/comparison-partials"
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
        assert!(result
            .compiled
            .modules
            .values()
            .any(|m| m.generated.typescript.contains("$comparison$stage")));
    }
}
