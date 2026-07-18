use crate::{ResolvedModule, ScopeKind, SymbolId, SymbolKind, SymbolNamespace, TypedType};
use seseragi_syntax::{SurfaceDecl, TypeRef};
use std::collections::{BTreeMap, BTreeSet};

use super::type_ref::typed_type_from_type_ref;

mod constructors;
mod imports;
mod prelude;
mod substitution;

pub(crate) use substitution::{
    instantiate_callable, instantiate_callable_indexed, substitute_remaining_scheme_parameters,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SemanticTypeKey {
    Adt {
        owner: SymbolId,
        arguments: Vec<SemanticValueType>,
    },
    Struct {
        owner: SymbolId,
        arguments: Vec<SemanticValueType>,
    },
    TypeParameter(SymbolId),
    SchemeParameter(String),
    ExternalNominal {
        canonical: String,
        arguments: Vec<SemanticValueType>,
    },
    Tuple(Vec<SemanticTypeKey>),
    Other,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticValueType {
    pub(crate) type_ref: TypedType,
    pub(crate) key: SemanticTypeKey,
}

pub(crate) fn semantic_values_are_compatible(
    expected: &SemanticValueType,
    actual: &SemanticValueType,
) -> bool {
    if let (
        TypedType::Record {
            fields: expected_fields,
            ..
        },
        TypedType::Record {
            fields: actual_fields,
            ..
        },
    ) = (&expected.type_ref, &actual.type_ref)
    {
        return record_is_compatible(expected_fields, actual_fields);
    }
    match (&expected.key, &actual.key) {
        (SemanticTypeKey::Invalid, _) | (_, SemanticTypeKey::Invalid) => true,
        (
            SemanticTypeKey::Adt {
                owner: expected_owner,
                arguments: expected_arguments,
            },
            SemanticTypeKey::Adt {
                owner: actual_owner,
                arguments: actual_arguments,
            },
        ) => {
            expected_owner == actual_owner
                && expected_arguments.len() == actual_arguments.len()
                && expected_arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| semantic_values_are_compatible(expected, actual))
        }
        (SemanticTypeKey::Adt { .. }, _) | (_, SemanticTypeKey::Adt { .. }) => false,
        (
            SemanticTypeKey::Struct {
                owner: expected_owner,
                arguments: expected_arguments,
            },
            SemanticTypeKey::Struct {
                owner: actual_owner,
                arguments: actual_arguments,
            },
        ) => {
            expected_owner == actual_owner
                && expected_arguments.len() == actual_arguments.len()
                && expected_arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| semantic_values_are_compatible(expected, actual))
        }
        (SemanticTypeKey::Struct { .. }, _) | (_, SemanticTypeKey::Struct { .. }) => false,
        (
            SemanticTypeKey::ExternalNominal {
                canonical: expected_canonical,
                arguments: expected_arguments,
            },
            SemanticTypeKey::ExternalNominal {
                canonical: actual_canonical,
                arguments: actual_arguments,
            },
        ) => {
            expected_canonical == actual_canonical
                && expected_arguments.len() == actual_arguments.len()
                && expected_arguments
                    .iter()
                    .zip(actual_arguments)
                    .all(|(expected, actual)| semantic_values_are_compatible(expected, actual))
        }
        (SemanticTypeKey::ExternalNominal { .. }, _)
        | (_, SemanticTypeKey::ExternalNominal { .. }) => false,
        (SemanticTypeKey::Tuple(expected_keys), SemanticTypeKey::Tuple(actual_keys)) => {
            let (
                TypedType::Tuple {
                    elements: expected_types,
                },
                TypedType::Tuple {
                    elements: actual_types,
                },
            ) = (&expected.type_ref, &actual.type_ref)
            else {
                return false;
            };
            expected_keys.len() == actual_keys.len()
                && expected_types.len() == actual_types.len()
                && expected_keys.len() == expected_types.len()
                && expected_keys
                    .iter()
                    .zip(actual_keys)
                    .zip(expected_types.iter().zip(actual_types))
                    .all(
                        |((expected_key, actual_key), (expected_type, actual_type))| {
                            semantic_values_are_compatible(
                                &SemanticValueType {
                                    type_ref: expected_type.clone(),
                                    key: expected_key.clone(),
                                },
                                &SemanticValueType {
                                    type_ref: actual_type.clone(),
                                    key: actual_key.clone(),
                                },
                            )
                        },
                    )
        }
        (SemanticTypeKey::Tuple(_), _) | (_, SemanticTypeKey::Tuple(_)) => false,
        _ => expected.type_ref == actual.type_ref,
    }
}

fn record_is_compatible(
    expected: &[crate::TypedRecordField],
    actual: &[crate::TypedRecordField],
) -> bool {
    expected.iter().all(|required| {
        let found = actual.iter().find(|field| field.name == required.name);
        if required.optional {
            return found.is_none_or(|field| field.type_ref == required.type_ref);
        }
        found.is_some_and(|field| !field.optional && field.type_ref == required.type_ref)
    })
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticAdt {
    pub(crate) name: String,
    pub(crate) type_parameters: Vec<SymbolId>,
    pub(crate) type_parameter_names: Vec<String>,
    pub(crate) variants: Vec<SemanticVariant>,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticVariant {
    pub(crate) constructor: SymbolId,
    pub(crate) canonical: String,
    pub(crate) spelling: String,
    pub(crate) payload: Option<SemanticValueType>,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticStruct {
    pub(crate) name: String,
    pub(crate) type_parameters: Vec<SymbolId>,
    pub(crate) type_parameter_names: Vec<String>,
    pub(crate) fields: Vec<SemanticStructField>,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticStructField {
    pub(crate) name: String,
    pub(crate) type_ref: SemanticValueType,
}

#[derive(Clone, Debug)]
pub(crate) struct SemanticConstructorSignature {
    pub(crate) symbol: String,
    pub(crate) type_parameters: Vec<String>,
    pub(crate) parameters: Vec<SemanticValueType>,
    pub(crate) result: SemanticValueType,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct SemanticTypeCatalog {
    adts: BTreeMap<SymbolId, SemanticAdt>,
    structs: BTreeMap<SymbolId, SemanticStruct>,
    constructors: BTreeMap<SymbolId, SymbolId>,
}

impl SemanticTypeCatalog {
    pub(crate) fn new(resolved: &ResolvedModule) -> Self {
        let declarations = resolved
            .declarations
            .iter()
            .filter_map(|declaration| {
                let (name_span, type_parameters, span) = match declaration {
                    SurfaceDecl::Type {
                        name_span,
                        type_parameters,
                        span,
                        ..
                    }
                    | SurfaceDecl::Newtype {
                        name_span,
                        type_parameters,
                        span,
                        ..
                    } => (name_span, type_parameters, span),
                    _ => return None,
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
                Some((
                    (name_span.start, name_span.end),
                    (
                        symbol.id,
                        symbol.spelling.clone(),
                        parameters,
                        type_parameters.clone(),
                    ),
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let owners = declarations
            .values()
            .map(|(owner, ..)| *owner)
            .collect::<BTreeSet<_>>();
        let struct_declarations = resolved
            .declarations
            .iter()
            .filter_map(|declaration| {
                let SurfaceDecl::Struct {
                    name_span,
                    type_parameters,
                    fields,
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
                Some((
                    symbol.id,
                    symbol.spelling.clone(),
                    parameters,
                    type_parameters.clone(),
                    fields,
                ))
            })
            .collect::<Vec<_>>();
        let struct_owners = struct_declarations
            .iter()
            .map(|(owner, ..)| *owner)
            .collect::<BTreeSet<_>>();

        let mut catalog = Self::default();
        for (owner, name, parameters, parameter_names) in declarations.values() {
            catalog.adts.insert(
                *owner,
                SemanticAdt {
                    name: name.clone(),
                    type_parameters: parameters.clone(),
                    type_parameter_names: parameter_names
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    variants: Vec::new(),
                },
            );
        }
        for (owner, name, parameters, parameter_names, fields) in struct_declarations {
            let fields = fields
                .iter()
                .map(|field| SemanticStructField {
                    name: field.name.clone(),
                    type_ref: SemanticValueType {
                        type_ref: typed_type_from_type_ref(&field.type_ref),
                        key: semantic_key_from_type_ref(
                            resolved,
                            &owners,
                            &struct_owners,
                            &field.type_ref,
                        ),
                    },
                })
                .collect();
            catalog.structs.insert(
                owner,
                SemanticStruct {
                    name,
                    type_parameters: parameters,
                    type_parameter_names: parameter_names
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect(),
                    fields,
                },
            );
        }
        for declaration in &resolved.declarations {
            let (name_span, variants) = match declaration {
                SurfaceDecl::Type {
                    name_span,
                    variants,
                    ..
                } => (
                    name_span,
                    variants
                        .iter()
                        .map(|variant| (variant.name_span, variant.payload.as_ref()))
                        .collect::<Vec<_>>(),
                ),
                SurfaceDecl::Newtype {
                    name_span,
                    representation,
                    ..
                } => (name_span, vec![(*name_span, Some(representation))]),
                _ => continue,
            };
            let Some((owner, ..)) = declarations.get(&(name_span.start, name_span.end)) else {
                continue;
            };
            let owner = *owner;
            let mut semantic_variants = Vec::new();
            for (variant_span, payload) in variants {
                let Some(symbol) = resolved.symbols.iter().find(|symbol| {
                    symbol.kind == SymbolKind::Constructor && symbol.origin == variant_span
                }) else {
                    continue;
                };
                let Some(canonical) = symbol.canonical.clone() else {
                    continue;
                };
                let payload = payload.map(|payload| SemanticValueType {
                    type_ref: typed_type_from_type_ref(payload),
                    key: semantic_key_from_type_ref(resolved, &owners, &struct_owners, payload),
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
        catalog.collect_prelude_sum_types(resolved);
        catalog.collect_imported_adts(resolved);
        catalog.collect_imported_structs(resolved);
        catalog
    }

    pub(crate) fn key_from_type_ref(
        &self,
        resolved: &ResolvedModule,
        type_ref: &TypeRef,
    ) -> SemanticTypeKey {
        let owners = self.adts.keys().copied().collect::<BTreeSet<_>>();
        let struct_owners = self.structs.keys().copied().collect::<BTreeSet<_>>();
        semantic_key_from_type_ref(resolved, &owners, &struct_owners, type_ref)
    }

    pub(crate) fn key_from_typed_type(
        &self,
        resolved: &ResolvedModule,
        type_ref: &TypedType,
    ) -> SemanticTypeKey {
        match type_ref {
            TypedType::Named { name, arguments } => {
                let mut owners = resolved
                    .symbols
                    .iter()
                    .filter(|symbol| {
                        symbol.kind == SymbolKind::Type
                            && symbol.spelling == *name
                            && self.adts.contains_key(&symbol.id)
                    })
                    .map(|symbol| symbol.id);
                let owner = owners.next().filter(|_| owners.next().is_none());
                let arguments = arguments
                    .iter()
                    .map(|argument| SemanticValueType {
                        type_ref: argument.clone(),
                        key: self.key_from_typed_type(resolved, argument),
                    })
                    .collect::<Vec<_>>();
                match owner {
                    Some(owner) => SemanticTypeKey::Adt { owner, arguments },
                    None => {
                        let mut owners = resolved
                            .symbols
                            .iter()
                            .filter(|symbol| {
                                symbol.kind == SymbolKind::Type
                                    && symbol.spelling == *name
                                    && self.structs.contains_key(&symbol.id)
                            })
                            .map(|symbol| symbol.id);
                        let owner = owners.next().filter(|_| owners.next().is_none());
                        owner.map_or(SemanticTypeKey::Other, |owner| SemanticTypeKey::Struct {
                            owner,
                            arguments,
                        })
                    }
                }
            }
            TypedType::ExternalNamed {
                canonical,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| SemanticValueType {
                        type_ref: argument.clone(),
                        key: self.key_from_typed_type(resolved, argument),
                    })
                    .collect::<Vec<_>>();
                let mut owners = resolved
                    .symbols
                    .iter()
                    .filter(|symbol| {
                        symbol.kind == SymbolKind::Type
                            && symbol.canonical.as_deref() == Some(canonical.as_str())
                            && self.adts.contains_key(&symbol.id)
                    })
                    .map(|symbol| symbol.id);
                let owner = owners.next().filter(|_| owners.next().is_none());
                match owner {
                    Some(owner) => SemanticTypeKey::Adt { owner, arguments },
                    None => {
                        let mut owners = resolved
                            .symbols
                            .iter()
                            .filter(|symbol| {
                                symbol.kind == SymbolKind::Type
                                    && symbol.canonical.as_deref() == Some(canonical.as_str())
                                    && self.structs.contains_key(&symbol.id)
                            })
                            .map(|symbol| symbol.id);
                        let owner = owners.next().filter(|_| owners.next().is_none());
                        match owner {
                            Some(owner) => SemanticTypeKey::Struct { owner, arguments },
                            None => SemanticTypeKey::ExternalNominal {
                                canonical: canonical.clone(),
                                arguments,
                            },
                        }
                    }
                }
            }
            TypedType::Tuple { elements } => SemanticTypeKey::Tuple(
                elements
                    .iter()
                    .map(|element| self.key_from_typed_type(resolved, element))
                    .collect(),
            ),
            TypedType::Hole => SemanticTypeKey::Invalid,
            TypedType::Record { .. } | TypedType::Function { .. } => SemanticTypeKey::Other,
        }
    }

    pub(crate) fn adt(&self, owner: SymbolId) -> Option<&SemanticAdt> {
        self.adts.get(&owner)
    }

    pub(crate) fn struct_type(&self, owner: SymbolId) -> Option<&SemanticStruct> {
        self.structs.get(&owner)
    }
}

fn semantic_key_from_type_ref(
    resolved: &ResolvedModule,
    owners: &BTreeSet<SymbolId>,
    struct_owners: &BTreeSet<SymbolId>,
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
                    .map(|symbol| (target, symbol.kind, symbol.canonical.as_deref()))
            }) {
                Some((owner, SymbolKind::Type, _)) if owners.contains(&owner) => {
                    SemanticTypeKey::Adt {
                        owner,
                        arguments: arguments
                            .iter()
                            .map(|argument| SemanticValueType {
                                type_ref: typed_type_from_type_ref(argument),
                                key: semantic_key_from_type_ref(
                                    resolved,
                                    owners,
                                    struct_owners,
                                    argument,
                                ),
                            })
                            .collect(),
                    }
                }
                Some((owner, SymbolKind::Type, _)) if struct_owners.contains(&owner) => {
                    SemanticTypeKey::Struct {
                        owner,
                        arguments: arguments
                            .iter()
                            .map(|argument| SemanticValueType {
                                type_ref: typed_type_from_type_ref(argument),
                                key: semantic_key_from_type_ref(
                                    resolved,
                                    owners,
                                    struct_owners,
                                    argument,
                                ),
                            })
                            .collect(),
                    }
                }
                Some((_, SymbolKind::Type, Some(canonical)))
                    if crate::prelude::is_external_nominal_type(canonical) =>
                {
                    SemanticTypeKey::ExternalNominal {
                        canonical: canonical.to_owned(),
                        arguments: arguments
                            .iter()
                            .map(|argument| SemanticValueType {
                                type_ref: typed_type_from_type_ref(argument),
                                key: semantic_key_from_type_ref(
                                    resolved,
                                    owners,
                                    struct_owners,
                                    argument,
                                ),
                            })
                            .collect(),
                    }
                }
                Some((parameter, SymbolKind::TypeParameter, _)) => {
                    SemanticTypeKey::TypeParameter(parameter)
                }
                _ => SemanticTypeKey::Other,
            }
        }
        TypeRef::Tuple { elements, .. } => SemanticTypeKey::Tuple(
            elements
                .iter()
                .map(|element| semantic_key_from_type_ref(resolved, owners, struct_owners, element))
                .collect(),
        ),
        TypeRef::Hole { .. } => SemanticTypeKey::Invalid,
        TypeRef::Record { .. } | TypeRef::Function { .. } => SemanticTypeKey::Other,
    }
}
