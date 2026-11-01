use seseragi_driver::{analyze_module, compile_module, BuildProfile, CompileInput};
use seseragi_semantics::{TypedDecl, TypedExpr, TypedInstanceEvidence};

const SOURCE: &str =
    include_str!("../../../examples/spec/fixtures/compile/html-saved-helpers.ssrg");

#[test]
fn inferred_html_helpers_lower_with_their_checked_constraints_in_both_profiles() {
    for profile in [BuildProfile::Development, BuildProfile::Release] {
        let compiled = compile_module(
            CompileInput::new("html-saved-雪.ssrg", "fixture/html-saved", SOURCE)
                .with_profile(profile),
        )
        .unwrap();
        assert!(compiled.diagnostics.diagnostics.is_empty());
        let evidence = compiled
            .typed_hir
            .declarations
            .iter()
            .find_map(|decl| match decl {
                TypedDecl::Let {
                    value:
                        TypedExpr::Call {
                            callee, evidence, ..
                        },
                    ..
                } if callee.ends_with("::savedStyle") => Some(evidence),
                _ => None,
            })
            .unwrap();
        assert_eq!(evidence.len(), 1);
        assert_eq!(
            evidence[0].trait_identity.as_deref(),
            Some("std/web/html::trait(StyleRecord)")
        );
        assert!(matches!(
            &evidence[0].evidence,
            TypedInstanceEvidence::Standard { identity, .. }
                if identity == "std/web/html::StyleRecord<Record>"
        ));
        let direct_evidence = compiled
            .typed_hir
            .declarations
            .iter()
            .find_map(|decl| match decl {
                TypedDecl::Let {
                    value:
                        TypedExpr::Call {
                            callee, evidence, ..
                        },
                    ..
                } if callee == "std/web/html::style" => Some(evidence),
                _ => None,
            })
            .unwrap();
        assert_eq!(evidence, direct_evidence);
        for feature in ["web.html.style", "web.html.div", "web.html.fragment"] {
            assert!(compiled
                .typescript_ir
                .runtime_requirements
                .iter()
                .any(|value| value == feature));
        }
    }
}

#[test]
fn inferred_html_helpers_keep_invalid_style_and_children_as_source_diagnostics() {
    for (helper, argument) in [
        ("style", "42"),
        ("style", "{ color: 42 }"),
        ("style", "{ variables: { accentColor: 42 } }"),
        ("div", "{ children: 42 }"),
        ("fragment", "42"),
    ] {
        let source = format!(
            "import * as html from \"std/web/html\"\nlet saved = html.{helper}\nlet invalid = saved {argument}\n"
        );
        let input = CompileInput::new("html-invalid.ssrg", "fixture/html-invalid", &source);
        let diagnostics = compile_module(input).unwrap_err();
        assert_eq!(analyze_module(input).diagnostics, diagnostics);
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key == "instance.missing"),
            "{source}\n{diagnostics:?}"
        );
    }
}
