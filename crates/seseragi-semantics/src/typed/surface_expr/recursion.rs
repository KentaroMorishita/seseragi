//! A rec group has one substitution environment while its bodies are checked.
//! Calls outside the group continue to instantiate the ordinary function scheme.
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use super::{PureCallIssue, PureExpressionContext, TopLevelPureFunction};
use crate::{SymbolId, TypedType};
use seseragi_syntax::ByteSpan;

type Variable = (SymbolId, String);
#[derive(Clone, Debug, PartialEq)]
enum Term {
    Variable(Variable),
    Shape(String, Vec<Term>),
    Hole,
}

#[derive(Clone)]
pub(super) struct RecursiveContext {
    members: Rc<BTreeMap<SymbolId, TopLevelPureFunction>>,
    substitutions: Rc<RefCell<BTreeMap<Variable, Term>>>,
    caller: SymbolId,
}

impl RecursiveContext {
    pub(super) fn new(members: BTreeMap<SymbolId, TopLevelPureFunction>, caller: SymbolId) -> Self {
        Self {
            members: Rc::new(members),
            substitutions: Rc::default(),
            caller,
        }
    }
    pub(super) fn for_member(&self, caller: SymbolId) -> Self {
        Self {
            caller,
            ..self.clone()
        }
    }
    fn variables(&self, member: SymbolId) -> BTreeMap<String, Variable> {
        self.members[&member]
            .type_parameters
            .iter()
            .map(|p| (p.name.clone(), (member, p.name.clone())))
            .collect()
    }
    pub(super) fn check(
        &self,
        target: SymbolId,
        arguments: &[TypedType],
        result: &TypedType,
    ) -> bool {
        let Some(signature) = self.members.get(&target) else {
            return true;
        };
        if signature.type_parameters.is_empty() {
            return true;
        }
        let callee_variables = self.variables(target);
        let caller_variables = self.variables(self.caller);
        let mut substitutions = self.substitutions.borrow_mut();
        for (template, actual) in signature.parameters.iter().zip(arguments) {
            if !unify(
                term(template, &callee_variables),
                term(actual, &caller_variables),
                &mut substitutions,
            ) {
                return false;
            }
        }
        let remaining = signature
            .parameters
            .iter()
            .skip(arguments.len())
            .rev()
            .fold(signature.result.clone(), |result, parameter| {
                TypedType::Function {
                    parameter: Box::new(parameter.clone()),
                    result: Box::new(result),
                }
            });
        unify(
            term(&remaining, &callee_variables),
            term(result, &caller_variables),
            &mut substitutions,
        )
    }
}

impl PureExpressionContext<'_> {
    pub(super) fn is_recursive_member(&self, span: ByteSpan) -> bool {
        self.target(span).is_some_and(|target| {
            self.recursive_groups
                .iter()
                .any(|group| group.members.contains_key(&target))
        })
    }
    pub(super) fn recursive_call_issue(
        &self,
        callee: ByteSpan,
        arguments: &[TypedType],
        result: &TypedType,
    ) -> Option<PureCallIssue> {
        let target = self.target(callee)?;
        self.recursive_groups
            .iter()
            .any(|group| !group.check(target, arguments, result))
            .then_some(PureCallIssue::PolymorphicRecursion { callee })
    }
}

fn term(value: &TypedType, variables: &BTreeMap<String, Variable>) -> Term {
    match value {
        TypedType::Hole => Term::Hole,
        TypedType::Named { name, arguments } => {
            let head = variables.get(name).map(|v| Term::Variable(v.clone()));
            if arguments.is_empty() {
                return head.unwrap_or_else(|| Term::Shape(name.clone(), vec![]));
            }
            let mut args: Vec<_> = arguments.iter().map(|v| term(v, variables)).collect();
            if let Some(head) = head {
                args.insert(0, head);
                Term::Shape("apply".into(), args)
            } else {
                Term::Shape(name.clone(), args)
            }
        }
        TypedType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => Term::Shape(
            canonical.clone(),
            arguments.iter().map(|v| term(v, variables)).collect(),
        ),
        TypedType::Function { parameter, result } => Term::Shape(
            "function".into(),
            vec![term(parameter, variables), term(result, variables)],
        ),
        TypedType::Tuple { elements } => Term::Shape(
            "tuple".into(),
            elements.iter().map(|v| term(v, variables)).collect(),
        ),
        TypedType::RequirementMerge { operands } => Term::Shape(
            "requirements".into(),
            operands.iter().map(|v| term(v, variables)).collect(),
        ),
        TypedType::Record { fields, .. } => {
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by_key(|f| &f.name);
            Term::Shape(
                "record".into(),
                fields
                    .iter()
                    .map(|f| Term::Shape(f.name.clone(), vec![term(&f.type_ref, variables)]))
                    .collect(),
            )
        }
    }
}
fn resolve(value: Term, substitutions: &BTreeMap<Variable, Term>) -> Term {
    if let Term::Variable(variable) = &value {
        if let Some(next) = substitutions.get(variable) {
            return resolve(next.clone(), substitutions);
        }
    }
    value
}
fn unify(left: Term, right: Term, substitutions: &mut BTreeMap<Variable, Term>) -> bool {
    let left = resolve(left, substitutions);
    let right = resolve(right, substitutions);
    if left == right || matches!(left, Term::Hole) || matches!(right, Term::Hole) {
        return true;
    }
    match (left, right) {
        (Term::Variable(variable), Term::Variable(other)) => {
            // Explicit binders remain universally quantified after the group.
            // Unify alpha-renamed member binders, but never specialize them or
            // equate two independent binders belonging to the same member.
            let mut members = BTreeMap::new();
            for candidate in substitutions.keys().chain([&variable, &other]) {
                let resolved = resolve(Term::Variable(candidate.clone()), substitutions);
                if resolved == Term::Variable(variable.clone())
                    || resolved == Term::Variable(other.clone())
                {
                    if let Some(previous) = members.insert(candidate.0, &candidate.1) {
                        if previous != &candidate.1 {
                            return false;
                        }
                    }
                }
            }
            substitutions.insert(variable, Term::Variable(other));
            true
        }
        (Term::Shape(left, a), Term::Shape(right, b)) if left == "record" && right == "record" => {
            a.into_iter().all(|field| {
                let Term::Shape(name, arguments) = field else {
                    return false;
                };
                b.iter()
                    .find_map(|candidate| match candidate {
                        Term::Shape(other, values) if *other == name => Some(values),
                        _ => None,
                    })
                    .is_some_and(|values| {
                        arguments
                            .into_iter()
                            .zip(values.iter().cloned())
                            .all(|(a, b)| unify(a, b, substitutions))
                    })
            })
        }
        (Term::Shape(left, a), Term::Shape(right, b)) => {
            left == right
                && a.len() == b.len()
                && a.into_iter()
                    .zip(b)
                    .all(|(a, b)| unify(a, b, substitutions))
        }
        _ => false,
    }
}
