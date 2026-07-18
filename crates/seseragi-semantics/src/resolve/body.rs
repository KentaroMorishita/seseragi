use crate::{
    prelude::{is_standalone_symbol, sum_type_for_symbol, trait_methods_named, PreludeSumType},
    ResolveIssue, ResolvedModule, ResolvedReference, ResolvedScope, ResolvedSymbol, ScopeId,
    ScopeKind, SymbolId, SymbolKind, SymbolNamespace,
};
use seseragi_syntax::{
    parse_module_interface, parse_surface_ast, ByteSpan, ModuleInterface, SurfaceModule,
};
use std::collections::BTreeMap;

mod declarations;
mod expression;
mod imports;
mod instances;
mod namespace;
mod pattern;
mod scheme_types;
#[cfg(test)]
mod tests;

pub fn resolve_module(source_name: impl Into<String>, source: &str) -> ResolvedModule {
    let source_name = source_name.into();
    let interface = parse_module_interface(source_name, source);
    resolve_module_from_interface(interface, source)
}

pub(crate) fn resolve_module_from_interface(
    interface: ModuleInterface,
    source: &str,
) -> ResolvedModule {
    let surface =
        expand_impl_operator_instances(parse_surface_ast(interface.source.clone(), source));
    resolve_surface_module(interface, surface, Vec::new())
}

pub fn resolve_linked_module(
    linked: seseragi_project::LinkedModule,
    source: &str,
) -> ResolvedModule {
    let surface =
        expand_impl_operator_instances(parse_surface_ast(linked.interface.source.clone(), source));
    let (dependency_instances, dependency_instance_issues) =
        instances::resolve_dependency_instances(&linked.dependencies);
    let mut resolver = Resolver::new(&linked.interface.module, module_origin(&surface));
    resolver.issues.extend(dependency_instance_issues);
    declarations::register_module_declarations(&mut resolver, &surface.declarations);
    let imports = imports::register_linked_imports(&mut resolver, &linked.dependencies);
    declarations::resolve_declarations(&mut resolver, &surface.declarations);
    finish_resolved_module(
        linked.interface,
        surface,
        imports,
        dependency_instances,
        resolver,
    )
}

fn expand_impl_operator_instances(mut surface: SurfaceModule) -> SurfaceModule {
    surface.declarations = surface
        .declarations
        .into_iter()
        .flat_map(|declaration| {
            let instances = seseragi_syntax::impl_operator_instances(&declaration);
            std::iter::once(declaration).chain(instances)
        })
        .collect();
    surface
}

fn resolve_surface_module(
    interface: ModuleInterface,
    surface: SurfaceModule,
    imports: Vec<crate::ResolvedImport>,
) -> ResolvedModule {
    let module_origin = module_origin(&surface);
    let mut resolver = Resolver::new(&interface.module, module_origin);
    declarations::register_module_declarations(&mut resolver, &surface.declarations);
    declarations::register_imports(&mut resolver, &interface, &surface);
    declarations::resolve_declarations(&mut resolver, &surface.declarations);

    finish_resolved_module(interface, surface, imports, Vec::new(), resolver)
}

fn finish_resolved_module(
    interface: ModuleInterface,
    surface: SurfaceModule,
    mut imports: Vec<crate::ResolvedImport>,
    dependency_instances: Vec<crate::ResolvedDependencyInstance>,
    mut resolver: Resolver,
) -> ResolvedModule {
    merge_selected_imports(&mut imports, resolver.namespace_imports.take_selected());
    ResolvedModule {
        schema: 2,
        stage: "resolved-ast".to_owned(),
        source: surface.source,
        module: interface.module,
        dependencies: interface.dependencies,
        imports,
        dependency_instances,
        declarations: surface.declarations,
        scopes: resolver.scopes,
        symbols: resolver.symbols,
        references: resolver.references,
        issues: resolver.issues,
    }
}

fn merge_selected_imports(
    imports: &mut Vec<crate::ResolvedImport>,
    selected: Vec<crate::ResolvedImport>,
) {
    for selected in selected {
        let same_binding = |candidate: &crate::ResolvedImport| {
            candidate.specifier == selected.specifier
                && candidate.module == selected.module
                && candidate.export.namespace == selected.export.namespace
                && candidate.export.symbol == selected.export.symbol
        };
        if imports.iter().any(|candidate| {
            candidate.in_scope
                && same_binding(candidate)
                && candidate.local_name == selected.local_name
        }) {
            continue;
        }
        if let Some(hidden) = imports
            .iter_mut()
            .find(|candidate| !candidate.in_scope && same_binding(candidate))
        {
            *hidden = selected;
        } else {
            imports.push(selected);
        }
    }
}

fn module_origin(surface: &SurfaceModule) -> ByteSpan {
    match (surface.declarations.first(), surface.declarations.last()) {
        (Some(first), Some(last)) => ByteSpan {
            start: declaration_span(first).start,
            end: declaration_span(last).end,
        },
        _ => ByteSpan { start: 0, end: 0 },
    }
}

pub(super) struct Resolver {
    module: String,
    scopes: Vec<ResolvedScope>,
    symbols: Vec<ResolvedSymbol>,
    references: Vec<ResolvedReference>,
    issues: Vec<ResolveIssue>,
    names: BTreeMap<(ScopeId, SymbolNamespace, String), SymbolId>,
    trait_methods: BTreeMap<(ScopeId, String), Vec<SymbolId>>,
    dependency_symbols: BTreeMap<(SymbolNamespace, String), SymbolId>,
    prelude_names: BTreeMap<(SymbolNamespace, String), SymbolId>,
    namespace_imports: namespace::NamespaceImports,
}

impl Resolver {
    fn new(module: &str, origin: ByteSpan) -> Self {
        Self {
            module: module.to_owned(),
            scopes: vec![ResolvedScope {
                id: ScopeId(0),
                parent: None,
                kind: ScopeKind::Module,
                origin,
            }],
            symbols: Vec::new(),
            references: Vec::new(),
            issues: Vec::new(),
            names: BTreeMap::new(),
            trait_methods: BTreeMap::new(),
            dependency_symbols: BTreeMap::new(),
            prelude_names: BTreeMap::new(),
            namespace_imports: namespace::NamespaceImports::default(),
        }
    }

    pub(super) fn module_scope(&self) -> ScopeId {
        ScopeId(0)
    }

    pub(super) fn is_module_binding(&self, scope: ScopeId, spelling: &str) -> bool {
        self.lookup(scope, SymbolNamespace::Module, spelling)
            .is_some()
    }

    pub(super) fn new_scope(
        &mut self,
        parent: ScopeId,
        kind: ScopeKind,
        origin: ByteSpan,
    ) -> ScopeId {
        let id = ScopeId(self.scopes.len() as u32);
        self.scopes.push(ResolvedScope {
            id,
            parent: Some(parent),
            kind,
            origin,
        });
        id
    }

    pub(super) fn register(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        kind: SymbolKind,
        spelling: &str,
        canonical: Option<String>,
        origin: ByteSpan,
    ) -> SymbolId {
        let name = (scope, namespace, spelling.to_owned());
        if let Some(existing) = self.names.get(&name).copied() {
            self.issues.push(ResolveIssue {
                code: "SES-N0002".to_owned(),
                message_key: "name.duplicate-definition".to_owned(),
                primary: origin,
            });
            return existing;
        }

        let id = self.push_symbol(scope, namespace, kind, spelling, canonical, origin);
        self.names.insert(name, id);
        id
    }

    fn push_symbol(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        kind: SymbolKind,
        spelling: &str,
        canonical: Option<String>,
        origin: ByteSpan,
    ) -> SymbolId {
        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(ResolvedSymbol {
            id,
            spelling: spelling.to_owned(),
            canonical,
            namespace,
            kind,
            scope,
            origin,
        });
        id
    }

    pub(super) fn dependency_symbol(
        &mut self,
        namespace: SymbolNamespace,
        kind: SymbolKind,
        spelling: &str,
        canonical: String,
    ) -> SymbolId {
        let key = (namespace, canonical.clone());
        if let Some(symbol) = self.dependency_symbols.get(&key) {
            return *symbol;
        }
        let symbol = self.push_symbol(
            self.module_scope(),
            namespace,
            kind,
            spelling,
            Some(canonical),
            ByteSpan { start: 0, end: 0 },
        );
        self.dependency_symbols.insert(key, symbol);
        symbol
    }

    pub(super) fn bind_alias(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        spelling: &str,
        target: SymbolId,
        origin: ByteSpan,
    ) -> SymbolId {
        let name = (scope, namespace, spelling.to_owned());
        if let Some(existing) = self.names.get(&name).copied() {
            self.issues.push(ResolveIssue {
                code: "SES-N0002".to_owned(),
                message_key: "name.duplicate-definition".to_owned(),
                primary: origin,
            });
            return existing;
        }
        self.names.insert(name, target);
        target
    }

    pub(super) fn register_module(
        &mut self,
        namespace: SymbolNamespace,
        kind: SymbolKind,
        spelling: &str,
        origin: ByteSpan,
    ) -> SymbolId {
        let canonical = match namespace {
            SymbolNamespace::Trait => format!("{}::trait({spelling})", self.module),
            _ => format!("{}::{spelling}", self.module),
        };
        self.register(
            self.module_scope(),
            namespace,
            kind,
            spelling,
            Some(canonical),
            origin,
        )
    }

    pub(super) fn register_trait_method(
        &mut self,
        trait_name: &str,
        spelling: &str,
        origin: ByteSpan,
    ) -> SymbolId {
        let scope = self.module_scope();
        let id = self.push_symbol(
            scope,
            SymbolNamespace::Value,
            SymbolKind::TraitMethod,
            spelling,
            Some(format!("{}::trait({trait_name})::{spelling}", self.module)),
            origin,
        );
        self.trait_methods
            .entry((scope, spelling.to_owned()))
            .or_default()
            .push(id);
        id
    }

    pub(super) fn reference(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        spelling: &str,
        origin: ByteSpan,
        report_unresolved: bool,
    ) -> Option<SymbolId> {
        let mut target = self.lookup(scope, namespace, spelling);
        let mut candidates = Vec::new();
        let mut namespace_issue = None;
        if target.is_none() && matches!(namespace, SymbolNamespace::Type | SymbolNamespace::Value) {
            match self.resolve_namespace_member(scope, namespace, spelling, origin) {
                Ok(resolved) => target = resolved,
                Err(issue) => namespace_issue = Some(issue),
            }
        }
        if target.is_none() && namespace == SymbolNamespace::Value {
            candidates = self.lookup_trait_methods(scope, spelling);
            if candidates.is_empty() {
                self.materialize_prelude_trait_methods(spelling);
                candidates = self.lookup_trait_methods(scope, spelling);
            }
            if let [candidate] = candidates.as_slice() {
                target = Some(*candidate);
            }
        }
        if target.is_none() {
            target = self.ensure_prelude(namespace, spelling);
        }
        self.references.push(ResolvedReference {
            spelling: spelling.to_owned(),
            namespace,
            target,
            candidates,
            origin,
        });
        if let Some(issue) = namespace_issue {
            self.issues.push(issue);
        } else if target.is_none()
            && self
                .references
                .last()
                .is_some_and(|reference| reference.candidates.is_empty())
            && report_unresolved
        {
            self.issues.push(ResolveIssue {
                code: "SES-N0001".to_owned(),
                message_key: "name.unresolved".to_owned(),
                primary: origin,
            });
        }
        target
    }

    fn lookup_trait_methods(&self, mut scope: ScopeId, spelling: &str) -> Vec<SymbolId> {
        loop {
            if let Some(methods) = self.trait_methods.get(&(scope, spelling.to_owned())) {
                return methods.clone();
            }
            let Some(parent) = self
                .scopes
                .get(scope.0 as usize)
                .and_then(|scope| scope.parent)
            else {
                return Vec::new();
            };
            scope = parent;
        }
    }

    fn lookup(
        &self,
        mut scope: ScopeId,
        namespace: SymbolNamespace,
        spelling: &str,
    ) -> Option<SymbolId> {
        loop {
            if let Some(symbol) = self.names.get(&(scope, namespace, spelling.to_owned())) {
                return Some(*symbol);
            }
            let parent = self
                .scopes
                .get(scope.0 as usize)
                .and_then(|scope| scope.parent);
            let parent = parent?;
            scope = parent;
        }
    }

    fn ensure_prelude(&mut self, namespace: SymbolNamespace, spelling: &str) -> Option<SymbolId> {
        let key = (namespace, spelling.to_owned());
        if let Some(symbol) = self.prelude_names.get(&key) {
            return Some(*symbol);
        }
        if let Some(sum_type) = sum_type_for_symbol(namespace, spelling) {
            self.materialize_prelude_sum_type(sum_type);
            return self.prelude_names.get(&key).copied();
        }
        if !is_standalone_symbol(namespace, spelling) {
            return None;
        }
        let id = self.push_symbol(
            self.module_scope(),
            namespace,
            SymbolKind::Prelude,
            spelling,
            Some(format!("std/prelude::{spelling}")),
            ByteSpan { start: 0, end: 0 },
        );
        self.prelude_names.insert(key, id);
        Some(id)
    }

    fn materialize_prelude_sum_type(&mut self, sum_type: &PreludeSumType) {
        let owner_key = (SymbolNamespace::Type, sum_type.name.to_owned());
        if self.prelude_names.contains_key(&owner_key) {
            return;
        }
        let origin = ByteSpan { start: 0, end: 0 };
        let owner = self.push_symbol(
            self.module_scope(),
            SymbolNamespace::Type,
            SymbolKind::Type,
            sum_type.name,
            Some(sum_type.canonical.to_owned()),
            origin,
        );
        self.prelude_names.insert(owner_key, owner);

        let declaration_scope = self.new_scope(self.module_scope(), ScopeKind::Declaration, origin);
        for parameter in sum_type.type_parameters {
            self.push_symbol(
                declaration_scope,
                SymbolNamespace::Type,
                SymbolKind::TypeParameter,
                parameter,
                Some(format!("{}::{parameter}", sum_type.canonical)),
                origin,
            );
        }
        for variant in sum_type.variants {
            let symbol = self.push_symbol(
                self.module_scope(),
                SymbolNamespace::Value,
                SymbolKind::Constructor,
                variant.name,
                Some(variant.canonical.to_owned()),
                origin,
            );
            self.prelude_names
                .insert((SymbolNamespace::Value, variant.name.to_owned()), symbol);
        }
    }

    fn materialize_prelude_trait_methods(&mut self, spelling: &str) {
        let methods = trait_methods_named(spelling);
        for method in methods {
            if self
                .trait_methods
                .get(&(self.module_scope(), spelling.to_owned()))
                .into_iter()
                .flatten()
                .any(|symbol| {
                    self.symbols
                        .get(symbol.0 as usize)
                        .and_then(|symbol| symbol.canonical.as_deref())
                        == Some(method.canonical)
                })
            {
                continue;
            }
            self.ensure_prelude(SymbolNamespace::Trait, method.trait_name);
            let symbol = self.push_symbol(
                self.module_scope(),
                SymbolNamespace::Value,
                SymbolKind::TraitMethod,
                method.name,
                Some(method.canonical.to_owned()),
                ByteSpan { start: 0, end: 0 },
            );
            self.trait_methods
                .entry((self.module_scope(), method.name.to_owned()))
                .or_default()
                .push(symbol);
        }
    }
}

pub(super) fn declaration_span(declaration: &seseragi_syntax::SurfaceDecl) -> ByteSpan {
    declaration.span()
}
