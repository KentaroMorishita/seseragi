use crate::{
    analyze_linked_module, analyze_module_interface, resolve_linked_module, TypedDecl, TypedExpr,
    TypedType,
};
use seseragi_project::{link_module, ModuleLinkTarget};
use seseragi_syntax::{parse_unlinked_module_interface, InterfaceType};
use std::collections::BTreeMap;

#[test]
fn transports_provider_show_and_effect_failure_through_a_facade() {
    let provider_source = "pub type InputError deriving Show =\n  | InvalidInput String\n\npub effect fn reject input: String =\n  fail (InvalidInput input)\n";
    let provider = final_target(
        "provider.ssrg",
        "fixture/closure::provider",
        provider_source,
        BTreeMap::new(),
    );
    let facade_source = "import { reject } from \"./provider\"\n\npub effect fn rejectViaFacade input: String =\n  reject input\n";
    let facade = analyzed_target(
        "facade.ssrg",
        "fixture/closure::facade",
        facade_source,
        BTreeMap::from([("./provider".to_owned(), provider)]),
    );

    assert!(!facade
        .interface()
        .exports
        .iter()
        .any(|export| export.namespace == "type" && export.name == "InputError"));
    assert_external_effect_failure(
        facade.interface(),
        "rejectViaFacade",
        "fixture/closure::provider::InputError",
        "fixture/closure::provider",
    );
    assert_eq!(facade.interface().instances.len(), 1);
    assert_eq!(
        facade.interface().instances[0].provider_module.as_deref(),
        Some("fixture/closure::provider")
    );
    assert_eq!(
        facade.interface().instances[0].type_identity.as_deref(),
        Some("fixture/closure::provider::InputError")
    );

    let main_source = "import { rejectViaFacade } from \"./facade\"\n\npub effect fn main =\n  rejectViaFacade \"lizard\"\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/closure::main", main_source);
    let linked = link_module(main, &BTreeMap::from([("./facade".to_owned(), facade)])).unwrap();
    let resolved = resolve_linked_module(linked.clone(), main_source);
    assert_eq!(resolved.dependency_instances.len(), 1);
    assert_eq!(
        resolved.dependency_instances[0].provider_module,
        "fixture/closure::provider"
    );
    assert_eq!(
        resolved.dependency_instances[0].type_identity.as_deref(),
        Some("fixture/closure::provider::InputError")
    );

    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::EffectFn {
            body: TypedExpr::EffectInvoke { effect, .. },
            effect: declared,
            ..
        } if effect.failure == external_input_error()
            && declared.failure == external_input_error()
    ));
    assert_eq!(analyzed.typed_hir.module_dependencies.len(), 1);
    assert_eq!(
        analyzed.typed_hir.module_dependencies[0].module,
        "fixture/closure::facade"
    );
    assert_external_effect_failure(
        &analyzed.typed_interface.clone().into_link_interface(),
        "main",
        "fixture/closure::provider::InputError",
        "fixture/closure::provider",
    );
    assert_eq!(analyzed.typed_interface.instances.len(), 1);
    assert_eq!(
        analyzed.typed_interface.instances[0]
            .provider_module
            .as_deref(),
        Some("fixture/closure::provider")
    );
}

#[test]
fn deduplicates_provider_show_reached_through_two_facades() {
    let provider_source = "pub type InputError deriving Show =\n  | InvalidInput String\n\npub effect fn reject input: String =\n  fail (InvalidInput input)\n";
    let provider = final_target(
        "provider.ssrg",
        "fixture/diamond::provider",
        provider_source,
        BTreeMap::new(),
    );
    let left = analyzed_target(
        "left.ssrg",
        "fixture/diamond::left",
        "import { reject } from \"./provider\"\n\npub effect fn rejectLeft input: String = reject input\n",
        BTreeMap::from([("./provider".to_owned(), provider.clone())]),
    );
    let right = analyzed_target(
        "right.ssrg",
        "fixture/diamond::right",
        "import { reject } from \"./provider\"\n\npub effect fn rejectRight input: String = reject input\n",
        BTreeMap::from([("./provider".to_owned(), provider)]),
    );
    let main_source = "import { rejectLeft } from \"./left\"\nimport { rejectRight } from \"./right\"\n\npub effect fn main =\n  do {\n    rejectLeft \"left\"\n    rejectRight \"right\"\n    succeed ()\n  }\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/diamond::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([("./left".to_owned(), left), ("./right".to_owned(), right)]),
    )
    .unwrap();
    let resolved = resolve_linked_module(linked.clone(), main_source);

    assert_eq!(resolved.dependency_instances.len(), 1);
    assert_eq!(
        resolved.dependency_instances[0].identity,
        "Show<fixture/diamond::provider::InputError>"
    );
    assert!(analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .is_ok());
}

#[test]
fn keeps_same_spelling_transitive_instances_separate_by_canonical_owner() {
    let first = final_target(
        "first-provider.ssrg",
        "fixture/owners::first-provider",
        "pub type InputError deriving Show = | FirstError\npub effect fn rejectFirst = fail FirstError\n",
        BTreeMap::new(),
    );
    let second = final_target(
        "second-provider.ssrg",
        "fixture/owners::second-provider",
        "pub type InputError deriving Show = | SecondError\npub effect fn rejectSecond = fail SecondError\n",
        BTreeMap::new(),
    );
    let first_facade = analyzed_target(
        "first-facade.ssrg",
        "fixture/owners::first-facade",
        "import { rejectFirst } from \"./provider\"\npub effect fn first = rejectFirst ()\n",
        BTreeMap::from([("./provider".to_owned(), first)]),
    );
    let second_facade = analyzed_target(
        "second-facade.ssrg",
        "fixture/owners::second-facade",
        "import { rejectSecond } from \"./provider\"\npub effect fn second = rejectSecond ()\n",
        BTreeMap::from([("./provider".to_owned(), second)]),
    );
    let main_source = "import { first } from \"./first\"\nimport { second } from \"./second\"\n\npub effect fn main = second ()\n";
    let main = parse_unlinked_module_interface("main.ssrg", "fixture/owners::main", main_source);
    let linked = link_module(
        main,
        &BTreeMap::from([
            ("./first".to_owned(), first_facade),
            ("./second".to_owned(), second_facade),
        ]),
    )
    .unwrap();
    let resolved = resolve_linked_module(linked.clone(), main_source);
    let identities = resolved
        .dependency_instances
        .iter()
        .map(|instance| instance.identity.as_str())
        .collect::<Vec<_>>();

    assert_eq!(identities.len(), 2);
    assert!(identities.contains(&"Show<fixture/owners::first-provider::InputError>"));
    assert!(identities.contains(&"Show<fixture/owners::second-provider::InputError>"));
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics("main.ssrg", main_source),
        linked,
        main_source,
    )
    .unwrap();
    assert!(matches!(
        &analyzed.typed_hir.declarations[0],
        TypedDecl::EffectFn {
            effect,
            ..
        } if effect.failure == TypedType::ExternalNamed {
            name: "InputError".to_owned(),
            canonical: "fixture/owners::second-provider::InputError".to_owned(),
            arguments: Vec::new(),
        }
    ));
}

fn analyzed_target(
    source_name: &str,
    module: &str,
    source: &str,
    dependencies: BTreeMap<String, ModuleLinkTarget>,
) -> ModuleLinkTarget {
    let unlinked = parse_unlinked_module_interface(source_name, module, source);
    let analyzed = analyze_linked_module(
        seseragi_syntax::parse_diagnostics(source_name, source),
        link_module(unlinked.clone(), &dependencies).unwrap(),
        source,
    )
    .unwrap();
    ModuleLinkTarget::same_package(
        unlinked.header,
        analyzed.typed_interface.into_link_interface(),
    )
    .unwrap()
}

fn final_target(
    source_name: &str,
    module: &str,
    source: &str,
    dependencies: BTreeMap<String, ModuleLinkTarget>,
) -> ModuleLinkTarget {
    if !dependencies.is_empty() {
        return analyzed_target(source_name, module, source, dependencies);
    }
    let unlinked = parse_unlinked_module_interface(source_name, module, source);
    let analyzed = analyze_module_interface(
        seseragi_syntax::parse_diagnostics(source_name, source),
        unlinked.interface.clone(),
        source,
    )
    .unwrap();
    ModuleLinkTarget::same_package(
        unlinked.header,
        analyzed.typed_interface.into_link_interface(),
    )
    .unwrap()
}

fn assert_external_effect_failure(
    interface: &seseragi_syntax::ModuleInterface,
    export_name: &str,
    canonical: &str,
    provider_module: &str,
) {
    let export = interface
        .exports
        .iter()
        .find(|export| export.name == export_name)
        .expect("effect export");
    let mut cursor = &export.scheme.type_ref;
    while let InterfaceType::Function { result, .. } = cursor {
        cursor = result;
    }
    assert!(matches!(
        cursor,
        InterfaceType::Named { name, arguments }
            if name == "Effect"
                && matches!(
                    arguments.as_slice(),
                    [_, InterfaceType::ExternalNamed {
                        canonical: actual,
                        provider_module: actual_provider,
                        ..
                    }, _] if actual == canonical && actual_provider == provider_module
                )
    ));
}

fn external_input_error() -> TypedType {
    TypedType::ExternalNamed {
        name: "InputError".to_owned(),
        canonical: "fixture/closure::provider::InputError".to_owned(),
        arguments: Vec::new(),
    }
}
