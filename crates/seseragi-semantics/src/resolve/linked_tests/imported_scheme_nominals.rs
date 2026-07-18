use crate::{
    analyze_linked_module, analyze_module_interface, ExternalTypeBinding, ExternalTypeProvider,
    TypedDecl, TypedDoStatement, TypedExpr, TypedType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::parse_unlinked_module_interface;
use std::collections::BTreeMap;

#[test]
fn preserves_direct_provider_types_from_an_imported_pure_scheme() {
    let domain_source = "pub type Hand =\n  | Rock\n  | Paper\n\npub type Outcome =\n  | Draw\n\npub fn decide first: Hand -> second: Hand -> Outcome = Draw\npub fn render outcome: Outcome -> String = \"draw\"\n";
    let main_source = "import { Rock, Paper, decide, render } from \"./domain\"\n\npub fn run unit: Unit -> String = render (decide Rock Paper)\n";
    let analyzed = analyze_one_dependency(
        main_source,
        "./domain",
        "fixture/scheme::domain",
        domain_source,
    );

    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed function");
    };
    let TypedExpr::Call { arguments, .. } = body else {
        panic!("expected render call");
    };
    assert!(matches!(
        arguments.as_slice(),
        [TypedExpr::Call {
            type_ref: TypedType::ExternalNamed {
                name,
                canonical,
                arguments,
            },
            ..
        }] if name == "Outcome"
            && canonical == "fixture/scheme::domain::Outcome"
            && arguments.is_empty()
    ));
    assert_binding(
        &analyzed.typed_hir.external_type_bindings,
        "Hand",
        "fixture/scheme::domain::Hand",
        "fixture/scheme::domain",
        "Hand",
    );
    assert_binding(
        &analyzed.typed_hir.external_type_bindings,
        "Outcome",
        "fixture/scheme::domain::Outcome",
        "fixture/scheme::domain",
        "Outcome",
    );
}

#[test]
fn preserves_transitive_provider_for_an_imported_effect_success() {
    let domain_source = "pub type Hand =\n  | Rock\n";
    let domain = final_target("domain.ssrg", "fixture/scheme::domain", domain_source);

    let facade_source = "import { Hand, Rock } from \"./domain\"\n\npub effect fn readHand -> Hand = succeed Rock\n";
    let facade_unlinked =
        parse_unlinked_module_interface("facade.ssrg", "fixture/scheme::facade", facade_source);
    let facade_linked = link_module(
        facade_unlinked.clone(),
        &BTreeMap::from([("./domain".to_owned(), domain)]),
    )
    .unwrap();
    let facade = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("facade.ssrg", facade_source),
        facade_linked,
        facade_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let facade = ModuleLinkTarget::same_package(facade_unlinked.header, facade).unwrap();

    let main_source = "import { readHand } from \"./facade\"\n\npub effect fn main =\n  do {\n    hand <- readHand ()\n    succeed hand\n  }\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/scheme::main", main_source);
    let linked = link_module(main, &BTreeMap::from([("./facade".to_owned(), facade)])).unwrap();
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::EffectFn { body, effect, .. } = &analyzed.typed_hir.declarations[0] else {
        panic!("expected typed effect function");
    };
    assert_eq!(effect.success, external_hand());
    let TypedExpr::DoBlock {
        statements, result, ..
    } = body
    else {
        panic!("expected typed do block");
    };
    assert!(matches!(
        statements.as_slice(),
        [TypedDoStatement::Bind {
            type_ref,
            value: TypedExpr::EffectInvoke { effect, .. },
            ..
        }] if type_ref == &external_hand() && effect.success == external_hand()
    ));
    assert!(matches!(
        result.as_ref(),
        TypedExpr::EffectCall {
            effect,
            arguments,
            ..
        } if effect.success == external_hand()
            && matches!(
                arguments.as_slice(),
                [TypedExpr::Variable { type_ref, .. }] if type_ref == &external_hand()
            )
    ));
    assert_binding(
        &analyzed.typed_hir.external_type_bindings,
        "Hand",
        "fixture/scheme::domain::Hand",
        "fixture/scheme::domain",
        "Hand",
    );
    assert!(!analyzed
        .typed_hir
        .module_dependencies
        .iter()
        .any(|dependency| { dependency.module == "fixture/scheme::domain" }));
}

#[test]
fn accepts_an_explicitly_imported_failure_from_an_imported_effect_scheme() {
    let input_source = "pub type InputError =\n  | InvalidInput\n\npub effect fn readHand -> Unit\nfails InputError =\n  fail InvalidInput\n";
    let main_source = "import { InputError, readHand } from \"./input\"\n\npub type AppError =\n  | InputFailure InputError\n\npub effect fn main =\n  do {\n    unit <- mapError InputFailure (readHand ())\n    succeed unit\n  }\n";
    let analyzed = analyze_one_dependency(
        main_source,
        "./input",
        "fixture/scheme::input",
        input_source,
    );

    assert_binding(
        &analyzed.typed_hir.external_type_bindings,
        "InputError",
        "fixture/scheme::input::InputError",
        "fixture/scheme::input",
        "InputError",
    );
    assert!(matches!(
        &analyzed.typed_hir.declarations[1],
        TypedDecl::EffectFn {
            body: TypedExpr::DoBlock { statements, .. },
            ..
        } if matches!(statements.as_slice(), [TypedDoStatement::Bind { .. }])
    ));
}

#[test]
fn rejects_same_spelling_mapper_and_effect_failures_from_distinct_owners() {
    let first_source = "pub type InputError =\n  | FirstInputError\n";
    let second_source = "pub type InputError =\n  | SecondInputError\n\npub effect fn readHand -> Unit\nfails InputError =\n  fail SecondInputError\n";
    let main_source = "import { InputError } from \"./first\"\nimport { readHand } from \"./second\"\n\npub type AppError =\n  | InputFailure InputError\n\npub effect fn main =\n  do {\n    unit <- mapError InputFailure (readHand ())\n    succeed unit\n  }\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/scheme::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([
            (
                "./first".to_owned(),
                final_target("first.ssrg", "fixture/scheme::first", first_source),
            ),
            (
                "./second".to_owned(),
                final_target("second.ssrg", "fixture/scheme::second", second_source),
            ),
        ]),
    )
    .unwrap();
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap_err();

    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "effect.map-error-failure-mismatch"
    );
}

#[test]
fn keeps_same_spelling_scheme_types_distinct_by_canonical_owner() {
    let first_source = "pub type User =\n  | FirstUser\n\npub fn makeFirst unit: Unit -> User = FirstUser\npub fn acceptFirst user: User -> Unit = ()\n";
    let second_source = "pub type User =\n  | SecondUser\n\npub fn makeSecond unit: Unit -> User = SecondUser\npub fn acceptSecond user: User -> Unit = ()\n";
    let main_source = "import { makeFirst, acceptFirst } from \"./first\"\nimport { makeSecond, acceptSecond } from \"./second\"\n\npub fn first unit: Unit -> Unit = acceptFirst (makeFirst ())\npub fn second unit: Unit -> Unit = acceptSecond (makeSecond ())\n";
    let targets = BTreeMap::from([
        (
            "./first".to_owned(),
            final_target("first.ssrg", "fixture/scheme::first", first_source),
        ),
        (
            "./second".to_owned(),
            final_target("second.ssrg", "fixture/scheme::second", second_source),
        ),
    ]);
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/scheme::main", main_source);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        link_module(main, &targets).unwrap(),
        main_source,
    )
    .unwrap();

    let user_bindings = analyzed
        .typed_hir
        .external_type_bindings
        .iter()
        .filter(|binding| binding.spelling == "User")
        .collect::<Vec<_>>();
    assert_eq!(user_bindings.len(), 2);
    assert_ne!(user_bindings[0].canonical, user_bindings[1].canonical);
    for (declaration, canonical) in analyzed.typed_hir.declarations.iter().zip([
        "fixture/scheme::first::User",
        "fixture/scheme::second::User",
    ]) {
        let TypedDecl::Fn {
            body: TypedExpr::Call { arguments, .. },
            ..
        } = declaration
        else {
            panic!("expected typed function call");
        };
        assert!(matches!(
            arguments.as_slice(),
            [TypedExpr::Call {
                type_ref: TypedType::ExternalNamed {
                    canonical: actual,
                    ..
                },
                ..
            }] if actual == canonical
        ));
    }
}

#[test]
fn preserves_nominal_identity_from_a_namespace_selected_scheme() {
    let domain_source = "pub type Hand =\n  | Rock\n\npub fn make unit: Unit -> Hand = Rock\npub fn accept hand: Hand -> Unit = ()\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn run unit: Unit -> Unit = domain.accept (domain.make ())\n";
    let analyzed = analyze_one_dependency(
        main_source,
        "./domain",
        "fixture/scheme::domain",
        domain_source,
    );

    assert_binding(
        &analyzed.typed_hir.external_type_bindings,
        "Hand",
        "fixture/scheme::domain::Hand",
        "fixture/scheme::domain",
        "Hand",
    );
    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::Fn {
            body: TypedExpr::Call { arguments, .. },
            ..
        } if matches!(
            arguments.as_slice(),
            [TypedExpr::Call {
                type_ref: TypedType::ExternalNamed { canonical, .. },
                ..
            }] if canonical == "fixture/scheme::domain::Hand"
        )
    ));
}

#[test]
fn rejects_imported_opaque_struct_literals_before_lowering() {
    let domain_source = "pub opaque struct Secret { value: Int }\n\npub fn secret value: Int -> Secret = Secret { value }\n";
    let target = final_target(
        "domain.ssrg",
        "fixture/opaque-struct::domain",
        domain_source,
    );

    for literal in ["Secret {}", "Secret { value: 42 }"] {
        let main_source = format!(
            "import {{ Secret }} from \"./domain\"\n\npub fn forge unit: Unit -> Secret = {literal}\n"
        );
        let main = parse_unlinked_module_interface(
            "main.ssrg",
            "fixture/opaque-struct::main",
            &main_source,
        );
        let linked = link_module(
            main,
            &BTreeMap::from([("./domain".to_owned(), target.clone())]),
        )
        .unwrap();
        let diagnostics = analyze_linked_module(
            seseragi_syntax::parse_diagnostics("main.ssrg", &main_source),
            linked,
            &main_source,
        )
        .unwrap_err();

        assert_eq!(diagnostics.diagnostics.len(), 1, "{literal}");
        let diagnostic = &diagnostics.diagnostics[0];
        assert_eq!(diagnostic.code, "SES-T0101", "{literal}");
        assert_eq!(
            diagnostic.message_key, "struct.representation-private",
            "{literal}"
        );
    }
}

fn analyze_one_dependency(
    main_source: &str,
    specifier: &str,
    module: &str,
    dependency_source: &str,
) -> crate::AnalyzedModule {
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/scheme::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([(
            specifier.to_owned(),
            final_target("dependency.ssrg", module, dependency_source),
        )]),
    )
    .unwrap();
    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap()
}

fn final_target(source_name: &str, module: &str, source: &str) -> ModuleLinkTarget {
    let unlinked = parse_unlinked_module_interface(source_name, module, source);
    let interface = analyze_module_interface(
        seseragi_syntax::parse_diagnostics(source_name, source),
        unlinked.interface.clone(),
        source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    ModuleLinkTarget::same_package(unlinked.header, interface).unwrap()
}

fn assert_binding(
    bindings: &[ExternalTypeBinding],
    spelling: &str,
    canonical: &str,
    provider_module: &str,
    provider_export: &str,
) {
    assert!(bindings.iter().any(|binding| {
        binding.spelling == spelling
            && binding.canonical == canonical
            && binding.provider
                == Some(ExternalTypeProvider {
                    module: provider_module.to_owned(),
                    export: provider_export.to_owned(),
                })
    }));
}

fn external_hand() -> TypedType {
    TypedType::ExternalNamed {
        name: "Hand".to_owned(),
        canonical: "fixture/scheme::domain::Hand".to_owned(),
        arguments: Vec::new(),
    }
}
