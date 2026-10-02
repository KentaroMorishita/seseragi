//! Rank-1 inference for non-recursive lambda initializers. Inference variables
//! live only in this pass; the ordinary checker validates the inferred type and
//! selects required dictionaries before a binding acquires a scheme.
use super::PureExpressionContext;
use crate::typed::functions::{substitute_type_parameters, TopLevelPureFunction};
use crate::typed::semantic_types::SemanticTypeKey;
use crate::typed::type_ref::{application_argument_type_from_expr, typed_type_contains_hole};
use crate::{SymbolId, TypedConstraint, TypedRecordField, TypedType};
use seseragi_syntax::{
    SurfaceBlockItem, SurfaceExpr, SurfacePattern, SurfaceRecordItem, TypeParameter,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
struct Scheme {
    value: TypedType,
    quantified: BTreeSet<String>,
    constraints: Vec<(TypedConstraint, Option<String>)>,
}
#[derive(Clone)]
pub(crate) struct InferredLambda {
    pub(crate) value: TypedType,
    arity: usize,
    pub(crate) parameters: Vec<TypeParameter>,
    pub(crate) constraints: Vec<TypedConstraint>,
    pub(crate) identities: Vec<Option<String>>,
}
impl InferredLambda {
    pub(crate) fn callable(
        &self,
        symbol: String,
        context: &PureExpressionContext<'_>,
    ) -> TopLevelPureFunction {
        let mut result = self.value.clone();
        let mut parameters = Vec::new();
        for _ in 0..self.arity {
            let TypedType::Function {
                parameter,
                result: next,
            } = result
            else {
                break;
            };
            parameters.push(*parameter);
            result = *next;
        }
        TopLevelPureFunction {
            symbol,
            trait_identity: None,
            trait_method: None,
            type_parameters: self.parameters.clone(),
            constraints: self.constraints.clone(),
            constraint_identities: self.identities.clone(),
            semantic_parameters: parameters
                .iter()
                .map(|value| context.semantic_value_from_typed_type(value).key)
                .collect(),
            parameters,
            semantic_result: context.semantic_value_from_typed_type(&result).key,
            result,
        }
    }
}
pub(crate) fn infer(
    expression: &SurfaceExpr,
    context: &PureExpressionContext<'_>,
) -> Option<InferredLambda> {
    let mut ungrouped = expression;
    while let SurfaceExpr::Grouped { value, .. } = ungrouped {
        ungrouped = value;
    }
    if !matches!(ungrouped, SurfaceExpr::Lambda { .. }) {
        if matches!(
            ungrouped,
            SurfaceExpr::Name {
                type_arguments: None,
                ..
            } | SurfaceExpr::Member {
                type_arguments: None,
                ..
            }
        ) {
            let mut signature = context
                .target(ungrouped.span())
                .and_then(|target| context.callable_value(target))?;
            if !signature.type_parameters.is_empty() || !signature.constraints.is_empty() {
                if context.is_recursive_member(ungrouped.span()) {
                    signature.type_parameters.clear();
                }
                return Some(InferredLambda {
                    value: signature
                        .parameters
                        .iter()
                        .rev()
                        .fold(signature.result.clone(), |result, parameter| {
                            function(parameter.clone(), result)
                        }),
                    arity: signature.parameters.len(),
                    parameters: signature.type_parameters,
                    constraints: signature.constraints,
                    identities: signature.constraint_identities,
                });
            }
        }
        return None;
    }
    let mut arity = 0;
    let mut body = ungrouped;
    loop {
        match body {
            SurfaceExpr::Lambda { body: inner, .. } => {
                arity += 1;
                body = inner;
            }
            SurfaceExpr::Grouped { value, .. } => body = value,
            _ => break,
        }
    }
    let mut inference = Inference {
        prefix: expression.span().start,
        next: 0,
        substitutions: BTreeMap::new(),
        obligations: Vec::new(),
        context,
    };
    let environment = inference_environment(context);
    let value = inference.expression(expression, &environment)?;
    inference.solve_outputs()?;
    let value = inference.resolve(&value);
    let mut quantified = inference.generalize(&value, &environment);
    let mut constraints = Vec::new();
    let mut identities = Vec::new();
    for (constraint, identity) in &inference.obligations {
        let constraint = TypedConstraint {
            name: constraint.name.clone(),
            arguments: constraint
                .arguments
                .iter()
                .map(|value| inference.resolve(value))
                .collect(),
        };
        let selected = context.select_call_evidence(
            std::slice::from_ref(&constraint),
            std::slice::from_ref(identity),
        );
        // Concrete dictionaries stay concrete. Captured evidence becomes a
        // monomorphic scheme obligation; only environment-free variables are
        // quantified. This also keeps nested dictionary indexes unambiguous.
        if selected.as_ref().is_ok_and(|evidence| {
            evidence.iter().all(|item| {
                !matches!(
                    item.evidence,
                    crate::TypedInstanceEvidence::Parameter { .. }
                )
            })
        }) {
            continue;
        }
        let free = constraint
            .arguments
            .iter()
            .flat_map(|value| inference.generalize(value, &environment))
            .collect::<BTreeSet<_>>();
        if selected.is_err() && free.is_empty() {
            return None;
        }
        quantified.extend(free);
        if !constraints
            .iter()
            .zip(&identities)
            .any(|(existing, existing_identity)| {
                existing == &constraint && existing_identity == identity
            })
        {
            constraints.push(constraint);
            identities.push(identity.clone());
        }
    }
    let substitutions = quantified
        .iter()
        .enumerate()
        .map(|(index, variable)| {
            let mut name = format!("Let{}T{}", expression.span().start, index);
            while context
                .resolution
                .resolved()
                .symbols
                .iter()
                .any(|symbol| symbol.spelling == name)
            {
                name.push('_');
            }
            (variable.clone(), named(&name))
        })
        .collect::<BTreeMap<_, _>>();
    Some(InferredLambda {
        arity,
        constraints: constraints
            .into_iter()
            .map(|constraint| TypedConstraint {
                name: constraint.name,
                arguments: constraint
                    .arguments
                    .iter()
                    .map(|value| substitute_type_parameters(value, &substitutions))
                    .collect(),
            })
            .collect(),
        identities,
        value: substitute_type_parameters(&value, &substitutions),
        parameters: substitutions
            .values()
            .map(|value| match value {
                TypedType::Named { name, .. } => TypeParameter::value(name.clone()),
                _ => unreachable!(),
            })
            .collect(),
    })
}
/// Solve all argument equations together before the bidirectional checker
/// commits to a callback expectation. These variables are never generalized:
/// lambda parameters and captured values remain monomorphic, while each named
/// scheme reference receives its own fresh instantiation.
pub(super) fn infer_call_arguments(
    signature: &TopLevelPureFunction,
    arguments: &[&SurfaceExpr],
    expected: Option<&crate::typed::semantic_types::SemanticValueType>,
    context: &PureExpressionContext<'_>,
) -> Option<Vec<Option<crate::typed::semantic_types::SemanticValueType>>> {
    if !arguments.iter().any(|argument| {
        let mut argument = *argument;
        while let SurfaceExpr::Grouped { value, .. } = argument {
            argument = value;
        }
        matches!(argument, SurfaceExpr::Lambda { .. })
            || context
                .target(argument.span())
                .and_then(|target| context.callable_value(target))
                .is_some_and(|callable| !callable.type_parameters.is_empty())
    }) {
        return None;
    }
    let environment = inference_environment(context);
    let mut inference = Inference {
        prefix: arguments.first()?.span().start,
        next: 0,
        substitutions: BTreeMap::new(),
        obligations: Vec::new(),
        context,
    };
    let mut result = inference.signature(signature);
    let mut inferred = Vec::new();
    for argument in arguments {
        let actual = inference.expression(argument, &environment)?;
        let next = inference.fresh();
        inference.unify(&result, &function(actual.clone(), next.clone()))?;
        inferred.push(actual);
        result = next;
    }
    if let Some(expected) =
        expected.filter(|expected| !typed_type_contains_hole(&expected.type_ref))
    {
        inference.unify(&result, &expected.type_ref)?;
    }
    inference.solve_outputs()?;
    Some(
        inferred
            .into_iter()
            .map(|value| {
                let value = inference.resolve(&value);
                (variables(&value).is_empty() && !typed_type_contains_hole(&value))
                    .then(|| context.semantic_value_from_typed_type(&value))
            })
            .collect(),
    )
}

fn inference_environment(context: &PureExpressionContext<'_>) -> BTreeMap<SymbolId, Scheme> {
    context
        .parameters
        .iter()
        .map(|(symbol, value)| {
            let quantified = context
                .callable(*symbol)
                .map(|callable| {
                    callable
                        .type_parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect()
                })
                .unwrap_or_default();
            let constraints = context
                .callable(*symbol)
                .map(|callable| {
                    callable
                        .constraints
                        .iter()
                        .cloned()
                        .zip(callable.constraint_identities.iter().cloned())
                        .collect()
                })
                .unwrap_or_default();
            (
                *symbol,
                Scheme {
                    value: value.type_ref.clone(),
                    quantified,
                    constraints,
                },
            )
        })
        .collect()
}

struct Inference<'a, 'b> {
    prefix: usize,
    next: usize,
    substitutions: BTreeMap<String, TypedType>,
    obligations: Vec<(TypedConstraint, Option<String>)>,
    context: &'a PureExpressionContext<'b>,
}
impl Inference<'_, '_> {
    fn solve_outputs(&mut self) -> Option<()> {
        for _ in 0..self.obligations.len() {
            let before = self.substitutions.clone();
            for (constraint, _) in self.obligations.clone() {
                if let [left, right, output] = constraint.arguments.as_slice() {
                    if let Ok((actual, _)) = self.context.select_binary_operator_evidence(
                        &constraint.name,
                        self.resolve(left),
                        self.resolve(right),
                    ) {
                        self.unify(output, &actual)?;
                    }
                }
            }
            if self.substitutions == before {
                break;
            }
        }
        Some(())
    }

    // The question-mark prefix cannot name a source type or type parameter.
    fn fresh(&mut self) -> TypedType {
        let value = named(&format!("?let{}_{}", self.prefix, self.next));
        self.next += 1;
        value
    }
    fn resolve(&self, value: &TypedType) -> TypedType {
        if let Some(variable) = variable(value) {
            if let Some(replacement) = self.substitutions.get(variable) {
                return self.resolve(replacement);
            }
        }
        map_children(value, |child| self.resolve(child))
    }
    fn unify(&mut self, left: &TypedType, right: &TypedType) -> Option<()> {
        let left = self.resolve(left);
        let right = self.resolve(right);
        if left == right {
            return Some(());
        }
        if let Some(name) = variable(&left).or_else(|| variable(&right)) {
            let other = if variable(&left).is_some() {
                &right
            } else {
                &left
            };
            if variables(other).contains(name) {
                return None;
            }
            self.substitutions.insert(name.to_owned(), other.clone());
            return Some(());
        }
        match (&left, &right) {
            (
                TypedType::Function {
                    parameter: lp,
                    result: lr,
                },
                TypedType::Function {
                    parameter: rp,
                    result: rr,
                },
            ) => {
                self.unify(lp, rp)?;
                self.unify(lr, rr)
            }
            (
                TypedType::Named {
                    name: ln,
                    arguments: la,
                },
                TypedType::Named {
                    name: rn,
                    arguments: ra,
                },
            ) if ln == rn => self.unify_many(la, ra),
            (
                TypedType::ExternalNamed {
                    canonical: ln,
                    arguments: la,
                    ..
                },
                TypedType::ExternalNamed {
                    canonical: rn,
                    arguments: ra,
                    ..
                },
            ) if ln == rn => self.unify_many(la, ra),
            (TypedType::Tuple { elements: le }, TypedType::Tuple { elements: re }) => {
                self.unify_many(le, re)
            }
            (
                TypedType::Record {
                    fields: lf,
                    closed: lc,
                },
                TypedType::Record {
                    fields: rf,
                    closed: rc,
                },
            ) if lc == rc && lf.len() == rf.len() => {
                for l in lf {
                    let r = rf
                        .iter()
                        .find(|r| r.name == l.name && r.optional == l.optional)?;
                    self.unify(&l.type_ref, &r.type_ref)?;
                }
                Some(())
            }
            _ => None,
        }
    }
    fn unify_many(&mut self, left: &[TypedType], right: &[TypedType]) -> Option<()> {
        if left.len() != right.len() {
            return None;
        }
        for (left, right) in left.iter().zip(right) {
            self.unify(left, right)?;
        }
        Some(())
    }
    fn generalize(
        &self,
        value: &TypedType,
        environment: &BTreeMap<SymbolId, Scheme>,
    ) -> BTreeSet<String> {
        let mut free = variables(&self.resolve(value));
        for scheme in environment.values() {
            for variable in variables(&self.resolve(&scheme.value)).difference(&scheme.quantified) {
                free.remove(variable);
            }
        }
        free
    }
    fn instantiate(&mut self, scheme: &Scheme) -> TypedType {
        let substitutions = scheme
            .quantified
            .iter()
            .map(|name| (name.clone(), self.fresh()))
            .collect();
        for (constraint, identity) in &scheme.constraints {
            self.obligations.push((
                TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(|value| {
                            substitute_type_parameters(&self.resolve(value), &substitutions)
                        })
                        .collect(),
                },
                identity.clone(),
            ));
        }
        substitute_type_parameters(&self.resolve(&scheme.value), &substitutions)
    }
    fn signature(&mut self, signature: &TopLevelPureFunction) -> TypedType {
        let substitutions = signature
            .type_parameters
            .iter()
            .map(|parameter| (parameter.name.clone(), self.fresh()))
            .collect::<BTreeMap<_, _>>();
        for (index, constraint) in signature.constraints.iter().enumerate() {
            self.obligations.push((
                TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(|value| substitute_type_parameters(value, &substitutions))
                        .collect(),
                },
                signature
                    .constraint_identities
                    .get(index)
                    .cloned()
                    .flatten(),
            ));
        }
        signature.parameters.iter().rev().fold(
            substitute_type_parameters(&signature.result, &substitutions),
            |result, parameter| {
                function(
                    substitute_type_parameters(parameter, &substitutions),
                    result,
                )
            },
        )
    }
    fn expression(
        &mut self,
        expression: &SurfaceExpr,
        environment: &BTreeMap<SymbolId, Scheme>,
    ) -> Option<TypedType> {
        match expression {
            SurfaceExpr::Grouped { value, .. } => self.expression(value, environment),
            SurfaceExpr::Lambda {
                parameter, body, ..
            } => {
                let parameter_type = parameter
                    .type_ref
                    .as_ref()
                    .map(|value| self.context.semantic_value_from_type_ref(value).type_ref)
                    .unwrap_or_else(|| self.fresh());
                let mut environment = environment.clone();
                if let Some(symbol) = self.context.lambda_parameter_symbol(parameter.name_span) {
                    environment.insert(
                        symbol,
                        Scheme {
                            value: parameter_type.clone(),
                            quantified: BTreeSet::new(),
                            constraints: Vec::new(),
                        },
                    );
                } else if parameter.name != "_" {
                    return None;
                }
                let result = self.expression(body, &environment)?;
                Some(function(
                    self.resolve(&parameter_type),
                    self.resolve(&result),
                ))
            }
            SurfaceExpr::Name {
                span,
                type_arguments: None,
                ..
            } => {
                let target = self
                    .context
                    .target(*span)
                    .or_else(|| self.context.operator_target(*span))?;
                if self.context.is_recursive_member(*span) {
                    let mut signature = self.context.callable_value(target)?;
                    signature.type_parameters.clear();
                    return Some(self.signature(&signature));
                }
                if let Some(scheme) = environment.get(&target) {
                    return Some(self.instantiate(scheme));
                }
                if let Some(signature) = self.context.callable_value(target) {
                    return Some(self.signature(&signature));
                }
                self.context
                    .resolution
                    .top_level_value_type(target)
                    .cloned()
                    .filter(|value| !typed_type_contains_hole(value))
            }
            SurfaceExpr::Application {
                function: callee,
                argument,
                ..
            } => {
                let callee = self.expression(callee, environment)?;
                let argument = self.expression(argument, environment)?;
                let result = self.fresh();
                self.unify(&callee, &function(argument, result.clone()))?;
                Some(self.resolve(&result))
            }
            SurfaceExpr::Tuple { elements, .. } => Some(TypedType::Tuple {
                elements: elements
                    .iter()
                    .map(|value| self.expression(value, environment))
                    .collect::<Option<_>>()?,
            }),
            SurfaceExpr::Array { elements, .. } | SurfaceExpr::List { elements, .. } => {
                let element = self.fresh();
                for value in elements {
                    let value = self.expression(value, environment)?;
                    self.unify(&element, &value)?;
                }
                Some(TypedType::Named {
                    name: if matches!(expression, SurfaceExpr::Array { .. }) {
                        "Array"
                    } else {
                        "List"
                    }
                    .to_owned(),
                    arguments: vec![self.resolve(&element)],
                })
            }
            SurfaceExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition = self.expression(condition, environment)?;
                self.unify(&condition, &named("Bool"))?;
                let yes = self.expression(then_branch, environment)?;
                let no = self.expression(else_branch, environment)?;
                self.unify(&yes, &no)?;
                Some(self.resolve(&yes))
            }
            SurfaceExpr::Record { items, .. } => {
                let mut fields = Vec::new();
                for item in items {
                    match item {
                        SurfaceRecordItem::Field { name, value, .. } => {
                            fields.push(TypedRecordField {
                                name: name.clone(),
                                optional: false,
                                type_ref: self.expression(value, environment)?,
                            })
                        }
                        SurfaceRecordItem::Spread { value, .. } => {
                            let value = self.expression(value, environment)?;
                            let TypedType::Record { fields: spread, .. } = self.resolve(&value)
                            else {
                                return None;
                            };
                            fields.extend(spread);
                        }
                    }
                }
                Some(TypedType::Record {
                    closed: true,
                    fields,
                })
            }
            SurfaceExpr::Member {
                receiver,
                field,
                span,
                ..
            } => {
                if let Some(signature) = self
                    .context
                    .target(*span)
                    .and_then(|target| self.context.callable_value(target))
                {
                    return Some(self.signature(&signature));
                }
                let receiver = self.expression(receiver, environment)?;
                match self.resolve(&receiver) {
                    TypedType::Record { fields, .. } => fields
                        .into_iter()
                        .find(|item| item.name == *field)
                        .map(|item| item.type_ref),
                    TypedType::Tuple { elements } => field
                        .parse::<usize>()
                        .ok()
                        .and_then(|index| elements.get(index).cloned()),
                    _ => None,
                }
            }
            SurfaceExpr::Index {
                receiver, index, ..
            } => {
                let receiver = self.expression(receiver, environment)?;
                let index = self.expression(index, environment)?;
                self.unify(&index, &named("Int"))?;
                let element = self.fresh();
                self.unify(
                    &receiver,
                    &TypedType::Named {
                        name: "Array".to_owned(),
                        arguments: vec![element.clone()],
                    },
                )?;
                Some(TypedType::Named {
                    name: "Maybe".to_owned(),
                    arguments: vec![self.resolve(&element)],
                })
            }
            SurfaceExpr::Block { items, result, .. } => {
                let mut environment = environment.clone();
                for item in items {
                    let SurfaceBlockItem::Let {
                        pattern: SurfacePattern::Name { name_span, .. },
                        type_ref,
                        value,
                        ..
                    } = item
                    else {
                        return None;
                    };
                    let obligation_start = self.obligations.len();
                    let inferred = self.expression(value, &environment)?;
                    let constraints = self.obligations.split_off(obligation_start);
                    if let Some(annotation) = type_ref {
                        self.unify(
                            &inferred,
                            &self
                                .context
                                .semantic_value_from_type_ref(annotation)
                                .type_ref,
                        )?;
                    }
                    let value = self.resolve(&inferred);
                    let mut quantified = self.generalize(&value, &environment);
                    for (constraint, _) in &constraints {
                        for argument in &constraint.arguments {
                            quantified.extend(self.generalize(argument, &environment));
                        }
                    }
                    // Obligations owned by the surrounding environment cannot
                    // disappear when a local binding is unused. Only constraints
                    // depending on this binding's quantified variables travel
                    // exclusively with its scheme.
                    for (constraint, identity) in &constraints {
                        let depends_on_quantified = constraint.arguments.iter().any(|argument| {
                            variables(&self.resolve(argument))
                                .iter()
                                .any(|variable| quantified.contains(variable))
                        });
                        if !depends_on_quantified {
                            self.obligations
                                .push((constraint.clone(), identity.clone()));
                        }
                    }
                    environment.insert(
                        self.context.binding_symbol(*name_span)?,
                        Scheme {
                            value,
                            quantified,
                            constraints,
                        },
                    );
                }
                self.expression(result, &environment)
            }
            SurfaceExpr::Binary {
                operator,
                operator_span,
                left,
                right,
                ..
            } => {
                if let Some(standard) = seseragi_syntax::standard_operator(operator) {
                    let left = self.expression(left, environment)?;
                    let right = self.expression(right, environment)?;
                    let arithmetic =
                        standard.kind == seseragi_syntax::StandardOperatorKind::Arithmetic;
                    let result = if arithmetic {
                        self.fresh()
                    } else {
                        self.unify(&left, &right)?;
                        named("Bool")
                    };
                    let arguments = if arithmetic {
                        vec![left, right, result.clone()]
                    } else {
                        vec![left]
                    };
                    self.obligations.push((
                        TypedConstraint {
                            name: standard.trait_name.to_owned(),
                            arguments,
                        },
                        self.context.trait_identity(standard.trait_name),
                    ));
                    return Some(result);
                }
                let signature = self
                    .context
                    .operator_target(*operator_span)
                    .and_then(|target| self.context.callable_value(target))?;
                let operator = self.signature(&signature);
                let left = self.expression(left, environment)?;
                let right = self.expression(right, environment)?;
                let result = self.fresh();
                self.unify(&operator, &function(left, function(right, result.clone())))?;
                Some(self.resolve(&result))
            }
            _ => {
                // Reuse established typing for closed expressions. Never hide
                // unresolved local inference variables behind a recovery hole.
                let locals = environment
                    .iter()
                    .map(|(symbol, scheme)| {
                        (
                            *symbol,
                            self.context
                                .semantic_value_from_typed_type(&self.resolve(&scheme.value)),
                        )
                    })
                    .collect();
                let analysis = super::type_surface_expression(
                    expression,
                    &self.context.with_locals(locals).without_expected(),
                );
                let value = application_argument_type_from_expr(&analysis.value);
                (!typed_type_contains_hole(&value)
                    && analysis.semantic_type != SemanticTypeKey::Invalid
                    && analysis.pure_call_issue.is_none())
                .then_some(value)
            }
        }
    }
}
fn named(name: &str) -> TypedType {
    TypedType::Named {
        name: name.to_owned(),
        arguments: Vec::new(),
    }
}
fn function(parameter: TypedType, result: TypedType) -> TypedType {
    TypedType::Function {
        parameter: Box::new(parameter),
        result: Box::new(result),
    }
}
fn variable(value: &TypedType) -> Option<&str> {
    match value {
        TypedType::Named { name, arguments }
            if name.starts_with("?let") && arguments.is_empty() =>
        {
            Some(name)
        }
        _ => None,
    }
}
fn variables(value: &TypedType) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    if let Some(variable) = variable(value) {
        result.insert(variable.to_owned());
    }
    map_children(value, |child| {
        result.extend(variables(child));
        child.clone()
    });
    result
}
fn map_children(value: &TypedType, mut map: impl FnMut(&TypedType) -> TypedType) -> TypedType {
    match value {
        TypedType::Named { name, arguments } => TypedType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(map).collect(),
        },
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypedType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: arguments.iter().map(map).collect(),
        },
        TypedType::Function { parameter, result } => function(map(parameter), map(result)),
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements.iter().map(map).collect(),
        },
        TypedType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| TypedRecordField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: map(&field.type_ref),
                })
                .collect(),
        },
        TypedType::RequirementMerge { operands } => TypedType::RequirementMerge {
            operands: operands.iter().map(map).collect(),
        },
        TypedType::Hole => TypedType::Hole,
    }
}
