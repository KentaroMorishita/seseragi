use super::Resolver;
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
                    let symbol = resolver.register(
                        resolver.module_scope(),
                        namespace,
                        symbol_kind(namespace, export.declaration_kind.as_deref()),
                        local_name,
                        Some(export.symbol.clone()),
                        *origin,
                    );
                    resolved.push(ResolvedImport {
                        symbol,
                        local_name: local_name.clone(),
                        origin: *origin,
                        export: export.clone(),
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
                }
                LinkedImport::Operator {
                    spelling,
                    origin,
                    export,
                    ..
                } => {
                    let symbol = resolver.register(
                        resolver.module_scope(),
                        SymbolNamespace::Operator,
                        SymbolKind::Operator,
                        spelling,
                        Some(export.symbol.clone()),
                        *origin,
                    );
                    resolved.push(ResolvedImport {
                        symbol,
                        local_name: spelling.clone(),
                        origin: *origin,
                        export: export.clone(),
                    });
                }
            }
        }
    }
    resolved
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
