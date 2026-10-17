use super::{
    interface,
    model::{apply_arguments, ContractMethod, ContractType, TypeIdentity},
    resolution,
};
use crate::{ResolvedModule, SymbolNamespace};
use seseragi_syntax::SurfaceDecl;
use std::collections::{BTreeMap, BTreeSet};

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
    normalize_with_policy(resolved, type_ref, active, true)
}

pub(super) fn normalize_coherence_type(
    resolved: &ResolvedModule,
    type_ref: ContractType,
) -> Option<ContractType> {
    normalize_with_policy(resolved, type_ref, &mut BTreeSet::new(), false)
}

fn normalize_with_policy(
    resolved: &ResolvedModule,
    type_ref: ContractType,
    active: &mut BTreeSet<TypeIdentity>,
    allow_partial_alias: bool,
) -> Option<ContractType> {
    Some(match type_ref {
        ContractType::Named(identity) => {
            named(resolved, identity, Vec::new(), active, allow_partial_alias)?
        }
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| {
                    normalize_with_policy(resolved, argument, active, allow_partial_alias)
                })
                .collect::<Option<Vec<_>>>()?;
            match *constructor {
                ContractType::Named(identity) => {
                    named(resolved, identity, arguments, active, allow_partial_alias)?
                }
                constructor => apply_arguments(
                    normalize_with_policy(resolved, constructor, active, allow_partial_alias)?,
                    arguments,
                )?,
            }
        }
        ContractType::Function { parameter, result } => ContractType::Function {
            parameter: Box::new(normalize_with_policy(
                resolved,
                *parameter,
                active,
                allow_partial_alias,
            )?),
            result: Box::new(normalize_with_policy(
                resolved,
                *result,
                active,
                allow_partial_alias,
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
                        normalize_with_policy(resolved, type_ref, active, allow_partial_alias)?,
                    ))
                })
                .collect::<Option<_>>()?,
        },
        ContractType::Tuple(elements) => ContractType::Tuple(
            elements
                .into_iter()
                .map(|element| {
                    normalize_with_policy(resolved, element, active, allow_partial_alias)
                })
                .collect::<Option<_>>()?,
        ),
        ContractType::RequirementMerge(operands) => ContractType::RequirementMerge(
            operands
                .into_iter()
                .map(|operand| {
                    normalize_with_policy(resolved, operand, active, allow_partial_alias)
                })
                .collect::<Option<_>>()?,
        ),
        leaf => leaf,
    })
}

fn named(
    resolved: &ResolvedModule,
    identity: TypeIdentity,
    mut arguments: Vec<ContractType>,
    active: &mut BTreeSet<TypeIdentity>,
    allow_partial_alias: bool,
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
    let arity = if let Some((parameters, _, _)) = local {
        parameters.len()
    } else if let Some(import) = imported {
        import.export.scheme.type_parameters.len()
    } else {
        return apply_arguments(ContractType::Named(identity), arguments);
    };
    // Missing alias parameters are not interchangeable constructor slots:
    // aliases can reorder, repeat, or erase them. Do not use padded Hole as
    // evidence of constructor equality or disjointness.
    if !allow_partial_alias && (arguments.len() < arity || arguments.contains(&ContractType::Hole))
    {
        return None;
    }
    if arguments.len() > arity || !active.insert(identity.clone()) {
        return None;
    }
    // An unapplied alias in a higher-kinded position represents its constructor.
    // Existing application semantics fill these holes when arguments arrive.
    arguments.resize(arity, ContractType::Hole);
    let expanded = if let Some((parameters, target, span)) = local {
        let symbols = resolution::declaration_type_parameters(resolved, *span, parameters)?;
        let substitutions = symbols.into_iter().zip(arguments).collect();
        resolution::contract_type(resolved, target, &BTreeMap::new(), &substitutions)?
    } else {
        let import = imported?;
        let substitutions = import
            .export
            .scheme
            .type_parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .zip(arguments)
            .collect();
        interface::contract_type(
            import.export.representation.as_ref()?,
            &BTreeMap::new(),
            &substitutions,
            import.scheme_type_bindings.as_deref().unwrap_or(&[]),
        )?
    };
    let normalized = normalize_with_policy(resolved, expanded, active, allow_partial_alias);
    active.remove(&identity);
    normalized
}
