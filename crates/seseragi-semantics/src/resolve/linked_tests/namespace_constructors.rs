use super::linked_program;
use crate::{analyze_linked_module, TypedDecl, TypedExpr, TypedPattern, TypedType};

const HAND_SOURCE: &str = "pub type Hand =\n  | Rock\n  | Paper\n  | Scissors\n";

#[test]
fn types_a_namespace_qualified_constructor_as_its_canonical_adt_value() {
    let main_source = "import * as domain from \"./domain\"\n\npub fn choose unit: Unit -> domain.Hand = domain.Rock\n";
    let analyzed = analyze(main_source, HAND_SOURCE).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Variable {
                name,
                type_ref: TypedType::Named { arguments, .. },
                ..
            },
            ..
        } if name == "fixture/game::domain::Rock" && arguments.is_empty()
    ));
}

#[test]
fn accepts_a_namespace_qualified_constructor_for_a_qualified_function_owner() {
    let domain_source = format!("{HAND_SOURCE}\npub fn accept hand: Hand -> Unit = ()\n");
    let main_source = "import * as domain from \"./domain\"\n\npub fn run unit: Unit -> Unit = domain.accept domain.Rock\n";
    let analyzed = analyze(main_source, &domain_source).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Call {
                callee,
                arguments,
                type_ref,
                ..
            },
            ..
        } if callee == "fixture/game::domain::accept"
            && *type_ref == TypedType::Named {
                name: "Unit".to_owned(),
                arguments: Vec::new(),
            }
            && matches!(
                arguments.as_slice(),
                [TypedExpr::Variable { name, .. }]
                    if name == "fixture/game::domain::Rock"
            )
    ));
}

#[test]
fn types_an_exhaustive_match_over_namespace_qualified_adt_patterns() {
    let main_source = "import * as domain from \"./domain\"\n\npub fn render hand: domain.Hand -> String =\n  match hand {\n    domain.Rock -> \"rock\"\n    domain.Paper -> \"paper\"\n    domain.Scissors -> \"scissors\"\n  }\n";
    let analyzed = analyze(main_source, HAND_SOURCE).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Match {
                exhaustive: true,
                arms,
                ..
            },
            ..
        } if arms.iter().zip(["Rock", "Paper", "Scissors"]).all(|(arm, name)| {
            matches!(
                &arm.pattern,
                TypedPattern::Constructor { symbol, .. }
                    if symbol == &format!("fixture/game::domain::{name}")
            )
        })
    ));
}

#[test]
fn types_an_exhaustive_tuple_match_over_namespace_qualified_adt_patterns() {
    let main_source = "import * as domain from \"./domain\"\n\npub fn same first: domain.Hand -> second: domain.Hand -> Bool =\n  match (first, second) {\n    (domain.Rock, domain.Rock) -> True\n    (domain.Rock, domain.Paper) -> False\n    (domain.Rock, domain.Scissors) -> False\n    (domain.Paper, domain.Rock) -> False\n    (domain.Paper, domain.Paper) -> True\n    (domain.Paper, domain.Scissors) -> False\n    (domain.Scissors, domain.Rock) -> False\n    (domain.Scissors, domain.Paper) -> False\n    (domain.Scissors, domain.Scissors) -> True\n  }\n";
    let analyzed = analyze(main_source, HAND_SOURCE).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Match {
                exhaustive: true,
                arms,
                ..
            },
            ..
        } if arms.len() == 9
            && arms.iter().all(|arm| matches!(
                &arm.pattern,
                TypedPattern::Tuple { elements, .. }
                    if elements.len() == 2
                        && elements.iter().all(|element| matches!(
                            element,
                            TypedPattern::Constructor { symbol, .. }
                                if symbol.starts_with("fixture/game::domain::")
                        ))
            ))
    ));
}

#[test]
fn reports_a_missing_namespace_qualified_adt_pattern() {
    let main_source = "import * as domain from \"./domain\"\n\npub fn render hand: domain.Hand -> String =\n  match hand {\n    domain.Rock -> \"rock\"\n    domain.Paper -> \"paper\"\n  }\n";
    let diagnostics = analyze(main_source, HAND_SOURCE).unwrap_err();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0301");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "match.non-exhaustive"
    );
}

#[test]
fn treats_named_and_namespace_qualified_constructor_patterns_as_the_same_arm() {
    for duplicate_arms in [
        "Rock -> \"named\"\n    domain.Rock -> \"qualified\"",
        "domain.Rock -> \"qualified\"\n    Rock -> \"named\"",
    ] {
        let main_source = format!(
            "import {{ Hand, Rock }} from \"./domain\"\nimport * as domain from \"./domain\"\n\npub fn render hand: Hand -> String =\n  match hand {{\n    {duplicate_arms}\n    domain.Paper -> \"paper\"\n    domain.Scissors -> \"scissors\"\n  }}\n"
        );
        let diagnostics = analyze(&main_source, HAND_SOURCE).unwrap_err();

        assert_eq!(diagnostics.diagnostics.len(), 1);
        assert_eq!(diagnostics.diagnostics[0].code, "SES-T0302");
        assert_eq!(
            diagnostics.diagnostics[0].message_key,
            "match.unreachable-arm"
        );
    }
}

fn analyze(
    main_source: &str,
    domain_source: &str,
) -> Result<crate::AnalyzedModule, seseragi_syntax::DiagnosticArtifact> {
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );
    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
}
