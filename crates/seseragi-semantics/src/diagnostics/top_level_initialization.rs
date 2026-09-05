use crate::typed::TypedResolution;
use crate::{ResolvedModule, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticSeverity, RelatedDiagnostic, SurfaceBlockItem,
    SurfaceComprehensionClause, SurfaceDecl, SurfaceDoItem, SurfaceExpr, SurfaceImplMember,
    SurfaceParameter, SurfaceTemplatePart,
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
            active_callables: BTreeSet::new(),
            active_methods: BTreeSet::new(),
            reported_values: BTreeSet::new(),
            local_callables: BTreeMap::new(),
            callable_arguments: BTreeMap::new(),
            diagnostics,
        };
        walker.walk_expr(binding.body, None);
    }
}

struct InitializationGraph<'a> {
    resolved: &'a ResolvedModule,
    resolution: TypedResolution<'a>,
    bindings: Vec<TopLevelBinding<'a>>,
    binding_indexes: BTreeMap<SymbolId, usize>,
    binding_names: BTreeMap<SymbolId, String>,
    binding_origins: BTreeMap<SymbolId, ByteSpan>,
    callables: BTreeMap<SymbolId, CallableBody<'a>>,
    method_callables: BTreeMap<String, CallableBody<'a>>,
    instance_callables: BTreeMap<(String, String), CallableBody<'a>>,
    value_initializers: BTreeMap<SymbolId, &'a SurfaceExpr>,
}

#[derive(Clone)]
struct CallableBody<'a> {
    parameters: Vec<SymbolId>,
    arity: usize,
    body: &'a SurfaceExpr,
}

struct TopLevelBinding<'a> {
    name: String,
    name_span: ByteSpan,
    index: usize,
    body: &'a SurfaceExpr,
}

impl<'a> InitializationGraph<'a> {
    fn new(resolved: &'a ResolvedModule) -> Self {
        let resolution = TypedResolution::new(resolved);
        let mut bindings = Vec::new();
        let mut binding_indexes = BTreeMap::new();
        let mut binding_names = BTreeMap::new();
        let mut binding_origins = BTreeMap::new();
        let mut callables = BTreeMap::new();
        let mut method_callables = BTreeMap::new();
        let mut instance_callables = BTreeMap::new();
        let mut value_initializers = BTreeMap::new();

        for (declaration_index, declaration) in resolved.declarations.iter().enumerate() {
            match declaration {
                SurfaceDecl::Let {
                    pattern,
                    body: Some(body),
                    ..
                } => {
                    let pattern_bindings = pattern.bindings();
                    let Some(primary_binding) = pattern_bindings.first() else {
                        continue;
                    };
                    bindings.push(TopLevelBinding {
                        name: primary_binding.name.clone(),
                        name_span: primary_binding.name_span,
                        index: declaration_index,
                        body,
                    });
                    for binding in pattern_bindings {
                        let Some(symbol) =
                            declaration_symbol(resolved, SymbolKind::Let, binding.name_span)
                        else {
                            continue;
                        };
                        binding_indexes.insert(symbol, declaration_index);
                        binding_names.insert(symbol, binding.name);
                        binding_origins.insert(symbol, binding.name_span);
                        value_initializers.insert(symbol, body);
                    }
                }
                SurfaceDecl::Fn {
                    name_span,
                    parameters,
                    body: Some(body),
                    ..
                }
                | SurfaceDecl::EffectFn {
                    name_span,
                    parameters,
                    body: Some(body),
                    ..
                } => {
                    let kind = if matches!(declaration, SurfaceDecl::Fn { .. }) {
                        SymbolKind::Function
                    } else {
                        SymbolKind::EffectFunction
                    };
                    if let Some(symbol) = declaration_symbol(resolved, kind, *name_span) {
                        callables.insert(
                            symbol,
                            CallableBody {
                                parameters: parameter_symbols(resolved, parameters),
                                arity: parameters.len().max(1),
                                body,
                            },
                        );
                    }
                }
                SurfaceDecl::Operator {
                    spelling_span,
                    parameters,
                    body: Some(body),
                    ..
                } => {
                    if let Some(symbol) =
                        declaration_symbol(resolved, SymbolKind::Operator, *spelling_span)
                    {
                        callables.insert(
                            symbol,
                            CallableBody {
                                parameters: parameter_symbols(resolved, parameters),
                                arity: parameters.len().max(1),
                                body,
                            },
                        );
                    }
                }
                SurfaceDecl::Impl {
                    target, members, ..
                } => {
                    for member in members {
                        let SurfaceImplMember::Method { method, .. } = member else {
                            continue;
                        };
                        let (Some(symbol), Some(body)) = (
                            resolution.inherent_method_symbol(target, &method.name),
                            method.body.as_ref(),
                        ) else {
                            continue;
                        };
                        method_callables.insert(
                            symbol,
                            CallableBody {
                                parameters: parameter_symbols(resolved, &method.parameters),
                                arity: method.parameters.len().max(1),
                                body,
                            },
                        );
                    }
                }
                SurfaceDecl::Instance {
                    type_parameters,
                    trait_name_span,
                    arguments,
                    methods,
                    ..
                } => {
                    let Some(identity) = local_instance_identity(
                        resolved,
                        &resolution,
                        *trait_name_span,
                        type_parameters,
                        arguments,
                    ) else {
                        continue;
                    };
                    for method in methods {
                        let Some(body) = method.body.as_ref() else {
                            continue;
                        };
                        instance_callables.insert(
                            (identity.clone(), method.name.clone()),
                            CallableBody {
                                parameters: parameter_symbols(resolved, &method.parameters),
                                arity: method.parameters.len().max(1),
                                body,
                            },
                        );
                    }
                }
                _ => {}
            }
        }

        Self {
            resolved,
            resolution,
            bindings,
            binding_indexes,
            binding_names,
            binding_origins,
            callables,
            method_callables,
            instance_callables,
            value_initializers,
        }
    }

    fn target_at(&self, span: ByteSpan, namespace: SymbolNamespace) -> Option<SymbolId> {
        self.resolved
            .references
            .iter()
            .find(|reference| reference.origin == span && reference.namespace == namespace)
            .and_then(|reference| reference.target)
    }

    fn inherent_method_symbol(&self, receiver: &SurfaceExpr, field: &str) -> Option<String> {
        let receiver = match receiver {
            SurfaceExpr::Grouped { value, .. } => value.as_ref(),
            receiver => receiver,
        };
        let SurfaceExpr::Name { span, .. } = receiver else {
            return None;
        };
        let target = self.target_at(*span, SymbolNamespace::Value)?;
        let type_ref = self.resolution.top_level_value_type(target)?;
        let receiver_type = self.resolution.semantic_value_from_typed_type(type_ref);
        self.resolution
            .inherent_method(&receiver_type.key, field)
            .map(|method| method.symbol.clone())
    }
}

fn local_instance_identity(
    resolved: &ResolvedModule,
    resolution: &TypedResolution<'_>,
    trait_name_span: ByteSpan,
    type_parameters: &[seseragi_syntax::TypeParameter],
    arguments: &[seseragi_syntax::TypeRef],
) -> Option<String> {
    let trait_symbol = resolved
        .references
        .iter()
        .find(|reference| {
            reference.origin == trait_name_span && reference.namespace == SymbolNamespace::Trait
        })
        .and_then(|reference| reference.target)
        .and_then(|target| resolved.symbols.iter().find(|symbol| symbol.id == target))?;
    let trait_identity = trait_symbol.canonical.as_deref()?;
    let binders = type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let arguments = arguments
        .iter()
        .map(|argument| crate::typed::instances::canonical_type_ref(argument, resolution, &binders))
        .collect::<Option<Vec<_>>>()?;
    Some(crate::typed::instances::canonical_instance_head_identity(
        trait_identity,
        &arguments,
    ))
}

fn parameter_symbols(resolved: &ResolvedModule, parameters: &[SurfaceParameter]) -> Vec<SymbolId> {
    parameters
        .iter()
        .filter_map(|parameter| {
            declaration_symbol(resolved, SymbolKind::Parameter, parameter.name_span)
        })
        .collect()
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
    active_callables: BTreeSet<SymbolId>,
    active_methods: BTreeSet<String>,
    reported_values: BTreeSet<SymbolId>,
    local_callables: BTreeMap<SymbolId, CallableBody<'source>>,
    callable_arguments: BTreeMap<SymbolId, &'source SurfaceExpr>,
    diagnostics: &'diagnostics mut Vec<Diagnostic>,
}

impl<'graph, 'source, 'diagnostics> DependencyWalker<'graph, 'source, 'diagnostics> {
    fn walk_expr(&mut self, expression: &'source SurfaceExpr, root_call: Option<ByteSpan>) {
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
            SurfaceExpr::Member {
                receiver,
                field,
                field_span,
                ..
            } => {
                self.walk_expr(receiver, root_call);
                if let Some(symbol) = self.graph.inherent_method_symbol(receiver, field) {
                    self.walk_method(
                        &symbol,
                        &[receiver.as_ref()],
                        root_call.or(Some(*field_span)),
                    );
                }
            }
            SurfaceExpr::Application { .. } => {
                let (callee, arguments) = flatten_application(expression);
                self.walk_expr(callee, root_call);
                for argument in &arguments {
                    self.walk_expr(argument, root_call);
                }
                self.walk_invoked_expr(callee, &arguments, root_call);
                self.walk_trait_dispatch(expression, &arguments, root_call);
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
            SurfaceExpr::Index {
                receiver, index, ..
            } => {
                self.walk_expr(receiver, root_call);
                self.walk_expr(index, root_call);
            }
            SurfaceExpr::Binary {
                operator_span,
                left,
                right,
                ..
            } => {
                self.walk_expr(left, root_call);
                self.walk_expr(right, root_call);
                self.walk_callable_at(
                    *operator_span,
                    SymbolNamespace::Operator,
                    &[left.as_ref(), right.as_ref()],
                    root_call,
                );
            }
            SurfaceExpr::InfixChain { first, steps, .. } => {
                self.walk_expr(first, root_call);
                for step in steps {
                    self.walk_expr(&step.operand, root_call);
                    self.walk_callable_at(
                        step.operator_span,
                        SymbolNamespace::Operator,
                        &[],
                        root_call,
                    );
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
                    if let SurfaceBlockItem::Function {
                        name_span, value, ..
                    } = item
                    {
                        if let Some(symbol) = declaration_symbol(
                            self.graph.resolved,
                            SymbolKind::Function,
                            *name_span,
                        ) {
                            let SurfaceBlockItem::Function { parameters, .. } = item else {
                                unreachable!();
                            };
                            self.local_callables.insert(
                                symbol,
                                CallableBody {
                                    parameters: parameter_symbols(self.graph.resolved, parameters),
                                    arity: parameters.len().max(1),
                                    body: value,
                                },
                            );
                        }
                    }
                }
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
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        let Some(target) = self.graph.target_at(callee_span, namespace) else {
            return;
        };
        self.walk_callable_symbol(target, arguments, root_call.or(Some(callee_span)));
    }

    fn walk_callable_symbol(
        &mut self,
        target: SymbolId,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        if !self.active_callables.insert(target) {
            return;
        }

        if let Some(argument) = self.callable_arguments.get(&target).copied() {
            self.walk_callable_value(argument, arguments, root_call);
        } else if let Some(callable) = self
            .graph
            .callables
            .get(&target)
            .cloned()
            .or_else(|| self.local_callables.get(&target).cloned())
        {
            self.walk_callable_body(&callable, arguments, root_call);
        } else if let Some(value) = self.graph.value_initializers.get(&target).copied() {
            self.walk_callable_value(value, arguments, root_call);
        }

        self.active_callables.remove(&target);
    }

    fn walk_callable_body(
        &mut self,
        callable: &CallableBody<'source>,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        if arguments.len() < callable.arity {
            return;
        }

        let previous = callable
            .parameters
            .iter()
            .zip(arguments)
            .map(|(parameter, argument)| {
                (
                    *parameter,
                    self.callable_arguments.insert(*parameter, argument),
                )
            })
            .collect::<Vec<_>>();
        self.walk_expr(callable.body, root_call);
        if arguments.len() > callable.arity {
            self.walk_callable_value(callable.body, &arguments[callable.arity..], root_call);
        }
        for (parameter, value) in previous {
            if let Some(value) = value {
                self.callable_arguments.insert(parameter, value);
            } else {
                self.callable_arguments.remove(&parameter);
            }
        }
    }

    fn walk_callable_value(
        &mut self,
        expression: &'source SurfaceExpr,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        match expression {
            SurfaceExpr::Name { span, .. } => {
                let Some(target) = self.graph.target_at(*span, SymbolNamespace::Value) else {
                    return;
                };
                self.walk_callable_symbol(target, arguments, root_call);
            }
            SurfaceExpr::Lambda {
                parameter,
                body,
                span,
            } => {
                if arguments.is_empty() {
                    return;
                }
                let parameter_symbol = declaration_symbol(
                    self.graph.resolved,
                    SymbolKind::Parameter,
                    parameter.name_span,
                );
                let previous = parameter_symbol
                    .map(|symbol| (symbol, self.callable_arguments.insert(symbol, arguments[0])));
                self.walk_expr(body, root_call.or(Some(*span)));
                if arguments.len() > 1 {
                    self.walk_callable_value(body, &arguments[1..], root_call.or(Some(*span)));
                }
                if let Some((symbol, value)) = previous {
                    if let Some(value) = value {
                        self.callable_arguments.insert(symbol, value);
                    } else {
                        self.callable_arguments.remove(&symbol);
                    }
                }
            }
            SurfaceExpr::Grouped { value, .. } => {
                self.walk_callable_value(value, arguments, root_call)
            }
            SurfaceExpr::Application { .. } => {
                let (callee, mut applied_arguments) = flatten_application(expression);
                applied_arguments.extend_from_slice(arguments);
                self.walk_invoked_expr(callee, &applied_arguments, root_call);
            }
            SurfaceExpr::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.walk_callable_value(then_branch, arguments, root_call);
                self.walk_callable_value(else_branch, arguments, root_call);
            }
            SurfaceExpr::Match { arms, .. } => {
                for arm in arms {
                    self.walk_callable_value(&arm.body, arguments, root_call);
                }
            }
            SurfaceExpr::Block { result, .. } => {
                self.walk_expr(expression, root_call);
                self.walk_callable_value(result, arguments, root_call);
            }
            _ => {}
        }
    }

    fn walk_invoked_expr(
        &mut self,
        expression: &'source SurfaceExpr,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        match expression {
            SurfaceExpr::Name { span, .. } => {
                self.walk_callable_at(*span, SymbolNamespace::Value, arguments, root_call)
            }
            SurfaceExpr::Lambda { .. } => {
                self.walk_callable_value(expression, arguments, root_call)
            }
            SurfaceExpr::Grouped { value, .. } => {
                self.walk_invoked_expr(value, arguments, root_call)
            }
            SurfaceExpr::Member {
                receiver,
                field,
                field_span,
                span,
                ..
            } => {
                let mut method_arguments = Vec::with_capacity(arguments.len() + 1);
                method_arguments.push(receiver.as_ref());
                method_arguments.extend_from_slice(arguments);
                if let Some(symbol) = self.graph.inherent_method_symbol(receiver, field) {
                    self.walk_method(&symbol, &method_arguments, root_call.or(Some(*field_span)));
                } else {
                    self.walk_callable_at(
                        *span,
                        SymbolNamespace::Value,
                        &method_arguments,
                        root_call,
                    );
                    self.walk_callable_at(
                        *field_span,
                        SymbolNamespace::Value,
                        &method_arguments,
                        root_call,
                    );
                }
            }
            SurfaceExpr::Application { .. } => {
                let (callee, mut applied_arguments) = flatten_application(expression);
                applied_arguments.extend_from_slice(arguments);
                self.walk_invoked_expr(callee, &applied_arguments, root_call)
            }
            _ => {}
        }
    }

    fn walk_method(
        &mut self,
        symbol: &str,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        if !self.active_methods.insert(symbol.to_owned()) {
            return;
        }
        if let Some(callable) = self.graph.method_callables.get(symbol).cloned() {
            self.walk_callable_body(&callable, arguments, root_call);
        }
        self.active_methods.remove(symbol);
    }

    fn walk_trait_dispatch(
        &mut self,
        expression: &'source SurfaceExpr,
        arguments: &[&'source SurfaceExpr],
        root_call: Option<ByteSpan>,
    ) {
        let (callee, _) = flatten_application(expression);
        let SurfaceExpr::Name { span, .. } = callee else {
            return;
        };
        let Some(target) = self.graph.target_at(*span, SymbolNamespace::Value) else {
            return;
        };
        if !self
            .graph
            .resolved
            .symbols
            .iter()
            .any(|symbol| symbol.id == target && symbol.kind == SymbolKind::TraitMethod)
        {
            return;
        }
        let Some((identity, method)) = self.graph.resolution.local_trait_dispatch(expression)
        else {
            return;
        };
        let key = format!("{identity}::{method}");
        if !self.active_methods.insert(key.clone()) {
            return;
        }
        if let Some(callable) = self
            .graph
            .instance_callables
            .get(&(identity, method))
            .cloned()
        {
            self.walk_callable_body(&callable, arguments, root_call.or(Some(expression.span())));
        }
        self.active_methods.remove(&key);
    }
}

fn flatten_application(expression: &SurfaceExpr) -> (&SurfaceExpr, Vec<&SurfaceExpr>) {
    let mut arguments = Vec::new();
    let mut callee = expression;
    while let SurfaceExpr::Application {
        function, argument, ..
    } = callee
    {
        arguments.push(argument.as_ref());
        callee = function.as_ref();
    }
    arguments.reverse();
    (callee, arguments)
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}
