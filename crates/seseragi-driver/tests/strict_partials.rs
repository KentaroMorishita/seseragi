use seseragi_driver::{compile_core_module, compile_module, BuildProfile, CompileInput};

#[test]
fn facade_imports_keep_partial_source_stages_and_dictionary_owners() {
    let loaded = seseragi_project::load_local_project(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/spec/fixtures/rust-backend-projects/strict-partials"
    ))
    .unwrap();
    let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
    assert_eq!(core.compiled.modules.len(), 3);
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let project =
            seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile)).unwrap();
        for (owner, module) in &project.compiled.modules {
            assert_eq!(module.core_ir, core.compiled.modules[owner].core_ir);
        }
        assert!(project.compiled.modules[&project.entry_module]
            .generated
            .typescript
            .contains("const __ssrg$partial$stage"));
    }
}

#[test]
fn source_staging_and_checked_types_are_identical_in_both_profiles() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/strict-partials.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/strict-partial-trace.ssrg"),
        "fn prefix<A> value: A -> suffix: String -> String where Show<A> = show value + suffix\npub effect fn main = do { let saved = prefix [9007199254740991 + 1]; println \"unreachable\" }\n",
    ] {
        let mut outputs = vec![];
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let input = CompileInput::new("partial.ssrg", "fixture/partial", source).with_profile(profile);
            let compiled = compile_module(input).unwrap();
            assert_eq!(compiled.core_ir, compile_core_module(input).unwrap().core_ir);
            outputs.push(compiled.generated.typescript);
        }
        // These sources have no erased newtype boundary. Strict checking of
        // either identical raw body therefore checks the other profile too.
        assert_eq!(outputs[0], outputs[1]);
    }
}

#[test]
fn unresolved_qualified_partial_does_not_invent_a_dictionary_or_a_value_restriction() {
    let source = "fn prefix<A> number: Int -> value: A -> String where Show<A> = show number + show value\npub effect fn main = do { let partial = prefix (9007199254740991 + 1); println \"created\" }\n";
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let result = compile_module(
            CompileInput::new("unresolved.ssrg", "fixture/unresolved", source)
                .with_profile(profile),
        );
        let diagnostics = result.unwrap_err();
        assert!(diagnostics
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0101"));
    }
    // Actual generalized lambdas retain independent instantiations; no value
    // restriction is introduced to repair partial-application staging.
    compile_module(CompileInput::new("polymorphic.ssrg", "fixture/polymorphic", "pub fn probe -> (String, String) = { let render = \\value -> show value; (render 42, render True) }" )).unwrap();
}
