use crate::{semantic_diagnostics, type_module, TypedType};

#[test]
fn explicit_primitive_owners_survive_unrelated_same_named_binders() {
    for (primitive, literal) in [
        ("Unit", "()"),
        ("Bool", "True"),
        ("Int", "42"),
        ("Float", "1.5"),
        ("Char", "'a'"),
        ("String", "\"sample\""),
    ] {
        let source = format!("fn unrelated<{primitive}> value:{primitive} -> {primitive} = value\nfn render<A> value:A -> String where Show<A> = show value\npub fn probe -> String = render<{primitive}> {literal}\n");
        let diagnostics = semantic_diagnostics("primitive-owner-雫.ssrg", &source);
        assert!(
            diagnostics.diagnostics.is_empty(),
            "{source}\n{diagnostics:#?}"
        );
        let wrong = format!(
            "// 雫🙂\npub fn wrong<{primitive}> value:{primitive} -> {primitive} = {literal}\n"
        );
        let diagnostics = semantic_diagnostics("primitive-binder-雫.ssrg", &wrong);
        let error = diagnostics
            .diagnostics
            .iter()
            .find(|d| d.message_key == "function.return-type-mismatch")
            .unwrap_or_else(|| panic!("{wrong}\n{diagnostics:#?}"));
        assert_eq!(&wrong[error.primary.start..error.primary.end], literal);
    }
}

#[test]
fn closed_structural_alias_heads_keep_scoped_value_slots_and_method_contracts() {
    for body in [
        "Int -> A",
        "{ value: A }",
        "(A, Int)",
        "{ value: (A, Int -> A), ready?: Bool }",
    ] {
        let source = format!("// 雫🙂\nalias Shape<A> = {body}\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\ninstance Tag<Shape> {{ fn tag<A> value: ({body}) -> String = \"shape\" }}\n");
        let diagnostics = semantic_diagnostics("artifact/structural-alias/main.ssrg", &source);
        assert!(
            diagnostics.diagnostics.is_empty(),
            "{source}\n{diagnostics:#?}"
        );
        let typed = type_module("artifact/structural-alias/main.ssrg", &source);
        let [TypedType::TypeFunction {
            parameter_kinds,
            body,
        }] = typed.instances[0].arguments.as_slice()
        else {
            panic!("{typed:#?}")
        };
        assert_eq!(parameter_kinds, &[0]);
        assert!(matches!(
            body.as_ref(),
            TypedType::Function { .. }
                | TypedType::Record { closed: true, .. }
                | TypedType::Tuple { .. }
        ));
        assert!(typed.instances[0].argument_identities[0].starts_with("lambda[0]"));
    }
}

#[test]
fn structural_alias_alpha_duplicates_report_original_positions() {
    for body in ["Int -> A", "{ value: A }", "(A, Int)"] {
        let source = format!("// 雫🙂\nalias Shape<A> = {body}\nalias Other<X> = Shape<X>\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\ninstance Tag<Shape> {{ fn tag<A> value: ({body}) -> String = \"first\" }}\ninstance Tag<Other> {{ fn tag<A> value: ({body}) -> String = \"second\" }}\n");
        let diagnostics = semantic_diagnostics("artifact/structural-alias/main.ssrg", &source);
        let error = diagnostics
            .diagnostics
            .iter()
            .find(|d| d.message_key == "trait.instance-duplicate")
            .unwrap_or_else(|| panic!("{source}\n{diagnostics:#?}"));
        assert!(source[error.primary.start..error.primary.end].starts_with("instance Tag<Other>"));
        assert!(error
            .related
            .iter()
            .any(|r| source[r.primary.start..r.primary.end].starts_with("instance Tag<Shape>")));
    }
}

#[test]
fn structural_aliases_reject_wrong_components_and_lost_free_parameters() {
    for (declarations, head, signature, key) in [
        (
            "alias Shape<A> = Int -> A",
            "Shape",
            "A -> Int",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "alias Shape<A> = { value: A }",
            "Shape",
            "{ other: A }",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "alias Shape<A> = (A, Int)",
            "Shape",
            "(Int, A)",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "type Local = | Local\nalias Shape<A> = Local -> A",
            "Shape",
            "Int -> A",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "alias Shape<A> = Int -> A",
            "Shape",
            "Int -> B",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "alias Shape<T, A> = { value: A }",
            "Shape<T>",
            "{ value: A }",
            "trait.instance-head-unresolved",
        ),
    ] {
        let instance = if head == "Shape<T>" {
            "instance<T>"
        } else {
            "instance"
        };
        let source = format!("// 雫🙂\n{declarations}\ntrait Tag<F<_>> {{ fn tag<A, B> value: F<A> -> String }}\n{instance} Tag<{head}> {{ fn tag<A, B> value: ({signature}) -> String = \"unused\" }}\n");
        let diagnostics = semantic_diagnostics("artifact/structural-alias/main.ssrg", &source);
        let error = diagnostics
            .diagnostics
            .iter()
            .find(|d| d.message_key == key)
            .unwrap_or_else(|| panic!("{source}\n{diagnostics:#?}"));
        assert!(error.primary.start < error.primary.end && error.primary.end <= source.len());
    }
}
