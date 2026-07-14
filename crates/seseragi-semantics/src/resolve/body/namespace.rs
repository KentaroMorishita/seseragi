use super::{scheme_types::export_scheme_type_bindings, Resolver};
use crate::{ResolveIssue, ResolvedImport, ScopeId, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{ByteSpan, InterfaceExport, ModuleHeader, ModuleInterface, Visibility};
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
    exports: BTreeMap<(String, String), InterfaceExport>,
    scheme_type_bindings: BTreeMap<(String, String), Option<Vec<crate::ExternalTypeBinding>>>,
    private_names: BTreeSet<(String, String)>,
}

struct NamespaceMember {
    specifier: String,
    module: String,
    origin: ByteSpan,
    export: InterfaceExport,
    scheme_type_bindings: Option<Vec<crate::ExternalTypeBinding>>,
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
        interface: &ModuleInterface,
        header: Option<&ModuleHeader>,
    ) {
        let exports = interface
            .exports
            .iter()
            .filter(|export| matches!(export.namespace.as_str(), "type" | "value"))
            .map(|export| {
                (
                    (export.namespace.clone(), export.name.clone()),
                    export.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let scheme_type_bindings = interface
            .exports
            .iter()
            .map(|export| {
                (
                    (export.namespace.clone(), export.name.clone()),
                    export_scheme_type_bindings(interface, export),
                )
            })
            .collect();
        let private_names = header
            .into_iter()
            .flat_map(|header| &header.names)
            .filter(|name| {
                matches!(name.namespace.as_str(), "type" | "value")
                    && name.visibility == Visibility::Private
            })
            .map(|name| (name.namespace.clone(), name.name.clone()))
            .collect();
        self.bindings.insert(
            local_name.to_owned(),
            NamespaceImport {
                specifier: specifier.to_owned(),
                module: module.to_owned(),
                origin,
                exports,
                scheme_type_bindings,
                private_names,
            },
        );
    }

    fn member(
        &self,
        alias: &str,
        namespace: SymbolNamespace,
        member: &str,
    ) -> Option<NamespaceMemberLookup> {
        let binding = self.bindings.get(alias)?;
        let namespace = namespace_name(namespace)?;
        let key = (namespace.to_owned(), member.to_owned());
        if let Some(export) = binding.exports.get(&key) {
            return Some(NamespaceMemberLookup::Member(Box::new(NamespaceMember {
                specifier: binding.specifier.clone(),
                module: binding.module.clone(),
                origin: binding.origin,
                export: export.clone(),
                scheme_type_bindings: binding.scheme_type_bindings.get(&key).cloned().flatten(),
            })));
        }
        if binding.private_names.contains(&key) {
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
        interface: &ModuleInterface,
        header: Option<&ModuleHeader>,
    ) {
        self.namespace_imports
            .register(local_name, specifier, module, origin, interface, header);
    }

    pub(super) fn resolve_namespace_member(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
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
        let Some(member) = self.namespace_imports.member(alias, namespace, member) else {
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
            namespace,
            symbol_kind(namespace, member.export.declaration_kind.as_deref()),
            &member.export.name,
            member.export.symbol.clone(),
        );
        let symbol = self.bind_alias(
            self.module_scope(),
            namespace,
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
            scheme_type_bindings: member.scheme_type_bindings,
        });
        Ok(Some(symbol))
    }
}

fn namespace_name(namespace: SymbolNamespace) -> Option<&'static str> {
    match namespace {
        SymbolNamespace::Type => Some("type"),
        SymbolNamespace::Value => Some("value"),
        _ => None,
    }
}

fn symbol_kind(namespace: SymbolNamespace, declaration_kind: Option<&str>) -> SymbolKind {
    match (namespace, declaration_kind) {
        (SymbolNamespace::Type, _) => SymbolKind::Type,
        (SymbolNamespace::Value, Some("constructor")) => SymbolKind::Constructor,
        _ => SymbolKind::Imported,
    }
}
