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

#[test]
fn constructor_coherence_accepts_provably_disjoint_type_functions() {
    let source = r#"
type Pair<A, B> = | Pair (A, B)
alias Flip<A, B> = Pair<B, A>
alias LeftInt<A> = Pair<A, Int>
alias LeftBool<A> = Pair<A, Bool>
alias RightInt<A> = Pair<Int, A>
alias FromInt<A> = Int -> A
alias ToInt<A> = A -> Int

trait Tag2<F<_, _>> { fn tag2<A, B> value: F<A, B> -> String }
trait Tag1<F<_>> { fn tag1<A> value: F<A> -> String }

instance Tag2<Pair> { fn tag2<A, B> value: Pair<A, B> -> String = "pair" }
instance Tag2<Flip> { fn tag2<A, B> value: Pair<B, A> -> String = "flip" }

instance Tag1<LeftInt> { fn tag1<A> value: Pair<A, Int> -> String = "left-int" }
instance Tag1<LeftBool> { fn tag1<A> value: Pair<A, Bool> -> String = "left-bool" }
instance Tag1<RightInt> { fn tag1<A> value: Pair<Int, A> -> String = "right-int" }
instance Tag1<FromInt> { fn tag1<A> value: (Int -> A) -> String = "from-int" }
instance Tag1<ToInt> { fn tag1<A> value: (A -> Int) -> String = "to-int" }
"#;
    assert!(
        diagnostics(source).diagnostics.is_empty(),
        "{:#?}",
        diagnostics(source)
    );
}

#[test]
fn constructor_coherence_keeps_relaxed_sections_conservative_across_eta_arities() {
    // F<_, Int, _> eta-reduces to a 1-slot section while Triple<_, _, Int> keeps
    // 2 slots. Instantiating F with `alias SwapLast<A, B, C> = Triple<A, C, B>`
    // makes both heads equal, so coherence must stay conservative (`unproved`).
    let source = r#"
type Triple<A, B, C> = | Triple (A, B, C)
trait Tag2<G<_, _>> { fn tag2<A, B> value: G<A, B> -> String }
instance<F<_, _, _>> Tag2<F<_, Int, _>> { fn tag2<A, B> value: F<A, Int, B> -> String = "generic" }
instance Tag2<Triple<_, _, Int>> { fn tag2<A, B> value: Triple<A, B, Int> -> String = "triple" }
"#;
    let artifact = diagnostics(source);
    assert!(
        artifact
            .diagnostics
            .iter()
            .any(|d| d.message_key == "trait.instance-coherence-unproved"),
        "expected trait.instance-coherence-unproved\n{artifact:#?}"
    );
}

#[test]
fn constructor_coherence_detects_overlap_between_blanket_hkt_and_eta_reduced_section() {
    // Triple<_, Int, _> eta-reduces to a 1-slot section whose body is still
    // partially applied (full arity 2). A blanket F<_, _> overlaps it.
    let source = r#"
type Triple<A, B, C> = | Triple (A, B, C)
trait Tag2<G<_, _>> { fn tag2<A, B> value: G<A, B> -> String }
instance<F<_, _>> Tag2<F> { fn tag2<A, B> value: F<A, B> -> String = "blanket" }
instance Tag2<Triple<_, Int, _>> { fn tag2<A, B> value: Triple<A, Int, B> -> String = "triple" }
"#;
    let artifact = diagnostics(source);
    assert!(
        artifact.diagnostics.iter().any(|d| matches!(
            d.message_key.as_str(),
            "trait.instance-overlap" | "trait.instance-duplicate"
        )),
        "expected overlap diagnostic\n{artifact:#?}"
    );
}

#[test]
fn inverse_hkt_inference_from_alias_values_is_rejected_with_position() {
    for (source, snippet) in [
        (
            "alias Reader<A> = Int -> A\ntrait Extract<F<_>> { fn extract<A> value: F<A> -> A }\ninstance Extract<Reader> { fn extract<A> value: (Int -> A) -> A = value 1 }\npub fn probe value: Reader<Int> -> Int = extract value\n",
            "extract value",
        ),
        (
            "type Pair<A, B> = | Pair (A, B)\nalias Flip<A, B> = Pair<B, A>\ntrait Tag2<F<_, _>> { fn tag2<A, B> value: F<A, B> -> String }\ninstance Tag2<Flip> { fn tag2<A, B> value: Pair<B, A> -> String = \"flip\" }\npub fn probe value: Flip<Int, Bool> -> String = tag2 value\n",
            "tag2 value",
        ),
    ] {
        let artifact = diagnostics(source);
        assert!(
            !artifact.diagnostics.is_empty(),
            "expected positioned diagnostic for {source}"
        );
        let start = source.find(snippet).unwrap();
        let end = start + snippet.len();
        assert!(
            artifact
                .diagnostics
                .iter()
                .any(|d| d.primary.start < end && d.primary.end > start),
            "expected diagnostic overlapping {snippet:?}\n{artifact:#?}"
        );
    }
}
