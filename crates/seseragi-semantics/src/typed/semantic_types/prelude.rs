use crate::{prelude::SUM_TYPES, ResolvedModule, SymbolKind, TypedType};

use super::{
    SemanticAdt, SemanticTypeCatalog, SemanticTypeKey, SemanticValueType, SemanticVariant,
};

impl SemanticTypeCatalog {
    pub(super) fn collect_prelude_sum_types(&mut self, resolved: &ResolvedModule) {
        for contract in SUM_TYPES {
            let Some(owner) = resolved.symbols.iter().find(|symbol| {
                symbol.kind == SymbolKind::Type
                    && symbol.canonical.as_deref() == Some(contract.canonical)
            }) else {
                continue;
            };
            let parameters = contract
                .type_parameters
                .iter()
                .filter_map(|parameter| {
                    let canonical = format!("{}::{parameter}", contract.canonical);
                    resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::TypeParameter
                            && symbol.canonical.as_deref() == Some(canonical.as_str())
                    })
                })
                .collect::<Vec<_>>();
            if parameters.len() != contract.type_parameters.len() {
                continue;
            }
            let semantic_variants = contract
                .variants
                .iter()
                .filter_map(|variant| {
                    let constructor = resolved.symbols.iter().find(|symbol| {
                        symbol.kind == SymbolKind::Constructor
                            && symbol.canonical.as_deref() == Some(variant.canonical)
                    })?;
                    let payload = variant.payload_parameter.map(|index| SemanticValueType {
                        type_ref: TypedType::Named {
                            name: contract.type_parameters[index].to_owned(),
                            arguments: Vec::new(),
                        },
                        key: SemanticTypeKey::TypeParameter(parameters[index].id),
                    });
                    Some(SemanticVariant {
                        constructor: constructor.id,
                        canonical: variant.canonical.to_owned(),
                        spelling: variant.name.to_owned(),
                        payload,
                    })
                })
                .collect::<Vec<_>>();
            if semantic_variants.len() != contract.variants.len() {
                continue;
            }
            for variant in &semantic_variants {
                self.constructors.insert(variant.constructor, owner.id);
            }
            self.adts.insert(
                owner.id,
                SemanticAdt {
                    name: contract.name.to_owned(),
                    type_parameters: parameters.iter().map(|parameter| parameter.id).collect(),
                    type_parameter_names: contract
                        .type_parameters
                        .iter()
                        .map(|parameter| (*parameter).to_owned())
                        .collect(),
                    variants: semantic_variants,
                },
            );
        }
    }
}
