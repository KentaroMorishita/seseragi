use seseragi_semantics::{semantic_diagnostics, type_module, TypedDecl, TypedParameter, TypedType};

#[test]
fn an_omitted_function_parameter_is_not_its_own_generic_unit_binder() {
    for binder in ["Unit", "A"] {
        for application in [
            "selected<Int> ()",
            "{ let unit = (); selected<Int> unit }",
            "{ let saved = selected<Int>; saved () }",
            "{ let saved = selected; saved<Int> () }",
            "{ let saved: Unit -> Int -> Bool = selected; saved () }",
        ] {
            let source = format!("fn selected<{binder}> -> ({binder} -> Bool) = \\value -> False\npub fn probe -> Bool = {{ let next: Int -> Bool = {application}; next 1 }}\n");
            let diagnostics = semantic_diagnostics("own-unit-function-λ.ssrg", &source);
            assert!(
                diagnostics.diagnostics.is_empty(),
                "{source}\n{diagnostics:#?}"
            );
        }
    }
}

#[test]
fn an_omitted_local_function_parameter_keeps_the_anonymous_owner() {
    let source = "pub fn probe -> Bool = { fn selected<Unit> -> (Unit -> Bool) = \\value -> False; let unit = (); let next: Int -> Bool = selected<Int> unit; next 1 }\n";
    let diagnostics = semantic_diagnostics("local-unit-function-λ.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn omitted_function_parameters_keep_one_checked_unit_source_stage() {
    let source = "fn selected<Unit> -> (Unit -> Bool) = \\value -> False\n";
    let typed = type_module("unit-function-λ.ssrg", source);
    let TypedDecl::Fn {
        parameters, scheme, ..
    } = &typed.declarations[0]
    else {
        panic!("{typed:#?}")
    };
    assert!(
        matches!(parameters.as_slice(), [TypedParameter::ImplicitUnit {
        type_ref: TypedType::ExternalNamed { canonical, arguments, .. }
    }] if canonical == "std/prelude::Unit" && arguments.is_empty())
    );
    assert!(
        matches!(&scheme.type_ref, TypedType::Function { parameter, .. }
        if matches!(parameter.as_ref(), TypedType::Named { name, arguments }
            if name == "Unit" && arguments.is_empty())),
        "{scheme:#?}"
    );
}

#[test]
fn an_explicit_generic_unit_parameter_remains_generic() {
    let source =
        "fn selected<Unit> value: Unit -> Unit = value\npub fn probe -> Int = selected<Int> 7\n";
    let diagnostics = semantic_diagnostics("explicit-unit-function-λ.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    for argument in ["()", "{ let unit = (); unit }"] {
        let source = format!("fn selected<Unit> value: Unit -> Unit = value\npub fn probe -> Int = selected<Int> {argument}\n");
        let diagnostics = semantic_diagnostics("invalid-unit-function-λ.ssrg", &source);
        assert_eq!(
            diagnostics.diagnostics.len(),
            1,
            "{source}\n{diagnostics:#?}"
        );
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "call.argument-type-mismatch"
        );
    }
}

#[test]
fn omitted_unit_is_distinct_from_a_nominal_and_a_caller_parameter() {
    for source in [
        "type Unit = | LocalUnit\nfn selected<A> -> (A -> Bool) = \\value -> False\npub fn probe -> Bool = { let next = selected<Int> LocalUnit; next 1 }\n",
        "fn selected<Unit> -> (Unit -> Bool) = \\value -> False\npub fn probe<A> value: A -> Bool = { let next = selected<Int> value; next 1 }\n",
    ] {
        let diagnostics = semantic_diagnostics("wrong-unit-owner-λ.ssrg", source);
        assert_eq!(diagnostics.diagnostics.len(), 1, "{source}\n{diagnostics:#?}");
        assert_eq!(diagnostics.diagnostics[0].message_key, "call.argument-type-mismatch");
        assert!(diagnostics.diagnostics[0].primary.start > 0);
    }
}
