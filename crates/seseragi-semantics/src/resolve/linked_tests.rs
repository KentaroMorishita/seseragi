use crate::{
    analyze_linked_module, analyze_module_interface, resolve_linked_module, SymbolKind,
    SymbolNamespace, TypedDecl, TypedExpr, TypedType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

mod imported_effect_invocations;
mod imported_instance_evidence;
mod imported_scheme_nominals;
mod namespace_constructors;
mod namespace_types;

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
    assert_eq!(analyzed.typed_hir.module_dependencies.len(), 1);
    let dependency = &analyzed.typed_hir.module_dependencies[0];
    assert_eq!(dependency.specifier, "./domain");
    assert_eq!(dependency.module, "fixture/game::domain");
    assert_eq!(dependency.imports.len(), 1);
    assert_eq!(dependency.imports[0].local, "next");
    assert_eq!(
        dependency.imports[0].canonical,
        "fixture/game::domain::increment"
    );
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
fn resolves_a_namespace_qualified_pure_function_call() {
    let main_source = "import * as domain from \"./domain\"\npub fn run value: Int -> Int = domain.increment value\n";
    let linked = linked_function_program(main_source);

    let resolved = resolve_linked_module(linked, main_source);

    assert!(resolved.issues.is_empty());
    assert_eq!(resolved.imports.len(), 1);
    let imported = &resolved.imports[0];
    assert_eq!(imported.local_name, "domain.increment");
    assert_eq!(imported.export.name, "increment");
    assert_eq!(imported.export.symbol, "fixture/game::domain::increment");
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "domain.increment"
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
fn types_a_namespace_qualified_pure_function_call() {
    let main_source = "import * as domain from \"./domain\"\npub fn run value: Int -> Int = domain.increment value\n";
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
    let dependency = &analyzed.typed_hir.module_dependencies[0];
    assert_eq!(dependency.imports.len(), 1);
    assert_eq!(dependency.imports[0].imported, "increment");
    assert_eq!(dependency.imports[0].local, "domain.increment");
    assert_eq!(
        dependency.imports[0].canonical,
        "fixture/game::domain::increment"
    );
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
fn reports_a_missing_namespace_value_member_as_unresolved() {
    let main_source = "import * as domain from \"./domain\"\npub fn run value: Int -> Int = domain.missing value\n";
    let linked = linked_function_program(main_source);

    let resolved = resolve_linked_module(linked.clone(), main_source);

    assert!(resolved.imports.is_empty());
    assert!(resolved.issues.iter().any(|issue| {
        issue.code == "SES-N0104" && issue.message_key == "module.export-unresolved"
    }));
    assert!(resolved
        .issues
        .iter()
        .all(|issue| issue.code != "SES-N0001"));
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "domain.missing"
            && reference.namespace == SymbolNamespace::Value
            && reference.target.is_none()
    }));
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
}

#[test]
fn reports_a_private_namespace_value_member() {
    let domain_source = "fn hidden value: Int -> Int = value\n";
    let main_source = "import * as domain from \"./domain\"\npub fn run value: Int -> Int = domain.hidden value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);

    assert!(resolved.imports.is_empty());
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
fn instantiates_an_imported_generic_function_independently_per_call() {
    let domain_source = "pub fn identity<A> value: A -> A = value\n";
    let main_source = "import { identity } from \"./domain\"\n\npub fn keepInt value: Int -> Int = identity value\npub fn keepString value: String -> String = identity value\n";
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

    let [TypedDecl::Fn { body: int_body, .. }, TypedDecl::Fn {
        body: string_body, ..
    }] = analyzed.typed_hir.declarations.as_slice()
    else {
        panic!("expected two imported generic callers");
    };
    assert!(matches!(
        int_body,
        TypedExpr::Call {
            callee,
            type_ref: TypedType::Named { name, arguments },
            ..
        } if callee == "fixture/game::domain::identity" && name == "Int" && arguments.is_empty()
    ));
    assert!(matches!(
        string_body,
        TypedExpr::Call {
            callee,
            type_ref: TypedType::Named { name, arguments },
            ..
        } if callee == "fixture/game::domain::identity" && name == "String" && arguments.is_empty()
    ));
}

#[test]
fn instantiates_an_imported_generic_function_with_a_user_defined_adt() {
    let domain_source = "pub type Hand =\n  | Rock\n\npub fn identity<A> value: A -> A = value\n";
    let main_source = "import { Hand, Rock, identity } from \"./domain\"\n\npub fn keep unit: Unit -> Hand = identity Rock\n";
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

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Call {
                callee,
                type_ref: TypedType::Named { name, arguments },
                ..
            },
            ..
        } if callee == "fixture/game::domain::identity" && name == "Hand" && arguments.is_empty()
    ));
}

#[test]
fn reports_inconsistent_imported_generic_arguments_as_a_type_mismatch() {
    let domain_source = "pub fn same<A> first: A -> second: A -> A = first\n";
    let main_source = "import { same } from \"./domain\"\n\npub fn invalid unit: Unit -> Int = same 1 \"wrong\"\n";
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

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
    assert_eq!(
        diagnostics.diagnostics[0].related[0].message,
        "argument 2 expected Int, received String"
    );
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
            if *type_ref == TypedType::ExternalNamed {
                name: "LocalHand".to_owned(),
                canonical: "fixture/game::domain::Hand".to_owned(),
                arguments: Vec::new(),
            }
    ));
}

#[test]
fn treats_two_named_aliases_of_one_imported_adt_as_the_same_nominal_type() {
    let domain_source = "pub type User =\n  | Alice\n";
    let main_source = "import { User as First, User as Second } from \"./domain\"\n\npub fn relay value: First -> Second = value\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert!(resolved.issues.is_empty());
    let first = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "First")
        .unwrap();
    let second = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "Second")
        .unwrap();
    assert_eq!(first.symbol, second.symbol);
    assert_eq!(first.export.symbol, "fixture/game::domain::User");
    assert_eq!(second.export.symbol, "fixture/game::domain::User");

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    let imports = &analyzed.typed_hir.module_dependencies[0].imports;
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].local, "First");
    assert_eq!(imports[1].local, "Second");
    assert_eq!(imports[0].canonical, imports[1].canonical);
}

#[test]
fn namespace_selected_constructor_reuses_the_hidden_dependency_symbol() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let main_source =
        "import * as domain from \"./domain\"\n\npub fn choose unit: Unit -> Unit = domain.Rock\n";
    let linked = linked_program(
        main_source,
        [("./domain", "fixture/game::domain", domain_source)],
    );

    let resolved = resolve_linked_module(linked, main_source);

    assert!(resolved.issues.is_empty());
    let selected = resolved
        .imports
        .iter()
        .find(|import| import.local_name == "domain.Rock")
        .unwrap();
    assert!(selected.in_scope);
    assert_eq!(selected.export.symbol, "fixture/game::domain::Rock");
    assert_eq!(
        resolved
            .symbols
            .iter()
            .filter(|symbol| { symbol.canonical.as_deref() == Some("fixture/game::domain::Rock") })
            .count(),
        1
    );
    assert!(!resolved.imports.iter().any(|import| {
        !import.in_scope && import.export.symbol == "fixture/game::domain::Rock"
    }));
    assert!(resolved.references.iter().any(|reference| {
        reference.spelling == "domain.Rock" && reference.target == Some(selected.symbol)
    }));
}

#[test]
fn accepts_an_imported_constructor_for_a_function_from_the_same_adt_owner() {
    let domain_source = "pub type User =\n  | Alice\n\npub fn accept user: User -> Unit = ()\n";
    let main_source = "import { Alice, accept } from \"./domain\"\n\npub fn run unit: Unit -> Unit = accept Alice\n";
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

    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Call { type_ref, .. },
            ..
        } if *type_ref == TypedType::Named {
            name: "Unit".to_owned(),
            arguments: Vec::new(),
        }
    ));
}

#[test]
fn rejects_same_spelling_adts_from_different_dependency_owners() {
    let first_source = "pub type User =\n  | Alice\n";
    let second_source = "pub type User =\n  | Bob\n\npub fn accept user: User -> Unit = ()\n";
    let main_source = "import { Alice } from \"./first\"\nimport { accept } from \"./second\"\n\npub fn run unit: Unit -> Unit = accept Alice\n";
    let linked = linked_program(
        main_source,
        [
            ("./first", "fixture/game::first", first_source),
            ("./second", "fixture/game::second", second_source),
        ],
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
fn preserves_a_namespace_only_dependency_edge_without_inventing_named_bindings() {
    let domain_source = "pub let answer: Int = 42\n";
    let main_source =
        "import * as domain from \"./domain\"\n\npub fn run unit: Unit -> Unit = ()\n";
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

    assert_eq!(analyzed.typed_hir.module_dependencies.len(), 1);
    assert_eq!(
        analyzed.typed_hir.module_dependencies[0].module,
        "fixture/game::domain"
    );
    assert!(analyzed.typed_hir.module_dependencies[0].imports.is_empty());
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

fn linked_program<const N: usize>(
    main_source: &str,
    dependencies: [(&str, &str, &str); N],
) -> seseragi_project::LinkedModule {
    let targets = dependencies
        .into_iter()
        .map(|(specifier, module, source)| {
            let unlinked = parse_unlinked_module_interface("dependency.ssrg", module, source);
            let interface = analyze_module_interface(
                seseragi_syntax::parse_diagnostics("dependency.ssrg", source),
                unlinked.interface.clone(),
                source,
            )
            .unwrap()
            .typed_interface
            .into_link_interface();
            (
                specifier.to_owned(),
                ModuleLinkTarget::same_package(unlinked.header, interface).unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/game::main", main_source);
    link_module(main, &targets).unwrap()
}
