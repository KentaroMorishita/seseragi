use crate::{
    analyze_linked_module, analyze_module_interface, resolve_linked_module, SymbolKind,
    SymbolNamespace, TypedDecl, TypedExpr, TypedType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

#[test]
fn resolves_an_imported_function_to_its_canonical_dependency_symbol() {
    let main_source =
        "import { increment as next } from \"./domain\"\npub fn run value: Int -> Int = next value\n";
    let linked = linked_function_program(main_source);

    let resolved = resolve_linked_module(linked, main_source);
    assert!(resolved.issues.is_empty());
    assert_eq!(resolved.imports.len(), 1);
    let imported = &resolved.imports[0];
    assert_eq!(imported.local_name, "next");
    assert_eq!(imported.export.symbol, "fixture/game::domain::increment");
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "next"
            && reference.namespace == SymbolNamespace::Value
            && reference.target == Some(imported.symbol)
    }));
    assert!(resolved.symbols.iter().any(|symbol| {
        symbol.id == imported.symbol
            && symbol.kind == SymbolKind::Imported
            && symbol.canonical.as_deref() == Some("fixture/game::domain::increment")
    }));
}

#[test]
fn types_a_call_from_an_imported_pure_function_scheme() {
    let main_source =
        "import { increment as next } from \"./domain\"\npub fn run value: Int -> Int = next value\n";
    let linked = linked_function_program(main_source);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call { callee, type_ref, .. }
            if callee == "fixture/game::domain::increment"
                && *type_ref == TypedType::Named {
                    name: "Int".to_owned(),
                    arguments: Vec::new(),
                }
    ));
}

#[test]
fn reports_an_imported_function_argument_type_mismatch() {
    let main_source =
        "import { increment } from \"./domain\"\npub fn run value: String -> Int = increment value\n";
    let linked = linked_function_program(main_source);

    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();
    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-T0101"));
}

#[test]
fn types_an_exhaustive_match_over_an_imported_adt_family() {
    let main_source = "import { Outcome, Player1Wins, Player2Wins, Draw } from \"./domain\"\n\npub fn render outcome: Outcome -> String =\n  match outcome {\n    Player1Wins -> \"Player 1 wins!\"\n    Player2Wins -> \"Player 2 wins!\"\n    Draw -> \"Draw!\"\n  }\n";
    let linked = linked_adt_program(main_source);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed function");
    };
    assert!(matches!(
        body,
        TypedExpr::Match {
            exhaustive: true,
            arms,
            ..
        } if arms.len() == 3
    ));
}

#[test]
fn reports_a_missing_imported_adt_constructor_arm() {
    let main_source = "import { Outcome, Player1Wins, Player2Wins } from \"./domain\"\n\npub fn render outcome: Outcome -> String =\n  match outcome {\n    Player1Wins -> \"Player 1 wins!\"\n    Player2Wins -> \"Player 2 wins!\"\n  }\n";
    let linked = linked_adt_program(main_source);
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert!(diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-T0301"));
}

#[test]
fn localizes_imported_function_schemes_to_a_type_alias_import() {
    let domain_source = "pub type Hand =\n  | Rock\n\npub fn keep hand: Hand -> Hand = hand\n";
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let domain_interface = analyze_module_interface(
        seseragi_syntax::parse_diagnostics("domain.ssrg", domain_source),
        domain.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let target = ModuleLinkTarget::same_package(domain.header, domain_interface).unwrap();
    let main_source = "import { Hand as LocalHand, Rock as LocalRock, keep } from \"./domain\"\n\npub fn round hand: LocalHand -> LocalHand = keep hand\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    let linked = link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap();

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call { type_ref, .. }
            if *type_ref == TypedType::Named {
                name: "LocalHand".to_owned(),
                arguments: Vec::new(),
            }
    ));
}

fn linked_function_program(main_source: &str) -> seseragi_project::LinkedModule {
    let domain_source = "pub fn increment value: Int -> Int = value + 1\n";
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let domain_interface = analyze_module_interface(
        seseragi_syntax::parse_diagnostics("domain.ssrg", domain_source),
        domain.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let target = ModuleLinkTarget::same_package(domain.header, domain_interface).unwrap();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap()
}

fn linked_adt_program(main_source: &str) -> seseragi_project::LinkedModule {
    let domain_source = "pub type Outcome =\n  | Player1Wins\n  | Player2Wins\n  | Draw\n";
    let domain =
        parse_unlinked_module_interface("domain.ssrg", "fixture/game::domain", domain_source);
    let domain_interface = analyze_module_interface(
        seseragi_syntax::parse_diagnostics("domain.ssrg", domain_source),
        domain.interface.clone(),
        domain_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let target = ModuleLinkTarget::same_package(domain.header, domain_interface).unwrap();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    link_module(main, &BTreeMap::from([("./domain".to_owned(), target)])).unwrap()
}
