use super::linked_program;
use crate::{analyze_linked_module, resolve_linked_module, SymbolNamespace, TypedDecl, TypedType};

#[test]
fn resolves_a_namespace_qualified_imported_type() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn keep value: domain.Hand -> domain.Hand = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);

    assert!(resolved.issues.is_empty());
    let selected = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "domain.Hand")
        .unwrap();
    assert!(selected.in_scope);
    assert_eq!(selected.export.namespace, "type");
    assert_eq!(selected.export.symbol, "fixture/game::domain::Hand");
    assert!(!resolved.imports.iter().any(|import| {
        !import.in_scope && import.export.symbol == "fixture/game::domain::Hand"
    }));
    assert_eq!(
        resolved
            .symbols
            .iter()
            .filter(|symbol| symbol.canonical.as_deref() == Some("fixture/game::domain::Hand"))
            .count(),
        1
    );
    assert_eq!(
        resolved
            .references
            .iter()
            .filter(|reference| {
                reference.spelling == "domain.Hand"
                    && reference.namespace == SymbolNamespace::Type
                    && reference.target == Some(selected.symbol)
            })
            .count(),
        2
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            parameters,
            scheme,
            ..
        } if matches!(
            &parameters[0],
            crate::TypedParameter::Named {
                type_ref: TypedType::Named { name: parameter, arguments: parameter_arguments },
                ..
            } if parameter == "domain.Hand" && parameter_arguments.is_empty()
        ) && scheme.type_ref == TypedType::Named {
                name: "domain.Hand".to_owned(),
                arguments: Vec::new(),
            }
    ));
}

#[test]
fn resolves_a_namespace_qualified_type_nested_in_a_prelude_type() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn keep value: Maybe<domain.Hand> -> Maybe<domain.Hand> = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::Fn { scheme, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed function");
    };
    assert_eq!(
        scheme.type_ref,
        TypedType::Named {
            name: "Maybe".to_owned(),
            arguments: vec![TypedType::Named {
                name: "domain.Hand".to_owned(),
                arguments: Vec::new(),
            }],
        }
    );
}

#[test]
fn treats_named_and_namespace_qualified_imports_as_the_same_type() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import { Hand } from \"./domain\"\nimport * as domain from \"./domain\"\n\npub fn qualify value: Hand -> domain.Hand = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert!(resolved.issues.is_empty());
    let named = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "Hand")
        .unwrap();
    let qualified = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "domain.Hand")
        .unwrap();
    assert_eq!(named.symbol, qualified.symbol);

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
}

#[test]
fn types_an_imported_generic_adt_constructor_and_exhaustive_match() {
    let domain_source = "pub type Box<A> =\n  | Boxed A\n";
    let main_source = "import { Box, Boxed } from \"./domain\"\n\npub fn unwrap value: Box<Int> -> Int =\n  match value {\n    Boxed item -> item\n  }\n\npub fn answer unit: Unit -> Int = unwrap $ Boxed 42\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let [TypedDecl::Fn {
        body: crate::TypedExpr::Match { exhaustive, .. },
        ..
    }, TypedDecl::Fn { body, .. }] = analyzed.typed_hir.declarations.as_slice()
    else {
        panic!("expected generic ADT consumer functions");
    };
    assert!(*exhaustive);
    assert!(matches!(
        body,
        crate::TypedExpr::Call {
            callee,
            type_ref: TypedType::Named { name, arguments },
            ..
        } if callee == "fixture/game::main::unwrap"
            && name == "Int"
            && arguments.is_empty()
    ));
}

#[test]
fn rejects_an_incompatible_imported_generic_adt_payload() {
    let domain_source = "pub type Box<A> =\n  | Boxed A\n";
    let main_source = "import { Box, Boxed } from \"./domain\"\n\npub fn invalid unit: Unit -> Box<Int> = Boxed \"wrong\"\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "SES-T0101" && diagnostic.message_key == "call.argument-type-mismatch"
    }));
}

#[test]
fn substitutes_an_imported_generic_parameter_nested_in_a_payload_type() {
    let domain_source = "pub type Envelope<A> =\n  | Wrapped Maybe<A>\n";
    let main_source = "import { Envelope, Wrapped } from \"./domain\"\n\npub fn unwrap value: Envelope<Int> -> Maybe<Int> =\n  match value {\n    Wrapped item -> item\n  }\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::Fn {
        body: crate::TypedExpr::Match { arms, .. },
        ..
    } = &analyzed.typed_hir.declarations[0]
    else {
        panic!("expected nested generic payload match");
    };
    assert!(matches!(
        &arms[0].body,
        crate::TypedExpr::Variable {
            type_ref: TypedType::Named { name, arguments },
            ..
        } if name == "Maybe"
            && matches!(arguments.as_slice(), [TypedType::Named { name, arguments }] if name == "Int" && arguments.is_empty())
    ));
}

#[test]
fn reports_a_missing_namespace_type_member_as_an_unresolved_export() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn keep value: domain.Missing -> domain.Missing = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert!(resolved
        .imports
        .iter()
        .all(|import| import.local_name != "domain.Missing"));
    assert!(resolved.issues.iter().any(|issue| {
        issue.code == "SES-N0104" && issue.message_key == "module.export-unresolved"
    }));
    assert!(resolved
        .issues
        .iter()
        .all(|issue| issue.code != "SES-N0001"));

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-N0104"));
    assert!(diagnostics
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code != "SES-N0001"));
}

#[test]
fn reports_a_private_namespace_type_member() {
    let domain_source = "type Hidden =\n  | Secret\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn keep value: domain.Hidden -> domain.Hidden = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert!(resolved.issues.iter().any(|issue| {
        issue.code == "SES-N0102" && issue.message_key == "module.private-symbol"
    }));
    assert!(resolved
        .issues
        .iter()
        .all(|issue| issue.code != "SES-N0001"));

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-N0102"));
}
