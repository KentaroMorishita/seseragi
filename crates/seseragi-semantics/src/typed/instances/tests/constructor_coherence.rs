use crate::semantic_diagnostics;

fn diagnostics(source: &str) -> seseragi_syntax::DiagnosticArtifact {
    semantic_diagnostics("artifact/constructor-coherence/main.ssrg", source)
}

#[test]
fn constructor_coherence_identifies_eta_equivalent_explicit_sections() {
    for (name, parameters, first, second, value) in [
        ("Box", "A", "Box", "Box<_>", "Box<A>"),
        ("Pair", "A, B", "Pair<Int>", "Pair<Int, _>", "Pair<Int, A>"),
    ] {
        for (first, second) in [(first, second), (second, first)] {
            let source = format!("// 雫🙂\ntype {name}<{parameters}> = | Value\ntrait Tag<F<_>> {{ fn tag<A> value: F<A> -> String }}\ninstance Tag<{first}> {{ fn tag<A> value: {value} -> String = \"first\" }}\ninstance Tag<{second}> {{ fn tag<A> value: {value} -> String = \"second\" }}\n");
            let artifact = diagnostics(&source);
            let error = artifact
                .diagnostics
                .iter()
                .find(|d| d.message_key == "trait.instance-duplicate")
                .unwrap_or_else(|| panic!("{source}\n{artifact:#?}"));
            assert_eq!(
                &source[error.primary.start..error.primary.end],
                source.lines().nth(4).unwrap()
            );
            assert_eq!(
                &source[error.related[0].primary.start..error.related[0].primary.end],
                source.lines().nth(3).unwrap()
            );
        }
    }
}

#[test]
fn constructor_coherence_refuses_unproved_applied_constructor_overlap() {
    for target in ["Pair<Bool, Int>", "Int"] {
        for reverse in [false, true] {
            let generic = "instance<F<_>> Show<Box<F<Bool>>> { fn show value: Box<F<Bool>> -> String = \"generic\" }";
            let concrete = format!("instance Show<Box<{target}>> {{ fn show value: Box<{target}> -> String = \"concrete\" }}");
            let (first, second) = if reverse {
                (concrete.as_str(), generic)
            } else {
                (generic, concrete.as_str())
            };
            let source = format!("// 雫🙂\ntype Box<A> = | Box A\ntype Pair<A, B> = | Pair (A, B)\n{first}\n{second}\n");
            let artifact = diagnostics(&source);
            let error = artifact
                .diagnostics
                .iter()
                .find(|d| d.message_key == "trait.instance-coherence-unproved")
                .unwrap_or_else(|| panic!("{source}\n{artifact:#?}"));
            assert_eq!(error.code, "SES-T0101");
            assert_eq!(&source[error.primary.start..error.primary.end], second);
            assert_eq!(
                &source[error.related[0].primary.start..error.related[0].primary.end],
                first
            );
            assert!(
                !artifact
                    .diagnostics
                    .iter()
                    .any(|d| d.message_key == "trait.instance-duplicate"),
                "unproved is not a proved overlap: {artifact:#?}"
            );
        }
    }
}

#[test]
fn constructor_coherence_keeps_single_hkt_heads_and_rigid_disjoint_owners() {
    let declarations = "type Box<A> = | Box A\ntype Other<A> = | Other A\ninstance<F<_>> Show<Box<F<Bool>>> { fn show value: Box<F<Bool>> -> String = \"generic\" }\n";
    for suffix in [
        "",
        "instance Show<Other<Int>> { fn show value: Other<Int> -> String = \"other\" }\n",
    ] {
        let source = format!("{declarations}{suffix}");
        assert!(
            diagnostics(&source).diagnostics.is_empty(),
            "{source}\n{:#?}",
            diagnostics(&source)
        );
    }
}

#[test]
fn constructor_coherence_refuses_unproved_import_without_using_foreign_spans() {
    let provider = crate::type_module_public_interface("fixture/provider/lib.ssrg", "pub trait Tag<A> { fn tag value: A -> String }\ninstance<F<_>> Tag<F<Bool>> { fn tag value: F<Bool> -> String = \"generic\" }\n").into_link_interface();
    assert_eq!(provider.instances.len(), 1);
    let source = "// 雫🙂\nimport { Tag } from \"./provider\"\ntype Box<A> = | Box A\ninstance Tag<Box<Int>> { fn tag value: Box<Int> -> String = \"concrete\" }\n";
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
    let error = artifact
        .diagnostics
        .iter()
        .find(|d| d.message_key == "trait.instance-coherence-unproved")
        .unwrap_or_else(|| panic!("{artifact:#?}"));
    assert_eq!(
        &source[error.primary.start..error.primary.end],
        source.lines().nth(3).unwrap()
    );
    for related in &error.related {
        assert!(
            source
                .get(related.primary.start..related.primary.end)
                .is_some(),
            "a provider's foreign byte span must not be used in main.ssrg"
        );
    }
}

#[test]
fn constructor_coherence_preserves_distinct_rigid_owners_inside_sections() {
    let source = "type Pair<A, B> = | Pair (A, B)\ntrait Tag<F<_>> { fn tag<A> value: F<A> -> String }\ninstance Tag<Pair<_, Int>> { fn tag<A> value: Pair<A, Int> -> String = \"pair\" }\ninstance Tag<Maybe> { fn tag<A> value: Maybe<A> -> String = \"maybe\" }\n";
    assert!(
        diagnostics(source).diagnostics.is_empty(),
        "{:#?}",
        diagnostics(source)
    );
}
