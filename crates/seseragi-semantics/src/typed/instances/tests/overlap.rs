use crate::semantic_diagnostics;

#[test]
fn overlap_rejects_generic_and_concrete_heads_even_when_unused() {
    for (first, second) in [("A", "Int"), ("Int", "A")] {
        let declaration = |argument: &str| {
            let binder = if argument == "A" { "<A>" } else { "" };
            format!("instance{binder} Show<Wrapped<{argument}>> {{ fn show value: Box<{argument}> -> String = \"box\" }}\n")
        };
        let source = format!(
            "// 雫🙂\ntype Box<A> = | Box A\nalias Wrapped<A> = Box<A>\n{}{}",
            declaration(first),
            declaration(second)
        );
        let diagnostics = semantic_diagnostics("artifact/overlap/main.ssrg", &source);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-T0202" && d.message_key == "trait.instance-duplicate"),
            "{source}\n{diagnostics:#?}"
        );
    }
}

#[test]
fn overlap_rejects_functional_dependency_conflicts_with_different_outputs() {
    let source = "type Box<A> = | Box A\ninstance<A> Add<Box<A>, Int, Int> { fn add left: Box<A> -> right: Int -> Int = right }\ninstance Add<Box<Bool>, Int, String> { fn add left: Box<Bool> -> right: Int -> String = \"wrong\" }\n";
    let diagnostics = semantic_diagnostics("artifact/overlap/main.ssrg", source);
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202" && d.message_key == "trait.instance-duplicate"),
        "{diagnostics:#?}"
    );
}

#[test]
fn overlap_rejects_deriving_and_a_concrete_user_instance() {
    let source = "type Box<A> deriving Show = | Box A\ninstance Show<Box<Int>> { fn show value: Box<Int> -> String = \"box\" }\n";
    let diagnostics = semantic_diagnostics("artifact/overlap/main.ssrg", source);
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202" && d.message_key == "trait.instance-duplicate"),
        "{diagnostics:#?}"
    );
}

fn diagnostics(source: &str) -> seseragi_syntax::DiagnosticArtifact {
    semantic_diagnostics("artifact/overlap/main.ssrg", source)
}

#[test]
fn overlap_preserves_repeated_binders_and_rejects_infinite_types() {
    for source in [
        "type Pair<A, B> = | Pair (A, B)\ninstance<A> Show<Pair<A, A>> { fn show value: Pair<A, A> -> String = \"same\" }\ninstance Show<Pair<Int, Bool>> { fn show value: Pair<Int, Bool> -> String = \"different\" }\n",
        "type Box<A> = | Box A\ntype Pair<A, B> = | Pair (A, B)\ninstance<A> Show<Pair<A, Box<A>>> { fn show value: Pair<A, Box<A>> -> String = \"left\" }\ninstance<B> Show<Pair<Box<B>, B>> { fn show value: Pair<Box<B>, B> -> String = \"right\" }\n",
        "type Box<A> = | Box A\ninstance Show<Box<Int>> { fn show value: Box<Int> -> String = \"int\" }\ninstance Show<Box<Bool>> { fn show value: Box<Bool> -> String = \"bool\" }\n",
    ] {
        assert!(diagnostics(source).diagnostics.is_empty(), "{source}\n{:#?}", diagnostics(source));
    }
    let source = "type Pair<A, B> = | Pair (A, B)\ninstance<A> Show<Pair<A, A>> { fn show value: Pair<A, A> -> String = \"same\" }\ninstance<B> Show<Pair<B, Int>> { fn show value: Pair<B, Int> -> String = \"integer\" }\n";
    assert!(
        diagnostics(source)
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202"),
        "{:#?}",
        diagnostics(source)
    );
}

#[test]
fn overlap_does_not_use_where_constraints_as_disjointness_or_priority() {
    let source = "type Box<A> = | Box A\ninstance<A> Show<Box<A>> where Eq<A> { fn show value: Box<A> -> String = \"eq\" }\ninstance<B> Show<Box<B>> where Ord<B> { fn show value: Box<B> -> String = \"ord\" }\n";
    assert!(
        diagnostics(source)
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202"),
        "{:#?}",
        diagnostics(source)
    );
}

#[test]
fn overlap_preserves_constructor_parameters_and_distinct_custom_traits() {
    let source = "type Box<A> = | Box A\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance<F<_>> Tag<F> { fn tag<A> value: F<A> -> String = \"generic\" }\ninstance Tag<Box> { fn tag<A> value: Box<A> -> String = \"box\" }\n";
    assert!(
        diagnostics(source)
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202"),
        "{:#?}",
        diagnostics(source)
    );
    let source = "type Box<A> = | Box A\ntrait Add<A, B, C> { fn plus left: A -> right: B -> C }\ninstance Add<Box<Int>, Int, Int> { fn plus left: Box<Int> -> right: Int -> Int = right }\ninstance Add<Box<Int>, Int, String> { fn plus left: Box<Int> -> right: Int -> String = \"str\" }\n";
    assert!(
        diagnostics(source).diagnostics.is_empty(),
        "{:#?}",
        diagnostics(source)
    );
}

#[test]
fn overlap_checks_every_standard_arithmetic_dependency_by_canonical_owner() {
    for (name, method) in [
        ("Add", "add"),
        ("Sub", "sub"),
        ("Mul", "mul"),
        ("Div", "div"),
        ("Rem", "rem"),
        ("Pow", "pow"),
    ] {
        let source = format!("type Box<A> = | Box A\ninstance<A> {name}<Box<A>, Int, Int> {{ fn {method} left: Box<A> -> right: Int -> Int = right }}\ninstance {name}<Box<Bool>, Int, String> {{ fn {method} left: Box<Bool> -> right: Int -> String = \"str\" }}\n");
        assert!(
            diagnostics(&source)
                .diagnostics
                .iter()
                .any(|d| d.code == "SES-T0202"),
            "{source}\n{:#?}",
            diagnostics(&source)
        );
    }
}

#[test]
fn overlap_diagnostic_has_both_declaration_ranges_in_utf8_bytes() {
    let source = "// 雫🙂\ntype Box<A> = | Box A\ninstance<A> Show<Box<A>> { fn show value: Box<A> -> String = \"first\" }\ninstance Show<Box<Int>> { fn show value: Box<Int> -> String = \"second\" }\n";
    let artifact = diagnostics(source);
    let error = artifact
        .diagnostics
        .iter()
        .find(|d| d.code == "SES-T0202")
        .unwrap();
    assert_eq!(
        &source[error.primary.start..error.primary.end],
        source.lines().nth(3).unwrap()
    );
    assert_eq!(
        &source[error.related[0].primary.start..error.related[0].primary.end],
        source.lines().nth(2).unwrap()
    );
}

#[test]
fn overlap_checks_a_local_instance_against_a_checked_imported_blanket() {
    let provider = crate::type_module_public_interface("fixture/provider/lib.ssrg", "pub trait Tag<A> { fn tag value: A -> String }\ninstance<A> Tag<A> { fn tag value: A -> String = \"generic\" }\n").into_link_interface();
    assert_eq!(provider.instances.len(), 1);
    let source = "import { Tag } from \"./provider\"\ntype Box<A> = | Box A\ninstance Tag<Box<Int>> { fn tag value: Box<Int> -> String = \"concrete\" }\n";
    let main =
        seseragi_syntax::parse_unlinked_module_interface("main.ssrg", "fixture/main", source);
    let linked = seseragi_project::link_module(
        main,
        &std::collections::BTreeMap::from([(
            "./provider".into(),
            seseragi_project::ModuleLinkTarget::external(provider),
        )]),
    )
    .unwrap();
    let artifact = crate::analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", source),
        linked,
        source,
    )
    .unwrap_err();
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|d| d.code == "SES-T0202" && d.message_key == "trait.instance-ambiguous"),
        "{artifact:#?}"
    );
}
