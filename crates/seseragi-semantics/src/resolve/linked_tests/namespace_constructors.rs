use super::linked_program;
use crate::{
    analyze_linked_module, resolve_linked_module, SymbolNamespace, TypedDecl, TypedExpr,
    TypedPattern, TypedType,
};

const HAND_SOURCE: &str = "pub type Hand =\n  | Rock\n  | Paper\n  | Scissors\n";

#[test]
fn constructs_public_structs_through_named_and_namespace_imports() {
    let domain =
        "pub struct Box<A> { value: A }\npub struct Empty {}\npub alias Wrapped<A> = Box<A>\n";
    let main = concat!(
        "import { Box as Named, Empty } from \"./domain\"\n",
        "import * as domain from \"./domain\"\n",
        "pub fn qualified value: Int -> Named<Int> = domain.Box<Int> { value }\n",
        "pub fn inferred value: Int -> Named<Int> = domain.Box { value }\n",
        "pub fn named value: Int -> domain.Box<Int> = Named<Int> { value }\n",
        "pub fn empty -> Empty = domain.Empty {}\n",
        "pub fn field -> Int = (domain.Box<Int> { value: 42 }).value\n",
        "pub fn direct -> Int = domain.Box<Int> { value: 42 }.value\n",
        "fn read box: Named<Int> -> Int = box.value\n",
        "pub fn argument -> Int = read domain.Box<Int> { value: 42 }\n",
        "pub fn wrapped -> Named<Int> = domain.Wrapped<Int> { value: 42 }\n",
    );
    analyze(main, domain).unwrap();
    let resolved = resolve_linked_module(
        linked_program(main, [("./domain", "fixture/game::domain", domain)]),
        main,
    );
    assert!(resolved.issues.is_empty());
    let named = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "Named")
        .unwrap();
    let qualified = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "domain.Box")
        .unwrap();
    assert_eq!(named.symbol, qualified.symbol);
    assert_eq!(qualified.export.namespace, "type");
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "domain.Box"
            && reference.namespace == SymbolNamespace::Type
            && reference.target == Some(named.symbol)
    }));
}

#[test]
fn qualified_struct_construction_keeps_visibility_and_field_checks() {
    let domain = concat!(
        "pub struct Box<A> { value: A }\n",
        "pub opaque struct Secret { value: Int }\n",
        "struct Hidden { value: Int }\n",
    );
    for (expression, code, key) in [
        ("domain.Missing {}", "SES-N0104", "module.export-unresolved"),
        (
            "domain.Hidden { value: 1 }",
            "SES-N0102",
            "module.private-symbol",
        ),
        (
            "domain.Secret { value: 1 }",
            "SES-T0101",
            "struct.representation-private",
        ),
        ("domain.Box<Int> {}", "SES-T0101", "struct.field-missing"),
        (
            "domain.Box<Int> { value: False }",
            "SES-T0101",
            "struct.field-type-mismatch",
        ),
        (
            "domain.Box<Int> { value: 1, extra: 2 }",
            "SES-T0101",
            "struct.field-unresolved",
        ),
        (
            "domain.Box<Int, Bool> { value: 1 }",
            "SES-T0101",
            "struct.type-argument-arity-mismatch",
        ),
    ] {
        let main = format!("import * as domain from \"./domain\"\nlet invalid = {expression}\n");
        let diagnostics = analyze(&main, domain).unwrap_err();
        let actual = diagnostics
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.message_key.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(actual, [(code, key)], "{expression}: {diagnostics:#?}");
    }
}

#[test]
fn qualified_record_payload_constructors_still_resolve_as_values() {
    let domain = "pub type Box<A> = | Box { value: A }\n";
    let main = concat!(
        "import * as domain from \"./domain\"\n",
        "pub fn boxed -> domain.Box<Int> = domain.Box { value: 42 }\n",
    );
    let analyzed = analyze(main, domain).unwrap();
    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn { body: TypedExpr::Call { callee, .. }, .. }
            if callee == "fixture/game::domain::Box"
    ));
    let main = main.replace("value: 42", "value: False");
    let diagnostics = analyze(&main, domain).unwrap_err();
    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(diagnostics.diagnostics[0].code, "SES-T0101");
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
}

#[test]
fn qualified_record_payloads_keep_application_order_and_single_name_blocks() {
    let domain = "pub type Box = | Box { value: Int }\npub type Number = | Number Int\n";
    let main = concat!(
        "import * as domain from \"./domain\"\n",
        "fn apply make: ({ value: Int } -> domain.Box) -> payload: { value: Int } -> domain.Box = make payload\n",
        "pub fn boxed -> domain.Box = apply domain.Box { value: 42 }\n",
        "let value = 42\n",
        "pub fn number -> domain.Number = domain.Number { value }\n",
    );
    analyze(main, domain).unwrap();
}

#[test]
fn types_a_namespace_qualified_constructor_as_its_canonical_adt_value() {
    let main_source = "import * as domain from \"./domain\"\n\npub fn choose unit: Unit -> domain.Hand = domain.Rock\n";
    let analyzed = analyze(main_source, HAND_SOURCE).unwrap();

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Variable {
                name,
                type_ref: TypedType::ExternalNamed { canonical, arguments, .. },
                ..
            },
            ..
        } if name == "fixture/game::domain::Rock"
            && canonical == "fixture/game::domain::Hand"
            && arguments.is_empty()
    ));
}

#[test]
fn comparison_still_rejects_distinct_owners_and_type_arguments() {
    let domain = "pub type Box<A> deriving Eq = | Box A\n";
    for expression in [
        "domain.Box 1 == Local 1",
        "domain.Box 1 == domain.Box \"one\"",
    ] {
        let main = format!(
            "import * as domain from \"./domain\"\ntype Box<A> deriving Eq = | Local A\npub fn check -> Bool = {expression}\n"
        );
        let diagnostics = analyze(&main, domain).unwrap_err();
        assert!(
            diagnostics
                .diagnostics
                .iter()
                .any(|item| item.code == "SES-T0101"),
            "{diagnostics:#?}"
        );
    }
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
