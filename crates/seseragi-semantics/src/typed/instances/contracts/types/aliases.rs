use super::{
    interface,
    model::{apply_arguments, ContractMethod, ContractType, TypeIdentity},
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
    method.type_ref = normalize_type(resolved, method.type_ref, &mut active)?;
    for constraint in &mut method.constraints {
        constraint.arguments = std::mem::take(&mut constraint.arguments)
            .into_iter()
            .map(|argument| normalize_type(resolved, argument, &mut active))
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
    normalize_impl(resolved, type_ref, active)
}

pub(super) fn normalize_coherence_type(
    resolved: &ResolvedModule,
    type_ref: ContractType,
) -> Option<ContractType> {
    normalize_impl(resolved, type_ref, &mut BTreeSet::new())
}

fn normalize_impl(
    resolved: &ResolvedModule,
    type_ref: ContractType,
    active: &mut BTreeSet<TypeIdentity>,
) -> Option<ContractType> {
    Some(match type_ref {
        ContractType::Named(identity) => named(resolved, identity, Vec::new(), active)?,
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| normalize_impl(resolved, argument, active))
                .collect::<Option<Vec<_>>>()?;
            match *constructor {
                ContractType::Named(identity) => named(resolved, identity, arguments, active)?,
                constructor => {
                    apply_arguments(normalize_impl(resolved, constructor, active)?, arguments)?
                }
            }
        }
        ContractType::Function { parameter, result } => ContractType::Function {
            parameter: Box::new(normalize_impl(resolved, *parameter, active)?),
            result: Box::new(normalize_impl(resolved, *result, active)?),
        },
        ContractType::Record { closed, fields } => ContractType::Record {
            closed,
            fields: fields
                .into_iter()
                .map(|(name, optional, type_ref)| {
                    Some((name, optional, normalize_impl(resolved, type_ref, active)?))
                })
                .collect::<Option<_>>()?,
        },
        ContractType::Tuple(elements) => ContractType::Tuple(
            elements
                .into_iter()
                .map(|element| normalize_impl(resolved, element, active))
                .collect::<Option<_>>()?,
        ),
        ContractType::RequirementMerge(operands) => ContractType::RequirementMerge(
            operands
                .into_iter()
                .map(|operand| normalize_impl(resolved, operand, active))
                .collect::<Option<_>>()?,
        ),
        leaf => leaf,
    })
}

fn named(
    resolved: &ResolvedModule,
    identity: TypeIdentity,
    arguments: Vec<ContractType>,
    active: &mut BTreeSet<TypeIdentity>,
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
    let partial = arguments.len() < arity || arguments.contains(&ContractType::Hole);
    // Mixed implicit and explicit sections need their own section/kind proof.
    // Never lose an implicit slot by flattening it into an explicit Hole.
    if partial
        && (parameters.iter().any(|parameter| parameter.arity != 0)
            || (arguments.len() < arity && arguments.contains(&ContractType::Hole)))
    {
        return None;
    }
    let supplied = if partial {
        (0..arity)
            .map(|index| ContractType::AliasSlot {
                owner: identity.clone(),
                index: index as u32,
            })
            .collect()
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
    let normalized = normalize_impl(resolved, expanded, active)?;
    let normalized = if partial {
        let constructor = constructors::reduce(resolved, normalized, &identity, arity)?;
        apply_arguments(constructor, arguments)?
    } else {
        normalized
    };
    active.remove(&identity);
    Some(normalized)
}
