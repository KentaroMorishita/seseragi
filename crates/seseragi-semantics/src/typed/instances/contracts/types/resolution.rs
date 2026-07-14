use super::model::{apply_arguments, ContractConstraint, ContractType, TypeIdentity};
use crate::{ResolvedModule, ScopeKind, SymbolId, SymbolKind, SymbolNamespace};
use seseragi_syntax::{SurfaceConstraint, SurfaceMethod, TypeRef};
use std::collections::BTreeMap;

pub(super) fn contract_constraint(
    resolved: &ResolvedModule,
    constraint: &SurfaceConstraint,
    binders: &BTreeMap<SymbolId, u32>,
    substitutions: &BTreeMap<SymbolId, ContractType>,
) -> Option<ContractConstraint> {
    let target = reference_target(resolved, SymbolNamespace::Trait, constraint.name_span)?;
    Some(ContractConstraint {
        trait_identity: symbol_identity(resolved, target)?,
        arguments: constraint
            .arguments
            .iter()
            .map(|argument| contract_type(resolved, argument, binders, substitutions))
            .collect::<Option<Vec<_>>>()?,
    })
}

pub(super) fn contract_type(
    resolved: &ResolvedModule,
    type_ref: &TypeRef,
    binders: &BTreeMap<SymbolId, u32>,
    substitutions: &BTreeMap<SymbolId, ContractType>,
) -> Option<ContractType> {
    match type_ref {
        TypeRef::Named {
            arguments, span, ..
        } => {
            let target = reference_target(resolved, SymbolNamespace::Type, *span)?;
            let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
            let arguments = arguments
                .iter()
                .map(|argument| contract_type(resolved, argument, binders, substitutions))
                .collect::<Option<Vec<_>>>()?;
            let constructor = if symbol.kind == SymbolKind::TypeParameter {
                if let Some(index) = binders.get(&target) {
                    ContractType::Binder(*index)
                } else if let Some(substitution) = substitutions.get(&target) {
                    return apply_arguments(substitution.clone(), arguments);
                } else {
                    ContractType::Parameter(target)
                }
            } else {
                ContractType::Named(symbol_identity(resolved, target)?)
            };
            apply_arguments(constructor, arguments)
        }
        TypeRef::Hole { .. } => Some(ContractType::Hole),
        TypeRef::Record { closed, fields, .. } => {
            let mut fields = fields
                .iter()
                .map(|field| {
                    Some((
                        field.name.clone(),
                        field.optional,
                        contract_type(resolved, &field.type_ref, binders, substitutions)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            fields.sort_by(|left, right| left.0.cmp(&right.0));
            Some(ContractType::Record {
                closed: *closed,
                fields,
            })
        }
        TypeRef::Tuple { elements, .. } => Some(ContractType::Tuple(
            elements
                .iter()
                .map(|element| contract_type(resolved, element, binders, substitutions))
                .collect::<Option<Vec<_>>>()?,
        )),
        TypeRef::Function {
            parameter, result, ..
        } => Some(ContractType::Function {
            parameter: Box::new(contract_type(resolved, parameter, binders, substitutions)?),
            result: Box::new(contract_type(resolved, result, binders, substitutions)?),
        }),
    }
}

pub(super) fn method_binders(
    resolved: &ResolvedModule,
    method: &SurfaceMethod,
) -> Option<BTreeMap<SymbolId, u32>> {
    let scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::Function && scope.origin == method.span)?;
    method
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let symbol = resolved.symbols.iter().find(|symbol| {
                symbol.scope == scope.id
                    && symbol.kind == SymbolKind::TypeParameter
                    && symbol.spelling == *name
            })?;
            Some((symbol.id, index as u32))
        })
        .collect()
}

pub(super) fn declaration_type_parameters(
    resolved: &ResolvedModule,
    span: seseragi_syntax::ByteSpan,
    names: &[String],
) -> Option<Vec<SymbolId>> {
    let scope = resolved
        .scopes
        .iter()
        .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == span)?;
    names
        .iter()
        .map(|name| {
            resolved
                .symbols
                .iter()
                .find(|symbol| {
                    symbol.scope == scope.id
                        && symbol.kind == SymbolKind::TypeParameter
                        && symbol.spelling == *name
                })
                .map(|symbol| symbol.id)
        })
        .collect()
}

fn reference_target(
    resolved: &ResolvedModule,
    namespace: SymbolNamespace,
    origin: seseragi_syntax::ByteSpan,
) -> Option<SymbolId> {
    resolved
        .references
        .iter()
        .find(|reference| reference.namespace == namespace && reference.origin == origin)?
        .target
}

fn symbol_identity(resolved: &ResolvedModule, target: SymbolId) -> Option<TypeIdentity> {
    let symbol = resolved.symbols.iter().find(|symbol| symbol.id == target)?;
    Some(match &symbol.canonical {
        Some(canonical) => TypeIdentity::Canonical(canonical.clone()),
        None => TypeIdentity::Symbol(target),
    })
}
