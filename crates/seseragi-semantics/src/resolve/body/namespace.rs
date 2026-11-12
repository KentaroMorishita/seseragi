use super::{
    scheme_types::{export_scheme_trait_bindings, export_scheme_type_bindings},
    Resolver,
};
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
    scheme_trait_bindings: BTreeMap<(String, String), Option<Vec<crate::ExternalTraitBinding>>>,
    private_names: BTreeSet<(String, String)>,
}

struct NamespaceMember {
    specifier: String,
    module: String,
    origin: ByteSpan,
    export: InterfaceExport,
    scheme_type_bindings: Option<Vec<crate::ExternalTypeBinding>>,
    scheme_trait_bindings: Option<Vec<crate::ExternalTraitBinding>>,
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
        let scheme_trait_bindings = interface
            .exports
            .iter()
            .map(|export| {
                (
                    (export.namespace.clone(), export.name.clone()),
                    export_scheme_trait_bindings(interface, export),
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
                scheme_trait_bindings,
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
                scheme_trait_bindings: binding.scheme_trait_bindings.get(&key).cloned().flatten(),
            })));
        }
        if binding.private_names.contains(&key) {
            return Some(NamespaceMemberLookup::Private);
        }
        Some(NamespaceMemberLookup::Missing)
    }

    /// The namespace imports that export `name`, as `(alias, specifier)`.
    pub(super) fn exporters(&self, namespace: SymbolNamespace, name: &str) -> Vec<(&str, &str)> {
        let Some(namespace) = namespace_name(namespace) else {
            return Vec::new();
        };
        let key = (namespace.to_owned(), name.to_owned());
        self.bindings
            .iter()
            .filter(|(_, binding)| binding.exports.contains_key(&key))
            .map(|(alias, binding)| (alias.as_str(), binding.specifier.as_str()))
            .collect()
    }

    /// The specifier an alias names and the public names it exports in
    /// `namespace`.
    fn exported_names(&self, alias: &str, namespace: SymbolNamespace) -> Option<(&str, Vec<&str>)> {
        let binding = self.bindings.get(alias)?;
        let namespace = namespace_name(namespace)?;
        let names = binding
            .exports
            .iter()
            .filter(|((export_namespace, _), export)| {
                export_namespace == namespace && export.visibility == Visibility::Public
            })
            .map(|((_, name), _)| name.as_str())
            .collect();
        Some((binding.specifier.as_str(), names))
    }

    fn select(&mut self, import: ResolvedImport) {
        self.selected.push(import);
    }

    pub(super) fn take_selected(&mut self) -> Vec<ResolvedImport> {
        std::mem::take(&mut self.selected)
    }
}

impl Resolver {
    pub(super) fn namespace_construction_namespace(&self, spelling: &str) -> SymbolNamespace {
        let Some((alias, member)) = spelling.split_once('.') else {
            return SymbolNamespace::Value;
        };
        match self
            .namespace_imports
            .member(alias, SymbolNamespace::Value, member)
        {
            Some(NamespaceMemberLookup::Missing) => SymbolNamespace::Type,
            _ => SymbolNamespace::Value,
        }
    }

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
        if self.lookup(scope, SymbolNamespace::Module, alias).is_none() {
            return Ok(None);
        }
        let member_name = member;
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
                    hint: None,
                    suggestion: None,
                });
            }
            NamespaceMemberLookup::Missing => {
                return Err(self.missing_namespace_member(
                    alias,
                    namespace,
                    member_name,
                    reference_origin,
                ));
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
            reexported_as: None,
            symbol,
            specifier: member.specifier,
            module: member.module,
            local_name: spelling.to_owned(),
            origin: member.origin,
            in_scope: true,
            export: member.export,
            member_owner: None,
            scheme_type_bindings: member.scheme_type_bindings,
            scheme_trait_bindings: member.scheme_trait_bindings,
            contract_trait_bindings: None,
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

impl Resolver {
    /// A missing export names the module it is missing from. A close export
    /// of that module is offered as the replacement; otherwise another module
    /// that exports the same name is pointed out.
    fn missing_namespace_member(
        &self,
        alias: &str,
        namespace: SymbolNamespace,
        member: &str,
        origin: ByteSpan,
    ) -> ResolveIssue {
        let (specifier, names) = self
            .namespace_imports
            .exported_names(alias, namespace)
            .unwrap_or_default();
        let suggestion =
            super::suggestion::closest_name(member, names).map(|name| format!("{alias}.{name}"));
        let mut hint = format!("`{specifier}` has no export `{member}`");
        if suggestion.is_none() {
            if let Some(elsewhere) = self.other_exporter(namespace, member, specifier) {
                hint.push_str(&format!("; {elsewhere}"));
            }
        }
        ResolveIssue {
            code: "SES-N0104".to_owned(),
            message_key: "module.export-unresolved".to_owned(),
            primary: origin,
            hint: Some(hint),
            suggestion,
        }
    }

    /// Where else `member` is exported: an imported namespace first, then an
    /// available standard module.
    fn other_exporter(
        &self,
        namespace: SymbolNamespace,
        member: &str,
        specifier: &str,
    ) -> Option<String> {
        if let Some((alias, _)) = self
            .namespace_imports
            .exporters(namespace, member)
            .into_iter()
            .find(|(_, other)| *other != specifier)
        {
            return Some(format!("write `{alias}.{member}`"));
        }
        let namespace = namespace_name(namespace)?;
        seseragi_project::standard_module_registry_surface()
            .modules
            .into_iter()
            .filter(|module| {
                module.specifier != specifier
                    && module.status == seseragi_project::StandardModuleStatus::Available
            })
            .find(|module| {
                module.public_interface.as_ref().is_some_and(|interface| {
                    interface.exports.iter().any(|export| {
                        export.namespace == namespace
                            && export.name == member
                            && export.visibility == Visibility::Public
                    })
                })
            })
            .map(|module| format!("`{}` exports it", module.specifier))
    }
}
