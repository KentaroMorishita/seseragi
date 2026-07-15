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
                    import.in_scope
                        && import.specifier == dependency.specifier
                        && import.module == dependency.module
                })
                .map(|import| TypedModuleImport {
                    namespace: import.export.namespace.clone(),
                    imported: import.export.name.clone(),
                    local: import.local_name.clone(),
                    canonical: import.export.symbol.clone(),
                    origin: import.origin,
                })
                .collect(),
        })
        .collect()
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
            scheme_type_bindings: None,
            scheme_trait_bindings: None,
            contract_trait_bindings: None,
        }
    }
}
