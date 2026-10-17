//! A bounded eta proof, not general type-level lambda unification.
use super::{ContractType, TypeIdentity};
use crate::{ResolvedModule, SymbolKind};
use seseragi_syntax::{InterfaceType, SurfaceDecl};
use std::collections::BTreeSet;

pub(super) fn reduce(
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

fn contains_unrepresented_slot(type_ref: &ContractType) -> bool {
    match type_ref {
        ContractType::AliasSlot { .. } | ContractType::Hole => true,
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
