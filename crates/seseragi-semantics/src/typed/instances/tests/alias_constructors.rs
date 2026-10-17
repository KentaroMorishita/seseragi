use crate::{semantic_diagnostics, type_module, TypedType};

#[test]
fn alias_constructor_eta_keeps_registration_method_contract_and_selected_owner() {
    let source = "type Box<A> = | Box A\nalias Wrapped<A> = Box<A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Wrapped> { fn tag<A> value: Box<A> -> String = \"box\" }\npub fn probe -> String = tag (Box 42)\n";
    let diagnostics = semantic_diagnostics("artifact/alias-eta/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let typed = type_module("artifact/alias-eta/main.ssrg", source);
    assert_eq!(typed.instances.len(), 1);
    assert_eq!(
        typed.instances[0].argument_identities,
        ["artifact/alias-eta::Box"]
    );
    assert!(
        matches!(&typed.instances[0].arguments[..], [TypedType::Named { name, arguments }] if name == "Box" && arguments.is_empty())
    );
}

#[test]
fn alias_constructor_eta_detects_duplicate_against_direct_constructor() {
    for (first, second) in [("Wrapped", "Box"), ("Box", "Wrapped")] {
        let source = format!("// 雫🙂\ntype Box<A> = | Box A\nalias Wrapped<A> = Box<A>\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\ninstance Tag<{first}> {{ fn tag<A> value: Box<A> -> String = \"first\" }}\ninstance Tag<{second}> {{ fn tag<A> value: Box<A> -> String = \"second\" }}\n");
        let diagnostics = semantic_diagnostics("artifact/alias-eta/main.ssrg", &source);
        let error = diagnostics
            .diagnostics
            .iter()
            .find(|d| d.message_key == "trait.instance-duplicate")
            .unwrap_or_else(|| panic!("{source}\n{diagnostics:#?}"));
        assert_eq!(
            &source[error.primary.start..error.primary.end],
            source.lines().nth(5).unwrap()
        );
        assert_eq!(
            &source[error.related[0].primary.start..error.related[0].primary.end],
            source.lines().nth(4).unwrap()
        );
    }
}

#[test]
fn alias_constructor_eta_preserves_fixed_prefix_and_missing_parameter_order() {
    let source = "type Pair<A, B> = | Pair (A, B)\nalias Same<A, B> = Pair<A, B>\nalias IntPair<A> = Pair<Int, A>\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<IntPair> { fn tag<A> value: Pair<Int, A> -> String = \"pair\" }\npub fn probe -> String = tag (Pair (42, True))\n";
    let diagnostics = semantic_diagnostics("artifact/alias-eta-prefix/main.ssrg", source);
    assert!(diagnostics.diagnostics.is_empty(), "{diagnostics:#?}");
    let typed = type_module("artifact/alias-eta-prefix/main.ssrg", source);
    assert_eq!(
        typed.instances[0].argument_identities,
        ["artifact/alias-eta-prefix::Pair<std/prelude::Int>"]
    );
}

#[test]
fn alias_constructor_eta_refuses_unrepresented_shapes_even_when_unused() {
    for (alias, use_head, signature) in [
        ("alias Flip<A, B> = Pair<B, A>", "Flip", "Pair<B, A>"),
        ("alias Repeat<A> = Pair<A, A>", "Repeat", "Pair<A, A>"),
        ("alias Erase<A> = Int", "Erase", "Int"),
        ("alias Identity<A> = A", "Identity", "A"),
        ("alias Nested<A> = Box<Box<A>>", "Nested", "Box<Box<A>>"),
        ("alias Applied<F<_>, A> = F<A>", "Applied", "Box<A>"),
        ("alias Same<A, B> = Pair<A, B>", "Same<_>", "Pair<A, B>"),
    ] {
        let source = format!("// 雫🙂\ntype Box<A> = | Box A\ntype Pair<A, B> = | Pair (A, B)\n{alias}\ntrait Tag<F<_>> {{ fn tag<A, B> value: F<A> -> String }}\ninstance Tag<{use_head}> {{ fn tag<A, B> value: {signature} -> String = \"unused\" }}\n");
        let diagnostics = semantic_diagnostics("artifact/alias-eta-refused/main.ssrg", &source);
        let error = diagnostics
            .diagnostics
            .iter()
            .find(|d| d.message_key == "trait.instance-head-unresolved")
            .unwrap_or_else(|| panic!("{source}\n{diagnostics:#?}"));
        assert_eq!(
            &source[error.primary.start..error.primary.end],
            source.lines().nth(5).unwrap()
        );
    }
}

#[test]
fn alias_constructor_eta_keeps_chains_partial_application_and_explicit_sections() {
    for head in ["IntPair", "Same<Int>", "IntPair<_>", "Chain"] {
        let source = format!("type Pair<A, B> = | Pair (A, B)\nalias Same<A, B> = Pair<A, B>\nalias IntPair<A> = Pair<Int, A>\nalias Chain<A> = IntPair<A>\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\ninstance Tag<{head}> {{ fn tag<A> value: Pair<Int, A> -> String = \"pair\" }}\npub fn probe -> String = tag (Pair (42, True))\n");
        let diagnostics = semantic_diagnostics("artifact/alias-eta-chain/main.ssrg", &source);
        assert!(
            diagnostics.diagnostics.is_empty(),
            "{source}\n{diagnostics:#?}"
        );
    }
}

#[test]
fn alias_constructor_eta_uses_checked_import_owner_under_local_shadowing() {
    let provider = crate::type_module_public_interface(
        "fixture/eta-provider/lib.ssrg",
        "pub type Box<A> = | Box A\npub alias Wrapped<A> = Box<A>\n",
    )
    .into_link_interface();
    let source = "import { Wrapped, Box as Foreign } from \"./provider\"\ntype Box<A> = | Local A\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Wrapped> { fn tag<A> value: Foreign<A> -> String = \"foreign\" }\npub fn probe value: Foreign<Int> -> String = tag value\n";
    let main =
        seseragi_syntax::parse_unlinked_module_interface("main.ssrg", "fixture/eta-main", source);
    let linked = seseragi_project::link_module(
        main,
        &std::collections::BTreeMap::from([(
            "./provider".into(),
            seseragi_project::ModuleLinkTarget::external(provider),
        )]),
    )
    .unwrap();
    let analyzed = crate::analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:#?}"));
    assert_eq!(
        analyzed.typed_hir.instances[0].argument_identities,
        ["fixture/eta-provider::Box"]
    );
    assert!(
        matches!(&analyzed.typed_hir.instances[0].arguments[..], [TypedType::ExternalNamed { canonical, arguments, .. }] if canonical == "fixture/eta-provider::Box" && arguments.is_empty())
    );
    let head = &analyzed.typed_interface.instances[0].head;
    assert!(
        matches!(head, seseragi_syntax::InterfaceType::Apply { arguments, .. } if matches!(&arguments[..], [seseragi_syntax::InterfaceType::ExternalNamed { canonical, arguments, .. }] if canonical == "fixture/eta-provider::Box" && arguments.is_empty()))
    );
}

#[test]
fn alias_constructor_eta_preserves_kind_and_method_binder_boundaries() {
    let declarations =
        "type Box<A> = | Box A\ntype Other<A> = | Other A\nalias Wrapped<A> = Box<A>\n";
    for (tail, expected) in [
        ("trait Tag<F<_, _>> { fn tag<A, B> value: F<A, B> -> String }\ninstance Tag<Wrapped> { fn tag<A, B> value: Box<A> -> String = \"bad\" }\n", "trait.instance-kind"),
        ("trait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Wrapped> { fn tag<A> value: Other<A> -> String = \"bad\" }\n", "trait.instance-method-signature"),
    ] {
        let source = format!("{declarations}{tail}");
        let diagnostics = semantic_diagnostics("artifact/alias-eta-boundary/main.ssrg", &source);
        assert!(diagnostics.diagnostics.iter().any(|d| d.message_key.starts_with(expected)), "{source}\n{diagnostics:#?}");
    }
}
