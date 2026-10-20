use seseragi_driver::{compile_core_module, compile_module, CompileInput};

#[test]
fn implicit_unit_methods_share_checked_signatures_and_provider_provenance() {
    for prefix in ["", "type Unit = | LocalUnit\n"] {
        for body in ["ready<Int> ()", "{ let saved = ready<Int>; saved () }"] {
            let source = format!("{prefix}trait Ready<A> {{ fn ready -> Int }}\ninstance Ready<Int> {{ fn ready -> Int = 7 }}\npub fn probe -> Int = {body}\n");
            let input = CompileInput::new("unit-λ.ssrg", "fixture/unit", &source);
            let core = compile_core_module(input.clone()).unwrap().core_ir;
            let compiled = compile_module(input).unwrap();
            assert_eq!(core, compiled.core_ir);
            assert_eq!(core.trait_methods[0].signature.parameters.len(), 1);
            assert!(compiled.diagnostics.diagnostics.is_empty());
        }
    }
}

#[test]
fn nominal_unit_argument_is_rejected_before_either_backend() {
    let source = "// 雫\ntype Unit = | LocalUnit\ntrait Ready<A> { fn ready -> Int }\ninstance Ready<Int> { fn ready -> Int = 7 }\npub fn probe -> Int = ready<Int> LocalUnit\n";
    let input = CompileInput::new("unit-owner-λ.ssrg", "fixture/unit-owner", source);
    let core = compile_core_module(input.clone()).unwrap_err();
    let ts = compile_module(input).unwrap_err();
    assert_eq!(core, ts);
    assert_eq!(core.diagnostics.len(), 1);
    assert_eq!(
        core.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
    let span = core.diagnostics[0].primary;
    assert_eq!(&source[span.start..span.end], "LocalUnit");
}

#[test]
fn saved_intrinsic_units_keep_method_binders_separate_in_both_profiles() {
    for parameter in ["Unit", "A"] {
        for body in [
            "{ let unit = (); load<Maybe, Int> unit }",
            "{ let unit: Unit = (); load<Maybe, Int> unit }",
            "{ let saved = load<Maybe, Int>; let unit = (); saved unit }",
        ] {
            let source = format!("trait Load<F<_>> {{ fn load<{parameter}> -> (F<{parameter}> -> Bool) where Show<{parameter}> }}\ninstance Load<Maybe> {{ fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }}\npub fn probe -> Bool = {{ let next: Maybe<Int> -> Bool = {body}; next (Just 1) }}\n");
            for profile in [
                seseragi_driver::BuildProfile::Development,
                seseragi_driver::BuildProfile::Release,
            ] {
                let input = CompileInput::new("saved-unit-λ.ssrg", "fixture/saved-unit", &source)
                    .with_profile(profile);
                let core = compile_core_module(input.clone()).unwrap().core_ir;
                let compiled = compile_module(input).unwrap();
                assert_eq!(core, compiled.core_ir);
                assert_eq!(core.trait_methods[0].signature.parameters.len(), 1);
                assert!(compiled.diagnostics.diagnostics.is_empty());
            }
        }
    }
}

#[test]
fn unit_literal_cannot_implement_a_same_named_universal_result() {
    let source = "pub fn wrong<Unit> value: Unit -> Unit = ()\n";
    for profile in [
        seseragi_driver::BuildProfile::Development,
        seseragi_driver::BuildProfile::Release,
    ] {
        let input = CompileInput::new("unit-result-λ.ssrg", "fixture/unit-result", source)
            .with_profile(profile);
        let core = compile_core_module(input.clone()).unwrap_err();
        let ts = compile_module(input).unwrap_err();
        assert_eq!(core, ts);
        assert_eq!(core.diagnostics.len(), 1);
        assert_eq!(core.diagnostics[0].code, "SES-T0101");
        let span = core.diagnostics[0].primary;
        assert_eq!(&source[span.start..span.end], "()");
    }
}

#[test]
fn anonymous_unit_callers_do_not_capture_the_method_parameter_identity() {
    for parameter in ["A", "Unit"] {
        let source = format!("trait Load<F<_>> {{ fn load<{parameter}> -> (F<{parameter}> -> Bool) where Show<{parameter}> }}\ninstance Load<Maybe> {{ fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }}\nfn selected<Unit> value: Unit -> Bool = {{ let unit = (); let saved = load<Maybe, Int>; let next: Maybe<Int> -> Bool = saved unit; next (Just 1) }}\npub fn probe -> Bool = selected 7\n");
        for profile in [
            seseragi_driver::BuildProfile::Development,
            seseragi_driver::BuildProfile::Release,
        ] {
            let input = CompileInput::new("unit-alpha-λ.ssrg", "fixture/unit-alpha", &source)
                .with_profile(profile);
            let core = compile_core_module(input.clone()).unwrap().core_ir;
            let compiled = compile_module(input).unwrap();
            assert_eq!(core, compiled.core_ir);
            assert!(compiled.diagnostics.diagnostics.is_empty());
        }
    }
}
