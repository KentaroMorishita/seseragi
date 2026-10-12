//! Deterministic public-surface coverage. History/orphan nodes never inflate counts.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticCompatibilityReport {
    pub schema: u32,
    pub kind: String,
    pub graph_digest: String,
    pub identity: Option<ArtifactIdentity>,
    pub counts: Vec<ClassificationCount>,
    pub symbols: BTreeMap<String, SymbolCompatibility>,
    pub types: BTreeMap<TypeId, TypeCompatibility>,
    pub declarations: BTreeMap<String, Vec<String>>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassificationCount {
    pub classification: SemanticClassification,
    pub count: usize,
    pub symbols: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SymbolCompatibility {
    pub public_name: String,
    pub type_ref: TypeId,
    pub classification: SemanticClassification,
    pub semantic_classification: SemanticClassification,
    pub projection_classification: Option<SemanticClassification>,
    pub runtime_abi_unresolved: bool,
    pub types: Vec<TypeId>,
    pub reasons: Vec<CompatibilityReason>,
    pub origins: Vec<Origin>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeCompatibility {
    pub classification: SemanticClassification,
    pub reasons: Vec<CompatibilityReason>,
    pub origins: Vec<Origin>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityReason {
    pub type_ref: TypeId,
    pub classification: SemanticClassification,
    pub rule: String,
    pub reason: String,
    pub policy: Option<ProducerIdentity>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompatibilityDiff {
    pub before_digest: String,
    pub after_digest: String,
    pub identity_changed: bool,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}
/// Evidence supplied by the projection phase. It never changes interpreted IR.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionEvidence {
    pub symbol: String,
    pub classification: SemanticClassification,
    pub rule: String,
    pub reason: String,
    pub policy: Option<ProducerIdentity>,
}
const CLASSES: [SemanticClassification; 7] = [
    SemanticClassification::Direct,
    SemanticClassification::Normalized,
    SemanticClassification::OpaquePreserved,
    SemanticClassification::SafeBoundary,
    SemanticClassification::PolicyRequired,
    SemanticClassification::RuntimeAbiUnresolved,
    SemanticClassification::Uninterpreted,
];
fn rank(class: SemanticClassification) -> usize {
    CLASSES.iter().position(|c| *c == class).unwrap()
}
fn label(class: SemanticClassification) -> &'static str {
    match class {
        SemanticClassification::Direct => "direct",
        SemanticClassification::Normalized => "normalized",
        SemanticClassification::OpaquePreserved => "opaque-preserved",
        SemanticClassification::SafeBoundary => "safe-boundary",
        SemanticClassification::PolicyRequired => "policy-required",
        SemanticClassification::RuntimeAbiUnresolved => "runtime-ABI-unresolved",
        SemanticClassification::Uninterpreted => "uninterpreted",
    }
}
fn sort_origins(origins: &mut Vec<Origin>) {
    origins.sort_by(|a, b| {
        (&a.source, a.start, a.end, &a.rule).cmp(&(&b.source, b.start, b.end, &b.rule))
    });
    origins.dedup();
}

impl SemanticCompatibilityReport {
    pub fn from_graph(graph: &SemanticGraph) -> Result<Self, ValidationError> {
        graph.validate()?;
        let mut symbols = BTreeMap::new();
        let mut types = BTreeMap::new();
        let mut declarations: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (identity, symbol) in graph
            .symbols
            .iter()
            .filter(|(identity, _)| !graph.private_symbols.contains(*identity))
        {
            let mut reachable = BTreeSet::new();
            let mut pending = vec![symbol.type_ref];
            while let Some(id) = pending.pop() {
                if !reachable.insert(id) {
                    continue;
                }
                let node = &graph.types[&id];
                pending.extend(crate::validate::references(node));
                match node {
                    SemanticType::Reference { symbol } => {
                        pending.push(graph.symbols[symbol].type_ref)
                    }
                    SemanticType::Namespace { members } => pending.extend(
                        members
                            .values()
                            .map(|symbol| graph.symbols[symbol].type_ref),
                    ),
                    _ => {}
                }
            }
            let mut reasons = Vec::new();
            let mut classification = SemanticClassification::Direct;
            for id in &reachable {
                let status = types.entry(*id).or_insert_with(|| type_status(graph, *id));
                if rank(status.classification) > rank(classification) {
                    classification = status.classification;
                }
                reasons.extend(status.reasons.clone());
            }
            let runtime_abi_unresolved = match symbol.kind {
                SymbolKind::Type => reasons
                    .iter()
                    .any(|r| r.classification == SemanticClassification::RuntimeAbiUnresolved),
                SymbolKind::Callable => symbol.runtime.as_ref().is_none_or(|runtime| {
                    runtime.evaluation == Evaluation::Unresolved
                        || runtime.failure == FailureBoundary::Unresolved
                }),
                SymbolKind::Namespace => false,
            };
            if runtime_abi_unresolved {
                reasons.push(CompatibilityReason { type_ref: symbol.type_ref,
                    classification: SemanticClassification::RuntimeAbiUnresolved,
                    rule: "runtime-binding-evidence".to_owned(), reason: "runtime linking, evaluation or failure evidence is unresolved; semantic interpretation alone does not prove runtime compatibility".to_owned(), policy: None });
            }
            let mut origins = symbol.origins.clone();
            for id in &reachable {
                origins.extend(graph.origins_for_type(*id));
            }
            sort_origins(&mut origins);
            for source in symbol
                .origins
                .iter()
                .map(|origin| origin.source.clone())
                .collect::<BTreeSet<_>>()
            {
                declarations
                    .entry(source)
                    .or_default()
                    .push(identity.clone());
            }
            symbols.insert(
                identity.clone(),
                SymbolCompatibility {
                    public_name: symbol.public_name.clone(),
                    type_ref: symbol.type_ref,
                    classification,
                    semantic_classification: classification,
                    projection_classification: None,
                    runtime_abi_unresolved,
                    types: reachable.into_iter().collect(),
                    reasons,
                    origins,
                },
            );
        }
        let counts = CLASSES
            .into_iter()
            .map(|classification| {
                let identities = symbols
                    .iter()
                    .filter(|(_, status)| status.classification == classification)
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                ClassificationCount {
                    classification,
                    count: identities.len(),
                    symbols: identities,
                }
            })
            .collect();
        Ok(Self {
            schema: 1,
            kind: "seseragi-interop-compatibility-report".to_owned(),
            graph_digest: graph.digest()?,
            identity: graph.identity.clone(),
            counts,
            symbols,
            types,
            declarations,
        })
    }
    /// Combine actual rendering/policy outcomes without altering cached interpretation.
    /// Unsafe or unresolved meaning can never be upgraded to direct by a renderer.
    pub fn from_projection(
        graph: &SemanticGraph,
        evidence: &[ProjectionEvidence],
    ) -> Result<Self, ValidationError> {
        let mut report = Self::from_graph(graph)?;
        let mut evidence = evidence.iter().collect::<Vec<_>>();
        evidence.sort_by_key(|item| {
            (
                &item.symbol,
                &item.rule,
                &item.reason,
                rank(item.classification),
                item.policy
                    .as_ref()
                    .map(|policy| (&policy.name, &policy.version)),
            )
        });
        evidence.dedup();
        for item in evidence {
            let Some(symbol) = report.symbols.get_mut(&item.symbol) else {
                return Err(ValidationError(vec![ValidationIssue {
                    subject: item.symbol.clone(),
                    reason: "projection evidence names no public symbol".to_owned(),
                }]));
            };
            if item.classification == SemanticClassification::SafeBoundary && item.policy.is_none()
            {
                return Err(ValidationError(vec![ValidationIssue {
                    subject: item.symbol.clone(),
                    reason: "safe boundary projection requires an applied policy identity"
                        .to_owned(),
                }]));
            }
            if item.reason.trim().is_empty() || item.rule.trim().is_empty() {
                return Err(ValidationError(vec![ValidationIssue {
                    subject: item.symbol.clone(),
                    reason: "projection evidence requires a rule and reason".to_owned(),
                }]));
            }
            if rank(item.classification) > rank(symbol.classification) {
                symbol.classification = item.classification;
            }
            symbol.projection_classification = Some(match symbol.projection_classification {
                Some(previous) if rank(previous) > rank(item.classification) => previous,
                _ => item.classification,
            });
            symbol.reasons.push(CompatibilityReason {
                type_ref: symbol.type_ref,
                classification: item.classification,
                rule: item.rule.clone(),
                reason: item.reason.clone(),
                policy: item.policy.clone(),
            });
        }
        report.counts = CLASSES
            .into_iter()
            .map(|classification| {
                let symbols = report
                    .symbols
                    .iter()
                    .filter(|(_, symbol)| symbol.classification == classification)
                    .map(|(identity, _)| identity.clone())
                    .collect::<Vec<_>>();
                ClassificationCount {
                    classification,
                    count: symbols.len(),
                    symbols,
                }
            })
            .collect();
        Ok(report)
    }
    pub fn to_json(&self) -> Result<String, ValidationError> {
        serde_json::to_string_pretty(self).map_err(ValidationError::serialization)
    }
    pub fn render_text(&self) -> String {
        let mut text = format!(
            "Semantic compatibility: {} public symbols\n",
            self.symbols.len()
        );
        for count in &self.counts {
            text.push_str(&format!(
                "{}: {}\n",
                label(count.classification),
                count.count
            ));
            for identity in &count.symbols {
                let symbol = &self.symbols[identity];
                text.push_str(&format!(
                    "  {identity}: type {}{}\n",
                    symbol.type_ref.0,
                    if symbol.runtime_abi_unresolved {
                        " (runtime ABI unresolved)"
                    } else {
                        ""
                    }
                ));
                for reason in &symbol.reasons {
                    text.push_str(&format!("    {}: {}\n", reason.rule, reason.reason));
                }
            }
        }
        text
    }
    pub fn diff(&self, before: &Self) -> CompatibilityDiff {
        CompatibilityDiff {
            before_digest: before.graph_digest.clone(),
            after_digest: self.graph_digest.clone(),
            identity_changed: before.identity != self.identity,
            added: self
                .symbols
                .keys()
                .filter(|id| !before.symbols.contains_key(*id))
                .cloned()
                .collect(),
            removed: before
                .symbols
                .keys()
                .filter(|id| !self.symbols.contains_key(*id))
                .cloned()
                .collect(),
            changed: self
                .symbols
                .iter()
                .filter(|(id, status)| before.symbols.get(*id).is_some_and(|old| old != *status))
                .map(|(id, _)| id.clone())
                .collect(),
        }
    }
}
fn type_status(graph: &SemanticGraph, id: TypeId) -> TypeCompatibility {
    let mut reasons = graph
        .normalizations
        .iter()
        .filter(|step| step.output == id)
        .map(|step| CompatibilityReason {
            type_ref: id,
            classification: step.classification,
            rule: step.rule.clone(),
            reason: step.reason.clone(),
            policy: step.policy.clone(),
        })
        .collect::<Vec<_>>();
    // Representation checks are independent of producer-supplied classifications:
    // a forged "direct" normalization cannot hide dynamic/unresolved meaning.
    let guard = match &graph.types[&id] {
        SemanticType::Unresolved { reason } => {
            Some((SemanticClassification::Uninterpreted, reason.as_str()))
        }
        SemanticType::Primitive {
            domain: PrimitiveDomain::Dynamic,
        } => Some((
            SemanticClassification::PolicyRequired,
            "dynamic domain requires explicit unsafe approval and cannot count as direct",
        )),
        SemanticType::Opaque { .. } => Some((
            SemanticClassification::OpaquePreserved,
            "nominal identity retained without structural projection proof",
        )),
        SemanticType::Projection { .. } | SemanticType::ShapeTransform { .. } => Some((
            SemanticClassification::PolicyRequired,
            "open semantic relation retained; projection requires normalization or explicit policy",
        )),
        _ => None,
    };
    if let Some((classification, reason)) = guard {
        reasons.push(CompatibilityReason {
            type_ref: id,
            classification,
            rule: "representation-safety".to_owned(),
            reason: reason.to_owned(),
            policy: None,
        });
    }
    if reasons.is_empty() {
        reasons.push(CompatibilityReason {
            type_ref: id,
            classification: SemanticClassification::Direct,
            rule: "semantic-relationship".to_owned(),
            reason: "meaning is represented directly by the validated shared semantic node"
                .to_owned(),
            policy: None,
        });
    }
    let classification = reasons
        .iter()
        .map(|reason| reason.classification)
        .max_by_key(|class| rank(*class))
        .unwrap();
    TypeCompatibility {
        classification,
        reasons,
        origins: graph.origins_for_type(id),
    }
}
