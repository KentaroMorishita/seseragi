use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use seseragi_semantics::TypedType;

#[test]
fn formerly_unrepresented_structural_heads_keep_original_sources_and_method_rejections() {
    for (source, wrong) in [
        (
            include_str!("../../../examples/spec/artifacts/semantic-diagnostics-schema-1/alias-constructor-function/main.ssrg"),
            include_str!("../../../examples/spec/artifacts/semantic-diagnostics-schema-1/alias-constructor-function-wrong-method/main.ssrg"),
        ),
        (
            include_str!("../../../examples/spec/artifacts/semantic-diagnostics-schema-1/alias-constructor-record/main.ssrg"),
            include_str!("../../../examples/spec/artifacts/semantic-diagnostics-schema-1/alias-constructor-record-wrong-method/main.ssrg"),
        ),
    ] {
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let compiled = compile_module(
                CompileInput::new("original-雫.ssrg", "fixture/structural", source)
                    .with_profile(profile),
            )
            .unwrap_or_else(|error| panic!("{error:#?}"));
            assert_eq!(compiled.typed_hir.instances.len(), 1);
            let error = compile_module(
                CompileInput::new("wrong-雫.ssrg", "fixture/structural", wrong)
                    .with_profile(profile),
            )
            .unwrap_err();
            assert!(error.diagnostics.iter().any(|diagnostic| {
                diagnostic.message_key == "trait.instance-method-signature-mismatch"
                    && diagnostic.primary.start < diagnostic.primary.end
                    && wrong
                        .get(diagnostic.primary.start..diagnostic.primary.end)
                        .is_some()
            }), "{error:#?}");
        }
    }
}

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

#[test]
fn nested_record_aliases_keep_invariant_field_identities() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/structural-alias-nested.ssrg");
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        compile_module(
            CompileInput::new("nested-雫.ssrg", "fixture/structural", source).with_profile(profile),
        )
        .unwrap_or_else(|e| panic!("{e:#?}"));
        let declarations = source.split("pub fn probe").next().unwrap();
        for inner in [
            "{ value: True, read: \\value: Int -> True }",
            "{ value: 7, read, extra: True }",
            "{ value: 7, read: \\value: Bool -> 7 }",
        ] {
            let wrong = format!(
                "{declarations}\npub fn probe -> Int = take<Nested, Int> {{ inner: {inner} }}\n"
            );
            let error = compile_module(
                CompileInput::new("nested-wrong-雫.ssrg", "fixture/structural", &wrong)
                    .with_profile(profile),
            )
            .unwrap_err();
            assert!(
                error
                    .diagnostics
                    .iter()
                    .any(|d| d.message_key == "call.argument-type-mismatch"
                        && d.primary.start < d.primary.end
                        && d.primary.end <= wrong.len()),
                "{wrong}\n{error:#?}"
            );
        }
    }
}
