//! Conservative value flow for records consumed by HTML styles. Scalar reads
//! do not carry record order; functions and containers can carry it unchanged.
use super::*;
use seseragi_lowering::{CoreComprehensionClause, CoreMonadDoStatement, CorePattern, SourceSpan};

type Origin = (String, usize, usize);
type Environment = BTreeMap<String, usize>;

pub(super) fn origin(span: &SourceSpan) -> Origin {
    (span.source.clone(), span.start, span.end)
}

fn carries(ty: &CoreType) -> bool {
    let name = match ty {
        CoreType::Named { name, .. } => name,
        CoreType::ExternalNamed { canonical, .. } => canonical,
        _ => return true,
    };
    !matches!(
        name.as_str(),
        "Unit"
            | "Int"
            | "Float"
            | "Byte"
            | "Char"
            | "Bool"
            | "String"
            | "std/prelude::Unit"
            | "std/prelude::Int"
            | "std/prelude::Float"
            | "std/prelude::Byte"
            | "std/prelude::Char"
            | "std/prelude::Bool"
            | "std/prelude::String"
            | "std/web/html::Style"
            | "std/web/html::WebUrl"
    )
}

#[derive(Default)]
struct Flow {
    edges: Vec<Vec<usize>>,
    globals: Environment,
    records: Vec<(Origin, usize)>,
}
impl Flow {
    fn node(&mut self) -> usize {
        let node = self.edges.len();
        self.edges.push(vec![]);
        node
    }
    fn global(&mut self, name: &str) -> usize {
        if let Some(node) = self.globals.get(name) {
            return *node;
        }
        let node = self.node();
        self.globals.insert(name.into(), node);
        node
    }
    fn name(&mut self, name: &str, env: &Environment, owner: &str) -> usize {
        if let Some(node) = env.get(name) {
            return *node;
        }
        self.global(&if name.contains("::") {
            name.into()
        } else {
            format!("{owner}::{name}")
        })
    }
    fn link(&mut self, a: Option<usize>, b: Option<usize>) {
        if let (Some(a), Some(b)) = (a, b) {
            self.edges[a].push(b);
            self.edges[b].push(a);
        }
    }
    fn evidence(&mut self, node: usize, evidence: &[CoreCallEvidence]) {
        for e in evidence {
            match &e.evidence {
                CoreInstanceEvidence::Local {
                    identity,
                    evidence_arguments,
                    ..
                }
                | CoreInstanceEvidence::Imported {
                    identity,
                    evidence_arguments,
                    ..
                } => {
                    let instance = self.global(&format!("instance:{identity}"));
                    self.link(Some(node), Some(instance));
                    self.evidence(node, evidence_arguments);
                }
                CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } => self.evidence(node, evidence_arguments),
                _ => {}
            }
        }
    }
    fn function(
        &mut self,
        node: usize,
        parameters: &[CoreParameter],
        body: &CoreExpr,
        outer: &Environment,
        owner: &str,
    ) {
        let mut env = outer.clone();
        for parameter in parameters {
            let argument = self.node();
            env.insert(parameter.id.clone(), argument);
            if carries(&parameter.type_ref) {
                self.link(Some(node), Some(argument));
            }
        }
        let result = self.expr(body, &env, owner);
        self.link(Some(node), result);
    }
    fn pattern(&mut self, pattern: &CorePattern, source: Option<usize>, env: &mut Environment) {
        match pattern {
            CorePattern::Binding { name, type_ref, .. } => {
                let node = self.node();
                env.insert(name.clone(), node);
                if carries(type_ref) {
                    self.link(Some(node), source);
                }
            }
            CorePattern::Constructor {
                argument: Some(argument),
                ..
            } => self.pattern(argument, source, env),
            CorePattern::Tuple { elements, .. } => {
                for p in elements {
                    self.pattern(p, source, env);
                }
            }
            CorePattern::Array { elements, rest, .. }
            | CorePattern::List { elements, rest, .. } => {
                for p in elements {
                    self.pattern(p, source, env);
                }
                if let Some(p) = rest {
                    self.pattern(p, source, env);
                }
            }
            CorePattern::Record { fields, .. } => {
                for field in fields {
                    self.pattern(&field.pattern, source, env);
                }
            }
            _ => {}
        }
    }
    fn expr(&mut self, expr: &CoreExpr, outer: &Environment, owner: &str) -> Option<usize> {
        let node = (matches!(expr, CoreExpr::Record { .. } | CoreExpr::Lambda { .. })
            || local::method_argument_type(expr).is_none_or(|ty| carries(&ty)))
        .then(|| self.node());
        match expr {
            CoreExpr::Variable { name, evidence, .. } => {
                let value = self.name(name, outer, owner);
                self.evidence(value, evidence);
                self.link(node, Some(value));
            }
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                ..
            }
            | CoreExpr::EffectInvoke {
                callee,
                arguments,
                evidence,
                ..
            } => {
                let function = self.name(callee, outer, owner);
                self.evidence(function, evidence);
                self.link(node, Some(function));
                for argument in arguments {
                    let argument = self.expr(argument, outer, owner);
                    self.link(Some(function), argument);
                }
            }
            CoreExpr::EffectOperation {
                operation,
                arguments,
                ..
            } => {
                let function = self.global(operation);
                self.link(node, Some(function));
                for argument in arguments {
                    let argument = self.expr(argument, outer, owner);
                    self.link(Some(function), argument);
                }
            }
            CoreExpr::Lambda {
                parameter, body, ..
            } => {
                self.function(
                    node.unwrap(),
                    std::slice::from_ref(parameter),
                    body,
                    outer,
                    owner,
                );
            }
            CoreExpr::Record {
                items,
                origin: span,
                ..
            } => {
                self.records.push((origin(span), node.unwrap()));
                for item in items {
                    let (CoreRecordValueItem::Field { value, .. }
                    | CoreRecordValueItem::Spread { value, .. }) = item;
                    let value = self.expr(value, outer, owner);
                    self.link(node, value);
                }
            }
            CoreExpr::Tuple { elements, .. }
            | CoreExpr::Array { elements, .. }
            | CoreExpr::List { elements, .. } => {
                for element in elements {
                    let element = self.expr(element, outer, owner);
                    self.link(node, element);
                }
            }
            CoreExpr::FieldAccess { receiver, .. }
            | CoreExpr::OptionalFieldAccess { receiver, .. } => {
                let receiver = self.expr(receiver, outer, owner);
                self.link(node, receiver);
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.expr(condition, outer, owner);
                for branch in [then_branch, else_branch] {
                    let branch = self.expr(branch, outer, owner);
                    self.link(node, branch);
                }
            }
            CoreExpr::Decision {
                scrutinee,
                branches,
                ..
            } => {
                let source = self.expr(scrutinee, outer, owner);
                for branch in branches {
                    let mut env = outer.clone();
                    for binding in &branch.bindings {
                        let value = self.node();
                        env.insert(binding.name.clone(), value);
                        if carries(&binding.type_ref) {
                            self.link(Some(value), source);
                        }
                    }
                    if let Some(guard) = &branch.guard {
                        self.expr(guard, &env, owner);
                    }
                    let value = self.expr(&branch.value, &env, owner);
                    self.link(node, value);
                }
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => {
                let mut env = outer.clone();
                let mut functions = BTreeMap::new();
                for (index, statement) in statements.iter().enumerate() {
                    match statement {
                        CoreStatement::Effect { value } => {
                            self.expr(value, &env, owner);
                        }
                        CoreStatement::PureLet { name, value, .. }
                        | CoreStatement::Bind { name, value, .. } => {
                            let value = self.expr(value, &env, owner);
                            let binding = self.node();
                            env.insert(name.clone(), binding);
                            self.link(Some(binding), value);
                        }
                        CoreStatement::LocalFunction {
                            name,
                            parameters,
                            body,
                            rec_group,
                            initializer_in_outer_scope,
                            ..
                        } => {
                            if *initializer_in_outer_scope {
                                let value = self.node();
                                self.function(value, parameters, body, &env, owner);
                                env.insert(name.clone(), value);
                            } else {
                                if let Some(group) = rec_group {
                                    for (member, declaration) in
                                        statements.iter().enumerate().skip(index)
                                    {
                                        let CoreStatement::LocalFunction {
                                            name,
                                            rec_group: Some(candidate),
                                            initializer_in_outer_scope: false,
                                            ..
                                        } = declaration
                                        else {
                                            break;
                                        };
                                        if candidate != group {
                                            break;
                                        }
                                        if let std::collections::btree_map::Entry::Vacant(entry) =
                                            functions.entry(member)
                                        {
                                            let value = self.node();
                                            entry.insert(value);
                                            env.insert(name.clone(), value);
                                        }
                                    }
                                }
                                let value = *functions.entry(index).or_insert_with(|| self.node());
                                env.insert(name.clone(), value);
                                self.function(value, parameters, body, &env, owner);
                            }
                        }
                    }
                }
                let result = self.expr(result, &env, owner);
                self.link(node, result);
            }
            CoreExpr::MonadDo {
                statements, result, ..
            } => {
                let mut env = outer.clone();
                for statement in statements {
                    match statement {
                        CoreMonadDoStatement::Expression { value } => {
                            self.expr(value, &env, owner);
                        }
                        CoreMonadDoStatement::PureLet { name, value, .. }
                        | CoreMonadDoStatement::Bind { name, value, .. } => {
                            let value = self.expr(value, &env, owner);
                            let binding = self.node();
                            env.insert(name.clone(), binding);
                            self.link(Some(binding), value);
                        }
                    }
                }
                let result = self.expr(result, &env, owner);
                self.link(node, result);
            }
            CoreExpr::ArrayComprehension {
                element, clauses, ..
            }
            | CoreExpr::ListComprehension {
                element, clauses, ..
            } => {
                let mut env = outer.clone();
                for clause in clauses {
                    match clause {
                        CoreComprehensionClause::Generator {
                            pattern, source, ..
                        } => {
                            let source = self.expr(source, &env, owner);
                            self.pattern(pattern, source, &mut env);
                        }
                        CoreComprehensionClause::Guard { condition, .. } => {
                            self.expr(condition, &env, owner);
                        }
                    }
                }
                let element = self.expr(element, &env, owner);
                self.link(node, element);
            }
            CoreExpr::Template { parts, .. } => {
                for part in parts {
                    if let CoreTemplatePart::Interpolation {
                        value, evidence, ..
                    } = part
                    {
                        let function = self.node();
                        if let Some(evidence) = evidence {
                            self.evidence(function, std::slice::from_ref(evidence));
                        }
                        let value = self.expr(value, outer, owner);
                        self.link(Some(function), value);
                    }
                }
            }
            CoreExpr::Binary {
                operator,
                left,
                right,
                evidence,
                ..
            } => {
                let function = self.global(&format!("operator:{operator}"));
                self.evidence(function, evidence);
                self.link(node, Some(function));
                for value in [left, right] {
                    let value = self.expr(value, outer, owner);
                    self.link(Some(function), value);
                }
            }
            CoreExpr::Unary {
                operator, operand, ..
            } => {
                let function = self.global(&format!("unary:{operator}"));
                self.link(node, Some(function));
                let value = self.expr(operand, outer, owner);
                self.link(Some(function), value);
            }
            CoreExpr::Unit { .. }
            | CoreExpr::Integer { .. }
            | CoreExpr::Float64 { .. }
            | CoreExpr::Char { .. }
            | CoreExpr::String { .. }
            | CoreExpr::Boolean { .. } => {}
        }
        node
    }
}

pub(super) fn ordered_records(
    module: &CoreModule,
    owners: &BTreeMap<String, String>,
) -> BTreeSet<Origin> {
    let mut flow = Flow::default();
    for binding in &module.bindings {
        let node = flow.global(&binding.symbol);
        let value = flow.expr(
            &binding.value,
            &Environment::new(),
            &owners[&binding.symbol],
        );
        flow.link(Some(node), value);
    }
    for function in &module.functions {
        let node = flow.global(&function.symbol);
        flow.function(
            node,
            &function.parameters,
            &function.body,
            &Environment::new(),
            &owners[&function.symbol],
        );
    }
    for instance in &module.instances {
        if let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation {
            let node = flow.global(&format!("instance:{}", instance.identity));
            for method in methods {
                let owner = owners
                    .get(&instance.identity)
                    .map(String::as_str)
                    .unwrap_or(&module.module);
                flow.function(
                    node,
                    &method.parameters,
                    &method.body,
                    &Environment::new(),
                    owner,
                );
            }
        }
    }
    let Some(sink) = flow.globals.get("std/web/html::style").copied() else {
        return BTreeSet::new();
    };
    let mut visited = BTreeSet::new();
    let mut pending = vec![sink];
    while let Some(node) = pending.pop() {
        if visited.insert(node) {
            pending.extend(flow.edges[node].iter().copied());
        }
    }
    flow.records
        .into_iter()
        .filter_map(|(origin, node)| visited.contains(&node).then_some(origin))
        .collect()
}
