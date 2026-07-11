use crate::{ResolvedModule, ScopeKind, SymbolId, SymbolKind, SymbolNamespace, TypedType};
use seseragi_syntax::{SurfaceDecl, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::type_ref::typed_type_from_type_ref;

mod substitution;

pub(crate) use substitution::instantiate_callable_result;
use substitution::substitute_type_parameters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SemanticTypeKey {
    Adt {
        owner: SymbolId,
        arguments: Vec<SemanticValueType>,
    },
    TypeParameter(SymbolId),
    Tuple(Vec<SemanticTypeKey>),
    Other,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticValueType {
    pub(crate) type_ref: TypedType,
    pub(crate) key: SemanticTypeKey,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticAdt {
    pub(crate) type_parameters: Vec<SymbolId>,
    pub(crate) variants: Vec<SemanticVariant>,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticVariant {
    pub(crate) constructor: SymbolId,
    pub(crate) canonical: String,
    pub(crate) spelling: String,
    pub(crate) payload: Option<SemanticValueType>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SemanticTypeCatalog {
    adts: BTreeMap<SymbolId, SemanticAdt>,
    constructors: BTreeMap<SymbolId, SymbolId>,
}

impl SemanticTypeCatalog {
    pub(crate) fn new(resolved: &ResolvedModule) -> Self {
        let declarations = resolved
            .declarations
            .iter()
            .filter_map(|declaration| {
                let SurfaceDecl::Type {
                    name_span,
                    type_parameters,
                    span,
                    ..
                } = declaration
                else {
                    return None;
                };
                let symbol = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Type && symbol.origin == *name_span
                })?;
                let scope = resolved
                    .scopes
                    .iter()
                    .find(|scope| scope.kind == ScopeKind::Declaration && scope.origin == *span);
                let parameters = scope
                    .map(|scope| {
                        resolved
                            .symbols
                            .iter()
                            .filter(|candidate| {
                                candidate.kind == SymbolKind::TypeParameter
                                    && candidate.scope == scope.id
                            })
                            .map(|candidate| candidate.id)
                            .take(type_parameters.len())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Some(((name_span.start, name_span.end), (symbol.id, parameters)))
            })
            .collect::<BTreeMap<_, _>>();
        let owners = declarations
            .values()
            .map(|(owner, _)| *owner)
            .collect::<BTreeSet<_>>();

        let mut catalog = Self::default();
        for (owner, parameters) in declarations.values() {
            catalog.adts.insert(
                *owner,
                SemanticAdt {
                    type_parameters: parameters.clone(),
                    variants: Vec::new(),
                },
            );
        }
        for declaration in &resolved.declarations {
            let SurfaceDecl::Type {
                name_span,
                variants,
                ..
            } = declaration
            else {
                continue;
            };
            let Some((owner, _)) = declarations.get(&(name_span.start, name_span.end)) else {
                continue;
            };
            let owner = *owner;
            let mut semantic_variants = Vec::new();
            for variant in variants {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Constructor && symbol.origin == variant.name_span
                }) else {
                    continue;
                };
                let Some(canonical) = symbol.canonical.clone() else {
                    continue;
                };
                let payload = variant.payload.as_ref().map(|payload| SemanticValueType {
                    type_ref: typed_type_from_type_ref(payload),
                    key: semantic_key_from_type_ref(resolved, &owners, payload),
                });
                catalog.constructors.insert(symbol.id, owner);
                semantic_variants.push(SemanticVariant {
                    constructor: symbol.id,
                    canonical,
                    spelling: symbol.spelling.clone(),
                    payload,
                });
            }
            if let Some(adt) = catalog.adts.get_mut(&owner) {
                adt.variants = semantic_variants;
            }
        }
        catalog
    }

    pub(crate) fn key_from_type_ref(
        &self,
        resolved: &ResolvedModule,
        type_ref: &TypeRef,
    ) -> SemanticTypeKey {
        let owners = self.adts.keys().copied().collect::<BTreeSet<_>>();
        semantic_key_from_type_ref(resolved, &owners, type_ref)
    }

    pub(crate) fn adt(&self, owner: SymbolId) -> Option<&SemanticAdt> {
        self.adts.get(&owner)
    }

    pub(crate) fn polymorphic_adt_key(
        &self,
        owner: SymbolId,
        type_ref: &TypedType,
    ) -> SemanticTypeKey {
        let Some(adt) = self.adts.get(&owner) else {
            return SemanticTypeKey::Invalid;
        };
        let TypedType::Named { arguments, .. } = type_ref else {
            return SemanticTypeKey::Invalid;
        };
        SemanticTypeKey::Adt {
            owner,
            arguments: adt
                .type_parameters
                .iter()
                .zip(arguments)
                .map(|(parameter, type_ref)| SemanticValueType {
                    type_ref: type_ref.clone(),
                    key: SemanticTypeKey::TypeParameter(*parameter),
                })
                .collect(),
        }
    }

    pub(crate) fn constructor(
        &self,
        constructor: SymbolId,
    ) -> Option<(SymbolId, &SemanticVariant)> {
        let owner = self.constructors.get(&constructor).copied()?;
        let variant = self
            .adts
            .get(&owner)?
            .variants
            .iter()
            .find(|variant| variant.constructor == constructor)?;
        Some((owner, variant))
    }

    pub(crate) fn instantiate_payload(
        &self,
        owner: SymbolId,
        arguments: &[SemanticValueType],
        payload: &SemanticValueType,
    ) -> SemanticValueType {
        let substitutions = self
            .adts
            .get(&owner)
            .into_iter()
            .flat_map(|adt| adt.type_parameters.iter())
            .copied()
            .zip(arguments.iter().cloned())
            .collect::<BTreeMap<_, _>>();
        substitute_type_parameters(payload, &substitutions)
    }
}

fn semantic_key_from_type_ref(
    resolved: &ResolvedModule,
    owners: &BTreeSet<SymbolId>,
    type_ref: &TypeRef,
) -> SemanticTypeKey {
    match type_ref {
        TypeRef::Named {
            arguments, span, ..
        } => {
            let target = resolved
                .references
                .iter()
                .find(|reference| {
                    reference.namespace == SymbolNamespace::Type && reference.origin == *span
                })
                .and_then(|reference| reference.target);
            match target.and_then(|target| {
                resolved
                    .symbols
                    .iter()
                    .find(|symbol| symbol.id == target)
                    .map(|symbol| (target, symbol.kind))
            }) {
                Some((owner, SymbolKind::Type)) if owners.contains(&owner) => {
                    SemanticTypeKey::Adt {
                        owner,
                        arguments: arguments
                            .iter()
                            .map(|argument| SemanticValueType {
                                type_ref: typed_type_from_type_ref(argument),
                                key: semantic_key_from_type_ref(resolved, owners, argument),
                            })
                            .collect(),
                    }
                }
                Some((parameter, SymbolKind::TypeParameter)) => {
                    SemanticTypeKey::TypeParameter(parameter)
                }
                _ => SemanticTypeKey::Other,
            }
        }
        TypeRef::Tuple { elements, .. } => SemanticTypeKey::Tuple(
            elements
                .iter()
                .map(|element| semantic_key_from_type_ref(resolved, owners, element))
                .collect(),
        ),
        TypeRef::Hole { .. } => SemanticTypeKey::Invalid,
        TypeRef::Record { .. } | TypeRef::Function { .. } => SemanticTypeKey::Other,
    }
}
