use crate::{
    analyze_linked_module, analyze_module_interface, ExternalTypeBinding, ExternalTypeProvider,
    TypedDecl, TypedDoStatement, TypedExpr, TypedType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::{parse_unlinked_module_interface, InterfaceType};
use std::collections::BTreeMap;

#[test]
fn preserves_an_imported_struct_payload_inside_an_adt_constructor() {
    let domain_source = "pub struct Head { status: Int }\npub type Event =\n  | Started Head\n";
    let main_source = "import * as domain from \"./domain\"\n\npub fn status event: domain.Event -> Int =\n  match event {\n    domain.Started head -> head.status\n  }\n";

    analyze_one_dependency(
        main_source,
        "./domain",
        "fixture/constructor-payload::domain",
        domain_source,
    );
}

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
fn preserves_transitive_nominal_identity_inside_a_prelude_generic() {
    let model_source =
        "pub type Action =\n  | SelectReadable\n\npub struct Envelope<A> { value: A }\n";
    let model = final_target("model.ssrg", "fixture/nested::model", model_source);

    let component_source = "import { Action, Envelope } from \"./model\"\nimport * as html from \"std/web/html\"\n\npub fn view unit: Unit -> html.Html<Action> = html.text \"Readable\"\n\npub fn wrapped unit: Unit -> Envelope<html.Html<Action>> = Envelope { value: view () }\n\npub fn items unit: Unit -> Array<html.Html<Action>> = [view ()]\n\nfn task unit: Unit -> Task<Unit> = pure ()\n\nfn dispatch unit: Unit -> html.EventAction<Task<Unit>> = html.Dispatch (task ())\n";
    let component_unlinked = parse_unlinked_module_interface(
        "component.ssrg",
        "fixture/nested::component",
        component_source,
    );
    let component_linked = link_module(
        component_unlinked.clone(),
        &BTreeMap::from([
            ("./model".to_owned(), model.clone()),
            (
                "std/web/html".to_owned(),
                seseragi_project::standard_module_target("std/web/html").unwrap(),
            ),
        ]),
    )
    .unwrap();
    let component = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("component.ssrg", component_source),
        component_linked,
        component_source,
    )
    .unwrap()
    .typed_interface
    .into_link_interface();
    let component = ModuleLinkTarget::same_package(component_unlinked.header, component).unwrap();

    let main_source = "import * as model from \"./model\"\nimport { Action as LocalAction } from \"./model\"\nimport { items, view, wrapped } from \"./component\"\nimport * as html from \"std/web/html\"\n\nfn direct unit: Unit -> html.Html<model.Action> = view ()\nfn aliased unit: Unit -> html.Html<LocalAction> = view ()\nfn userGeneric unit: Unit -> model.Envelope<html.Html<model.Action>> = wrapped ()\npub fn nested unit: Unit -> Array<html.Html<model.Action>> = items ()\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/nested::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([
            ("./model".to_owned(), model),
            ("./component".to_owned(), component),
            (
                "std/web/html".to_owned(),
                seseragi_project::standard_module_target("std/web/html").unwrap(),
            ),
        ]),
    )
    .unwrap();

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();

    let TypedDecl::Fn { body, .. } = &analyzed.typed_hir.declarations[3] else {
        panic!("expected typed function");
    };
    assert!(matches!(
        body,
        TypedExpr::Call {
            type_ref: TypedType::Named { name, arguments },
            ..
        } if name == "Array"
            && matches!(
                arguments.as_slice(),
                [TypedType::ExternalNamed {
                    canonical,
                    arguments,
                    ..
                }] if canonical == "std/web/html::Html"
                    && matches!(
                        arguments.as_slice(),
                        [TypedType::ExternalNamed { canonical, .. }]
                            if canonical == "fixture/nested::model::Action"
                    )
            )
    ));
}

#[test]
fn preserves_imported_generic_identity_in_public_struct_fields() {
    let domain_source = "pub struct Box<A> { value: A }\n";
    let domain = final_target(
        "domain.ssrg",
        "fixture/struct-fields::domain",
        domain_source,
    );

    let context_source = "import { Box } from \"./domain\"\nimport * as signals from \"std/signal\"\n\npub struct AppContext {\n  count: signals.Signal<Int>,\n  boxes: Array<Box<String>>,\n  optional: Maybe<Box<String>>,\n}\n";
    let context_unlinked = parse_unlinked_module_interface(
        "context.ssrg",
        "fixture/struct-fields::context",
        context_source,
    );
    let context_linked = link_module(
        context_unlinked.clone(),
        &BTreeMap::from([
            ("./domain".to_owned(), domain.clone()),
            (
                "std/signal".to_owned(),
                seseragi_project::standard_module_target("std/signal").unwrap(),
            ),
        ]),
    )
    .unwrap();
    let analyzed_context = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("context.ssrg", context_source),
        context_linked,
        context_source,
    )
    .unwrap();
    let context_export = analyzed_context
        .typed_interface
        .exports
        .iter()
        .find(|export| export.name == "AppContext")
        .expect("missing AppContext export");
    assert!(matches!(
        context_export.representation.as_ref(),
        Some(InterfaceType::Record { fields, .. })
            if matches!(
                fields.as_slice(),
                [
                    seseragi_syntax::InterfaceRecordField {
                        type_ref: InterfaceType::ExternalNamed { canonical: count, .. },
                        ..
                    },
                    seseragi_syntax::InterfaceRecordField {
                        type_ref: InterfaceType::Named { name: boxes, arguments: box_arguments },
                        ..
                    },
                    seseragi_syntax::InterfaceRecordField {
                        type_ref: InterfaceType::Named { name: optional, arguments: optional_arguments },
                        ..
                    }
                ] if count == "std/signal::Signal"
                    && boxes == "Array"
                    && matches!(
                        box_arguments.as_slice(),
                        [InterfaceType::ExternalNamed { canonical, .. }]
                            if canonical == "fixture/struct-fields::domain::Box"
                    )
                    && optional == "Maybe"
                    && matches!(
                        optional_arguments.as_slice(),
                        [InterfaceType::ExternalNamed { canonical, .. }]
                            if canonical == "fixture/struct-fields::domain::Box"
                    )
            )
    ));
    let context = analyzed_context.typed_interface.into_link_interface();
    let context = ModuleLinkTarget::same_package(context_unlinked.header, context).unwrap();

    let main_source = "import { AppContext } from \"./context\"\nimport { Box } from \"./domain\"\nimport * as domain from \"./domain\"\nimport * as signals from \"std/signal\"\n\nfn renderCount count: signals.Signal<Int> -> Unit = ()\nfn renderBoxes boxes: Array<Box<String>> -> Unit = ()\nfn renderQualified boxes: Array<domain.Box<String>> -> Unit = ()\nfn renderOptional value: Maybe<Box<String>> -> Unit = ()\n\npub fn useCount context: AppContext -> Unit = renderCount context.count\npub fn useBoxes context: AppContext -> Unit = renderBoxes context.boxes\npub fn useQualified context: AppContext -> Unit = renderQualified context.boxes\npub fn useOptional context: AppContext -> Unit = renderOptional context.optional\n";
    let main =
        parse_unlinked_module_interface("main.ssrg", "fixture/struct-fields::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([
            ("./context".to_owned(), context),
            ("./domain".to_owned(), domain),
            (
                "std/signal".to_owned(),
                seseragi_project::standard_module_target("std/signal").unwrap(),
            ),
        ]),
    )
    .unwrap();

    analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
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
            pattern: crate::TypedPattern::Binding { type_ref, .. },
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
fn validates_explicit_imported_parameterized_failure_identity() {
    let dependency_source = "pub type RemoteError<A> =\n  | Remote A\n\npub effect fn reject -> Never\nfails RemoteError<String> =\n  fail (Remote \"bad\")\n";
    let accepted_source = "import { RemoteError, reject } from \"./dependency\"\n\npub effect fn main -> Never\nfails RemoteError<String> =\n  reject ()\n";
    let accepted = analyze_one_dependency(
        accepted_source,
        "./dependency",
        "fixture/scheme::dependency",
        dependency_source,
    );
    let TypedDecl::EffectFn {
        effect,
        body: TypedExpr::EffectInvoke {
            effect: body_effect,
            ..
        },
        ..
    } = &accepted.typed_hir.declarations[0]
    else {
        panic!("expected imported effect invocation");
    };
    assert!(matches!(
        &effect.failure,
        TypedType::Named { name, arguments }
            if name == "RemoteError"
                && arguments == &[TypedType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }]
    ));
    assert!(matches!(
        &body_effect.failure,
        TypedType::ExternalNamed { canonical, arguments, .. }
            if canonical == "fixture/scheme::dependency::RemoteError"
                && arguments == &[TypedType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }]
    ));
    assert_binding(
        &accepted.typed_hir.external_type_bindings,
        "RemoteError",
        "fixture/scheme::dependency::RemoteError",
        "fixture/scheme::dependency",
        "RemoteError",
    );
    assert!(matches!(
        &accepted.typed_interface.exports[0].scheme.type_ref,
        InterfaceType::Function { result, .. }
            if matches!(
                result.as_ref(),
                InterfaceType::Named { name, arguments }
                    if name == "Effect"
                        && matches!(
                            arguments.get(1),
                            Some(InterfaceType::ExternalNamed { canonical, arguments, .. })
                                if canonical == "fixture/scheme::dependency::RemoteError"
                                    && matches!(
                                        arguments.as_slice(),
                                        [InterfaceType::Named { name, arguments }]
                                            if name == "String" && arguments.is_empty()
                                    )
                        )
            )
    ));

    let rejected_source = "import { RemoteError, reject } from \"./dependency\"\n\npub effect fn main -> Never\nfails RemoteError<Int> =\n  reject ()\n";
    let main =
        parse_unlinked_module_interface("main.ssrg", "fixture/scheme::main", rejected_source);
    let linked = link_module(
        main,
        &BTreeMap::from([(
            "./dependency".to_owned(),
            final_target(
                "dependency.ssrg",
                "fixture/scheme::dependency",
                dependency_source,
            ),
        )]),
    )
    .unwrap();
    let diagnostics = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", rejected_source),
        linked,
        rejected_source,
    )
    .unwrap_err();
    assert_eq!(diagnostics.diagnostics.len(), 1);
    assert_eq!(
        diagnostics.diagnostics[0].message_key,
        "effect.explicit-failure-mismatch"
    );
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
