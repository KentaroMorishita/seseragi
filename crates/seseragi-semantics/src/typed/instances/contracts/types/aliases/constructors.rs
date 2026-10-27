//! Checked eta/section projection, not general type-level lambda unification.
use super::{ContractType, TypeIdentity};
use crate::{ResolvedModule, SymbolKind};
use seseragi_syntax::{InterfaceType, SurfaceDecl, TypeParameter};
use std::collections::BTreeSet;

pub(super) fn reduce(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    supplied: Vec<ContractType>,
) -> Option<ContractType> {
    // Preserve the existing eta proof, including checked facade bindings that
    // describe an ordinary nominal constructor only by its explicit arity.
    if parameters.iter().all(|parameter| parameter.arity == 0) {
        if let Some(constructor) = eta_reduce(resolved, template.clone(), owner, parameters.len()) {
            return super::apply_arguments(constructor, supplied);
        }
    }
    section_reduce(
        resolved,
        template.clone(),
        owner,
        parameters,
        supplied.clone(),
    )
    .or_else(|| type_function(resolved, template, owner, parameters, supplied))
}

/// A transparent alias may use a remaining parameter more than once, in a
/// different position, or not at all. Keep that mapping instead of flattening
/// its variables into nominal holes. The finite producer proves value-kinded
/// domains and a closed value-kinded body; higher-order inverse inference is
/// deliberately not introduced here.
fn type_function(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    supplied: Vec<ContractType>,
) -> Option<ContractType> {
    if parameters.iter().any(|parameter| parameter.arity != 0)
        || template_kinds(resolved, &template, owner, parameters)? != Vec::<u32>::new()
    {
        return None;
    }
    let remaining = (0..parameters.len())
        .filter(|index| {
            supplied
                .get(*index)
                .is_none_or(|argument| *argument == ContractType::Hole)
        })
        .collect::<Vec<_>>();
    if remaining.is_empty() {
        return None;
    }
    for argument in supplied
        .iter()
        .filter(|argument| **argument != ContractType::Hole)
    {
        if !remaining_kinds(resolved, argument)?.is_empty() {
            return None;
        }
    }
    fn replace(
        value: &ContractType,
        owner: &TypeIdentity,
        supplied: &[ContractType],
        remaining: &[usize],
        depth: u32,
    ) -> Option<ContractType> {
        use seseragi_syntax::type_functions::{shift_type_function_parameters, TypeFunctionTerm};
        match value {
            ContractType::AliasSlot {
                owner: actual,
                index,
            } if actual == owner => {
                if let Some(index) = remaining
                    .iter()
                    .position(|remaining| *remaining == *index as usize)
                {
                    Some(ContractType::TypeFunctionParameter {
                        depth,
                        index: index as u32,
                    })
                } else {
                    shift_type_function_parameters(
                        supplied.get(*index as usize)?,
                        depth.checked_add(1)?,
                        0,
                    )
                }
            }
            ContractType::AliasSlot { .. } | ContractType::Hole => None,
            ContractType::TypeFunction {
                parameter_kinds,
                body,
            } => Some(ContractType::TypeFunction {
                parameter_kinds: parameter_kinds.clone(),
                body: Box::new(replace(
                    body,
                    owner,
                    supplied,
                    remaining,
                    depth.checked_add(1)?,
                )?),
            }),
            _ => value
                .map_type_children(&mut |child| replace(child, owner, supplied, remaining, depth)),
        }
    }
    Some(ContractType::TypeFunction {
        parameter_kinds: vec![0; remaining.len()],
        body: Box::new(replace(&template, owner, &supplied, &remaining, 0)?),
    })
}

fn template_kinds(
    resolved: &ResolvedModule,
    value: &ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
) -> Option<Vec<u32>> {
    Some(match value {
        ContractType::AliasSlot {
            owner: actual,
            index,
        } if actual == owner => {
            vec![0; parameters.get(*index as usize)?.arity as usize]
        }
        ContractType::Named(TypeIdentity::Canonical(canonical)) => {
            rigid_parameter_kinds(resolved, canonical)?
        }
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let kinds = template_kinds(resolved, constructor, owner, parameters)?;
            if arguments.len() > kinds.len() {
                return None;
            }
            for (argument, kind) in arguments.iter().zip(&kinds) {
                if template_kinds(resolved, argument, owner, parameters)? != vec![0; *kind as usize]
                {
                    return None;
                }
            }
            kinds[arguments.len()..].to_vec()
        }
        // The finite producer accepts canonical constructor applications and
        // scoped value slots. Structural function/record/row constructors need
        // their own admission proof; no unknown body is silently exported.
        // No wildcard, unresolved free binder or nested unproved function is
        // evidence of the required closed constructor kind.
        _ => return None,
    })
}

fn section_reduce(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    supplied: Vec<ContractType>,
) -> Option<ContractType> {
    let ContractType::Apply {
        constructor,
        arguments,
    } = template
    else {
        return None;
    };
    let ContractType::Named(TypeIdentity::Canonical(canonical)) = constructor.as_ref() else {
        return None;
    };
    // The extension needs the actual declaration's parameter kinds, not an
    // arity-only facade descriptor or the spelling of an unrelated nominal.
    let kinds = rigid_parameter_kinds(resolved, canonical)?;
    if arguments.len() != kinds.len() {
        return None;
    }
    let remaining = parameters
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            supplied
                .get(*index)
                .is_none_or(|argument| *argument == ContractType::Hole)
        })
        .map(|(index, parameter)| (parameter.arity == 0).then_some(index as u32))
        .collect::<Option<Vec<_>>>()?;
    let mut seen = Vec::new();
    let mut projected = Vec::new();
    for (argument, kind) in arguments.into_iter().zip(kinds) {
        match argument {
            ContractType::AliasSlot {
                owner: actual,
                index,
            } if actual == *owner => {
                let parameter = parameters.get(index as usize)?;
                if parameter.arity != kind {
                    return None;
                }
                match supplied.get(index as usize) {
                    Some(argument) if *argument != ContractType::Hole => {
                        // Closed fixed arguments have their own checked kinds.
                        // General higher-order unification is deliberately not
                        // inferred from an unresolved binder or display name.
                        if remaining_kinds(resolved, argument)? != vec![0; kind as usize] {
                            return None;
                        }
                        projected.push(argument.clone());
                    }
                    _ => {
                        seen.push(index);
                        projected.push(ContractType::Hole);
                    }
                }
            }
            fixed => {
                // A private AliasSlot cannot pass this recursive kind proof.
                // Explicit holes are admitted only as slots of a checked
                // canonical constructor application, never as unknown leaves.
                if remaining_kinds(resolved, &fixed)? != vec![0; kind as usize] {
                    return None;
                }
                projected.push(fixed);
            }
        }
    }
    // The existing section representation fills its slots left to right, once
    // each. A permutation, repetition or erasure of an unbound alias parameter
    // needs a different representation and remains unproved.
    if seen != remaining {
        return None;
    }
    // Only implicit suffix slots are eta-reduced. Preserve explicit source
    // sections, as well as every fixed/non-trailing argument.
    if supplied.len() < parameters.len() {
        while projected.last() == Some(&ContractType::Hole) {
            projected.pop();
        }
    }
    super::apply_arguments(*constructor, projected)
}

fn eta_reduce(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    arity: usize,
) -> Option<ContractType> {
    let ContractType::Apply {
        constructor,
        mut arguments,
    } = template
    else {
        return None;
    };
    for index in (0..arity).rev() {
        if arguments.pop()?
            != (ContractType::AliasSlot {
                owner: owner.clone(),
                index: index as u32,
            })
        {
            return None;
        }
    }
    if arguments.iter().any(contains_unrepresented_slot) {
        return None;
    }
    let ContractType::Named(TypeIdentity::Canonical(canonical)) = constructor.as_ref() else {
        return None;
    };
    if rigid_arity(resolved, canonical)? != arguments.len() + arity {
        return None;
    }
    super::apply_arguments(*constructor, arguments)
}

pub(super) fn omitted_suffix(resolved: &ResolvedModule, type_ref: &ContractType) -> Option<usize> {
    let (constructor, arguments) = match type_ref {
        ContractType::Apply {
            constructor,
            arguments,
        } => (constructor.as_ref(), arguments.as_slice()),
        constructor => (constructor, &[][..]),
    };
    let ContractType::Named(TypeIdentity::Canonical(canonical)) = constructor else {
        return None;
    };
    rigid_parameter_kinds(resolved, canonical)
        .map(|kinds| kinds.len())
        .or_else(|| rigid_arity(resolved, canonical))?
        .checked_sub(arguments.len())
}

fn remaining_kinds(resolved: &ResolvedModule, type_ref: &ContractType) -> Option<Vec<u32>> {
    match type_ref {
        ContractType::TypeFunction {
            parameter_kinds, ..
        } => Some(parameter_kinds.clone()),
        ContractType::Named(TypeIdentity::Canonical(canonical)) => {
            rigid_parameter_kinds(resolved, canonical)
        }
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let kinds = remaining_kinds(resolved, constructor)?;
            if arguments.len() > kinds.len() {
                return None;
            }
            let mut remaining = Vec::new();
            for (argument, kind) in arguments.iter().zip(&kinds) {
                if *argument == ContractType::Hole {
                    remaining.push(*kind);
                } else if remaining_kinds(resolved, argument)? != vec![0; *kind as usize] {
                    return None;
                }
            }
            remaining.extend_from_slice(&kinds[arguments.len()..]);
            Some(remaining)
        }
        ContractType::Function { .. }
        | ContractType::Record { .. }
        | ContractType::Tuple(_)
        | ContractType::RequirementMerge(_)
            if !contains_unrepresented_slot(type_ref) =>
        {
            Some(Vec::new())
        }
        _ => None,
    }
}

fn rigid_parameter_kinds(resolved: &ResolvedModule, canonical: &str) -> Option<Vec<u32>> {
    let mut candidates = BTreeSet::new();
    for symbol in resolved.symbols.iter().filter(|symbol| {
        symbol.namespace == crate::SymbolNamespace::Type
            && symbol.canonical.as_deref() == Some(canonical)
    }) {
        if canonical == format!("std/prelude::{}", symbol.spelling) {
            if let Some(arity) = crate::prelude::type_constructor_arity(&symbol.spelling) {
                candidates.insert(vec![0; arity as usize]);
            }
        }
        for declaration in &resolved.declarations {
            let (name_span, parameters) = match declaration {
                SurfaceDecl::Type {
                    name_span,
                    type_parameters,
                    ..
                }
                | SurfaceDecl::Newtype {
                    name_span,
                    type_parameters,
                    ..
                }
                | SurfaceDecl::Struct {
                    name_span,
                    type_parameters,
                    ..
                } => (name_span, type_parameters),
                _ => continue,
            };
            if *name_span == symbol.origin {
                candidates.insert(
                    parameters
                        .iter()
                        .map(|parameter| parameter.arity)
                        .collect::<Vec<_>>(),
                );
            }
        }
        for import in resolved.imports.iter().filter(|import| {
            import.symbol == symbol.id
                && import.export.namespace == "type"
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("type" | "opaque-type" | "struct" | "opaque-struct" | "newtype")
                )
        }) {
            let parameters = &import.export.scheme.type_parameters;
            if let Some(InterfaceType::TypeConstructor { arity, .. }) =
                &import.export.representation
            {
                if *arity as usize != parameters.len() {
                    continue;
                }
            }
            candidates.insert(
                parameters
                    .iter()
                    .map(|parameter| parameter.arity)
                    .collect::<Vec<_>>(),
            );
        }
    }
    if let Some(sum) = crate::prelude::SUM_TYPES
        .iter()
        .find(|sum| sum.canonical == canonical)
    {
        candidates.insert(vec![0; sum.type_parameters.len()]);
    }
    (candidates.len() == 1).then(|| candidates.pop_first().unwrap())
}

fn contains_unrepresented_slot(type_ref: &ContractType) -> bool {
    match type_ref {
        ContractType::AliasSlot { .. } | ContractType::Hole => true,
        ContractType::TypeFunction { body, .. } => contains_unrepresented_slot(body),
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            contains_unrepresented_slot(constructor)
                || arguments.iter().any(contains_unrepresented_slot)
        }
        ContractType::Function { parameter, result } => {
            contains_unrepresented_slot(parameter) || contains_unrepresented_slot(result)
        }
        ContractType::Tuple(elements) | ContractType::RequirementMerge(elements) => {
            elements.iter().any(contains_unrepresented_slot)
        }
        ContractType::Record { fields, .. } => fields
            .iter()
            .any(|(_, _, field)| contains_unrepresented_slot(field)),
        _ => false,
    }
}

fn rigid_arity(resolved: &ResolvedModule, canonical: &str) -> Option<usize> {
    let mut arities = BTreeSet::new();
    for symbol in resolved.symbols.iter().filter(|symbol| {
        symbol.kind == SymbolKind::Type && symbol.canonical.as_deref() == Some(canonical)
    }) {
        if canonical == format!("std/prelude::{}", symbol.spelling) {
            if let Some(arity) = crate::prelude::type_constructor_arity(&symbol.spelling) {
                arities.insert(arity as usize);
            }
        }
        for declaration in &resolved.declarations {
            let (name_span, parameters) = match declaration {
                SurfaceDecl::Type {
                    name_span,
                    type_parameters,
                    ..
                }
                | SurfaceDecl::Newtype {
                    name_span,
                    type_parameters,
                    ..
                }
                | SurfaceDecl::Struct {
                    name_span,
                    type_parameters,
                    ..
                } => (name_span, type_parameters),
                _ => continue,
            };
            if *name_span == symbol.origin {
                if parameters.iter().any(|parameter| parameter.arity != 0) {
                    return None;
                }
                arities.insert(parameters.len());
            }
        }
        for import in resolved
            .imports
            .iter()
            .filter(|import| import.symbol == symbol.id && import.export.namespace == "type")
        {
            if import.export.declaration_kind.as_deref() == Some("alias") {
                continue;
            }
            if import
                .export
                .scheme
                .type_parameters
                .iter()
                .any(|parameter| parameter.arity != 0)
            {
                return None;
            }
            let arity = match &import.export.representation {
                Some(InterfaceType::TypeConstructor { arity, .. }) => *arity as usize,
                _ => import.export.scheme.type_parameters.len(),
            };
            arities.insert(arity);
        }
    }
    // An alias facade may name a checked nominal provider without importing
    // that provider directly. Use its canonical binding and explicit arity.
    for binding in resolved
        .imports
        .iter()
        .flat_map(|import| import.scheme_type_bindings.iter().flatten())
    {
        if binding.canonical == canonical {
            if let Some(arity) = binding.arity {
                arities.insert(arity as usize);
            }
        }
    }
    if let Some(sum) = crate::prelude::SUM_TYPES
        .iter()
        .find(|sum| sum.canonical == canonical)
    {
        arities.insert(sum.type_parameters.len());
    }
    (arities.len() == 1).then(|| *arities.first().unwrap())
}
