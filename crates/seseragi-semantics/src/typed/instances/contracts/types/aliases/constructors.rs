//! Checked eta/section projection, not general type-level lambda unification.
use super::{ContractType, TypeIdentity};
use crate::{ResolvedModule, ScopeKind, SymbolId, SymbolKind};
use seseragi_syntax::{InterfaceType, SurfaceDecl, SurfaceImplMember, TypeParameter};
use std::collections::BTreeSet;

pub(super) fn reduce(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    remaining: &[usize],
    explicit_holes: bool,
    binder_arities: &[u32],
) -> Option<ContractType> {
    if remaining.is_empty() {
        return None;
    }
    // Preserve the existing eta proof, including checked facade bindings that
    // describe an ordinary nominal constructor only by its explicit arity,
    // and higher-kinded aliases whose constructor parameter is already supplied.
    if let Some(constructor) = eta_reduce(
        resolved,
        template.clone(),
        owner,
        remaining,
        explicit_holes,
        binder_arities,
    ) {
        return Some(constructor);
    }
    section_reduce(
        resolved,
        template.clone(),
        owner,
        parameters,
        remaining,
        explicit_holes,
        binder_arities,
    )
    .or_else(|| type_function(resolved, template, owner, parameters, remaining))
}

/// A transparent alias may use a remaining parameter more than once, in a
/// different position, or not at all. Keep that mapping instead of flattening
/// its variables into nominal holes. The finite producer proves value-kinded
/// remaining domains and a closed value-kinded body; higher-order inverse
/// inference is deliberately not introduced here.
fn type_function(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    remaining: &[usize],
) -> Option<ContractType> {
    if remaining.iter().any(|index| {
        parameters
            .get(*index)
            .is_none_or(|parameter| parameter.arity != 0)
    }) || template_kinds(resolved, &template, owner, parameters, false)? != Vec::<u32>::new()
    {
        return None;
    }
    fn replace(
        value: &ContractType,
        owner: &TypeIdentity,
        remaining: &[usize],
        depth: u32,
    ) -> Option<ContractType> {
        use seseragi_syntax::type_functions::TypeFunctionTerm;
        match value {
            ContractType::AliasSlot {
                owner: actual,
                index,
            } if actual == owner => {
                let index = remaining
                    .iter()
                    .position(|remaining| *remaining == *index as usize)?;
                Some(ContractType::TypeFunctionParameter {
                    depth,
                    index: index as u32,
                })
            }
            ContractType::AliasSlot { .. } | ContractType::Hole => None,
            ContractType::TypeFunction {
                parameter_kinds,
                body,
            } => Some(ContractType::TypeFunction {
                parameter_kinds: parameter_kinds.clone(),
                body: Box::new(replace(body, owner, remaining, depth.checked_add(1)?)?),
            }),
            _ => value.map_type_children(&mut |child| replace(child, owner, remaining, depth)),
        }
    }
    Some(ContractType::TypeFunction {
        parameter_kinds: vec![0; remaining.len()],
        body: Box::new(replace(&template, owner, remaining, 0)?),
    })
}

fn template_kinds(
    resolved: &ResolvedModule,
    value: &ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    allow_requirement_merge: bool,
) -> Option<Vec<u32>> {
    Some(match value {
        ContractType::AliasSlot {
            owner: actual,
            index,
        } if actual == owner => {
            vec![0; parameters.get(*index as usize)?.arity as usize]
        }
        ContractType::TypeFunction {
            parameter_kinds, ..
        } => parameter_kinds.clone(),
        ContractType::Named(TypeIdentity::Canonical(canonical)) => {
            rigid_parameter_kinds(resolved, canonical)?
        }
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let kinds = template_kinds(resolved, constructor, owner, parameters, false)?;
            if arguments.len() > kinds.len() {
                return None;
            }
            let is_effect_or_stream = matches!(
                constructor.as_ref(),
                ContractType::Named(TypeIdentity::Canonical(canonical))
                    if matches!(canonical.as_str(), "std/prelude::Effect" | "std/prelude::Stream")
            );
            for (index, (argument, kind)) in arguments.iter().zip(&kinds).enumerate() {
                let allow_merge = is_effect_or_stream && index == 0;
                if template_kinds(resolved, argument, owner, parameters, allow_merge)?
                    != vec![0; *kind as usize]
                {
                    return None;
                }
            }
            kinds[arguments.len()..].to_vec()
        }
        ContractType::Function { parameter, result } => {
            if !template_kinds(resolved, parameter, owner, parameters, false)?.is_empty()
                || !template_kinds(resolved, result, owner, parameters, false)?.is_empty()
            {
                return None;
            }
            vec![]
        }
        ContractType::Tuple(elements) => {
            for element in elements {
                if !template_kinds(resolved, element, owner, parameters, false)?.is_empty() {
                    return None;
                }
            }
            vec![]
        }
        ContractType::Record {
            closed: true,
            fields,
        } => {
            for (_, _, field) in fields {
                if !template_kinds(resolved, field, owner, parameters, false)?.is_empty() {
                    return None;
                }
            }
            vec![]
        }
        ContractType::RequirementMerge(operands) if allow_requirement_merge => {
            for operand in operands {
                if !template_kinds(resolved, operand, owner, parameters, true)?.is_empty() {
                    return None;
                }
            }
            vec![]
        }
        // Structural bodies have a value kind only when every component
        // proves that kind in the same alias-slot scope. An open row, merge
        // outside Effect/Stream, private slot or higher-order domain still has
        // no closed-body proof. No wildcard, unresolved free binder or nested
        // unproved function is evidence of the required closed constructor kind.
        _ => return None,
    })
}

fn section_reduce(
    resolved: &ResolvedModule,
    template: ContractType,
    owner: &TypeIdentity,
    parameters: &[TypeParameter],
    remaining: &[usize],
    explicit_holes: bool,
    binder_arities: &[u32],
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
    let remaining = remaining
        .iter()
        .map(|&index| (parameters.get(index)?.arity == 0).then_some(index as u32))
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
                seen.push(index);
                projected.push(ContractType::Hole);
            }
            fixed => {
                // A private AliasSlot cannot pass this recursive kind proof.
                // Explicit holes are admitted only as slots of a checked
                // canonical constructor application, never as unknown leaves.
                if remaining_kinds(resolved, &fixed, binder_arities)? != vec![0; kind as usize] {
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
    if !explicit_holes {
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
    remaining: &[usize],
    explicit_holes: bool,
    binder_arities: &[u32],
) -> Option<ContractType> {
    let ContractType::Apply {
        constructor,
        mut arguments,
    } = template
    else {
        return None;
    };
    for &index in remaining.iter().rev() {
        if arguments.pop()?
            != (ContractType::AliasSlot {
                owner: owner.clone(),
                index: index as u32,
            })
        {
            return None;
        }
    }
    if contains_unrepresented_slot(&constructor)
        || arguments.iter().any(contains_unrepresented_slot)
    {
        return None;
    }
    let reduced = super::apply_arguments(*constructor, arguments)?;
    let expected = vec![0; remaining.len()];
    let proved = remaining_kinds(resolved, &reduced, binder_arities)
        .is_some_and(|kinds| kinds == expected)
        || match &reduced {
            ContractType::Named(TypeIdentity::Canonical(canonical)) => {
                rigid_arity(resolved, canonical) == Some(remaining.len())
            }
            ContractType::Apply {
                constructor,
                arguments,
            } => matches!(
                constructor.as_ref(),
                ContractType::Named(TypeIdentity::Canonical(canonical))
                    if rigid_arity(resolved, canonical) == Some(arguments.len() + remaining.len())
            ),
            _ => false,
        };
    if !proved {
        return None;
    }
    if explicit_holes {
        super::apply_arguments(reduced, vec![ContractType::Hole; remaining.len()])
    } else {
        Some(reduced)
    }
}

pub(super) fn omitted_suffix(resolved: &ResolvedModule, type_ref: &ContractType) -> Option<usize> {
    let (constructor, arguments) = match type_ref {
        ContractType::Apply {
            constructor,
            arguments,
        } => (constructor.as_ref(), arguments.as_slice()),
        constructor => (constructor, &[][..]),
    };
    let arity = match constructor {
        ContractType::Named(TypeIdentity::Canonical(canonical)) => {
            rigid_parameter_kinds(resolved, canonical)
                .map(|kinds| kinds.len())
                .or_else(|| rigid_arity(resolved, canonical))?
        }
        ContractType::Parameter(symbol_id) => type_parameter_arity(resolved, *symbol_id)? as usize,
        _ => return None,
    };
    arity.checked_sub(arguments.len())
}

pub(super) fn remaining_kinds(
    resolved: &ResolvedModule,
    type_ref: &ContractType,
    binder_arities: &[u32],
) -> Option<Vec<u32>> {
    match type_ref {
        ContractType::TypeFunction {
            parameter_kinds, ..
        } => Some(parameter_kinds.clone()),
        ContractType::Binder(index) => binder_arities
            .get(*index as usize)
            .map(|arity| vec![0; *arity as usize]),
        ContractType::Parameter(symbol_id) => {
            type_parameter_arity(resolved, *symbol_id).map(|arity| vec![0; arity as usize])
        }
        ContractType::Named(TypeIdentity::Canonical(canonical)) => {
            rigid_parameter_kinds(resolved, canonical)
                .or_else(|| rigid_arity(resolved, canonical).map(|arity| vec![0; arity]))
        }
        ContractType::Apply {
            constructor,
            arguments,
        } => {
            let kinds = remaining_kinds(resolved, constructor, binder_arities)?;
            if arguments.len() > kinds.len() {
                return None;
            }
            let mut remaining = Vec::new();
            for (argument, kind) in arguments.iter().zip(&kinds) {
                if *argument == ContractType::Hole {
                    remaining.push(*kind);
                } else if remaining_kinds(resolved, argument, binder_arities)?
                    != vec![0; *kind as usize]
                {
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

fn type_parameter_arity(resolved: &ResolvedModule, id: SymbolId) -> Option<u32> {
    let symbol = resolved
        .symbols
        .iter()
        .find(|symbol| symbol.id == id && symbol.kind == SymbolKind::TypeParameter)?;
    let scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.id == symbol.scope)?;
    let find_in = |parameters: &[TypeParameter]| {
        parameters
            .iter()
            .find(|parameter| parameter.name == symbol.spelling)
            .map(|parameter| parameter.arity)
    };
    for declaration in &resolved.declarations {
        match scope.kind {
            ScopeKind::Declaration if declaration.span() == scope.origin => {
                let parameters = match declaration {
                    SurfaceDecl::Type {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Newtype {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Struct {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Alias {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Trait {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Instance {
                        type_parameters, ..
                    }
                    | SurfaceDecl::Impl {
                        type_parameters, ..
                    } => type_parameters.as_slice(),
                    _ => continue,
                };
                if let Some(arity) = find_in(parameters) {
                    return Some(arity);
                }
            }
            ScopeKind::Function => match declaration {
                SurfaceDecl::Fn {
                    type_parameters,
                    span,
                    ..
                } if *span == scope.origin => {
                    if let Some(arity) = find_in(type_parameters) {
                        return Some(arity);
                    }
                }
                SurfaceDecl::Trait { methods, .. } | SurfaceDecl::Instance { methods, .. } => {
                    for method in methods {
                        if method.span == scope.origin {
                            if let Some(arity) = find_in(&method.type_parameters) {
                                return Some(arity);
                            }
                        }
                    }
                }
                SurfaceDecl::Impl { members, .. } => {
                    for member in members {
                        if let SurfaceImplMember::Method { method, .. } = member {
                            if method.span == scope.origin {
                                if let Some(arity) = find_in(&method.type_parameters) {
                                    return Some(arity);
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    None
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
