use seseragi_syntax::{InterfaceExport, InterfaceScheme, InterfaceType, ModuleInterface};
use std::collections::BTreeSet;

pub(super) fn capture_types(
    export: &InterfaceExport,
    provider: &ModuleInterface,
) -> InterfaceExport {
    let mut export = export.clone();
    let parameters = capture_scheme(&mut export.scheme, provider, &BTreeSet::new());
    if let Some(representation) = &mut export.representation {
        capture_type(representation, provider, &parameters);
    }
    for method in &mut export.methods {
        capture_scheme(&mut method.scheme, provider, &parameters);
    }
    export
}

fn capture_scheme(
    scheme: &mut InterfaceScheme,
    provider: &ModuleInterface,
    outer_parameters: &BTreeSet<String>,
) -> BTreeSet<String> {
    let parameters = outer_parameters
        .iter()
        .cloned()
        .chain(scheme.type_parameters.iter().map(|p| p.name.clone()))
        .collect();
    capture_type(&mut scheme.type_ref, provider, &parameters);
    for constraint in &mut scheme.constraints {
        for argument in &mut constraint.arguments {
            capture_type(argument, provider, &parameters);
        }
    }
    parameters
}

fn capture_type(
    value: &mut InterfaceType,
    provider: &ModuleInterface,
    parameters: &BTreeSet<String>,
) {
    match value {
        InterfaceType::Named { name, arguments } => {
            for argument in arguments.iter_mut() {
                capture_type(argument, provider, parameters);
            }
            if parameters.contains(name) {
                return;
            }
            // A facade need not publish the types mentioned by a callable.
            // Capture their defining owners before its lexical scope is lost.
            let mut owners = provider
                .exports
                .iter()
                .filter(|export| export.namespace == "type" && export.name == *name);
            if let Some(owner) = owners.next().filter(|_| owners.next().is_none()) {
                *value = InterfaceType::ExternalNamed {
                    name: name.clone(),
                    canonical: owner.symbol.clone(),
                    provider_module: provider.module.clone(),
                    provider_export: owner.name.clone(),
                    arguments: std::mem::take(arguments),
                };
            }
        }
        InterfaceType::ExternalNamed { arguments, .. } | InterfaceType::Apply { arguments, .. } => {
            for argument in arguments {
                capture_type(argument, provider, parameters);
            }
        }
        InterfaceType::Function { parameter, result } => {
            capture_type(parameter, provider, parameters);
            capture_type(result, provider, parameters);
        }
        InterfaceType::Record { fields, .. } => {
            for field in fields {
                capture_type(&mut field.type_ref, provider, parameters);
            }
        }
        InterfaceType::Tuple { elements }
        | InterfaceType::RequirementMerge { operands: elements } => {
            for element in elements {
                capture_type(element, provider, parameters);
            }
        }
        InterfaceType::TypeFunction { body, .. } => capture_type(body, provider, parameters),
        InterfaceType::TypeConstructor { .. }
        | InterfaceType::TypeFunctionParameter { .. }
        | InterfaceType::Hole => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{link_module, standard_module_target, ModuleLinkTarget};
    use seseragi_syntax::parse_unlinked_module_interface;
    use std::collections::BTreeMap;

    #[test]
    fn named_reexports_capture_hidden_queue_types_and_keep_the_calling_scheme() {
        let provider = standard_module_target("std/queue").unwrap();
        let linked = link_module(
            parse_unlinked_module_interface(
                "api.ssrg",
                "fixture/api",
                "pub import { unbounded as create, offer } from \"std/queue\"\n",
            ),
            &BTreeMap::from([("std/queue".into(), provider.clone())]),
        )
        .unwrap();
        assert_eq!(linked.interface.exports.len(), 2);
        let create = linked
            .interface
            .exports
            .iter()
            .find(|e| e.name == "create")
            .unwrap();
        let original = provider
            .interface()
            .exports
            .iter()
            .find(|e| e.name == "unbounded")
            .unwrap();
        assert_eq!(create.symbol, "std/queue::unbounded");
        assert_eq!(
            create.scheme.type_parameters,
            original.scheme.type_parameters
        );
        assert_eq!(create.call_arity, original.call_arity);
        assert_eq!(create.declaration_kind, original.declaration_kind);
        let InterfaceType::Function { parameter, result } = &create.scheme.type_ref else {
            panic!("{create:#?}");
        };
        assert_eq!(**parameter, named("Unit"));
        let InterfaceType::Named { name, arguments } = &**result else {
            panic!("{result:#?}");
        };
        assert_eq!(name, "Effect");
        assert_eq!(arguments.len(), 3);
        assert_eq!(arguments[2], queue_type());
        let offer = linked
            .interface
            .exports
            .iter()
            .find(|e| e.name == "offer")
            .unwrap();
        let InterfaceType::Function { parameter, result } = &offer.scheme.type_ref else {
            panic!("{offer:#?}");
        };
        assert_eq!(**parameter, named("A"));
        let InterfaceType::Function { parameter, .. } = &**result else {
            panic!("{offer:#?}");
        };
        assert_eq!(**parameter, queue_type());

        let again = link_module(
            parse_unlinked_module_interface(
                "facade.ssrg",
                "fixture/facade",
                "pub import { create as again } from \"./api\"\n",
            ),
            &BTreeMap::from([(
                "./api".into(),
                ModuleLinkTarget::external(linked.interface.clone()),
            )]),
        )
        .unwrap();
        assert_eq!(again.interface.exports[0].scheme, create.scheme);
    }

    #[test]
    fn a_quantified_name_wins_over_a_provider_type_with_the_same_spelling() {
        let mut interface = standard_module_target("std/queue")
            .unwrap()
            .interface()
            .clone();
        let mut export = interface
            .exports
            .iter()
            .find(|e| e.name == "offer")
            .unwrap()
            .clone();
        export.scheme.type_parameters[0].name = "Queue".into();
        export.scheme.type_ref = InterfaceType::Function {
            parameter: Box::new(named("Queue")),
            result: Box::new(named("Queue")),
        };
        interface.exports.push(export.clone());
        assert_eq!(capture_types(&export, &interface).scheme, export.scheme);
    }

    fn named(name: &str) -> InterfaceType {
        InterfaceType::Named {
            name: name.into(),
            arguments: vec![],
        }
    }

    fn queue_type() -> InterfaceType {
        InterfaceType::ExternalNamed {
            name: "Queue".into(),
            canonical: "std/queue::Queue".into(),
            provider_module: "std/queue".into(),
            provider_export: "Queue".into(),
            arguments: vec![named("A")],
        }
    }
}
