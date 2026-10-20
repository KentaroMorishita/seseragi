use crate::{ExternalTraitBinding, ExternalTypeBinding, ResolvedModule, SymbolId};
use seseragi_syntax::{InterfaceMethod, SurfaceMethod, TypeParameter, TypeRef};
use std::collections::BTreeMap;

mod aliases;
mod coherence;
mod interface;
mod model;
mod operator;
mod resolution;

pub(super) use operator::{imported_operator_method_matches, local_operator_method_matches};

pub(crate) use coherence::{dependency_head, local_head, CoherenceHead, HeadRelation};
use model::{ContractMethod, ContractType};
use resolution::{contract_constraint, contract_type, declaration_type_parameters, method_binders};

/// Shared proof used by semantic alias projection as well as instance contracts.
/// The caller has already identified a transparent alias by its resolved symbol.
pub(crate) fn represented_alias_constructor(resolved: &ResolvedModule, owner: SymbolId) -> bool {
    let Some(symbol) = resolved.symbols.iter().find(|symbol| symbol.id == owner) else {
        return false;
    };
    let identity = symbol.canonical.clone().map_or(
        model::TypeIdentity::Symbol(owner),
        model::TypeIdentity::Canonical,
    );
    aliases::normalize_type(
        resolved,
        ContractType::Named(identity),
        &mut Default::default(),
    )
    .is_some()
}

pub(in crate::typed::instances) fn unrepresented_alias_head(
    resolved: &ResolvedModule,
    type_ref: &TypeRef,
) -> bool {
    let Some(raw) = contract_type(resolved, type_ref, &BTreeMap::new(), &BTreeMap::new()) else {
        // Name resolution reports an unknown type separately.
        return false;
    };
    aliases::normalize_type(resolved, raw, &mut Default::default()).is_none()
}

/// Instance registration and selected evidence must use the same expanded
/// identity as method contracts. Alias substitution keeps resolved symbol
/// owners and binders, including imported representation bindings.
pub(in crate::typed::instances) fn canonical_type_identity(
    resolved: &ResolvedModule,
    type_ref: &TypeRef,
    binders: &BTreeMap<&str, usize>,
) -> Option<String> {
    let type_ref = contract_type(resolved, type_ref, &BTreeMap::new(), &BTreeMap::new())?;
    let type_ref = aliases::normalize_type(resolved, type_ref, &mut Default::default())?;
    render_identity(resolved, &type_ref, binders)
}

pub(in crate::typed::instances) fn has_unrepresented_parameters(
    resolved: &ResolvedModule,
    origin: seseragi_syntax::ByteSpan,
    parameters: &[TypeParameter],
    arguments: &[TypeRef],
) -> bool {
    fn contains(type_ref: &ContractType, id: SymbolId) -> bool {
        match type_ref {
            ContractType::Parameter(actual) => *actual == id,
            ContractType::Apply {
                constructor,
                arguments,
            } => {
                contains(constructor, id) || arguments.iter().any(|argument| contains(argument, id))
            }
            ContractType::Function { parameter, result } => {
                contains(parameter, id) || contains(result, id)
            }
            ContractType::Tuple(elements) | ContractType::RequirementMerge(elements) => {
                elements.iter().any(|element| contains(element, id))
            }
            ContractType::Record { fields, .. } => {
                fields.iter().any(|(_, _, field)| contains(field, id))
            }
            _ => false,
        }
    }
    let Some(parameters) = declaration_type_parameters(resolved, origin, parameters) else {
        return false;
    };
    let types = arguments
        .iter()
        .filter_map(|argument| {
            let type_ref = contract_type(resolved, argument, &BTreeMap::new(), &BTreeMap::new())?;
            aliases::normalize_type(resolved, type_ref, &mut Default::default())
        })
        .collect::<Vec<_>>();
    parameters
        .iter()
        .any(|parameter| !types.iter().any(|type_ref| contains(type_ref, *parameter)))
}

fn render_identity(
    resolved: &ResolvedModule,
    type_ref: &ContractType,
    binders: &BTreeMap<&str, usize>,
) -> Option<String> {
    use model::TypeIdentity;
    Some(match type_ref {
        ContractType::Binder(index) => format!("${index}"),
        ContractType::Parameter(id) => {
            let symbol = resolved.symbols.iter().find(|symbol| symbol.id == *id)?;
            format!("${}", binders.get(symbol.spelling.as_str())?)
        }
        ContractType::Named(TypeIdentity::Canonical(identity)) => identity.clone(),
        ContractType::Named(TypeIdentity::Symbol(id)) => resolved
            .symbols
            .iter()
            .find(|symbol| symbol.id == *id)?
            .spelling
            .clone(),
        ContractType::Apply {
            constructor,
            arguments,
        } => format!(
            "{}<{}>",
            render_identity(resolved, constructor, binders)?,
            arguments
                .iter()
                .map(|argument| render_identity(resolved, argument, binders))
                .collect::<Option<Vec<_>>>()?
                .join(",")
        ),
        ContractType::Function { parameter, result } => format!(
            "({}->{})",
            render_identity(resolved, parameter, binders)?,
            render_identity(resolved, result, binders)?
        ),
        ContractType::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(|element| render_identity(resolved, element, binders))
                .collect::<Option<Vec<_>>>()?
                .join(",")
        ),
        ContractType::Record { closed, fields } => format!(
            "{}{{{}}}",
            if *closed { "" } else { "open" },
            fields
                .iter()
                .map(|(name, optional, type_ref)| Some(format!(
                    "{name}{}:{}",
                    if *optional { "?" } else { "" },
                    render_identity(resolved, type_ref, binders)?
                )))
                .collect::<Option<Vec<_>>>()?
                .join(",")
        ),
        ContractType::Hole => "_".to_owned(),
        ContractType::AliasSlot { .. } => return None,
        ContractType::RequirementMerge(operands) => operands
            .iter()
            .map(|operand| render_identity(resolved, operand, binders))
            .collect::<Option<Vec<_>>>()?
            .join("&"),
    })
}

pub(super) struct ImportedMethodContext<'a> {
    pub(super) trait_parameters: &'a [TypeParameter],
    pub(super) bindings: &'a [ExternalTypeBinding],
    pub(super) trait_bindings: &'a [ExternalTraitBinding],
    pub(super) trait_name: &'a str,
    pub(super) trait_canonical: &'a str,
}

pub(super) fn method_contract_matches(
    resolved: &ResolvedModule,
    trait_span: seseragi_syntax::ByteSpan,
    trait_parameters: &[TypeParameter],
    instance_arguments: &[TypeRef],
    expected: &SurfaceMethod,
    actual: &SurfaceMethod,
) -> Option<bool> {
    if trait_parameters.len() != instance_arguments.len()
        || expected.type_parameters.len() != actual.type_parameters.len()
    {
        return Some(false);
    }

    let expected_binders = method_binders(resolved, expected)?;
    let actual_binders = method_binders(resolved, actual)?;
    let trait_symbols = declaration_type_parameters(resolved, trait_span, trait_parameters)?;
    let mut substitutions = BTreeMap::new();
    for (symbol, argument) in trait_symbols.into_iter().zip(instance_arguments) {
        substitutions.insert(
            symbol,
            contract_type(resolved, argument, &BTreeMap::new(), &BTreeMap::new())?,
        );
    }

    let expected = contract_method(resolved, expected, &expected_binders, &substitutions)?;
    let actual = contract_method(resolved, actual, &actual_binders, &BTreeMap::new())?;
    // An unproved alias contract is not permission to skip signature checking.
    Some(
        match (
            aliases::normalize(resolved, expected),
            aliases::normalize(resolved, actual),
        ) {
            (Some(expected), Some(actual)) => expected == actual,
            _ => false,
        },
    )
}

pub(super) fn imported_method_contract_matches(
    resolved: &ResolvedModule,
    instance_arguments: &[TypeRef],
    expected: &InterfaceMethod,
    actual: &SurfaceMethod,
    context: ImportedMethodContext<'_>,
) -> Option<bool> {
    if context.trait_parameters.len() != instance_arguments.len()
        || expected.scheme.type_parameters.len() != actual.type_parameters.len()
    {
        return Some(false);
    }

    let actual_binders = method_binders(resolved, actual)?;
    let substitutions = context
        .trait_parameters
        .iter()
        .zip(instance_arguments)
        .map(|(parameter, argument)| {
            Some((
                parameter.name.clone(),
                contract_type(resolved, argument, &BTreeMap::new(), &BTreeMap::new())?,
            ))
        })
        .collect::<Option<BTreeMap<_, _>>>()?;
    let expected = interface::contract_method_from_interface(
        expected,
        &substitutions,
        context.bindings,
        context.trait_bindings,
        context.trait_name,
        context.trait_canonical,
    )?;
    let actual = contract_method(resolved, actual, &actual_binders, &BTreeMap::new())?;
    // An unproved alias contract is not permission to skip signature checking.
    Some(
        match (
            aliases::normalize(resolved, expected),
            aliases::normalize(resolved, actual),
        ) {
            (Some(expected), Some(actual)) => expected == actual,
            _ => false,
        },
    )
}

fn contract_method(
    resolved: &ResolvedModule,
    method: &SurfaceMethod,
    binders: &BTreeMap<SymbolId, u32>,
    substitutions: &BTreeMap<SymbolId, ContractType>,
) -> Option<ContractMethod> {
    let result = contract_type(resolved, &method.return_type, binders, substitutions)?;
    let type_ref = method
        .parameters
        .iter()
        .rev()
        .try_fold(result, |result, parameter| {
            Some(ContractType::Function {
                parameter: Box::new(contract_type(
                    resolved,
                    &parameter.type_ref,
                    binders,
                    substitutions,
                )?),
                result: Box::new(result),
            })
        })?;
    // Omitted source parameters mean one anonymous Prelude Unit parameter.
    // Compare the full function type, so a returned function remains distinct
    // from this parameter while explicit currying can still match the contract.
    let type_ref = if method.parameters.is_empty() {
        ContractType::Function {
            parameter: Box::new(ContractType::Named(model::TypeIdentity::Canonical(
                "std/prelude::Unit".to_owned(),
            ))),
            result: Box::new(type_ref),
        }
    } else {
        type_ref
    };
    let mut constraints = method
        .constraints
        .iter()
        .map(|constraint| contract_constraint(resolved, constraint, binders, substitutions))
        .collect::<Option<Vec<_>>>()?;
    constraints.sort();
    Some(ContractMethod {
        type_ref,
        constraints,
    })
}
