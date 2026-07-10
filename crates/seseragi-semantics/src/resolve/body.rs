use crate::{
    ResolveIssue, ResolvedModule, ResolvedReference, ResolvedScope, ResolvedSymbol, ScopeId,
    ScopeKind, SymbolId, SymbolKind, SymbolNamespace,
};
use seseragi_syntax::{
    parse_module_interface, parse_surface_ast, ByteSpan, ModuleInterface, SurfaceModule,
};
use std::collections::BTreeMap;

mod declarations;
mod expression;
#[cfg(test)]
mod tests;

pub fn resolve_module(source_name: impl Into<String>, source: &str) -> ResolvedModule {
    let source_name = source_name.into();
    let interface = parse_module_interface(source_name, source);
    let surface = parse_surface_ast(interface.source.clone(), source);
    resolve_surface_module(interface, surface)
}

fn resolve_surface_module(interface: ModuleInterface, surface: SurfaceModule) -> ResolvedModule {
    let module_origin = module_origin(&surface);
    let mut resolver = Resolver::new(&interface.module, module_origin);
    declarations::register_module_declarations(&mut resolver, &surface.declarations);
    declarations::register_imports(&mut resolver, &interface, &surface);
    declarations::resolve_declarations(&mut resolver, &surface.declarations);

    ResolvedModule {
        schema: 2,
        stage: "resolved-ast".to_owned(),
        source: surface.source,
        module: interface.module,
        dependencies: interface.dependencies,
        declarations: surface.declarations,
        scopes: resolver.scopes,
        symbols: resolver.symbols,
        references: resolver.references,
        issues: resolver.issues,
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
        }
    }

    pub(super) fn module_scope(&self) -> ScopeId {
        ScopeId(0)
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
        self.names
            .entry((scope, namespace, spelling.to_owned()))
            .or_insert(id);
        id
    }

    pub(super) fn register_module(
        &mut self,
        namespace: SymbolNamespace,
        kind: SymbolKind,
        spelling: &str,
        origin: ByteSpan,
    ) -> SymbolId {
        self.register(
            self.module_scope(),
            namespace,
            kind,
            spelling,
            Some(format!("{}::{spelling}", self.module)),
            origin,
        )
    }

    pub(super) fn reference(
        &mut self,
        scope: ScopeId,
        namespace: SymbolNamespace,
        spelling: &str,
        origin: ByteSpan,
        report_unresolved: bool,
    ) -> Option<SymbolId> {
        let target = self
            .lookup(scope, namespace, spelling)
            .or_else(|| self.ensure_prelude(namespace, spelling));
        self.references.push(ResolvedReference {
            spelling: spelling.to_owned(),
            namespace,
            target,
            origin,
        });
        if target.is_none() && report_unresolved {
            self.issues.push(ResolveIssue {
                code: "SES-N0001".to_owned(),
                message_key: "name.unresolved".to_owned(),
                primary: origin,
            });
        }
        target
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
        if !is_prelude_name(namespace, spelling) {
            return None;
        }
        let id = self.register(
            self.module_scope(),
            namespace,
            SymbolKind::Prelude,
            spelling,
            Some(format!("std/prelude::{spelling}")),
            ByteSpan { start: 0, end: 0 },
        );
        Some(id)
    }
}

fn is_prelude_name(namespace: SymbolNamespace, spelling: &str) -> bool {
    match namespace {
        SymbolNamespace::Type => matches!(
            spelling,
            "Unit"
                | "Never"
                | "Bool"
                | "Int"
                | "Float"
                | "String"
                | "Array"
                | "List"
                | "Maybe"
                | "Either"
                | "Effect"
                | "Console"
                | "ConsoleError"
                | "Stdin"
                | "StdinError"
        ),
        SymbolNamespace::Value => matches!(
            spelling,
            "print" | "println" | "readLine" | "succeed" | "fail" | "mapError"
        ),
        SymbolNamespace::Operator => matches!(
            spelling,
            "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">="
        ),
        _ => false,
    }
}

pub(super) fn declaration_span(declaration: &seseragi_syntax::SurfaceDecl) -> ByteSpan {
    match declaration {
        seseragi_syntax::SurfaceDecl::Let { span, .. }
        | seseragi_syntax::SurfaceDecl::EffectFn { span, .. }
        | seseragi_syntax::SurfaceDecl::Fn { span, .. }
        | seseragi_syntax::SurfaceDecl::Newtype { span, .. }
        | seseragi_syntax::SurfaceDecl::Alias { span, .. }
        | seseragi_syntax::SurfaceDecl::Type { span, .. }
        | seseragi_syntax::SurfaceDecl::Struct { span, .. }
        | seseragi_syntax::SurfaceDecl::Trait { span, .. }
        | seseragi_syntax::SurfaceDecl::Operator { span, .. }
        | seseragi_syntax::SurfaceDecl::Instance { span, .. } => *span,
    }
}
