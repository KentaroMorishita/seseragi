use super::{
    interface,
    model::{
        apply_arguments, normalize_requirement_merge, ContractMethod, ContractType, TypeIdentity,
    },
    resolution,
};
use crate::{ResolvedModule, SymbolNamespace};
use seseragi_syntax::SurfaceDecl;
use std::collections::{BTreeMap, BTreeSet};

mod constructors;

/// Registry contracts contain canonical types, while source methods may name
/// transparent aliases. Expand those aliases by resolved owner, keeping method
/// binders and instance parameters as identities rather than guessing spellings.
pub(super) fn normalize(
    resolved: &ResolvedModule,
    mut method: ContractMethod,
) -> Option<ContractMethod> {
    let mut active = BTreeSet::new();
    method.type_ref = normalize_impl(resolved, method.type_ref, &mut active, false, &[])?;
    for constraint in &mut method.constraints {
        constraint.arguments = std::mem::take(&mut constraint.arguments)
            .into_iter()
            .map(|argument| normalize_impl(resolved, argument, &mut active, false, &[]))
            .collect::<Option<_>>()?;
    }
    method.constraints.sort();
    Some(method)
}

pub(super) fn normalize_type(
    resolved: &ResolvedModule,
    type_ref: ContractType,
    active: &mut BTreeSet<TypeIdentity>,
) -> Option<ContractType> {
    normalize_impl(resolved, type_ref, active, true, &[])
}

pub(super) fn normalize_coherence_type(
    resolved: &ResolvedModule,
    type_ref: ContractType,
    binder_arities: &[u32],
) -> Option<ContractType> {
    normalize_impl(
        resolved,
        type_ref,
        &mut BTreeSet::new(),
        true,
        binder_arities,
    )
}

pub(super) fn omitted_constructor_suffix(
    resolved: &ResolvedModule,
    normalized: &ContractType,
) -> Option<usize> {
    constructors::omitted_suffix(resolved, normalized)
}

fn normalize_impl(
    resolved: &ResolvedModule,
    type_ref: ContractType,
    active: &mut BTreeSet<TypeIdentity>,
    for_head: bool,
    binder_arities: &[u32],
) -> Option<ContractType> {
    Some(match type_ref {
        ContractType::TypeFunction {
            parameter_kinds,
            body,
        } => ContractType::TypeFunction {
            parameter_kinds,
            body: Box::new(normalize_impl(
                resolved,
                *body,
                active,
                for_head,
                binder_arities,
            )?),
        },
        ContractType::Named(identity) => named(
            resolved,
            identity,
            Vec::new(),
            active,
            for_head,
            binder_arities,
        )?,
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| {
                    normalize_impl(resolved, argument, active, for_head, binder_arities)
                })
                .collect::<Option<Vec<_>>>()?;
            match *constructor {
                ContractType::Named(identity) => named(
                    resolved,
                    identity,
                    arguments,
                    active,
                    for_head,
                    binder_arities,
                )?,
                constructor => apply_arguments(
                    normalize_impl(resolved, constructor, active, for_head, binder_arities)?,
                    arguments,
                )?,
            }
        }
        ContractType::Function { parameter, result } => ContractType::Function {
            parameter: Box::new(normalize_impl(
                resolved,
                *parameter,
                active,
                for_head,
                binder_arities,
            )?),
            result: Box::new(normalize_impl(
                resolved,
                *result,
                active,
                for_head,
                binder_arities,
            )?),
        },
        ContractType::Record { closed, fields } => ContractType::Record {
            closed,
            fields: fields
                .into_iter()
                .map(|(name, optional, type_ref)| {
                    Some((
                        name,
                        optional,
                        normalize_impl(resolved, type_ref, active, for_head, binder_arities)?,
                    ))
                })
                .collect::<Option<_>>()?,
        },
        ContractType::Tuple(elements) => ContractType::Tuple(
            elements
                .into_iter()
                .map(|element| normalize_impl(resolved, element, active, for_head, binder_arities))
                .collect::<Option<_>>()?,
        ),
        ContractType::RequirementMerge(operands) => normalize_requirement_merge(
            operands
                .into_iter()
                .map(|operand| normalize_impl(resolved, operand, active, for_head, binder_arities))
                .collect::<Option<Vec<_>>>()?,
        ),
        leaf => leaf,
    })
}

fn named(
    resolved: &ResolvedModule,
    identity: TypeIdentity,
    arguments: Vec<ContractType>,
    active: &mut BTreeSet<TypeIdentity>,
    for_head: bool,
    binder_arities: &[u32],
) -> Option<ContractType> {
    let owner = resolved.symbols.iter().find(|symbol| {
        symbol.namespace == SymbolNamespace::Type
            && match &identity {
                TypeIdentity::Canonical(canonical) => symbol.canonical.as_ref() == Some(canonical),
                TypeIdentity::Symbol(id) => symbol.id == *id,
            }
    });
    let local = owner.and_then(|owner| {
        resolved.declarations.iter().find_map(|declaration| {
            let SurfaceDecl::Alias {
                name_span,
                type_parameters,
                target,
                span,
                ..
            } = declaration
            else {
                return None;
            };
            (*name_span == owner.origin).then_some((type_parameters, target, span))
        })
    });
    let imported = owner.and_then(|owner| {
        resolved.imports.iter().find(|import| {
            import.symbol == owner.id && import.export.declaration_kind.as_deref() == Some("alias")
        })
    });
    let parameters = if let Some((parameters, _, _)) = local {
        parameters.as_slice()
    } else if let Some(import) = imported {
        import.export.scheme.type_parameters.as_slice()
    } else {
        return apply_arguments(ContractType::Named(identity), arguments);
    };
    let arity = parameters.len();
    if arguments.len() > arity || !active.insert(identity.clone()) {
        return None;
    }
    // Method signatures compare fully supplied value-type arguments, including
    // ordinary inference holes. Constructor identity/coherence must instead
    // prove every explicit hole from an alias template before using its head.
    let partial = arguments.len() < arity || (for_head && arguments.contains(&ContractType::Hole));
    // Mixed implicit and explicit sections need their own section/kind proof.
    // Never lose an implicit slot by flattening it into an explicit Hole.
    if partial
        && (parameters.iter().enumerate().any(|(index, parameter)| {
            parameter.arity != 0
                && arguments
                    .get(index)
                    .is_none_or(|argument| *argument == ContractType::Hole)
        }) || (arguments.len() < arity && arguments.contains(&ContractType::Hole)))
    {
        return None;
    }
    let mut remaining = Vec::new();
    let supplied = if partial {
        let mut supplied = Vec::with_capacity(arity);
        for (index, parameter) in parameters.iter().enumerate() {
            match arguments.get(index) {
                Some(argument) if *argument != ContractType::Hole => {
                    if constructors::remaining_kinds(resolved, argument, binder_arities)?
                        != vec![0; parameter.arity as usize]
                    {
                        return None;
                    }
                    supplied.push(argument.clone());
                }
                _ => {
                    remaining.push(index);
                    supplied.push(ContractType::AliasSlot {
                        owner: identity.clone(),
                        index: index as u32,
                    });
                }
            }
        }
        supplied
    } else {
        arguments.clone()
    };
    let expanded = if let Some((parameters, target, span)) = local {
        let symbols = resolution::declaration_type_parameters(resolved, *span, parameters)?;
        let substitutions = symbols.into_iter().zip(supplied).collect();
        resolution::contract_type(resolved, target, &BTreeMap::new(), &substitutions)?
    } else {
        let import = imported?;
        let substitutions = import
            .export
            .scheme
            .type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .zip(supplied)
            .collect();
        interface::contract_type(
            import.export.representation.as_ref()?,
            &BTreeMap::new(),
            &substitutions,
            import.scheme_type_bindings.as_deref().unwrap_or(&[]),
        )?
    };
    let normalized = normalize_impl(resolved, expanded, active, for_head, binder_arities)?;
    let normalized = if partial {
        constructors::reduce(
            resolved,
            normalized,
            &identity,
            parameters,
            &remaining,
            arguments.len() == arity,
            binder_arities,
        )?
    } else {
        normalized
    };
    active.remove(&identity);
    Some(normalized)
}
