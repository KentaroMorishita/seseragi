use crate::{SymbolId, TypedType};
use std::collections::BTreeMap;

use super::substitution::substitute_type_parameters;
use super::{
    SemanticConstructorSignature, SemanticTypeCatalog, SemanticTypeKey, SemanticValueType,
    SemanticVariant,
};

impl SemanticTypeCatalog {
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

    pub(crate) fn constructor_signatures(
        &self,
    ) -> impl Iterator<Item = (SymbolId, SemanticConstructorSignature)> + '_ {
        self.constructors.keys().filter_map(|constructor| {
            self.constructor_signature(*constructor)
                .map(|signature| (*constructor, signature))
        })
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

    fn constructor_signature(&self, constructor: SymbolId) -> Option<SemanticConstructorSignature> {
        let (owner, variant) = self.constructor(constructor)?;
        let adt = self.adts.get(&owner)?;
        Some(SemanticConstructorSignature {
            symbol: variant.canonical.clone(),
            type_parameters: adt.type_parameter_names.clone(),
            parameters: variant.payload.clone().into_iter().collect(),
            result: self.polymorphic_adt_value(owner)?,
        })
    }

    fn polymorphic_adt_value(&self, owner: SymbolId) -> Option<SemanticValueType> {
        let adt = self.adts.get(&owner)?;
        let arguments = adt
            .type_parameters
            .iter()
            .zip(&adt.type_parameter_names)
            .map(|(parameter, name)| SemanticValueType {
                type_ref: TypedType::Named {
                    name: name.clone(),
                    arguments: Vec::new(),
                },
                key: SemanticTypeKey::TypeParameter(*parameter),
            })
            .collect::<Vec<_>>();
        Some(SemanticValueType {
            type_ref: TypedType::Named {
                name: adt.name.clone(),
                arguments: arguments
                    .iter()
                    .map(|argument| argument.type_ref.clone())
                    .collect(),
            },
            key: SemanticTypeKey::Adt { owner, arguments },
        })
    }
}
