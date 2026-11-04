use super::{LinkedDependency, ModuleLinkTarget};
use seseragi_syntax::{ByteSpan, InterfaceType, ModuleInterface, Visibility};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct TypeProvider {
    module: String,
    export: String,
    canonical: String,
}

pub(super) fn retain_struct_definitions(
    dependencies: &mut Vec<LinkedDependency>,
    targets: &BTreeMap<String, ModuleLinkTarget>,
    providers: &[ModuleInterface],
) {
    let mut pending = Vec::new();
    let mut retained = BTreeSet::new();
    for dependency in dependencies.iter() {
        for export in &dependency.interface.exports {
            if export.namespace == "type" {
                retained.insert(export.symbol.clone());
            }
            collect_providers(&export.scheme.type_ref, &mut pending);
            if let Some(representation) = &export.representation {
                collect_providers(representation, &mut pending);
            }
        }
    }
    let mut visited = BTreeSet::new();
    while let Some(provider) = pending.pop() {
        if !visited.insert(provider.clone()) || retained.contains(&provider.canonical) {
            continue;
        }
        let interface = providers
            .iter()
            .chain(targets.values().map(ModuleLinkTarget::interface))
            .find(|interface| interface.module == provider.module)
            .cloned()
            .or_else(|| {
                crate::standard_module_target(&provider.module)
                    .map(|target| target.interface().clone())
            });
        let Some(mut interface) = interface else {
            continue;
        };
        let Some(export) = interface.exports.iter().find(|export| {
            export.namespace == "type"
                && export.name == provider.export
                && export.symbol == provider.canonical
                && export.visibility == Visibility::Public
                && export.declaration_kind.as_deref() == Some("struct")
                && matches!(export.representation, Some(InterfaceType::Record { .. }))
        }) else {
            continue;
        };
        collect_providers(export.representation.as_ref().unwrap(), &mut pending);
        let export = export.clone();
        retained.insert(export.symbol.clone());
        interface.exports = vec![export];
        interface.instances.clear();
        interface.operators.clear();
        interface.namespace_deprecations.clear();
        interface.operator_deprecations.clear();
        dependencies.push(LinkedDependency {
            specifier: provider.module,
            origin: ByteSpan { start: 0, end: 0 },
            interface,
            header: None,
            imports: Vec::new(),
        });
    }
}

fn collect_providers(type_ref: &InterfaceType, providers: &mut Vec<TypeProvider>) {
    match type_ref {
        InterfaceType::ExternalNamed {
            canonical,
            provider_module,
            provider_export,
            arguments,
            ..
        } => {
            providers.push(TypeProvider {
                module: provider_module.clone(),
                export: provider_export.clone(),
                canonical: canonical.clone(),
            });
            for argument in arguments {
                collect_providers(argument, providers);
            }
        }
        InterfaceType::Named { arguments, .. } | InterfaceType::Apply { arguments, .. } => {
            for argument in arguments {
                collect_providers(argument, providers);
            }
        }
        InterfaceType::TypeFunction { body, .. } => collect_providers(body, providers),
        InterfaceType::Function { parameter, result } => {
            collect_providers(parameter, providers);
            collect_providers(result, providers);
        }
        InterfaceType::Record { fields, .. } => {
            for field in fields {
                collect_providers(&field.type_ref, providers);
            }
        }
        InterfaceType::Tuple { elements }
        | InterfaceType::RequirementMerge { operands: elements } => {
            for element in elements {
                collect_providers(element, providers);
            }
        }
        InterfaceType::TypeFunctionParameter { .. }
        | InterfaceType::TypeConstructor { .. }
        | InterfaceType::Hole => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{link_module, link_module_with_type_providers};
    use seseragi_syntax::{parse_unlinked_module_interface, InterfaceRecordField};

    fn external(module: &str, name: &str) -> InterfaceType {
        InterfaceType::ExternalNamed {
            name: name.into(),
            canonical: format!("{module}::{name}"),
            provider_module: module.into(),
            provider_export: name.into(),
            arguments: Vec::new(),
        }
    }

    fn model(field: InterfaceType) -> ModuleLinkTarget {
        let mut interface = parse_unlinked_module_interface(
            "model.ssrg",
            "fixture::model",
            "pub alias Span = { span: Int }\n",
        )
        .interface;
        interface.exports[0].representation = Some(InterfaceType::Record {
            closed: true,
            fields: vec![InterfaceRecordField {
                name: "span".into(),
                optional: false,
                type_ref: field,
            }],
        });
        ModuleLinkTarget::external(interface)
    }

    fn main() -> seseragi_syntax::UnlinkedModuleInterface {
        parse_unlinked_module_interface(
            "main.ssrg",
            "fixture::main",
            "import { Span } from \"./model\"\n",
        )
    }

    #[test]
    fn retains_nested_standard_structs_without_source_edges_or_value_imports() {
        let linked = link_module(
            main(),
            &BTreeMap::from([("./model".into(), model(external("std/regex", "RegexMatch")))]),
        )
        .unwrap();
        assert_eq!(linked.interface.dependencies.len(), 1);
        let hidden = &linked.dependencies[1..];
        let retained = hidden
            .iter()
            .flat_map(|dependency| &dependency.interface.exports)
            .map(|export| export.symbol.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            retained,
            BTreeSet::from([
                "std/regex::RegexMatch",
                "std/regex::RegexSpan",
                "std/regex::RegexCapture",
            ])
        );
        for dependency in hidden {
            assert!(dependency.imports.is_empty());
            assert!(dependency.interface.instances.is_empty());
            assert!(dependency.interface.operators.is_empty());
            assert!(dependency.interface.exports.iter().all(|export| {
                export.namespace == "type" && export.declaration_kind.as_deref() == Some("struct")
            }));
        }
    }

    #[test]
    fn retains_reachable_source_structs_and_stops_at_recursive_owners() {
        let mut provider = parse_unlinked_module_interface(
            "provider.ssrg",
            "fixture::provider",
            "pub struct Box<A> { value: A }\n",
        )
        .interface;
        provider.exports[0].representation = Some(InterfaceType::Record {
            closed: true,
            fields: vec![InterfaceRecordField {
                name: "next".into(),
                optional: false,
                type_ref: external("fixture::provider", "Box"),
            }],
        });
        let linked = link_module_with_type_providers(
            main(),
            &BTreeMap::from([(
                "./model".into(),
                model(external("fixture::provider", "Box")),
            )]),
            &[provider],
        )
        .unwrap();
        assert_eq!(linked.interface.dependencies.len(), 1);
        assert_eq!(linked.dependencies.len(), 2);
        assert_eq!(
            linked.dependencies[1].interface.exports[0].symbol,
            "fixture::provider::Box"
        );
    }

    #[test]
    fn rejects_mismatched_provider_identity_and_opaque_representations() {
        let mut mismatched = external("std/regex", "RegexSpan");
        if let InterfaceType::ExternalNamed { canonical, .. } = &mut mismatched {
            *canonical = "fixture::shadow::RegexSpan".into();
        }
        for field in [mismatched, external("std/regex", "Regex")] {
            let linked =
                link_module(main(), &BTreeMap::from([("./model".into(), model(field))])).unwrap();
            assert_eq!(linked.dependencies.len(), 1);
        }
    }
}
