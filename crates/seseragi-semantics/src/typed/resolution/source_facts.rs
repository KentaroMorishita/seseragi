use super::{SemanticTypeKey, SemanticValueType, TypedResolution};
use crate::SymbolId;
use std::collections::BTreeMap;

pub(super) type BindingFacts = BTreeMap<SymbolId, Option<String>>;

impl TypedResolution<'_> {
    pub(crate) fn enable_binding_facts(&mut self) {
        *self.binding_facts.get_mut() = Some(BTreeMap::new());
        // Top-level pattern inference is complete before recording starts.
        // Parameters and inferred local patterns are recorded at the same
        // environment boundary used by their actual expression typing.
        for (binding, key) in &self.semantic_values {
            self.record_binding_key(*binding, key);
        }
    }

    pub(crate) fn record_receiver_bindings(
        &self,
        bindings: &BTreeMap<SymbolId, SemanticValueType>,
    ) {
        for (binding, value) in bindings {
            self.record_binding_key(*binding, &value.key);
        }
    }

    fn record_binding_key(&self, binding: SymbolId, key: &SemanticTypeKey) {
        let mut facts = self.binding_facts.borrow_mut();
        let Some(facts) = facts.as_mut() else { return };
        // An initially unknown inference contributes no proof. A concrete
        // scoped parameter or conflicting known type permanently blocks an
        // unrelated nominal with matching display spelling.
        if matches!(key, SemanticTypeKey::Other | SemanticTypeKey::Invalid) {
            return;
        }
        let owner = match key {
            SemanticTypeKey::Adt { owner, .. } | SemanticTypeKey::Struct { owner, .. } => self
                .symbol(*owner)
                .and_then(|symbol| symbol.canonical.clone()),
            SemanticTypeKey::ExternalNominal { canonical, .. } => Some(canonical.clone()),
            _ => None,
        };
        facts
            .entry(binding)
            .and_modify(|previous| {
                if *previous != owner {
                    *previous = None;
                }
            })
            .or_insert(owner);
    }

    pub(crate) fn nominal_bindings(&self) -> Vec<crate::AnalysisNominalBinding> {
        self.binding_facts
            .borrow()
            .as_ref()
            .into_iter()
            .flat_map(|facts| facts.iter())
            .filter_map(|(binding, owner)| {
                Some(crate::AnalysisNominalBinding {
                    binding: binding.0,
                    owner: owner.clone()?,
                })
            })
            .collect()
    }
}
