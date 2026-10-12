use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SEMANTIC_SCHEMA: u32 = 1;
pub const SEMANTIC_KIND: &str = "seseragi-interop-semantic-graph";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypeId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticGraph {
    pub schema: u32,
    pub kind: String,
    pub types: BTreeMap<TypeId, SemanticType>,
    pub symbols: BTreeMap<String, SemanticSymbol>,
    pub sources: BTreeMap<String, SourceIdentity>,
    pub type_origins: BTreeMap<TypeId, Vec<Origin>>,
    pub normalizations: Vec<Normalization>,
    pub identity: Option<ArtifactIdentity>,
}

impl Default for SemanticGraph {
    fn default() -> Self {
        Self {
            schema: SEMANTIC_SCHEMA,
            kind: SEMANTIC_KIND.to_owned(),
            types: BTreeMap::new(),
            symbols: BTreeMap::new(),
            sources: BTreeMap::new(),
            type_origins: BTreeMap::new(),
            normalizations: Vec::new(),
            identity: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceIdentity {
    pub digest: String,
    pub byte_length: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub frontend: ProducerIdentity,
    pub settings_digest: String,
    pub source_digests: BTreeMap<String, String>,
    pub runtime_contract_digest: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProducerIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Normalization {
    pub output: TypeId,
    pub inputs: Vec<TypeId>,
    pub rule: String,
    pub classification: SemanticClassification,
    pub reason: String,
    pub policy: Option<ProducerIdentity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticClassification {
    Direct,
    Normalized,
    OpaquePreserved,
    SafeBoundary,
    PolicyRequired,
    RuntimeAbiUnresolved,
    Uninterpreted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Origin {
    pub source: String,
    pub start: usize,
    pub end: usize,
    pub rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticSymbol {
    pub public_name: String,
    pub declaration_name: String,
    pub kind: SymbolKind,
    pub type_ref: TypeId,
    pub origins: Vec<Origin>,
    pub runtime: Option<RuntimeBinding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SymbolKind {
    Type,
    Callable,
    Namespace,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SemanticType {
    Primitive {
        domain: PrimitiveDomain,
    },
    Literal {
        value: LiteralValue,
    },
    Reference {
        symbol: String,
    },
    Generic {
        parameters: Vec<GenericParameter>,
        body: TypeId,
    },
    Parameter {
        binder: TypeId,
        index: u32,
    },
    Apply {
        constructor: TypeId,
        arguments: Vec<TypeId>,
    },
    Shape {
        identity: ShapeIdentity,
        fields: BTreeMap<String, ShapeField>,
    },
    Opaque {
        identity: String,
        arguments: Vec<TypeId>,
    },
    Tuple {
        elements: Vec<TypeId>,
    },
    Union {
        members: Vec<TypeId>,
    },
    Intersection {
        members: Vec<TypeId>,
    },
    Collection {
        element: TypeId,
        mutable: bool,
    },
    Nullable {
        value: TypeId,
    },
    Optional {
        value: TypeId,
    },
    Async {
        value: TypeId,
    },
    Callable {
        parameters: Vec<CallParameter>,
        result: TypeId,
        receiver: Option<TypeId>,
    },
    Overload {
        signatures: Vec<TypeId>,
    },
    Namespace {
        members: BTreeMap<String, String>,
    },
    Projection {
        relation: ProjectionRelation,
        subject: TypeId,
        key: Option<TypeId>,
    },
    /// Explicit, reason-bearing state; never masquerades as Any/Unknown.
    Unresolved {
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrimitiveDomain {
    Unit,
    Never,
    Boolean,
    String,
    Integer,
    Float64,
    BigInteger,
    Unknown,
    Object,
    Null,
    Undefined,
    Dynamic,
    Binary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum LiteralValue {
    String(String),
    Boolean(bool),
    // JSON consumers include JavaScript hosts. Decimal text preserves the
    // complete domain instead of rounding integers beyond binary64 precision.
    Integer(#[serde(with = "integer_text")] i64),
}

mod integer_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
        let text = String::deserialize(deserializer)?;
        let value = text.parse::<i64>().map_err(serde::de::Error::custom)?;
        if value.to_string() != text {
            return Err(serde::de::Error::custom(
                "integer literal requires canonical decimal text",
            ));
        }
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShapeIdentity {
    ReadonlySnapshot,
    MutableIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShapeField {
    pub type_ref: TypeId,
    pub optional: bool,
    pub readonly: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenericParameter {
    pub name: String,
    pub constraint: Option<TypeId>,
    pub default: Option<TypeId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CallParameter {
    pub name: String,
    pub origins: Vec<Origin>,
    pub type_ref: TypeId,
    pub optional: bool,
    pub rest: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionRelation {
    Keys,
    Member,
    Extract,
}

/// Type checking never executes or resolves this locator. Adapters supply it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeBinding {
    pub exact_identity: String,
    pub export_path: Vec<String>,
    pub targets: Vec<String>,
    pub convention: CallingConvention,
    pub evaluation: Evaluation,
    pub failure: FailureBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallingConvention {
    Value,
    Function,
    Constructor,
    Method,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Evaluation {
    PureApproved,
    Effectful,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureBoundary {
    CannotFailApproved,
    Throw,
    Rejection,
    ThrowOrRejection,
    Unresolved,
}
