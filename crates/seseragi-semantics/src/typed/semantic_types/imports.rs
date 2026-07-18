use crate::{ResolvedModule, SymbolKind, TypedType};
use seseragi_syntax::InterfaceType;
use std::collections::BTreeMap;

use super::{
    SemanticAdt, SemanticStruct, SemanticStructField, SemanticTypeCatalog, SemanticTypeKey,
    SemanticValueType, SemanticVariant,
};
use crate::typed::type_ref::typed_type_from_interface_type;

impl SemanticTypeCatalog {
    pub(super) fn collect_imported_adts(&mut self, resolved: &ResolvedModule) {
        for owner_import in resolved.imports.iter().filter(|import| {
            import.export.namespace == "type"
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("type" | "opaque-type" | "newtype")
                )
        }) {
            let owner = owner_import.symbol;
            let owner_canonical = &owner_import.export.symbol;
            let type_parameters = owner_import
                .export
                .scheme
                .type_parameters
                .iter()
                .filter_map(|parameter| {
                    let canonical = format!("{owner_canonical}::{}", parameter.name);
                    resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::TypeParameter
                            && symbol.canonical.as_deref() == Some(canonical.as_str())
                    })
                })
                .collect::<Vec<_>>();
            if type_parameters.len() != owner_import.export.scheme.type_parameters.len() {
                continue;
            }
            let parameter_ids = type_parameters
                .iter()
                .map(|parameter| (parameter.spelling.clone(), parameter.id))
                .collect::<BTreeMap<_, _>>();
            let variants = resolved
                .imports
                .iter()
                .filter(|import| import.export.constructor_of.as_ref() == Some(owner_canonical))
                .filter_map(|import| {
                    let payload = constructor_payload(&import.export.scheme.type_ref)?;
                    self.constructors.insert(import.symbol, owner);
                    Some(SemanticVariant {
                        constructor: import.symbol,
                        canonical: import.export.symbol.clone(),
                        spelling: import.local_name.clone(),
                        payload: payload.map(|type_ref| {
                            self.imported_payload(resolved, type_ref, &parameter_ids)
                        }),
                    })
                })
                .collect();
            self.adts.insert(
                owner,
                SemanticAdt {
                    name: owner_import.local_name.clone(),
                    type_parameters: type_parameters
                        .iter()
                        .map(|parameter| parameter.id)
                        .collect(),
                    type_parameter_names: type_parameters
                        .iter()
                        .map(|parameter| parameter.spelling.clone())
                        .collect(),
                    variants,
                },
            );
        }
    }

    pub(super) fn collect_imported_structs(&mut self, resolved: &ResolvedModule) {
        for owner_import in resolved.imports.iter().filter(|import| {
            import.export.namespace == "type"
                && matches!(
                    import.export.declaration_kind.as_deref(),
                    Some("struct" | "opaque-struct")
                )
        }) {
            let fields = match owner_import.export.representation.as_ref() {
                Some(InterfaceType::Record { fields, .. }) => fields.as_slice(),
                None if owner_import.export.declaration_kind.as_deref()
                    == Some("opaque-struct") =>
                {
                    &[]
                }
                _ => continue,
            };
            let owner = owner_import.symbol;
            let owner_canonical = &owner_import.export.symbol;
            let type_parameters = owner_import
                .export
                .scheme
                .type_parameters
                .iter()
                .filter_map(|parameter| {
                    let canonical = format!("{owner_canonical}::{}", parameter.name);
                    resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::TypeParameter
                            && symbol.canonical.as_deref() == Some(canonical.as_str())
                    })
                })
                .collect::<Vec<_>>();
            if type_parameters.len() != owner_import.export.scheme.type_parameters.len() {
                continue;
            }
            let parameter_ids = type_parameters
                .iter()
                .map(|parameter| (parameter.spelling.clone(), parameter.id))
                .collect::<BTreeMap<_, _>>();
            let fields = fields
                .iter()
                .filter_map(|field| {
                    let type_ref = typed_type_from_interface_type(field.type_ref.clone())?;
                    Some(SemanticStructField {
                        name: field.name.clone(),
                        type_ref: self.imported_payload(resolved, type_ref, &parameter_ids),
                    })
                })
                .collect::<Vec<_>>();
            self.structs.insert(
                owner,
                SemanticStruct {
                    name: owner_import.local_name.clone(),
                    type_parameters: type_parameters
                        .iter()
                        .map(|parameter| parameter.id)
                        .collect(),
                    type_parameter_names: type_parameters
                        .iter()
                        .map(|parameter| parameter.spelling.clone())
                        .collect(),
                    fields,
                    construction_allowed: owner_import.export.declaration_kind.as_deref()
                        == Some("struct"),
                },
            );
        }
    }

    fn imported_payload(
        &self,
        resolved: &ResolvedModule,
        type_ref: TypedType,
        parameters: &BTreeMap<String, crate::SymbolId>,
    ) -> SemanticValueType {
        let key = match &type_ref {
            TypedType::Named { name, arguments } if arguments.is_empty() => parameters
                .get(name)
                .copied()
                .map(SemanticTypeKey::TypeParameter)
                .unwrap_or_else(|| self.key_from_typed_type(resolved, &type_ref)),
            TypedType::Named { arguments, .. } | TypedType::ExternalNamed { arguments, .. } => {
                let arguments = arguments
                    .iter()
                    .cloned()
                    .map(|argument| self.imported_payload(resolved, argument, parameters))
                    .collect::<Vec<_>>();
                match self.key_from_typed_type(resolved, &type_ref) {
                    SemanticTypeKey::Adt { owner, .. } => SemanticTypeKey::Adt { owner, arguments },
                    SemanticTypeKey::Struct { owner, .. } => {
                        SemanticTypeKey::Struct { owner, arguments }
                    }
                    SemanticTypeKey::ExternalNominal { canonical, .. } => {
                        SemanticTypeKey::ExternalNominal {
                            canonical,
                            arguments,
                        }
                    }
                    _ => SemanticTypeKey::Other,
                }
            }
            TypedType::Tuple { elements } => SemanticTypeKey::Tuple(
                elements
                    .iter()
                    .cloned()
                    .map(|element| self.imported_payload(resolved, element, parameters).key)
                    .collect(),
            ),
            _ => SemanticTypeKey::Other,
        };
        SemanticValueType { type_ref, key }
    }
}

fn constructor_payload(type_ref: &InterfaceType) -> Option<Option<TypedType>> {
    match type_ref {
        InterfaceType::Function { parameter, .. } => {
            Some(Some(typed_type_from_interface_type((**parameter).clone())?))
        }
        InterfaceType::Named { .. } | InterfaceType::ExternalNamed { .. } => Some(None),
        _ => None,
    }
}
