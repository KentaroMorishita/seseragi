use super::{SemanticTypeKey, TypedResolution};
use seseragi_syntax::{ByteSpan, SurfaceExpr};
use std::collections::BTreeMap;

pub(super) type ReceiverFacts = BTreeMap<(usize, usize), Option<String>>;

impl TypedResolution<'_> {
    pub(crate) fn enable_receiver_facts(&mut self) {
        *self.receiver_facts.get_mut() = Some(BTreeMap::new());
    }

    pub(crate) fn record_receiver_fact(&self, expression: &SurfaceExpr, key: &SemanticTypeKey) {
        let SurfaceExpr::Name { span, .. } = expression else {
            return;
        };
        let mut facts = self.receiver_facts.borrow_mut();
        let Some(facts) = facts.as_mut() else { return };
        let owner = match key {
            SemanticTypeKey::Adt { owner, .. } | SemanticTypeKey::Struct { owner, .. } => self
                .symbol(*owner)
                .and_then(|symbol| symbol.canonical.clone()),
            SemanticTypeKey::ExternalNominal { canonical, .. } => Some(canonical.clone()),
            _ => None,
        };
        // A scoped parameter, unresolved key or conflicting inference never
        // gains methods from a nominal with a matching display spelling.
        facts
            .entry((span.start, span.end))
            .and_modify(|previous| {
                if *previous != owner {
                    *previous = None;
                }
            })
            .or_insert(owner);
    }

    pub(crate) fn nominal_receivers(&self) -> Vec<crate::AnalysisNominalReceiver> {
        self.receiver_facts
            .borrow()
            .as_ref()
            .into_iter()
            .flat_map(|facts| facts.iter())
            .filter_map(|(&(start, end), owner)| {
                Some(crate::AnalysisNominalReceiver {
                    range: ByteSpan { start, end },
                    owner: owner.clone()?,
                })
            })
            .collect()
    }
}
