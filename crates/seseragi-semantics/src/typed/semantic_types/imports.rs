use crate::{ResolvedModule, TypedType};
use seseragi_syntax::InterfaceType;

use super::{
    SemanticAdt, SemanticTypeCatalog, SemanticTypeKey, SemanticValueType, SemanticVariant,
};
use crate::typed::type_ref::typed_type_from_interface_type;

impl SemanticTypeCatalog {
    pub(super) fn collect_imported_adts(&mut self, resolved: &ResolvedModule) {
        for owner_import in resolved.imports.iter().filter(|import| {
            import.export.namespace == "type"
                && import.export.declaration_kind.as_deref() == Some("type")
                && import.export.scheme.type_parameters.is_empty()
        }) {
            let owner = owner_import.symbol;
            let owner_canonical = &owner_import.export.symbol;
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
                        payload: payload.map(|type_ref| SemanticValueType {
                            type_ref,
                            key: SemanticTypeKey::Other,
                        }),
                    })
                })
                .collect();
            self.adts.insert(
                owner,
                SemanticAdt {
                    name: owner_import.local_name.clone(),
                    type_parameters: Vec::new(),
                    type_parameter_names: Vec::new(),
                    variants,
                },
            );
        }
    }
}

fn constructor_payload(type_ref: &InterfaceType) -> Option<Option<TypedType>> {
    match type_ref {
        InterfaceType::Function { parameter, .. } => {
            Some(Some(typed_type_from_interface_type((**parameter).clone())?))
        }
        InterfaceType::Named { .. } => Some(None),
        _ => None,
    }
}
