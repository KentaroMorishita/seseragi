use seseragi_semantics::{semantic_diagnostics, type_module, TypedType};

#[test]
fn omitted_trait_parameters_accept_one_anonymous_unit_argument() {
    for source in [
        "trait Ready<A> { fn ready -> Bool }\ninstance Ready<Int> { fn ready -> Bool = False }\npub fn probe -> Bool = ready<Int> ()\n",
        "trait Ready<A> { fn ready -> A }\ninstance Ready<Int> { fn ready -> Int = 7 }\npub fn probe -> Int = ready ()\n",
        "trait Ready<A> { fn ready -> A }\npub fn probe<A> -> A where Ready<A> = ready ()\n",
        "trait Ready<A> { fn ready -> A }\ninstance Ready<Int> { fn ready -> Int = 7 }\npub fn probe -> Int = { let saved: Unit -> Int = ready; saved () }\n",
        "trait Load<F<_>> { fn load<A> -> (F<A> -> Bool) where Show<A> }\ninstance Load<Maybe> { fn load<A> -> (Maybe<A> -> Bool) where Show<A> = \\value -> False }\npub fn probe -> Bool = { let saved: Unit -> Maybe<Int> -> Bool = load; let next = saved (); next (Just 1) }\n",
    ] {
        let diagnostics = semantic_diagnostics("implicit-unit.ssrg", source);
        assert!(diagnostics.diagnostics.is_empty(), "{source}\n{diagnostics:#?}");
    }
}

#[test]
fn implicit_unit_trait_catalog_retains_one_canonical_source_stage() {
    let source = "type Unit = | LocalUnit\ntrait Ready<A> { fn ready -> (A -> Bool) }\n";
    let typed = type_module("implicit-unit-shadow.ssrg", source);
    let declaration = &typed.trait_methods[0];
    assert_eq!(declaration.signature.parameters.len(), 1);
    assert!(matches!(
        declaration.signature.parameters.as_slice(),
        [TypedType::ExternalNamed { canonical, arguments, .. }]
            if canonical == "std/prelude::Unit" && arguments.is_empty()
    ));
    assert!(matches!(
        declaration.signature.result,
        TypedType::Function { .. }
    ));
}

#[test]
fn omitted_trait_parameters_do_not_consume_a_returned_function_stage() {
    let source = "trait Ready<A> { fn ready -> (A -> Bool) }\ninstance Ready<Int> { fn ready -> (Int -> Bool) = \\value -> False }\npub fn probe -> Bool = { let next = ready<Int> (); next 1 }\n";
    let diagnostics = semantic_diagnostics("returned-function.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn anonymous_unit_is_distinct_from_a_same_named_source_nominal() {
    let prefix = "type Unit = | LocalUnit\ntrait Ready<A> { fn ready -> Int }\ninstance Ready<Int> { fn ready -> Int = 7 }\n";
    for body in [
        "ready<Int> ()",
        "{ let saved = ready<Int>; saved () }",
        "{ let unit = (); ready<Int> unit }",
    ] {
        let source = format!("{prefix}pub fn probe -> Int = {body}\n");
        let diagnostics = semantic_diagnostics("unit-owner.ssrg", &source);
        assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    }
    let source = format!("{prefix}pub fn probe -> Int = ready<Int> LocalUnit\n");
    let diagnostics = semantic_diagnostics("unit-owner-invalid.ssrg", &source);
    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
    assert_eq!(
        &source[diagnostics.diagnostics[0].primary.start..diagnostics.diagnostics[0].primary.end],
        "LocalUnit"
    );
}

#[test]
fn implicit_unit_success_retains_its_checked_identity_across_an_effect_bind() {
    for body in [
        "pub effect fn main = do { value <- genuine (); println (show value) }",
        "pub effect fn main = println (show ())",
    ] {
        let source =
            format!("struct Unit {{value: Int}}\neffect fn genuine = succeed ()\n{body}\n");
        let diagnostics = semantic_diagnostics("unit-effect.ssrg", &source);
        assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    }
    let diagnostics = semantic_diagnostics(
        "nominal-unit-display.ssrg",
        "struct Unit {value: Int}\npub effect fn main = println (show (Unit {value: 1}))\n",
    );
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-T0201"));
}

#[test]
fn saved_units_preserve_the_implicit_method_parameter_identity() {
    for parameter in ["Unit", "A"] {
        for body in [
            "{ let unit = (); load<Maybe, Int> unit }",
            "{ let unit: Unit = (); load<Maybe, Int> unit }",
            "{ let (unit, value) = ((), 1); load<Maybe, Int> unit }",
            "{ let unit = (); let alias = unit; load<Maybe, Int> alias }",
            "{ let saved = load<Maybe, Int>; let unit = (); saved unit }",
        ] {
            let source = format!("trait Load<F<_>> {{ fn load<{parameter}> -> (F<{parameter}> -> Bool) where Show<{parameter}> }}\ninstance Load<Maybe> {{ fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }}\npub fn probe -> Bool = {{ let next: Maybe<Int> -> Bool = {body}; next (Just 1) }}\n");
            let diagnostics = semantic_diagnostics("saved-unit-λ.ssrg", &source);
            assert!(
                diagnostics.diagnostics.is_empty(),
                "{source}\n{diagnostics:#?}"
            );
        }
    }
    let source = "trait Load<F<_>> { fn load<Unit> -> (F<Unit> -> Bool) where Show<Unit> }\ninstance Load<Maybe> { fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }\npub fn probe<Unit> value: Unit -> Bool = { let unit = (); let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }\n";
    let diagnostics = semantic_diagnostics("caller-unit-binder-λ.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn implicit_method_unit_does_not_accept_a_source_type_or_caller_binder() {
    for source in [
        "type Unit = | LocalUnit\ntrait Ready<A> { fn ready<Unit> -> Bool }\ninstance Ready<Int> { fn ready<A> -> Bool = False }\npub fn probe -> Bool = { let unit = LocalUnit; ready<Int, Int> unit }\n",
        "trait Ready<A> { fn ready<Unit> -> Bool }\ninstance Ready<Int> { fn ready<A> -> Bool = False }\npub fn probe<Unit> unit: Unit -> Bool = ready<Int, Int> unit\n",
        "type Unit = | LocalUnit\nalias ForeignUnit = Unit\ntrait Ready<A> { fn ready<Unit> -> Bool }\ninstance Ready<Int> { fn ready<A> -> Bool = False }\nfn selected<Unit> value: Unit -> Bool = { let unit: ForeignUnit = LocalUnit; ready<Int, Int> unit }\npub fn probe -> Bool = selected 7\n",
    ] {
        let diagnostics = semantic_diagnostics("saved-unit-owner-λ.ssrg", source);
        assert_eq!(diagnostics.diagnostics.len(), 1, "{source}\n{diagnostics:#?}");
        assert_eq!(diagnostics.diagnostics[0].message_key, "call.argument-type-mismatch");
        let span = diagnostics.diagnostics[0].primary;
        assert_eq!(&source[span.start..span.end], "unit");
    }
}

#[test]
fn unit_literal_is_not_the_value_of_a_same_named_type_parameter() {
    let source = "pub fn wrong<Unit> value: Unit -> Unit = ()\n";
    let diagnostics = semantic_diagnostics("unit-literal-binder-λ.ssrg", source);
    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    let span = diagnostics.diagnostics[0].primary;
    assert_eq!(&source[span.start..span.end], "()");
}

#[test]
fn anonymous_method_unit_is_independent_of_caller_and_method_binder_names() {
    for method_parameter in ["A", "Unit"] {
        for caller_parameter in ["B", "Unit"] {
            for body in [
                "{ let unit = (); let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }",
                "{ let next: Maybe<Int> -> Bool = load<Maybe, Int> (); next (Just 1) }",
                "{ let saved = load<Maybe, Int>; let unit = (); let next: Maybe<Int> -> Bool = saved unit; next (Just 1) }",
                "{ let unit = (); let next: Maybe<Int> -> Bool = load unit; next (Just 1) }",
                "{ let unit: PreludeUnit = (); let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }",
                "{ let (unit, n) = ((), 7); let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }",
                "{ let state = {unit: (), n: 7}; let unit = state.unit; let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }",
            ] {
                let source = format!("alias PreludeUnit = Unit\ntrait Load<F<_>> {{ fn load<{method_parameter}> -> (F<{method_parameter}> -> Bool) where Show<{method_parameter}> }}\ninstance Load<Maybe> {{ fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }}\nfn selected<{caller_parameter}> value: {caller_parameter} -> Bool = {body}\npub fn probe -> Bool = selected 7\n");
                let diagnostics = semantic_diagnostics("caller-unit-alpha-λ.ssrg", &source);
                assert!(diagnostics.diagnostics.is_empty(), "{source}\n{diagnostics:#?}");
            }
        }
    }
}

#[test]
fn a_closed_unit_alias_annotation_does_not_accept_a_caller_type_parameter() {
    let source = "alias PreludeUnit = Unit\npub fn wrong<Unit> value: Unit -> Bool = { let unit: PreludeUnit = value; False }\n";
    let diagnostics = semantic_diagnostics("unit-alias-binder-λ.ssrg", source);
    assert_eq!(diagnostics.diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics.diagnostics[0].message_key, "let.type-mismatch");
    let span = diagnostics.diagnostics[0].primary;
    assert_eq!(&source[span.start..span.end], "unit");
}

#[test]
fn checked_unit_values_cross_a_caller_binder_without_becoming_that_binder() {
    for method_parameter in ["A", "Unit"] {
        for body in [
            "{ let unit = (); fn selected<Unit> value: Unit -> Bool = { let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }; selected 7 }",
            "{ let unit = (); fn selected<Unit> value: Unit -> Bool = { let next: Maybe<Int> -> Bool = load unit; next (Just 1) }; selected 7 }",
            "selected 7",
        ] {
            let source = format!("trait Load<F<_>> {{ fn load<{method_parameter}> -> (F<{method_parameter}> -> Bool) where Show<{method_parameter}> }}\ninstance Load<Maybe> {{ fn load<U> -> (Maybe<U> -> Bool) where Show<U> = \\value -> False }}\nfn source -> Unit = ()\nfn selected<Unit> value: Unit -> Bool = {{ let unit = source (); let next: Maybe<Int> -> Bool = load<Maybe, Int> unit; next (Just 1) }}\npub fn probe -> Bool = {body}\n");
            let diagnostics = semantic_diagnostics("captured-unit-alpha-λ.ssrg", &source);
            assert!(diagnostics.diagnostics.is_empty(), "{source}\n{diagnostics:#?}");
        }
    }
}
