use seseragi_driver::{compile_module, BuildProfile, CompileInput};

#[test]
fn abstract_method_authority_retains_declaration_binders_and_constraints() {
    let source = "trait Pick<A> { fn pick<B> value: B -> marker: A -> B where Show<B> }\npub fn probe -> Unit = ()";
    let module = compile_module(CompileInput::new(
        "contract-λ.ssrg",
        "fixture/contracts",
        source,
    ))
    .unwrap();
    let [declaration] = module.core_ir.trait_methods.as_slice() else {
        panic!("declaration")
    };
    assert_eq!(declaration.trait_identity, "fixture/contracts::trait(Pick)");
    assert_eq!(declaration.method, "pick");
    assert_eq!(declaration.trait_parameters[0].name, "A");
    assert_eq!(declaration.signature.type_parameters[0].name, "B");
    assert_eq!(declaration.signature.parameters.len(), 2);
    assert_eq!(
        declaration.constraints[0].trait_identity.as_deref(),
        Some("std/prelude::Show")
    );
    assert_eq!(declaration.origin.source, "contract-λ.ssrg");
    let ordinary = compile_module(CompileInput::new(
        "standard.ssrg",
        "fixture/standard",
        "pub fn probe -> String = { let render = show; render 42 }",
    ))
    .unwrap();
    assert!(ordinary.core_ir.trait_methods.is_empty());
}

#[test]
fn method_value_stages_share_checked_core_in_both_profiles() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-defect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-method-children.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/generalized-method-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/generalized-method-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/generalized-method-defect.ssrg"),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/generalized-method-effects.ssrg"
        ),
        include_str!(
            "../../../examples/spec/fixtures/rust-backend/generalized-method-children.ssrg"
        ),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-references.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-defect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/scoped-method-children.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-order.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/trait-operator-children.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-defect.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-trace.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-effects.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-reference-children.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/method-dictionaries.ssrg"),
        include_str!("../../../examples/spec/fixtures/compile/constrained-operator-alias.ssrg"),
    ] {
        let outputs = [BuildProfile::Development, BuildProfile::Release].map(|profile| {
            compile_module(
                CompileInput::new("method-λ.ssrg", "fixture/method", source).with_profile(profile),
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
fn generalized_method_aliases_still_require_the_selected_instance() {
    for source in [
        "fn saved<A> value: A -> String where Show<A> = { let render = show; render value }\npub fn probe -> String = saved 42",
        "fn saved<Int> value: Int -> String where Show<Int> = { let render: Int -> String = show; render value }\npub fn probe -> String = saved 'λ'",
        "fn saved<Array<_>, A> value: Array<A> -> String where Show<Array<A>> = { let render: Array<A> -> String = show; render value }",
    ] {
        compile_module(CompileInput::new("scope.ssrg", "fixture/scope", source)).unwrap();
    }
    let source =
        "type Missing = | Missing\npub fn probe -> String = { let render = show; render Missing }";
    let error = compile_module(CompileInput::new(
        "missing-λ.ssrg",
        "fixture/missing",
        source,
    ))
    .unwrap_err();
    assert!(error
        .diagnostics
        .iter()
        .any(|d| d.message_key == "instance.missing"));
    assert!(error.diagnostics.iter().all(
        |d| source.is_char_boundary(d.primary.start) && source.is_char_boundary(d.primary.end)
    ));
}

#[test]
fn method_references_keep_imported_owners_and_child_dictionaries() {
    for fixture in [
        "method-references",
        "scoped-methods",
        "trait-operator-order",
        "generalized-methods",
        "trait-methods",
    ] {
        let loaded = seseragi_project::load_local_project(
            std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/spec/fixtures/rust-backend-projects/"
            ))
            .join(fixture),
        )
        .unwrap();
        let core = seseragi_driver::compile_core_local_project(&loaded).unwrap();
        assert_eq!(core.compiled.modules.len(), 3);
        for profile in [BuildProfile::Development, BuildProfile::Release] {
            let result =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(profile))
                    .unwrap();
            for (id, module) in &result.compiled.modules {
                assert_eq!(module.core_ir, core.compiled.modules[id].core_ir);
            }
        }
    }
}
