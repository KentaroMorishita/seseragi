use crate::{ExternalTypeBinding, ResolvedModule, SymbolId, TypedRecordField, TypedType};
use seseragi_syntax::InterfaceType;
use std::collections::{BTreeMap, BTreeSet};

use super::super::semantic_types::{SemanticTypeKey, SemanticValueType};
use super::super::type_ref::typed_type_from_interface_type;

pub(super) struct ImportedTypeContext {
    names: BTreeMap<(String, String), String>,
    canonical_names: BTreeMap<String, String>,
    owners: BTreeMap<(String, String), SymbolId>,
    canonical_owners: BTreeMap<String, SymbolId>,
}

impl ImportedTypeContext {
    pub(super) fn new(resolved: &ResolvedModule) -> Self {
        let names = resolved
            .imports
            .iter()
            .filter(|import| import.in_scope && import.export.namespace == "type")
            .map(|import| {
                (
                    (import.module.clone(), import.export.name.clone()),
                    import.local_name.clone(),
                )
            })
            .collect();
        let owners = resolved
            .imports
            .iter()
            .filter(|import| {
                import.export.namespace == "type"
                    && import.export.declaration_kind.as_deref() == Some("type")
            })
            .map(|import| {
                (
                    (import.module.clone(), import.export.name.clone()),
                    import.symbol,
                )
            })
            .collect();
        let canonical_names = resolved
            .imports
            .iter()
            .filter(|import| import.export.namespace == "type")
            .map(|import| (import.export.symbol.clone(), import.local_name.clone()))
            .collect();
        let canonical_owners = resolved
            .imports
            .iter()
            .filter(|import| import.export.namespace == "type")
            .map(|import| (import.export.symbol.clone(), import.symbol))
            .collect();
        Self {
            names,
            canonical_names,
            owners,
            canonical_owners,
        }
    }

    pub(super) fn semantic_value(
        &self,
        type_ref: InterfaceType,
        module: &str,
        type_parameters: &BTreeSet<String>,
        bindings: &[ExternalTypeBinding],
    ) -> Option<SemanticValueType> {
        let key = self.semantic_key(&type_ref, module, type_parameters, bindings)?;
        let type_ref = typed_type_from_interface_type(type_ref)?;
        Some(SemanticValueType {
            type_ref: self.localize(type_ref, module, type_parameters, bindings),
            key,
        })
    }

    fn localize(
        &self,
        type_ref: TypedType,
        module: &str,
        type_parameters: &BTreeSet<String>,
        bindings: &[ExternalTypeBinding],
    ) -> TypedType {
        match type_ref {
            TypedType::Named { name, arguments } => {
                let localized_name = if type_parameters.contains(&name) {
                    name.clone()
                } else {
                    self.names
                        .get(&(module.to_owned(), name.clone()))
                        .cloned()
                        .unwrap_or_else(|| name.clone())
                };
                let arguments = arguments
                    .into_iter()
                    .map(|argument| self.localize(argument, module, type_parameters, bindings))
                    .collect();
                match bindings.iter().find(|binding| binding.spelling == name) {
                    Some(binding) => TypedType::ExternalNamed {
                        name: localized_name,
                        canonical: binding.canonical.clone(),
                        arguments,
                    },
                    None => TypedType::Named {
                        name: localized_name,
                        arguments,
                    },
                }
            }
            TypedType::ExternalNamed {
                name,
                canonical,
                arguments,
            } => TypedType::ExternalNamed {
                name: self
                    .canonical_names
                    .get(&canonical)
                    .cloned()
                    .unwrap_or(name),
                canonical,
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.localize(argument, module, type_parameters, bindings))
                    .collect(),
            },
            TypedType::Record { closed, fields } => TypedType::Record {
                closed,
                fields: fields
                    .into_iter()
                    .map(|field| TypedRecordField {
                        name: field.name,
                        optional: field.optional,
                        type_ref: self.localize(field.type_ref, module, type_parameters, bindings),
                    })
                    .collect(),
            },
            TypedType::Tuple { elements } => TypedType::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| self.localize(element, module, type_parameters, bindings))
                    .collect(),
            },
            TypedType::Function { parameter, result } => TypedType::Function {
                parameter: Box::new(self.localize(*parameter, module, type_parameters, bindings)),
                result: Box::new(self.localize(*result, module, type_parameters, bindings)),
            },
            TypedType::Hole => TypedType::Hole,
        }
    }

    fn semantic_key(
        &self,
        type_ref: &InterfaceType,
        module: &str,
        type_parameters: &BTreeSet<String>,
        bindings: &[ExternalTypeBinding],
    ) -> Option<SemanticTypeKey> {
        match type_ref {
            InterfaceType::Named { name, arguments } => {
                if arguments.is_empty() && type_parameters.contains(name) {
                    return Some(SemanticTypeKey::SchemeParameter(name.clone()));
                }
                let arguments = arguments
                    .iter()
                    .cloned()
                    .map(|argument| {
                        self.semantic_value(argument, module, type_parameters, bindings)
                    })
                    .collect::<Option<Vec<_>>>()?;
                if let Some(binding) = bindings.iter().find(|binding| binding.spelling == *name) {
                    return Some(self.canonical_owners.get(&binding.canonical).map_or_else(
                        || SemanticTypeKey::ExternalNominal {
                            canonical: binding.canonical.clone(),
                            arguments: arguments.clone(),
                        },
                        |owner| SemanticTypeKey::Adt {
                            owner: *owner,
                            arguments: arguments.clone(),
                        },
                    ));
                }
                self.owners.get(&(module.to_owned(), name.clone())).map_or(
                    Some(SemanticTypeKey::Other),
                    |owner| {
                        Some(SemanticTypeKey::Adt {
                            owner: *owner,
                            arguments,
                        })
                    },
                )
            }
            InterfaceType::ExternalNamed {
                canonical,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .cloned()
                    .map(|argument| {
                        self.semantic_value(argument, module, type_parameters, bindings)
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(match self.canonical_owners.get(canonical) {
                    Some(owner) => SemanticTypeKey::Adt {
                        owner: *owner,
                        arguments,
                    },
                    None => SemanticTypeKey::ExternalNominal {
                        canonical: canonical.clone(),
                        arguments,
                    },
                })
            }
            InterfaceType::Tuple { elements } => Some(SemanticTypeKey::Tuple(
                elements
                    .iter()
                    .map(|element| self.semantic_key(element, module, type_parameters, bindings))
                    .collect::<Option<Vec<_>>>()?,
            )),
            InterfaceType::Hole => Some(SemanticTypeKey::Invalid),
            InterfaceType::Function { .. }
            | InterfaceType::TypeConstructor { .. }
            | InterfaceType::Apply { .. }
            | InterfaceType::Record { .. } => Some(SemanticTypeKey::Other),
        }
    }
}

pub(super) fn flatten_function(type_ref: InterfaceType) -> (Vec<InterfaceType>, InterfaceType) {
    let mut parameters = Vec::new();
    let mut cursor = type_ref;
    loop {
        match cursor {
            InterfaceType::Function { parameter, result } => {
                parameters.push(*parameter);
                cursor = *result;
            }
            result => return (parameters, result),
        }
    }
}
