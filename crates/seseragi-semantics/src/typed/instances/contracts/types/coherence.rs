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
                aliases::normalize_coherence_type(resolved, argument)
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

/// A failed first-order unification is not a proof of higher-order disjointness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeadRelation {
    Overlap,
    Disjoint,
    Unproved,
}

impl CoherenceHead {
    pub(crate) fn relation(&self, other: &Self, trait_identity: &str) -> HeadRelation {
        if self.arguments.len() != other.arguments.len() {
            return HeadRelation::Unproved;
        }
        let count = crate::prelude::determining_instance_arguments(trait_identity)
            .unwrap_or(self.arguments.len());
        if count > self.arguments.len() {
            return HeadRelation::Unproved;
        }
        let pairs = self
            .arguments
            .iter()
            .zip(&other.arguments)
            .take(count)
            .map(|(left, right)| {
                Some((
                    term(left, &self.parameters, false)?,
                    term(right, &other.parameters, true)?,
                ))
            })
            .collect::<Option<Vec<_>>>();
        let Some(pairs) = pairs else {
            return HeadRelation::Unproved;
        };
        let mut substitutions = BTreeMap::new();
        if pairs
            .iter()
            .all(|(left, right)| unify(left.clone(), right.clone(), &mut substitutions))
        {
            return HeadRelation::Overlap;
        }
        // Rigid nominal owners remain distinct even under a non-eta section.
        // Relaxing that section must not discard an already available proof.
        if pairs.iter().any(|(left, right)| {
            matches!((rigid_root(left), rigid_root(right)), (Some(left), Some(right)) if left != right)
        }) {
            return HeadRelation::Disjoint;
        }
        // Forget each applied higher-kinded variable and non-eta section.
        // This is a superset of the possible heads, never an overlap witness.
        // Only failure even in that relaxation proves rigid disjointness.
        let mut substitutions = BTreeMap::new();
        if pairs
            .into_iter()
            .all(|(left, right)| unify(relax(left), relax(right), &mut substitutions))
        {
            HeadRelation::Unproved
        } else {
            HeadRelation::Disjoint
        }
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
    // Slots belong to the nearest Section; they are not instance variables,
    // wildcard types, or method binders, and never leave this private proof.
    Slot(u32),
    Section {
        arity: u32,
        body: Box<Term>,
    },
    // Used only by the conservative disjointness relaxation.
    Unknown,
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
        } => {
            let mut slots = 0;
            let body = arguments.iter().try_fold(
                term(constructor, parameters, side)?,
                |constructor, argument| {
                    let argument = if matches!(argument, ContractType::Hole) {
                        let slot = Term::Slot(slots);
                        slots += 1;
                        slot
                    } else {
                        term(argument, parameters, side)?
                    };
                    Some(Term::Apply(Box::new(constructor), Box::new(argument)))
                },
            )?;
            eta_section(slots, body)
        }
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

fn rigid_root(value: &Term) -> Option<&TypeIdentity> {
    match value {
        Term::Named(identity) => Some(identity),
        Term::Apply(constructor, _) => rigid_root(constructor),
        Term::Section { body, .. } => rigid_root(body),
        _ => None,
    }
}

fn contains_slot(value: &Term, slot: u32) -> bool {
    match value {
        Term::Slot(index) => *index == slot,
        Term::Apply(left, right) | Term::Function(left, right) => {
            contains_slot(left, slot) || contains_slot(right, slot)
        }
        Term::Tuple(elements) | Term::RequirementMerge(elements) => {
            elements.iter().any(|value| contains_slot(value, slot))
        }
        Term::Record { fields, .. } => fields
            .iter()
            .any(|(_, _, value)| contains_slot(value, slot)),
        // Nested sections bind their own slots.
        Term::Section { .. } | Term::Variable { .. } | Term::Named(_) | Term::Unknown => false,
    }
}

fn eta_section(mut arity: u32, mut body: Term) -> Term {
    while arity > 0 {
        let Term::Apply(ref constructor, ref argument) = body else {
            break;
        };
        if **argument != Term::Slot(arity - 1) || contains_slot(constructor, arity - 1) {
            break;
        }
        body = *constructor.clone();
        arity -= 1;
    }
    if arity == 0 {
        body
    } else {
        Term::Section {
            arity,
            body: Box::new(body),
        }
    }
}

fn applied_variable(value: &Term) -> bool {
    match value {
        Term::Variable { arity, .. } => *arity > 0,
        Term::Apply(constructor, _) => applied_variable(constructor),
        _ => false,
    }
}

fn relax(value: Term) -> Term {
    match value {
        Term::Apply(ref constructor, _) if applied_variable(constructor) => Term::Unknown,
        Term::Section { .. } | Term::Slot(_) => Term::Unknown,
        Term::Apply(constructor, argument) => {
            Term::Apply(Box::new(relax(*constructor)), Box::new(relax(*argument)))
        }
        Term::Function(parameter, result) => {
            Term::Function(Box::new(relax(*parameter)), Box::new(relax(*result)))
        }
        Term::Tuple(elements) => Term::Tuple(elements.into_iter().map(relax).collect()),
        Term::RequirementMerge(elements) => {
            Term::RequirementMerge(elements.into_iter().map(relax).collect())
        }
        Term::Record { closed, fields } => Term::Record {
            closed,
            fields: fields
                .into_iter()
                .map(|(name, optional, field)| (name, optional, relax(field)))
                .collect(),
        },
        leaf => leaf,
    }
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
        Term::Section { body, .. } => occurs(variable, &body, substitutions),
        Term::Named(_) | Term::Slot(_) | Term::Unknown => false,
    }
}

fn known_arity(value: &Term, substitutions: &Substitutions) -> Option<u32> {
    match dereference(value.clone(), substitutions) {
        Term::Variable { arity, .. } => Some(arity),
        Term::Apply(constructor, _) => known_arity(&constructor, substitutions)?.checked_sub(1),
        Term::Section { arity, .. } => Some(arity),
        Term::Unknown => None,
        Term::Named(_) => None, // rigid constructors are kind-checked by the source contract
        _ => Some(0),
    }
}

fn unify(left: Term, right: Term, substitutions: &mut Substitutions) -> bool {
    let left = dereference(left, substitutions);
    let right = dereference(right, substitutions);
    if matches!(left, Term::Unknown) || matches!(right, Term::Unknown) {
        return true;
    }
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
            assert_eq!(
                generic.relation(&concrete, trait_identity),
                HeadRelation::Overlap
            );
            assert_eq!(
                concrete.relation(&generic, trait_identity),
                HeadRelation::Overlap
            );
        }
        // A user trait with the same name has no built-in dependency.
        for trait_identity in ["fixture/custom::Iterable", "fixture/custom::Reducible"] {
            assert_eq!(
                generic.relation(&concrete, trait_identity),
                HeadRelation::Disjoint
            );
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
        assert_eq!(
            value.relation(&constructor, "fixture/trait::Tag"),
            HeadRelation::Disjoint
        );
        let hole = CoherenceHead {
            arguments: vec![ContractType::Hole],
            parameters: vec![],
        };
        assert_eq!(
            value.relation(&hole, "fixture/trait::Tag"),
            HeadRelation::Unproved
        );
    }
    fn head(argument: ContractType, parameters: Vec<TypeParameter>) -> CoherenceHead {
        CoherenceHead {
            arguments: vec![argument],
            parameters,
        }
    }

    #[test]
    fn sections_keep_slot_order_multiplicity_and_erasure_separate_from_binders() {
        let pair = Term::Named(TypeIdentity::Canonical("fixture/types::Pair".into()));
        let apply = |left, right| Term::Apply(Box::new(left), Box::new(right));
        let repeated = eta_section(1, apply(apply(pair.clone(), Term::Slot(0)), Term::Slot(0)));
        let reversed = eta_section(2, apply(apply(pair.clone(), Term::Slot(1)), Term::Slot(0)));
        let erased = eta_section(1, pair.clone());
        assert!(matches!(repeated, Term::Section { arity: 1, .. }));
        assert!(matches!(reversed, Term::Section { arity: 2, .. }));
        assert!(matches!(erased, Term::Section { arity: 1, .. }));
        assert_ne!(repeated, reversed);
        assert_ne!(repeated, erased);
        assert_eq!(
            eta_section(2, apply(apply(pair.clone(), Term::Slot(0)), Term::Slot(1))),
            pair
        );
        assert!(!unify(
            repeated.clone(),
            Term::Slot(0),
            &mut BTreeMap::new()
        ));
        // Relaxed matching is not a proof of equality.
        assert!(unify(
            relax(repeated),
            relax(reversed),
            &mut BTreeMap::new()
        ));
    }

    #[test]
    fn applied_constructors_need_a_disjointness_proof_and_keep_rigid_owners() {
        let generic = head(
            boxed(ContractType::Apply {
                constructor: Box::new(ContractType::Binder(0)),
                arguments: vec![nominal("std/prelude::Bool")],
            }),
            vec![TypeParameter::constructor("F", 1)],
        );
        let nonprefix = head(
            boxed(ContractType::Apply {
                constructor: Box::new(nominal("fixture/types::Pair")),
                arguments: vec![nominal("std/prelude::Bool"), nominal("std/prelude::Int")],
            }),
            vec![],
        );
        let erased = head(boxed(nominal("std/prelude::Int")), vec![]);
        let same_prefix = head(
            boxed(ContractType::Apply {
                constructor: Box::new(nominal("fixture/types::Other")),
                arguments: vec![nominal("std/prelude::Bool")],
            }),
            vec![],
        );
        for other in [nonprefix, erased] {
            assert_eq!(
                generic.relation(&other, "fixture/trait::Tag"),
                HeadRelation::Unproved
            );
            assert_eq!(
                other.relation(&generic, "fixture/trait::Tag"),
                HeadRelation::Unproved
            );
        }
        assert_eq!(
            generic.relation(&same_prefix, "fixture/trait::Tag"),
            HeadRelation::Overlap
        );
        let unrelated = head(nominal("different/module::Box"), vec![]);
        assert_eq!(
            generic.relation(&unrelated, "fixture/trait::Tag"),
            HeadRelation::Disjoint
        );
        // A disjoint non-determining result cannot override a standard FD.
        let left = CoherenceHead {
            arguments: vec![generic.arguments[0].clone(), nominal("std/prelude::Int")],
            parameters: generic.parameters,
        };
        let right = CoherenceHead {
            arguments: vec![
                boxed(nominal("std/prelude::Int")),
                nominal("std/prelude::Bool"),
            ],
            parameters: vec![],
        };
        assert_eq!(
            left.relation(&right, "std/prelude::Iterable"),
            HeadRelation::Unproved
        );
        assert_eq!(
            left.relation(&right, "fixture/custom::Iterable"),
            HeadRelation::Disjoint
        );
    }
    #[test]
    fn concrete_constructor_witnesses_are_never_reported_disjoint() {
        fn apply(constructor: ContractType, arguments: Vec<ContractType>) -> ContractType {
            super::super::model::apply_arguments(constructor, arguments).unwrap()
        }
        let pair = |left, right| apply(nominal("fixture/types::Pair"), vec![left, right]);
        let generic = head(
            pair(
                apply(ContractType::Binder(0), vec![nominal("std/prelude::Bool")]),
                apply(ContractType::Binder(0), vec![nominal("std/prelude::Int")]),
            ),
            vec![TypeParameter::constructor("F", 1)],
        );
        for shape in 0..7 {
            let instantiate = |argument: ContractType| match shape {
                0 => argument,
                1 => nominal("std/prelude::Int"),
                2 => boxed(argument),
                3 => pair(argument, nominal("std/prelude::Int")),
                4 => pair(nominal("std/prelude::Bool"), argument),
                5 => pair(argument.clone(), argument),
                _ => boxed(pair(nominal("std/prelude::Bool"), argument)),
            };
            let concrete = head(
                pair(
                    instantiate(nominal("std/prelude::Bool")),
                    instantiate(nominal("std/prelude::Int")),
                ),
                vec![],
            );
            assert_ne!(
                generic.relation(&concrete, "fixture/trait::Tag"),
                HeadRelation::Disjoint,
                "constructor witness {shape}"
            );
            assert_ne!(
                concrete.relation(&generic, "fixture/trait::Tag"),
                HeadRelation::Disjoint,
                "reverse constructor witness {shape}"
            );
        }
    }
}
