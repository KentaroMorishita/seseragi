//! Coherence unifies checked heads, independently of dictionary selection.
//! Constraints do not make overlapping heads disjoint: there is no instance
//! priority, and later instances can satisfy a currently missing constraint.
use super::{
    aliases, interface,
    model::{ContractType, TypeIdentity},
    resolution,
};
use crate::{ResolvedDependencyInstance, ResolvedModule, SymbolNamespace, TypedInstance};
use seseragi_syntax::{InterfaceType, SurfaceDecl, TypeParameter};
use std::collections::BTreeMap;

pub(crate) struct CoherenceHead {
    arguments: Vec<ContractType>,
    parameters: Vec<TypeParameter>,
}

pub(crate) fn local_head(
    resolved: &ResolvedModule,
    instance: &TypedInstance,
) -> Option<CoherenceHead> {
    let declaration = resolved
        .declarations
        .iter()
        .find(|declaration| declaration.span() == instance.origin)?;
    let arguments = if let SurfaceDecl::Instance {
        arguments,
        type_parameters,
        span,
        ..
    } = declaration
    {
        let symbols = resolution::declaration_type_parameters(resolved, *span, type_parameters)?;
        let binders = symbols
            .into_iter()
            .enumerate()
            .map(|(index, symbol)| (symbol, index as u32))
            .collect();
        arguments
            .iter()
            .map(|argument| {
                let argument =
                    resolution::contract_type(resolved, argument, &binders, &BTreeMap::new())?;
                aliases::normalize_type(resolved, argument, &mut Default::default())
            })
            .collect::<Option<Vec<_>>>()?
    } else {
        // Derived instances have exactly the declaration's nominal head and
        // parameters. Never infer its owner from a display name or payload.
        let name_span = match declaration {
            SurfaceDecl::Type { name_span, .. }
            | SurfaceDecl::Newtype { name_span, .. }
            | SurfaceDecl::Struct { name_span, .. } => *name_span,
            _ => return None,
        };
        let owner = resolved
            .symbols
            .iter()
            .find(|symbol| symbol.namespace == SymbolNamespace::Type && symbol.origin == name_span)?
            .canonical
            .clone()?;
        let constructor = ContractType::Named(TypeIdentity::Canonical(owner));
        vec![super::model::apply_arguments(
            constructor,
            (0..instance.type_parameters.len())
                .map(|index| ContractType::Binder(index as u32))
                .collect(),
        )?]
    };
    Some(CoherenceHead {
        arguments,
        parameters: instance.type_parameters.clone(),
    })
}

pub(crate) fn dependency_head(instance: &ResolvedDependencyInstance) -> Option<CoherenceHead> {
    let InterfaceType::Apply { arguments, .. } = &instance.head else {
        return None;
    };
    let binders = instance
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.name.clone(), index as u32))
        .collect();
    let arguments = arguments
        .iter()
        .map(|argument| interface::contract_type(argument, &binders, &BTreeMap::new(), &[]))
        .collect::<Option<_>>()?;
    Some(CoherenceHead {
        arguments,
        parameters: instance.type_parameters.clone(),
    })
}

impl CoherenceHead {
    pub(crate) fn conflicts(&self, other: &Self, trait_identity: &str) -> bool {
        if self.arguments.len() != other.arguments.len() {
            return false;
        }
        let count = crate::prelude::determining_instance_arguments(trait_identity)
            .unwrap_or(self.arguments.len());
        if count > self.arguments.len() {
            return false;
        }
        let mut substitutions = BTreeMap::new();
        self.arguments
            .iter()
            .zip(&other.arguments)
            .take(count)
            .all(|(left, right)| {
                let Some(left) = term(left, &self.parameters, false) else {
                    return false;
                };
                let Some(right) = term(right, &other.parameters, true) else {
                    return false;
                };
                unify(left, right, &mut substitutions)
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Term {
    Variable {
        side: bool,
        index: u32,
        arity: u32,
    },
    Named(TypeIdentity),
    Apply(Box<Term>, Box<Term>),
    Function(Box<Term>, Box<Term>),
    Record {
        closed: bool,
        fields: Vec<(String, bool, Term)>,
    },
    Tuple(Vec<Term>),
    RequirementMerge(Vec<Term>),
}
type Variable = (bool, u32);
type Substitutions = BTreeMap<Variable, Term>;

fn term(type_ref: &ContractType, parameters: &[TypeParameter], side: bool) -> Option<Term> {
    Some(match type_ref {
        ContractType::Binder(index) => Term::Variable {
            side,
            index: *index,
            arity: parameters.get(*index as usize)?.arity,
        },
        ContractType::Named(identity) => Term::Named(identity.clone()),
        ContractType::Apply {
            constructor,
            arguments,
        } => arguments.iter().try_fold(
            term(constructor, parameters, side)?,
            |constructor, argument| {
                Some(Term::Apply(
                    Box::new(constructor),
                    Box::new(term(argument, parameters, side)?),
                ))
            },
        )?,
        ContractType::Function { parameter, result } => Term::Function(
            Box::new(term(parameter, parameters, side)?),
            Box::new(term(result, parameters, side)?),
        ),
        ContractType::Record { closed, fields } => Term::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|(name, optional, field)| {
                    Some((name.clone(), *optional, term(field, parameters, side)?))
                })
                .collect::<Option<_>>()?,
        },
        ContractType::Tuple(elements) => Term::Tuple(
            elements
                .iter()
                .map(|element| term(element, parameters, side))
                .collect::<Option<_>>()?,
        ),
        ContractType::RequirementMerge(operands) => Term::RequirementMerge(
            operands
                .iter()
                .map(|operand| term(operand, parameters, side))
                .collect::<Option<_>>()?,
        ),
        // A hole or an unscoped parameter is not a wildcard in a checked head.
        ContractType::Hole | ContractType::Parameter(_) => return None,
    })
}

fn dereference(mut value: Term, substitutions: &Substitutions) -> Term {
    while let Term::Variable { side, index, .. } = value {
        let Some(replacement) = substitutions.get(&(side, index)) else {
            break;
        };
        value = replacement.clone();
    }
    value
}

fn occurs(variable: Variable, value: &Term, substitutions: &Substitutions) -> bool {
    match dereference(value.clone(), substitutions) {
        Term::Variable { side, index, .. } => variable == (side, index),
        Term::Apply(left, right) | Term::Function(left, right) => {
            occurs(variable, &left, substitutions) || occurs(variable, &right, substitutions)
        }
        Term::Record { fields, .. } => fields
            .iter()
            .any(|(_, _, field)| occurs(variable, field, substitutions)),
        Term::Tuple(elements) | Term::RequirementMerge(elements) => elements
            .iter()
            .any(|element| occurs(variable, element, substitutions)),
        Term::Named(_) => false,
    }
}

fn known_arity(value: &Term, substitutions: &Substitutions) -> Option<u32> {
    match dereference(value.clone(), substitutions) {
        Term::Variable { arity, .. } => Some(arity),
        Term::Apply(constructor, _) => known_arity(&constructor, substitutions)?.checked_sub(1),
        Term::Named(_) => None, // rigid constructors are kind-checked by the source contract
        _ => Some(0),
    }
}

fn unify(left: Term, right: Term, substitutions: &mut Substitutions) -> bool {
    let left = dereference(left, substitutions);
    let right = dereference(right, substitutions);
    if left == right {
        return true;
    }
    if let Term::Variable { side, index, arity } = &left {
        if known_arity(&right, substitutions).is_some_and(|actual| actual != *arity)
            || occurs((*side, *index), &right, substitutions)
        {
            return false;
        }
        substitutions.insert((*side, *index), right);
        return true;
    }
    if matches!(right, Term::Variable { .. }) {
        return unify(right, left, substitutions);
    }
    match (left, right) {
        (Term::Named(left), Term::Named(right)) => left == right,
        (Term::Apply(lc, la), Term::Apply(rc, ra))
        | (Term::Function(lc, la), Term::Function(rc, ra)) => {
            unify(*lc, *rc, substitutions) && unify(*la, *ra, substitutions)
        }
        (Term::Tuple(left), Term::Tuple(right))
        | (Term::RequirementMerge(left), Term::RequirementMerge(right)) => {
            unify_all(left, right, substitutions)
        }
        (
            Term::Record {
                closed: lc,
                fields: left,
            },
            Term::Record {
                closed: rc,
                fields: right,
            },
        ) => {
            lc == rc
                && left.len() == right.len()
                && left
                    .into_iter()
                    .zip(right)
                    .all(|((ln, lo, lt), (rn, ro, rt))| {
                        ln == rn && lo == ro && unify(lt, rt, substitutions)
                    })
        }
        _ => false,
    }
}
fn unify_all(left: Vec<Term>, right: Vec<Term>, substitutions: &mut Substitutions) -> bool {
    left.len() == right.len()
        && left
            .into_iter()
            .zip(right)
            .all(|(left, right)| unify(left, right, substitutions))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal(name: &str) -> ContractType {
        ContractType::Named(TypeIdentity::Canonical(name.into()))
    }

    fn boxed(value: ContractType) -> ContractType {
        ContractType::Apply {
            constructor: Box::new(nominal("fixture/types::Box")),
            arguments: vec![value],
        }
    }

    #[test]
    fn collection_dependencies_compare_the_determining_head_and_canonical_trait() {
        let generic = CoherenceHead {
            arguments: vec![boxed(ContractType::Binder(0)), nominal("std/prelude::Int")],
            parameters: vec![TypeParameter::value("A")],
        };
        let concrete = CoherenceHead {
            arguments: vec![
                boxed(nominal("std/prelude::Bool")),
                nominal("std/prelude::String"),
            ],
            parameters: vec![],
        };
        for trait_identity in ["std/prelude::Iterable", "std/prelude::Reducible"] {
            assert!(generic.conflicts(&concrete, trait_identity));
            assert!(concrete.conflicts(&generic, trait_identity));
        }
        // A user trait with the same name has no built-in dependency.
        for trait_identity in ["fixture/custom::Iterable", "fixture/custom::Reducible"] {
            assert!(!generic.conflicts(&concrete, trait_identity));
        }
    }

    #[test]
    fn constructor_kinds_and_unrepresented_holes_are_not_wildcards() {
        let value = CoherenceHead {
            arguments: vec![ContractType::Binder(0)],
            parameters: vec![TypeParameter::value("A")],
        };
        let constructor = CoherenceHead {
            arguments: vec![ContractType::Binder(0)],
            parameters: vec![TypeParameter::constructor("F", 1)],
        };
        assert!(!value.conflicts(&constructor, "fixture/trait::Tag"));
        let hole = CoherenceHead {
            arguments: vec![ContractType::Hole],
            parameters: vec![],
        };
        assert!(!value.conflicts(&hole, "fixture/trait::Tag"));
    }
}
