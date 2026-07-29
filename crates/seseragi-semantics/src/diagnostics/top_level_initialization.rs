use crate::{ResolvedModule, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceBlockItem,
    SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr, SurfaceTemplatePart,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect_top_level_initialization_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let graph = InitializationGraph::new(resolved);
    for binding in &graph.bindings {
        let mut walker = DependencyWalker {
            graph: &graph,
            binding,
            visited_callables: BTreeSet::new(),
            reported_values: BTreeSet::new(),
            diagnostics,
        };
        walker.walk_expr(binding.body, None);
    }
}

struct InitializationGraph<'a> {
    resolved: &'a ResolvedModule,
    bindings: Vec<TopLevelBinding<'a>>,
    binding_indexes: BTreeMap<SymbolId, usize>,
    binding_names: BTreeMap<SymbolId, String>,
    binding_origins: BTreeMap<SymbolId, ByteSpan>,
    callables: BTreeMap<SymbolId, &'a SurfaceExpr>,
}

struct TopLevelBinding<'a> {
    name: String,
    name_span: ByteSpan,
    index: usize,
    body: &'a SurfaceExpr,
}

impl<'a> InitializationGraph<'a> {
    fn new(resolved: &'a ResolvedModule) -> Self {
        let mut bindings = Vec::new();
        let mut binding_indexes = BTreeMap::new();
        let mut binding_names = BTreeMap::new();
        let mut binding_origins = BTreeMap::new();
        let mut callables = BTreeMap::new();

        for (declaration_index, declaration) in resolved.declarations.iter().enumerate() {
            match declaration {
                SurfaceDecl::Let {
                    pattern,
                    body: Some(body),
                    ..
                } => {
                    for binding in pattern.bindings() {
                        let Some(symbol) =
                            declaration_symbol(resolved, SymbolKind::Let, binding.name_span)
                        else {
                            continue;
                        };
                        bindings.push(TopLevelBinding {
                            name: binding.name.clone(),
                            name_span: binding.name_span,
                            index: declaration_index,
                            body,
                        });
                        binding_indexes.insert(symbol, declaration_index);
                        binding_names.insert(symbol, binding.name);
                        binding_origins.insert(symbol, binding.name_span);
                    }
                }
                SurfaceDecl::Fn {
                    name_span,
                    body: Some(body),
                    ..
                }
                | SurfaceDecl::EffectFn {
                    name_span,
                    body: Some(body),
                    ..
                } => {
                    let kind = if matches!(declaration, SurfaceDecl::Fn { .. }) {
                        SymbolKind::Function
                    } else {
                        SymbolKind::EffectFunction
                    };
                    if let Some(symbol) = declaration_symbol(resolved, kind, *name_span) {
                        callables.insert(symbol, body);
                    }
                }
                SurfaceDecl::Operator {
                    spelling_span,
                    body: Some(body),
                    ..
                } => {
                    if let Some(symbol) =
                        declaration_symbol(resolved, SymbolKind::Operator, *spelling_span)
                    {
                        callables.insert(symbol, body);
                    }
                }
                _ => {}
            }
        }

        Self {
            resolved,
            bindings,
            binding_indexes,
            binding_names,
            binding_origins,
            callables,
        }
    }

    fn target_at(&self, span: ByteSpan, namespace: SymbolNamespace) -> Option<SymbolId> {
        self.resolved
            .references
            .iter()
            .find(|reference| reference.origin == span && reference.namespace == namespace)
            .and_then(|reference| reference.target)
    }
}

fn declaration_symbol(
    resolved: &ResolvedModule,
    kind: SymbolKind,
    origin: ByteSpan,
) -> Option<SymbolId> {
    resolved
        .symbols
        .iter()
        .find(|symbol| symbol.kind == kind && symbol.origin == origin)
        .map(|symbol| symbol.id)
}

struct DependencyWalker<'graph, 'source, 'diagnostics> {
    graph: &'graph InitializationGraph<'source>,
    binding: &'graph TopLevelBinding<'source>,
    visited_callables: BTreeSet<SymbolId>,
    reported_values: BTreeSet<SymbolId>,
    diagnostics: &'diagnostics mut Vec<Diagnostic>,
}

impl DependencyWalker<'_, '_, '_> {
    fn walk_expr(&mut self, expression: &SurfaceExpr, root_call: Option<ByteSpan>) {
        match expression {
            SurfaceExpr::Unit { .. }
            | SurfaceExpr::Integer { .. }
            | SurfaceExpr::Float { .. }
            | SurfaceExpr::String { .. }
            | SurfaceExpr::Boolean { .. }
            | SurfaceExpr::Error { .. } => {}
            SurfaceExpr::Name { span, .. } => self.walk_name(*span, root_call),
            SurfaceExpr::Template { parts, .. } => {
                for part in parts {
                    if let SurfaceTemplatePart::Interpolation { value, .. } = part {
                        self.walk_expr(value, root_call);
                    }
                }
            }
            SurfaceExpr::Member { receiver, .. } => self.walk_expr(receiver, root_call),
            SurfaceExpr::Application {
                function, argument, ..
            } => {
                self.walk_expr(function, root_call);
                self.walk_expr(argument, root_call);
                if let Some(callee_span) = application_head_span(function) {
                    self.walk_callable_at(callee_span, SymbolNamespace::Value, root_call);
                }
            }
            SurfaceExpr::Prefix { operand, .. } => self.walk_expr(operand, root_call),
            SurfaceExpr::Assignment { target, value, .. } => {
                self.walk_expr(target, root_call);
                self.walk_expr(value, root_call);
            }
            // A lambda closes over module values but does not read them until it is called.
            SurfaceExpr::Lambda { .. } => {}
            SurfaceExpr::EffectfulFor { source, .. } => self.walk_expr(source, root_call),
            SurfaceExpr::Tuple { elements, .. }
            | SurfaceExpr::Array { elements, .. }
            | SurfaceExpr::List { elements, .. } => {
                for element in elements {
                    self.walk_expr(element, root_call);
                }
            }
            SurfaceExpr::Record { items, .. } | SurfaceExpr::Struct { items, .. } => {
                for item in items {
                    self.walk_expr(item.value(), root_call);
                }
            }
            SurfaceExpr::ArrayComprehension {
                element, clauses, ..
            }
            | SurfaceExpr::ListComprehension {
                element, clauses, ..
            } => {
                for clause in clauses {
                    match clause {
                        SurfaceComprehensionClause::Generator { source, .. } => {
                            self.walk_expr(source, root_call)
                        }
                        SurfaceComprehensionClause::Guard { condition, .. } => {
                            self.walk_expr(condition, root_call)
                        }
                    }
                }
                self.walk_expr(element, root_call);
            }
            SurfaceExpr::Binary {
                operator_span,
                left,
                right,
                ..
            } => {
                self.walk_expr(left, root_call);
                self.walk_expr(right, root_call);
                self.walk_callable_at(*operator_span, SymbolNamespace::Operator, root_call);
            }
            SurfaceExpr::InfixChain { first, steps, .. } => {
                self.walk_expr(first, root_call);
                for step in steps {
                    self.walk_expr(&step.operand, root_call);
                    self.walk_callable_at(step.operator_span, SymbolNamespace::Operator, root_call);
                }
            }
            SurfaceExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.walk_expr(condition, root_call);
                self.walk_expr(then_branch, root_call);
                self.walk_expr(else_branch, root_call);
            }
            SurfaceExpr::Match {
                scrutinee, arms, ..
            } => {
                self.walk_expr(scrutinee, root_call);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.walk_expr(guard, root_call);
                    }
                    self.walk_expr(&arm.body, root_call);
                }
            }
            SurfaceExpr::Block { items, result, .. } => {
                for item in items {
                    if let SurfaceBlockItem::Let { value, .. } = item {
                        self.walk_expr(value, root_call);
                    }
                }
                self.walk_expr(result, root_call);
            }
            SurfaceExpr::Do { items, result, .. } => {
                for item in items {
                    match item {
                        SurfaceDoItem::Bind { value, .. }
                        | SurfaceDoItem::Let { value, .. }
                        | SurfaceDoItem::Expression { value, .. } => {
                            self.walk_expr(value, root_call)
                        }
                    }
                }
                if let Some(result) = result {
                    self.walk_expr(result, root_call);
                }
            }
            SurfaceExpr::Grouped { value, .. } => self.walk_expr(value, root_call),
        }
    }

    fn walk_name(&mut self, span: ByteSpan, root_call: Option<ByteSpan>) {
        let Some(target) = self.graph.target_at(span, SymbolNamespace::Value) else {
            return;
        };
        let Some(target_index) = self.graph.binding_indexes.get(&target).copied() else {
            return;
        };
        if target_index < self.binding.index || !self.reported_values.insert(target) {
            return;
        }

        let cycle = target_index == self.binding.index;
        let primary = root_call.unwrap_or(span);
        let target_name = self
            .graph
            .binding_names
            .get(&target)
            .map(String::as_str)
            .unwrap_or("value");
        let target_origin = self
            .graph
            .binding_origins
            .get(&target)
            .copied()
            .unwrap_or(span);
        let mut related = vec![RelatedDiagnostic {
            message: if root_call.is_some() {
                format!("{target_name} is not initialized before this call")
            } else {
                format!("{target_name} is initialized at this later declaration")
            },
            primary: byte_range(if root_call.is_some() {
                span
            } else {
                target_origin
            }),
        }];
        related.push(RelatedDiagnostic {
            message: format!("{} is initialized here", self.binding.name),
            primary: byte_range(self.binding.name_span),
        });
        self.diagnostics.push(Diagnostic {
            type_difference: None,
            id: String::new(),
            code: "SES-N0201".to_owned(),
            severity: DiagnosticSeverity::Error,
            message_key: if cycle {
                "module.initialization-cycle".to_owned()
            } else {
                "module.initialization-order".to_owned()
            },
            primary: byte_range(primary),
            related,
            fixes: Vec::new(),
        });
    }

    fn walk_callable_at(
        &mut self,
        callee_span: ByteSpan,
        namespace: SymbolNamespace,
        root_call: Option<ByteSpan>,
    ) {
        let Some(target) = self.graph.target_at(callee_span, namespace) else {
            return;
        };
        let Some(body) = self.graph.callables.get(&target).copied() else {
            return;
        };
        if !self.visited_callables.insert(target) {
            return;
        }
        self.walk_expr(body, root_call.or(Some(callee_span)));
    }
}

fn application_head_span(expression: &SurfaceExpr) -> Option<ByteSpan> {
    match expression {
        SurfaceExpr::Name { span, .. } => Some(*span),
        SurfaceExpr::Application { function, .. }
        | SurfaceExpr::Grouped {
            value: function, ..
        } => application_head_span(function),
        _ => None,
    }
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
