//! Source-neutral projection decisions and explicit immutable adapter policies.
//! A candidate representation is not permission to reinterpret a host object.
//! Generated JSON adapters validate data; raw ABI projection remains separate.
use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PROJECTION_POLICY_KIND: &str = "seseragi-semantic-projection-policy";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ProjectionRepresentation {
    Direct {
        domain: PrimitiveDomain,
    },
    Boundary {
        carrier: String,
    },
    Opaque {
        identity: String,
        arguments: Vec<TypeId>,
    },
    RecordCandidate {
        fields: BTreeMap<String, ShapeField>,
    },
    TaggedDomainCandidate {
        discriminators: Vec<String>,
        variants: Vec<TypeId>,
    },
    GenericRelation {
        parameters: Vec<GenericParameter>,
        body: TypeId,
    },
    ParameterRelation {
        binder: TypeId,
        index: u32,
    },
    ApplicationRelation {
        constructor: TypeId,
        arguments: Vec<TypeId>,
    },
    NullableBoundary {
        value: TypeId,
    },
    OptionalBoundary {
        value: TypeId,
    },
    AsyncBoundary {
        value: TypeId,
    },
    CollectionSnapshot {
        element: TypeId,
    },
    FiniteDomain {
        members: Vec<TypeId>,
    },
    PolicyRequired {
        reason: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TypeProjection {
    pub type_ref: TypeId,
    pub representation: ProjectionRepresentation,
    pub origins: Vec<Origin>,
}

/// Determine what can be represented without granting implicit adapter policy.
/// In particular, class/opaque and mutable identity never become record data.
pub fn projection_plan(graph: &SemanticGraph) -> Result<Vec<TypeProjection>, String> {
    graph.validate().map_err(|e| e.to_string())?;
    if graph.types.len() > 16_384 {
        return Err("projection type work budget exceeded".into());
    }
    Ok(graph.types.iter().map(|(id,node)| {
        let representation = match node {
            SemanticType::Primitive { domain: PrimitiveDomain::Unknown | PrimitiveDomain::Dynamic } => ProjectionRepresentation::PolicyRequired { reason: "unknown/dynamic domain has no implicit native value codec".into() },
            SemanticType::Primitive { domain: PrimitiveDomain::Null } => ProjectionRepresentation::Boundary { carrier: "Js.Null".into() },
            SemanticType::Primitive { domain: PrimitiveDomain::Undefined } => ProjectionRepresentation::Boundary { carrier: "Js.Undefined".into() },
            SemanticType::Primitive { domain } => ProjectionRepresentation::Direct { domain: *domain },
            SemanticType::Opaque { identity, arguments } => ProjectionRepresentation::Opaque { identity: identity.clone(), arguments: arguments.clone() },
            SemanticType::Shape { identity: ShapeIdentity::MutableIdentity, .. } => ProjectionRepresentation::Opaque { identity: format!("mutable:type:{}",id.0), arguments: vec![] },
            SemanticType::Shape { identity: ShapeIdentity::ReadonlySnapshot, fields } => ProjectionRepresentation::RecordCandidate { fields: fields.clone() },
            SemanticType::Generic { parameters, body } => ProjectionRepresentation::GenericRelation { parameters: parameters.clone(), body: *body },
            SemanticType::Parameter { binder,index } => ProjectionRepresentation::ParameterRelation { binder: *binder, index: *index },
            SemanticType::Apply { constructor, arguments } => ProjectionRepresentation::ApplicationRelation { constructor: *constructor, arguments: arguments.clone() },
            SemanticType::Nullable { value } => ProjectionRepresentation::NullableBoundary { value: *value },
            SemanticType::Optional { value } => ProjectionRepresentation::OptionalBoundary { value: *value },
            SemanticType::Async { value } => ProjectionRepresentation::AsyncBoundary { value: *value },
            SemanticType::Collection { element, mutable: false } => ProjectionRepresentation::CollectionSnapshot { element: *element },
            SemanticType::Collection { mutable: true,.. } => ProjectionRepresentation::PolicyRequired { reason: "mutable collection identity requires a boundary/ownership contract".into() },
            SemanticType::Union { members } => {
                let discriminators = tagged_discriminators(graph,members);
                if let Some(value) = projection_nullable_value(graph,*id) {
                    ProjectionRepresentation::NullableBoundary { value }
                } else if !discriminators.is_empty() {
                    ProjectionRepresentation::TaggedDomainCandidate { discriminators, variants: members.clone() }
                } else if members.iter().all(|member| matches!(projection_node(graph,*member), Some((_,SemanticType::Literal { .. })))) {
                    ProjectionRepresentation::FiniteDomain { members: members.clone() }
                } else {
                    ProjectionRepresentation::PolicyRequired { reason: "general union needs a proven representation and explicit policy".into() }
                }
            },
            SemanticType::Unresolved { reason } => ProjectionRepresentation::PolicyRequired { reason: reason.clone() },
            _ => ProjectionRepresentation::PolicyRequired { reason: "relation/identity remains on its raw boundary until a proven codec and explicit projection policy exist".into() },
        };
        TypeProjection { type_ref: *id, representation, origins: graph.origins_for_type(*id) }
    }).collect())
}

/// Resolve only alias references, with a finite cycle/depth bound.
pub fn projection_node(graph: &SemanticGraph, mut id: TypeId) -> Option<(TypeId, &SemanticType)> {
    let mut seen = BTreeSet::new();
    for _ in 0..64 {
        if !seen.insert(id) {
            return None;
        }
        match graph.types.get(&id)? {
            SemanticType::Reference { symbol } => id = graph.symbols.get(symbol)?.type_ref,
            node => return Some((id, node)),
        }
    }
    None
}

/// Null-only union handling never folds explicit undefined into absence.
pub fn projection_nullable_value(graph: &SemanticGraph, id: TypeId) -> Option<TypeId> {
    match projection_node(graph, id)?.1 {
        SemanticType::Nullable { value } => Some(*value),
        SemanticType::Union { members } if members.len() == 2 => {
            let null = members
                .iter()
                .filter(|id| {
                    matches!(
                        projection_node(graph, **id),
                        Some((
                            _,
                            SemanticType::Primitive {
                                domain: PrimitiveDomain::Null
                            }
                        ))
                    )
                })
                .count();
            (null == 1).then(|| {
                *members
                    .iter()
                    .find(|id| {
                        !matches!(
                            projection_node(graph, **id),
                            Some((
                                _,
                                SemanticType::Primitive {
                                    domain: PrimitiveDomain::Null
                                }
                            ))
                        )
                    })
                    .unwrap()
            })
        }
        _ => None,
    }
}

/// A tagged ADT candidate needs readonly snapshots, required literal-string
/// tags, and pairwise distinct tag values. The policy chooses the discriminator.
pub fn tagged_discriminators(graph: &SemanticGraph, variants: &[TypeId]) -> Vec<String> {
    if variants.len() < 2 || variants.len() > 64 {
        return vec![];
    }
    let Some((
        _,
        SemanticType::Shape {
            identity: ShapeIdentity::ReadonlySnapshot,
            fields,
        },
    )) = projection_node(graph, variants[0])
    else {
        return vec![];
    };
    fields
        .keys()
        .filter(|name| {
            let mut tags = BTreeSet::new();
            variants.iter().all(|id| {
                let Some((
                    _,
                    SemanticType::Shape {
                        identity: ShapeIdentity::ReadonlySnapshot,
                        fields,
                    },
                )) = projection_node(graph, *id)
                else {
                    return false;
                };
                let Some(field) = fields.get(*name).filter(|f| !f.optional && f.readonly) else {
                    return false;
                };
                let Some((
                    _,
                    SemanticType::Literal {
                        value: LiteralValue::String(tag),
                    },
                )) = projection_node(graph, field.type_ref)
                else {
                    return false;
                };
                tags.insert(tag.clone())
            })
        })
        .cloned()
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ProjectionSelector {
    Type { type_ref: TypeId },
    Symbol { symbol: String },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AdapterMapping {
    /// Input is std/json::Json, never an arbitrary raw JS object or opaque class.
    JsonSnapshot,
    JsonNullable,
    JsonTaggedUnion {
        discriminator: String,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionPolicy {
    pub selector: ProjectionSelector,
    pub function_name: String,
    pub mapping: AdapterMapping,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionPolicySet {
    pub schema: u32,
    pub kind: String,
    pub producer: ProducerIdentity,
    pub graph_digest: String,
    pub artifact_identity: ArtifactIdentity,
    pub policies: Vec<ProjectionPolicy>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppliedProjectionPolicy {
    pub selector: ProjectionSelector,
    pub type_ref: TypeId,
    pub function_name: String,
    pub mapping: AdapterMapping,
    pub producer: ProducerIdentity,
    pub origins: Vec<Origin>,
    pub input_boundary: String,
}

/// Freshness is checked against complete graph, frontend/source/settings/ABI
/// identity. No policy mutates the semantic graph or overrides identity proof.
pub fn validate_projection_policies(
    graph: &SemanticGraph,
    set: &ProjectionPolicySet,
) -> Result<Vec<AppliedProjectionPolicy>, String> {
    graph.validate().map_err(|e| e.to_string())?;
    if set.schema != 1 || set.kind != PROJECTION_POLICY_KIND {
        return Err("unsupported projection policy schema/kind".into());
    }
    if set.producer.name.trim().is_empty() || set.producer.version.trim().is_empty() {
        return Err("policy producer identity is incomplete".into());
    }
    if graph.identity.as_ref() != Some(&set.artifact_identity)
        || graph.digest().map_err(|e| e.to_string())? != set.graph_digest
    {
        return Err(
            "stale projection policy: graph/source/settings/runtime identity differs".into(),
        );
    }
    if set.policies.len() > 64 {
        return Err("projection policy count budget exceeded".into());
    }
    let mut selectors = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut applied = vec![];
    for policy in &set.policies {
        if !selectors.insert(policy.selector.clone()) || !names.insert(policy.function_name.clone())
        {
            return Err("duplicate policy selector or generated function name".into());
        }
        if !valid_adapter_name(&policy.function_name) {
            return Err("adapter name is not a safe Seseragi function identifier".into());
        }
        let id = match &policy.selector {
            ProjectionSelector::Type { type_ref } => *type_ref,
            ProjectionSelector::Symbol { symbol } => {
                if graph.private_symbols.contains(symbol) {
                    return Err("private declaration is not a public policy surface".into());
                }
                graph
                    .symbols
                    .get(symbol)
                    .ok_or("policy symbol is absent")?
                    .type_ref
            }
        };
        let (_, node) = projection_node(graph, id).ok_or("policy target is missing or cyclic")?;
        match &policy.mapping {
            AdapterMapping::JsonSnapshot
                if matches!(
                    node,
                    SemanticType::Shape {
                        identity: ShapeIdentity::ReadonlySnapshot,
                        ..
                    }
                ) => {}
            AdapterMapping::JsonNullable if projection_nullable_value(graph, id).is_some() => {}
            AdapterMapping::JsonTaggedUnion { discriminator } => {
                let SemanticType::Union { members } = node else {
                    return Err("ADT policy requires a union domain".into());
                };
                if !tagged_discriminators(graph, members).contains(discriminator) {
                    return Err("ADT policy lacks required distinct readonly literal tags".into());
                }
            }
            _ => {
                return Err(
                    "policy cannot turn mutable/nominal/opaque/unproven meaning into native data"
                        .into(),
                )
            }
        }
        applied.push(AppliedProjectionPolicy {
            selector: policy.selector.clone(),
            type_ref: id,
            function_name: policy.function_name.clone(),
            mapping: policy.mapping.clone(),
            producer: set.producer.clone(),
            origins: graph.origins_for_type(id),
            input_boundary: "std/json::Json".into(),
        });
    }
    Ok(applied)
}
fn valid_adapter_name(name: &str) -> bool {
    name.len() <= 80
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && !matches!(
            name,
            "fn" | "for"
                | "then"
                | "when"
                | "type"
                | "pub"
                | "effect"
                | "match"
                | "if"
                | "else"
                | "let"
                | "rec"
                | "do"
                | "with"
                | "fails"
                | "import"
                | "from"
                | "json"
        )
}
