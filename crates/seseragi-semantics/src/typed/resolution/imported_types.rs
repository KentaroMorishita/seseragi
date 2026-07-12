use crate::{ResolvedModule, SymbolId, TypedRecordField, TypedType};
use seseragi_syntax::InterfaceType;
use std::collections::{BTreeMap, BTreeSet};

use super::super::semantic_types::{SemanticTypeKey, SemanticValueType};
use super::super::type_ref::typed_type_from_interface_type;

pub(super) struct ImportedTypeContext {
    names: BTreeMap<(String, String), String>,
    owners: BTreeMap<(String, String), SymbolId>,
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
        Self { names, owners }
    }

    pub(super) fn semantic_value(
        &self,
        type_ref: InterfaceType,
        module: &str,
        type_parameters: &BTreeSet<String>,
    ) -> Option<SemanticValueType> {
        let key = self.semantic_key(&type_ref, module, type_parameters)?;
        let type_ref = typed_type_from_interface_type(type_ref)?;
        Some(SemanticValueType {
            type_ref: self.localize(type_ref, module, type_parameters),
            key,
        })
    }

    fn localize(
        &self,
        type_ref: TypedType,
        module: &str,
        type_parameters: &BTreeSet<String>,
    ) -> TypedType {
        match type_ref {
            TypedType::Named { name, arguments } => TypedType::Named {
                name: if type_parameters.contains(&name) {
                    name
                } else {
                    self.names
                        .get(&(module.to_owned(), name.clone()))
                        .cloned()
                        .unwrap_or(name)
                },
                arguments: arguments
                    .into_iter()
                    .map(|argument| self.localize(argument, module, type_parameters))
                    .collect(),
            },
            TypedType::Record { closed, fields } => TypedType::Record {
                closed,
                fields: fields
                    .into_iter()
                    .map(|field| TypedRecordField {
                        name: field.name,
                        optional: field.optional,
                        type_ref: self.localize(field.type_ref, module, type_parameters),
                    })
                    .collect(),
            },
            TypedType::Tuple { elements } => TypedType::Tuple {
                elements: elements
                    .into_iter()
                    .map(|element| self.localize(element, module, type_parameters))
                    .collect(),
            },
            TypedType::Function { parameter, result } => TypedType::Function {
                parameter: Box::new(self.localize(*parameter, module, type_parameters)),
                result: Box::new(self.localize(*result, module, type_parameters)),
            },
            TypedType::Hole => TypedType::Hole,
        }
    }

    fn semantic_key(
        &self,
        type_ref: &InterfaceType,
        module: &str,
        type_parameters: &BTreeSet<String>,
    ) -> Option<SemanticTypeKey> {
        match type_ref {
            InterfaceType::Named { name, arguments } => {
                if arguments.is_empty() && type_parameters.contains(name) {
                    return Some(SemanticTypeKey::SchemeParameter(name.clone()));
                }
                let Some(owner) = self.owners.get(&(module.to_owned(), name.clone())) else {
                    return Some(SemanticTypeKey::Other);
                };
                let arguments = arguments
                    .iter()
                    .cloned()
                    .map(|argument| self.semantic_value(argument, module, type_parameters))
                    .collect::<Option<Vec<_>>>()?;
                Some(SemanticTypeKey::Adt {
                    owner: *owner,
                    arguments,
                })
            }
            InterfaceType::Tuple { elements } => Some(SemanticTypeKey::Tuple(
                elements
                    .iter()
                    .map(|element| self.semantic_key(element, module, type_parameters))
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
