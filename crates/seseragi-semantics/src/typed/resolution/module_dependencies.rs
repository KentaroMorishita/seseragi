use crate::{ResolvedModule, TypedModuleDependency, TypedModuleImport};

pub(super) fn collect_module_dependencies(resolved: &ResolvedModule) -> Vec<TypedModuleDependency> {
    resolved
        .dependencies
        .iter()
        .map(|dependency| TypedModuleDependency {
            specifier: dependency.specifier.clone(),
            module: dependency.module.clone(),
            origin: dependency.origin,
            imports: resolved
                .imports
                .iter()
                .filter(|import| {
                    (import.in_scope || import.export.namespace == "method")
                        && import.specifier == dependency.specifier
                        && import.module == dependency.module
                })
                .map(|import| TypedModuleImport {
                    reexported_as: import.reexported_as.clone(),
                    namespace: import.export.namespace.clone(),
                    imported: import.export.name.clone(),
                    local: import.local_name.clone(),
                    canonical: import.export.symbol.clone(),
                    foreign_scalar_result: foreign_scalar_result(&import.export),
                    origin: import.origin,
                })
                .collect(),
        })
        .collect()
}

fn foreign_scalar_result(
    export: &seseragi_syntax::InterfaceExport,
) -> Option<crate::ForeignScalarResultCodec> {
    use seseragi_syntax::InterfaceType;
    if export.namespace != "value"
        || export.declaration_kind.as_deref() != Some("generic-foreign-function")
        || !export.scheme.constraints.is_empty()
        || export.scheme.type_parameters.is_empty()
        || export
            .scheme
            .type_parameters
            .iter()
            .any(|parameter| parameter.arity != 0)
    {
        return None;
    }
    let mut result = &export.scheme.type_ref;
    let mut parameter_count = 0;
    while let InterfaceType::Function { result: next, .. } = result {
        parameter_count += 1;
        result = next;
    }
    // Task returns an Effect; an unapplied result binder can only be a pure
    // foreign return. A returned callable has no direct scalar result proof.
    let InterfaceType::Named { name, arguments } = result else {
        return None;
    };
    if parameter_count == 0
        || !arguments.is_empty()
        || export
            .call_arity
            .is_some_and(|arity| arity != parameter_count)
    {
        return None;
    }
    let matching = export
        .scheme
        .type_parameters
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.name == *name)
        .collect::<Vec<_>>();
    let [(type_parameter_index, _)] = matching.as_slice() else {
        return None;
    };
    Some(crate::ForeignScalarResultCodec {
        parameter_count,
        type_parameter_index: *type_parameter_index,
        type_parameter_count: export.scheme.type_parameters.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ResolvedImport;
    use seseragi_syntax::{
        ByteSpan, InterfaceDependency, InterfaceExport, InterfaceScheme, InterfaceType, Visibility,
    };

    #[test]
    fn keeps_edges_and_only_their_source_visible_canonical_bindings() {
        let mut resolved = crate::resolve_module("artifact/imports/main.ssrg", "");
        resolved.dependencies = vec![dependency("./domain", "fixture/game::domain")];
        resolved.imports = vec![import(true, "next"), import(false, "Hidden")];

        let dependencies = collect_module_dependencies(&resolved);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].specifier, "./domain");
        assert_eq!(dependencies[0].module, "fixture/game::domain");
        assert_eq!(dependencies[0].imports.len(), 1);
        assert_eq!(dependencies[0].imports[0].imported, "increment");
        assert_eq!(dependencies[0].imports[0].local, "next");
        assert_eq!(
            dependencies[0].imports[0].canonical,
            "fixture/game::domain::increment"
        );
    }

    #[test]
    fn keeps_a_dependency_edge_without_resolved_value_or_type_bindings() {
        let mut resolved = crate::resolve_module("artifact/imports/main.ssrg", "");
        resolved.dependencies = vec![dependency("./types", "fixture/game::types")];

        let dependencies = collect_module_dependencies(&resolved);

        assert_eq!(dependencies.len(), 1);
        assert!(dependencies[0].imports.is_empty());
    }

    #[test]
    fn scalar_result_descriptor_requires_original_pure_foreign_binder_proof() {
        let interface = seseragi_syntax::parse_unlinked_module_interface("foreign.ssrg", "fixture::foreign", "pub foreign \"typescript\" from \"./host.mjs\" {\n pure fn choose<A, B> left: A -> right: B -> B\n}\n").interface;
        let export = interface
            .exports
            .iter()
            .find(|export| export.name == "choose")
            .unwrap();
        assert_eq!(
            foreign_scalar_result(export),
            Some(crate::ForeignScalarResultCodec {
                parameter_count: 2,
                type_parameter_index: 1,
                type_parameter_count: 2
            })
        );
        let mut ordinary = export.clone();
        ordinary.declaration_kind = Some("function".into());
        assert!(foreign_scalar_result(&ordinary).is_none());
        let mut arity = export.clone();
        arity.call_arity = Some(1);
        assert!(foreign_scalar_result(&arity).is_none());
        let mut abstract_slot = export.clone();
        abstract_slot.scheme.type_parameters[1].arity = 1;
        assert!(foreign_scalar_result(&abstract_slot).is_none());
        let mut duplicate = export.clone();
        duplicate
            .scheme
            .type_parameters
            .push(seseragi_syntax::TypeParameter::value("B"));
        assert!(foreign_scalar_result(&duplicate).is_none());
        let task = seseragi_syntax::parse_unlinked_module_interface(
            "foreign.ssrg",
            "fixture::foreign",
            "pub foreign \"typescript\" from \"./host.mjs\" {\n task fn echo<A> value: A -> A\n}\n",
        )
        .interface;
        assert!(task.exports.iter().any(|export| export.name == "echo"));
        assert!(task
            .exports
            .iter()
            .all(|export| foreign_scalar_result(export).is_none()));
        let wrapped = seseragi_syntax::parse_unlinked_module_interface("foreign.ssrg", "fixture::foreign", "pub foreign \"typescript\" from \"./host.mjs\" {\n opaque type Schema<A>\n pure fn echo<A> value: A -> Schema<A>\n}\n").interface;
        assert!(wrapped.exports.iter().any(|export| export.name == "echo"));
        assert!(wrapped
            .exports
            .iter()
            .all(|export| foreign_scalar_result(export).is_none()));
    }

    fn dependency(specifier: &str, module: &str) -> InterfaceDependency {
        InterfaceDependency {
            specifier: specifier.to_owned(),
            module: module.to_owned(),
            origin: ByteSpan { start: 0, end: 31 },
            imports: Vec::new(),
        }
    }

    fn import(in_scope: bool, local_name: &str) -> ResolvedImport {
        ResolvedImport {
            reexported_as: None,
            symbol: crate::SymbolId(if in_scope { 1 } else { 2 }),
            specifier: "./domain".to_owned(),
            module: "fixture/game::domain".to_owned(),
            local_name: local_name.to_owned(),
            origin: ByteSpan { start: 9, end: 13 },
            in_scope,
            export: InterfaceExport {
                symbol: "fixture/game::domain::increment".to_owned(),
                namespace: "value".to_owned(),
                name: "increment".to_owned(),
                constructor_of: None,
                call_arity: None,
                visibility: Visibility::Public,
                declaration_kind: Some("function".to_owned()),
                declaration: ByteSpan { start: 4, end: 13 },
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref: InterfaceType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                },
                methods: Vec::new(),
                representation: None,
            },
            member_owner: None,
            scheme_type_bindings: None,
            scheme_trait_bindings: None,
            contract_trait_bindings: None,
        }
    }
}
