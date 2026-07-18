use super::{
    scheme_types::{
        export_contract_trait_bindings, export_scheme_trait_bindings, export_scheme_type_bindings,
    },
    Resolver,
};
use crate::{ResolvedImport, SymbolKind, SymbolNamespace};
use seseragi_project::{LinkedDependency, LinkedImport};

pub(super) fn register_linked_imports(
    resolver: &mut Resolver,
    dependencies: &[LinkedDependency],
) -> Vec<ResolvedImport> {
    let mut resolved = Vec::new();
    for dependency in dependencies {
        for import in &dependency.imports {
            match import {
                LinkedImport::Symbol {
                    local_name,
                    origin,
                    export,
                } => {
                    let Some(namespace) = namespace(&export.namespace) else {
                        continue;
                    };
                    let symbol = resolver.dependency_symbol(
                        namespace,
                        symbol_kind(namespace, export.declaration_kind.as_deref()),
                        &export.name,
                        export.symbol.clone(),
                    );
                    let symbol = resolver.bind_alias(
                        resolver.module_scope(),
                        namespace,
                        local_name,
                        symbol,
                        *origin,
                    );
                    resolved.push(ResolvedImport {
                        symbol,
                        specifier: dependency.specifier.clone(),
                        module: dependency.interface.module.clone(),
                        local_name: local_name.clone(),
                        origin: *origin,
                        in_scope: true,
                        export: export.clone(),
                        member_owner: None,
                        scheme_type_bindings: export_scheme_type_bindings(
                            &dependency.interface,
                            export,
                        ),
                        scheme_trait_bindings: export_scheme_trait_bindings(
                            &dependency.interface,
                            export,
                        ),
                        contract_trait_bindings: export_contract_trait_bindings(
                            &dependency.interface,
                            export,
                        ),
                    });
                }
                LinkedImport::Namespace {
                    local_name,
                    origin,
                    module,
                } => {
                    resolver.register(
                        resolver.module_scope(),
                        SymbolNamespace::Module,
                        SymbolKind::ModuleImport,
                        local_name,
                        Some(format!("{module}::*")),
                        *origin,
                    );
                    resolver.register_namespace_import(
                        local_name,
                        &dependency.specifier,
                        &dependency.interface.module,
                        *origin,
                        &dependency.interface,
                        dependency.header.as_ref(),
                    );
                }
                LinkedImport::Operator {
                    spelling,
                    origin,
                    export,
                    ..
                } => {
                    let symbol = resolver.dependency_symbol(
                        SymbolNamespace::Operator,
                        SymbolKind::Operator,
                        &export.name,
                        export.symbol.clone(),
                    );
                    let symbol = resolver.bind_alias(
                        resolver.module_scope(),
                        SymbolNamespace::Operator,
                        spelling,
                        symbol,
                        *origin,
                    );
                    resolved.push(ResolvedImport {
                        symbol,
                        specifier: dependency.specifier.clone(),
                        module: dependency.interface.module.clone(),
                        local_name: spelling.clone(),
                        origin: *origin,
                        in_scope: true,
                        export: export.clone(),
                        member_owner: None,
                        scheme_type_bindings: export_scheme_type_bindings(
                            &dependency.interface,
                            export,
                        ),
                        scheme_trait_bindings: export_scheme_trait_bindings(
                            &dependency.interface,
                            export,
                        ),
                        contract_trait_bindings: export_contract_trait_bindings(
                            &dependency.interface,
                            export,
                        ),
                    });
                }
            }
        }
        add_dependency_nominal_members(resolver, dependency, &mut resolved);
    }
    resolved
}

fn add_dependency_nominal_members(
    resolver: &mut Resolver,
    dependency: &LinkedDependency,
    resolved: &mut Vec<ResolvedImport>,
) {
    for owner in dependency.interface.exports.iter().filter(|export| {
        export.namespace == "type"
            && matches!(
                export.declaration_kind.as_deref(),
                Some("type" | "opaque-type" | "newtype" | "struct" | "opaque-struct")
            )
    }) {
        for parameter in &owner.scheme.type_parameters {
            resolver.dependency_symbol(
                SymbolNamespace::Type,
                SymbolKind::TypeParameter,
                &parameter.name,
                format!("{}::{}", owner.symbol, parameter.name),
            );
        }
        ensure_dependency_member(resolver, dependency, owner, resolved);
        let Some(owner_symbol) = resolved
            .iter()
            .find(|import| {
                import.export.namespace == "type" && import.export.symbol == owner.symbol
            })
            .map(|import| import.symbol)
        else {
            continue;
        };
        for constructor in dependency
            .interface
            .exports
            .iter()
            .filter(|export| export.constructor_of.as_ref() == Some(&owner.symbol))
        {
            ensure_dependency_member(resolver, dependency, constructor, resolved);
        }
        for method in &owner.methods {
            ensure_dependency_method(resolver, dependency, owner, owner_symbol, method, resolved);
        }
    }
}

fn ensure_dependency_member(
    resolver: &mut Resolver,
    dependency: &LinkedDependency,
    export: &seseragi_syntax::InterfaceExport,
    resolved: &mut Vec<ResolvedImport>,
) {
    if resolved.iter().any(|import| {
        import.export.namespace == export.namespace && import.export.symbol == export.symbol
    }) {
        return;
    }
    let Some(namespace) = namespace(&export.namespace) else {
        return;
    };
    let symbol = resolver.dependency_symbol(
        namespace,
        symbol_kind(namespace, export.declaration_kind.as_deref()),
        &export.name,
        export.symbol.clone(),
    );
    resolved.push(ResolvedImport {
        symbol,
        specifier: dependency.specifier.clone(),
        module: dependency.interface.module.clone(),
        local_name: export.name.clone(),
        origin: seseragi_syntax::ByteSpan { start: 0, end: 0 },
        in_scope: false,
        export: export.clone(),
        member_owner: None,
        scheme_type_bindings: export_scheme_type_bindings(&dependency.interface, export),
        scheme_trait_bindings: export_scheme_trait_bindings(&dependency.interface, export),
        contract_trait_bindings: export_contract_trait_bindings(&dependency.interface, export),
    });
}

fn ensure_dependency_method(
    resolver: &mut Resolver,
    dependency: &LinkedDependency,
    owner: &seseragi_syntax::InterfaceExport,
    owner_symbol: crate::SymbolId,
    method: &seseragi_syntax::InterfaceMethod,
    resolved: &mut Vec<ResolvedImport>,
) {
    let canonical = format!("{}::{}", owner.symbol, method.name);
    if resolved
        .iter()
        .any(|import| import.export.namespace == "method" && import.export.symbol == canonical)
    {
        return;
    }
    let export = seseragi_syntax::InterfaceExport {
        symbol: canonical,
        namespace: "method".to_owned(),
        name: method.name.clone(),
        constructor_of: None,
        visibility: seseragi_syntax::Visibility::Public,
        declaration_kind: Some("inherent-method".to_owned()),
        declaration: method.origin,
        scheme: method.scheme.clone(),
        methods: Vec::new(),
        representation: None,
    };
    let symbol = resolver.dependency_symbol(
        SymbolNamespace::Value,
        SymbolKind::Imported,
        &export.name,
        export.symbol.clone(),
    );
    resolved.push(ResolvedImport {
        symbol,
        specifier: dependency.specifier.clone(),
        module: dependency.interface.module.clone(),
        local_name: export.name.clone(),
        origin: seseragi_syntax::ByteSpan { start: 0, end: 0 },
        in_scope: false,
        member_owner: Some(owner_symbol),
        scheme_type_bindings: export_scheme_type_bindings(&dependency.interface, &export),
        scheme_trait_bindings: export_scheme_trait_bindings(&dependency.interface, &export),
        contract_trait_bindings: None,
        export,
    });
}

fn namespace(namespace: &str) -> Option<SymbolNamespace> {
    match namespace {
        "type" => Some(SymbolNamespace::Type),
        "value" => Some(SymbolNamespace::Value),
        "trait" => Some(SymbolNamespace::Trait),
        "operator" => Some(SymbolNamespace::Operator),
        _ => None,
    }
}

fn symbol_kind(namespace: SymbolNamespace, declaration_kind: Option<&str>) -> SymbolKind {
    match (namespace, declaration_kind) {
        (SymbolNamespace::Type, _) => SymbolKind::Type,
        (SymbolNamespace::Trait, _) => SymbolKind::Trait,
        (SymbolNamespace::Operator, _) => SymbolKind::Operator,
        (SymbolNamespace::Value, Some("constructor")) => SymbolKind::Constructor,
        _ => SymbolKind::Imported,
    }
}
