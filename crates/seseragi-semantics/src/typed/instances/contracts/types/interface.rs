use super::model::{
    apply_arguments, ContractConstraint, ContractMethod, ContractType, TypeIdentity,
};
use crate::prelude::{is_standalone_symbol, sum_type_for_symbol};
use crate::{ExternalTypeBinding, SymbolNamespace};
use seseragi_syntax::{InterfaceConstraint, InterfaceMethod, InterfaceType};
use std::collections::BTreeMap;

pub(super) fn contract_method_from_interface(
    method: &InterfaceMethod,
    substitutions: &BTreeMap<String, ContractType>,
    bindings: &[ExternalTypeBinding],
    trait_name: &str,
    trait_canonical: &str,
) -> Option<ContractMethod> {
    let binders = method
        .scheme
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index as u32))
        .collect::<BTreeMap<_, _>>();
    let mut constraints = method
        .scheme
        .constraints
        .iter()
        .map(|constraint| {
            contract_constraint(
                constraint,
                &binders,
                substitutions,
                bindings,
                trait_name,
                trait_canonical,
            )
        })
        .collect::<Option<Vec<_>>>()?;
    constraints.sort();
    Some(ContractMethod {
        type_ref: contract_type(&method.scheme.type_ref, &binders, substitutions, bindings)?,
        constraints,
    })
}

fn contract_constraint(
    constraint: &InterfaceConstraint,
    binders: &BTreeMap<String, u32>,
    substitutions: &BTreeMap<String, ContractType>,
    bindings: &[ExternalTypeBinding],
    trait_name: &str,
    trait_canonical: &str,
) -> Option<ContractConstraint> {
    let trait_identity = if constraint.name == trait_name {
        TypeIdentity::Canonical(trait_canonical.to_owned())
    } else if is_standalone_symbol(SymbolNamespace::Trait, &constraint.name) {
        TypeIdentity::Canonical(format!("std/prelude::{}", constraint.name))
    } else {
        return None;
    };
    Some(ContractConstraint {
        trait_identity,
        arguments: constraint
            .arguments
            .iter()
            .map(|argument| contract_type(argument, binders, substitutions, bindings))
            .collect::<Option<Vec<_>>>()?,
    })
}

fn contract_type(
    type_ref: &InterfaceType,
    binders: &BTreeMap<String, u32>,
    substitutions: &BTreeMap<String, ContractType>,
    bindings: &[ExternalTypeBinding],
) -> Option<ContractType> {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            named_type(name, arguments, binders, substitutions, bindings)
        }
        InterfaceType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => apply_arguments(
            ContractType::Named(TypeIdentity::Canonical(canonical.clone())),
            contract_types(arguments, binders, substitutions, bindings)?,
        ),
        InterfaceType::TypeConstructor { name, .. } => {
            named_type(name, &[], binders, substitutions, bindings)
        }
        InterfaceType::Apply {
            constructor,
            arguments,
        } => named_type(constructor, arguments, binders, substitutions, bindings),
        InterfaceType::Function { parameter, result } => Some(ContractType::Function {
            parameter: Box::new(contract_type(parameter, binders, substitutions, bindings)?),
            result: Box::new(contract_type(result, binders, substitutions, bindings)?),
        }),
        InterfaceType::Record { closed, fields } => {
            let mut fields = fields
                .iter()
                .map(|field| {
                    Some((
                        field.name.clone(),
                        field.optional,
                        contract_type(&field.type_ref, binders, substitutions, bindings)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            fields.sort_by(|left, right| left.0.cmp(&right.0));
            Some(ContractType::Record {
                closed: *closed,
                fields,
            })
        }
        InterfaceType::Tuple { elements } => Some(ContractType::Tuple(contract_types(
            elements,
            binders,
            substitutions,
            bindings,
        )?)),
        InterfaceType::Hole => Some(ContractType::Hole),
    }
}

fn named_type(
    name: &str,
    arguments: &[InterfaceType],
    binders: &BTreeMap<String, u32>,
    substitutions: &BTreeMap<String, ContractType>,
    bindings: &[ExternalTypeBinding],
) -> Option<ContractType> {
    let constructor = if let Some(index) = binders.get(name) {
        ContractType::Binder(*index)
    } else if let Some(substitution) = substitutions.get(name) {
        substitution.clone()
    } else if let Some(binding) = bindings.iter().find(|binding| binding.spelling == name) {
        ContractType::Named(TypeIdentity::Canonical(binding.canonical.clone()))
    } else if let Some(sum_type) = sum_type_for_symbol(SymbolNamespace::Type, name) {
        ContractType::Named(TypeIdentity::Canonical(sum_type.canonical.to_owned()))
    } else if is_standalone_symbol(SymbolNamespace::Type, name) {
        ContractType::Named(TypeIdentity::Canonical(format!("std/prelude::{name}")))
    } else {
        return None;
    };
    apply_arguments(
        constructor,
        contract_types(arguments, binders, substitutions, bindings)?,
    )
}

fn contract_types(
    types: &[InterfaceType],
    binders: &BTreeMap<String, u32>,
    substitutions: &BTreeMap<String, ContractType>,
    bindings: &[ExternalTypeBinding],
) -> Option<Vec<ContractType>> {
    types
        .iter()
        .map(|type_ref| contract_type(type_ref, binders, substitutions, bindings))
        .collect()
}
