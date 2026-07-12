use super::Resolver;
use crate::{ResolveIssue, ResolvedImport, ScopeId, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{ByteSpan, InterfaceExport, ModuleHeader, Visibility};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct NamespaceImports {
    bindings: BTreeMap<String, NamespaceImport>,
    selected: Vec<ResolvedImport>,
}

struct NamespaceImport {
    specifier: String,
    module: String,
    origin: ByteSpan,
    values: BTreeMap<String, InterfaceExport>,
    private_values: BTreeSet<String>,
}

struct NamespaceMember {
    specifier: String,
    module: String,
    origin: ByteSpan,
    export: InterfaceExport,
}

enum NamespaceMemberLookup {
    Member(Box<NamespaceMember>),
    Private,
    Missing,
}

impl NamespaceImports {
    fn register(
        &mut self,
        local_name: &str,
        specifier: &str,
        module: &str,
        origin: ByteSpan,
        exports: &[InterfaceExport],
        header: Option<&ModuleHeader>,
    ) {
        let values = exports
            .iter()
            .filter(|export| export.namespace == "value")
            .map(|export| (export.name.clone(), export.clone()))
            .collect::<BTreeMap<_, _>>();
        let private_values = header
            .into_iter()
            .flat_map(|header| &header.names)
            .filter(|name| name.namespace == "value" && name.visibility == Visibility::Private)
            .map(|name| name.name.clone())
            .collect();
        self.bindings.insert(
            local_name.to_owned(),
            NamespaceImport {
                specifier: specifier.to_owned(),
                module: module.to_owned(),
                origin,
                values,
                private_values,
            },
        );
    }

    fn member(&self, alias: &str, member: &str) -> Option<NamespaceMemberLookup> {
        let binding = self.bindings.get(alias)?;
        if let Some(export) = binding.values.get(member) {
            return Some(NamespaceMemberLookup::Member(Box::new(NamespaceMember {
                specifier: binding.specifier.clone(),
                module: binding.module.clone(),
                origin: binding.origin,
                export: export.clone(),
            })));
        }
        if binding.private_values.contains(member) {
            return Some(NamespaceMemberLookup::Private);
        }
        Some(NamespaceMemberLookup::Missing)
    }

    fn select(&mut self, import: ResolvedImport) {
        self.selected.push(import);
    }

    pub(super) fn take_selected(&mut self) -> Vec<ResolvedImport> {
        std::mem::take(&mut self.selected)
    }
}

impl Resolver {
    pub(super) fn register_namespace_import(
        &mut self,
        local_name: &str,
        specifier: &str,
        module: &str,
        origin: ByteSpan,
        exports: &[InterfaceExport],
        header: Option<&ModuleHeader>,
    ) {
        self.namespace_imports
            .register(local_name, specifier, module, origin, exports, header);
    }

    pub(super) fn resolve_namespace_value(
        &mut self,
        scope: ScopeId,
        spelling: &str,
        reference_origin: ByteSpan,
    ) -> Result<Option<SymbolId>, ResolveIssue> {
        let Some((alias, member)) = spelling.split_once('.') else {
            return Ok(None);
        };
        if member.contains('.') {
            return Ok(None);
        }
        if self.lookup(scope, SymbolNamespace::Module, alias).is_none() {
            return Ok(None);
        }
        let Some(member) = self.namespace_imports.member(alias, member) else {
            return Ok(None);
        };
        let member = match member {
            NamespaceMemberLookup::Member(member) => member,
            NamespaceMemberLookup::Private => {
                return Err(ResolveIssue {
                    code: "SES-N0102".to_owned(),
                    message_key: "module.private-symbol".to_owned(),
                    primary: reference_origin,
                });
            }
            NamespaceMemberLookup::Missing => {
                return Err(ResolveIssue {
                    code: "SES-N0104".to_owned(),
                    message_key: "module.export-unresolved".to_owned(),
                    primary: reference_origin,
                });
            }
        };
        let symbol = self.dependency_symbol(
            SymbolNamespace::Value,
            symbol_kind(member.export.declaration_kind.as_deref()),
            &member.export.name,
            member.export.symbol.clone(),
        );
        let symbol = self.bind_alias(
            self.module_scope(),
            SymbolNamespace::Value,
            spelling,
            symbol,
            member.origin,
        );
        self.namespace_imports.select(ResolvedImport {
            symbol,
            specifier: member.specifier,
            module: member.module,
            local_name: spelling.to_owned(),
            origin: member.origin,
            in_scope: true,
            export: member.export,
        });
        Ok(Some(symbol))
    }
}

fn symbol_kind(declaration_kind: Option<&str>) -> SymbolKind {
    match declaration_kind {
        Some("constructor") => SymbolKind::Constructor,
        _ => SymbolKind::Imported,
    }
}
