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
