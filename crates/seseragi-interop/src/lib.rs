//! Source-independent external type semantics.
//!
//! Frontends own parsing and source-language checking. This graph owns resolved
//! type relationships, generic scopes and explicit runtime boundaries. It has
//! no dependency on a parser, compiler AST, filesystem or host module loader.

mod linkage;
mod model;
mod normalize;
mod report;
mod validate;

pub use linkage::*;
pub use model::*;
pub use normalize::NormalizationBudget;
pub use report::*;
pub use validate::{ValidationError, ValidationIssue};

use sha2::{Digest, Sha256};

impl SemanticGraph {
    pub fn related_types(&self, id: TypeId) -> Vec<TypeId> {
        let Some(node) = self.types.get(&id) else {
            return vec![];
        };
        let mut relations = validate::references(node);
        if let SemanticType::Reference { symbol } = node {
            if let Some(symbol) = self.symbols.get(symbol) {
                relations.push(symbol.type_ref);
            }
        }
        relations
    }
    /// Closed relations are normalized by the shared semantic core, using a
    /// finite work budget. No source-language parser or host checker is needed.
    pub fn normalize_closed(&mut self, budget: NormalizationBudget) -> Result<(), ValidationError> {
        self.validate()?;
        normalize::normalize(self, budget);
        self.validate()
    }
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate::validate(self)
    }

    /// Never emit an invalid graph or silently accept an unknown schema.
    pub fn to_json(&self) -> Result<String, ValidationError> {
        self.validate()?;
        serde_json::to_string(self).map_err(ValidationError::serialization)
    }

    pub fn from_json(source: &str) -> Result<Self, ValidationError> {
        let graph: Self = serde_json::from_str(source).map_err(ValidationError::serialization)?;
        graph.validate()?;
        Ok(graph)
    }

    /// A persisted graph is usable only for the exact declared producer,
    /// inputs, settings and runtime contract. Plain decoding is not freshness.
    pub fn from_json_for(
        source: &str,
        expected: &ArtifactIdentity,
    ) -> Result<Self, ValidationError> {
        let graph = Self::from_json(source)?;
        if graph.identity.as_ref() != Some(expected) {
            return Err(ValidationError(vec![ValidationIssue {
                subject: "artifact".to_owned(),
                reason: "stale or incompatible semantic artifact identity".to_owned(),
            }]));
        }
        Ok(graph)
    }

    /// Direct origins and the origins of each interpretation step remain
    /// separate in serialization; clients may request their transitive union.
    pub fn origins_for_type(&self, id: TypeId) -> Vec<Origin> {
        let mut origins = Vec::new();
        let mut pending = vec![id];
        let mut seen = std::collections::BTreeSet::new();
        while let Some(current) = pending.pop() {
            if !seen.insert(current) {
                continue;
            }
            origins.extend(
                self.type_origins
                    .get(&current)
                    .into_iter()
                    .flatten()
                    .cloned(),
            );
            for step in self
                .normalizations
                .iter()
                .filter(|step| step.output == current)
            {
                pending.extend(&step.inputs);
            }
        }
        origins.sort_by(|a, b| {
            (&a.source, a.start, a.end, &a.rule).cmp(&(&b.source, b.start, b.end, &b.rule))
        });
        origins.dedup();
        origins
    }

    /// Identity of the complete canonical graph, including provenance/ABI.
    pub fn digest(&self) -> Result<String, ValidationError> {
        Ok(format!("{:x}", Sha256::digest(self.to_json()?.as_bytes())))
    }
}
