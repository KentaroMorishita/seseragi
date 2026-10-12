//! Bounded JSON Schema data frontend. No TypeScript parser, filesystem, or host
//! execution participates in interpretation. Unknown validation remains explicit
//! policy, rather than becoming Any or an incorrectly narrowed snapshot.
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::{Map, Value};
use seseragi_interop::*;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const MAX_BYTES: usize = 1_048_576;
const MAX_NODES: usize = 512;
const MAX_DEPTH: usize = 32;

#[derive(Debug)]
pub struct SchemaError(String);
impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for SchemaError {}

/// Interpret JSON Schema 2020-12's closed data subset. `settings` is an opaque
/// projection input; the frontend only records its exact identity. All origins
/// conservatively cover the source document and identify a JSON Pointer in the
/// rule, rather than inventing exact token spans from a value-only JSON parser.
pub fn interpret_schema(
    source: &str,
    file: &str,
    public_name: &str,
    settings: &str,
) -> Result<SemanticGraph, SchemaError> {
    if source.len() > MAX_BYTES {
        return Err(SchemaError("schema source byte budget exceeded".into()));
    }
    if !valid_name(public_name) {
        return Err(SchemaError(
            "public type name requires an ASCII type identifier".into(),
        ));
    }
    let StrictJson(value) = serde_json::from_str(source)
        .map_err(|e| SchemaError(format!("invalid schema JSON: {e}")))?;
    if let Some(dialect) = value.get("$schema") {
        if dialect.as_str() != Some("https://json-schema.org/draft/2020-12/schema") {
            return Err(SchemaError(
                "unsupported JSON Schema dialect; no interpretation was performed".into(),
            ));
        }
    }
    let defs = match value.get("$defs") {
        None => BTreeMap::new(),
        Some(Value::Object(map)) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        Some(_) => return Err(SchemaError("$defs must be an object".into())),
    };
    if defs.len() > 64 {
        return Err(SchemaError("schema definition budget exceeded".into()));
    }
    let mut builder = Builder {
        graph: SemanticGraph::default(),
        file: file.into(),
        bytes: source.len(),
        definitions: BTreeMap::new(),
        nodes: 0,
        resource_scopes: resource_scopes(&value),
    };
    let source_digest = digest(source);
    builder.graph.sources.insert(
        file.into(),
        SourceIdentity {
            digest: source_digest.clone(),
            byte_length: source.len(),
        },
    );
    builder.graph.identity = Some(ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "seseragi-json-schema".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        },
        settings_digest: digest(settings),
        source_digests: BTreeMap::from([(file.into(), source_digest)]),
        runtime_contract_digest: None,
    });
    for (index, name) in defs.keys().enumerate() {
        let symbol = format!("SchemaDef{index}");
        if symbol == public_name {
            return Err(SchemaError(
                "public type name conflicts with generated definition name".into(),
            ));
        }
        builder
            .definitions
            .insert(format!("#/$defs/{}", pointer(name)), symbol.clone());
        // Forward references are indexed before any declaration is interpreted.
        builder.graph.symbols.insert(
            symbol.clone(),
            SemanticSymbol {
                public_name: symbol.clone(),
                declaration_name: symbol,
                kind: SymbolKind::Type,
                type_ref: TypeId(0),
                origins: vec![builder.origin(&format!("#/$defs/{}", pointer(name)))],
                runtime: None,
            },
        );
    }
    for (name, value) in &defs {
        let path = format!("#/$defs/{}", pointer(name));
        let id = builder.schema(value, &path, 0)?;
        let symbol = builder.definitions[&path].clone();
        builder.graph.symbols.get_mut(&symbol).unwrap().type_ref = id;
    }
    let id = builder.schema(&value, "#", 0)?;
    builder.graph.symbols.insert(
        public_name.into(),
        SemanticSymbol {
            public_name: public_name.into(),
            declaration_name: public_name.into(),
            kind: SymbolKind::Type,
            type_ref: id,
            origins: vec![builder.origin("#")],
            runtime: None,
        },
    );
    builder
        .graph
        .normalize_closed(NormalizationBudget::default())
        .map_err(|e| SchemaError(e.to_string()))?;
    Ok(builder.graph)
}
fn valid_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn pointer(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}
fn resource_scopes(value: &Value) -> bool {
    match value {
        Value::Object(m) => {
            m.keys()
                .any(|k| matches!(k.as_str(), "$id" | "$anchor" | "$dynamicAnchor"))
                || m.values().any(resource_scopes)
        }
        Value::Array(a) => a.iter().any(resource_scopes),
        _ => false,
    }
}
struct Builder {
    graph: SemanticGraph,
    file: String,
    bytes: usize,
    definitions: BTreeMap<String, String>,
    nodes: usize,
    resource_scopes: bool,
}
impl Builder {
    fn origin(&self, path: &str) -> Origin {
        Origin {
            source: self.file.clone(),
            start: 0,
            end: self.bytes,
            rule: format!("json-schema:{path}"),
        }
    }
    fn node(
        &mut self,
        node: SemanticType,
        path: &str,
        inputs: Vec<TypeId>,
        classification: SemanticClassification,
        reason: &str,
    ) -> Result<TypeId, SchemaError> {
        if self.nodes >= MAX_NODES {
            return Err(SchemaError("schema semantic node budget exceeded".into()));
        }
        let id = TypeId(self.nodes as u32);
        self.nodes += 1;
        self.graph.types.insert(id, node);
        self.graph.type_origins.insert(id, vec![self.origin(path)]);
        self.graph.normalizations.push(Normalization {
            output: id,
            inputs,
            rule: format!("json-schema:{path}"),
            classification,
            reason: reason.into(),
            policy: None,
        });
        Ok(id)
    }
    fn policy(
        &mut self,
        path: &str,
        reason: &str,
        inputs: Vec<TypeId>,
    ) -> Result<TypeId, SchemaError> {
        self.node(
            SemanticType::Unresolved {
                reason: reason.into(),
            },
            path,
            inputs,
            SemanticClassification::PolicyRequired,
            reason,
        )
    }
    fn primitive(&mut self, domain: PrimitiveDomain, path: &str) -> Result<TypeId, SchemaError> {
        self.node(
            SemanticType::Primitive { domain },
            path,
            vec![],
            SemanticClassification::Direct,
            "JSON data primitive domain",
        )
    }
    fn compose(
        &mut self,
        mut members: Vec<TypeId>,
        union: bool,
        path: &str,
    ) -> Result<TypeId, SchemaError> {
        if members.len() == 1 {
            return Ok(members[0]);
        }
        if members.is_empty() {
            return self.policy(
                path,
                "empty schema composition requires explicit validation policy",
                vec![],
            );
        }
        members.sort();
        members.dedup();
        let node = if union {
            SemanticType::Union {
                members: members.clone(),
            }
        } else {
            SemanticType::Intersection {
                members: members.clone(),
            }
        };
        self.node(
            node,
            path,
            members,
            SemanticClassification::Direct,
            "schema domain composition preserves all validation branches",
        )
    }
    fn schema(&mut self, schema: &Value, path: &str, depth: usize) -> Result<TypeId, SchemaError> {
        if depth > MAX_DEPTH {
            return Err(SchemaError("schema recursion depth budget exceeded".into()));
        }
        if schema == &Value::Bool(false) {
            return self.primitive(PrimitiveDomain::Never, path);
        }
        if schema == &Value::Bool(true) {
            return self.policy(
                path,
                "unconstrained JSON domain requires a JSON value boundary policy",
                vec![],
            );
        }
        let Some(map) = schema.as_object() else {
            return Err(SchemaError(format!(
                "schema at {path} must be object or boolean"
            )));
        };
        for key in map.keys() {
            if !matches!(
                key.as_str(),
                "$schema"
                    | "$defs"
                    | "$ref"
                    | "title"
                    | "description"
                    | "$comment"
                    | "default"
                    | "examples"
                    | "type"
                    | "properties"
                    | "required"
                    | "additionalProperties"
                    | "items"
                    | "enum"
                    | "const"
                    | "anyOf"
                    | "allOf"
            ) {
                return self.policy(path,&format!("validation keyword {key} needs an explicit semantic policy; no constraint was discarded"),vec![]);
            }
        }
        if map
            .get("$schema")
            .is_some_and(|v| v.as_str() != Some("https://json-schema.org/draft/2020-12/schema"))
        {
            return self.policy(path, "nested schema dialect is unsupported", vec![]);
        }
        let mut parts = vec![];
        if let Some(reference) = map.get("$ref") {
            let Some(reference) = reference.as_str() else {
                return Err(SchemaError("$ref must be a string".into()));
            };
            if self.resource_scopes {
                parts.push(self.policy(path,"reference resource scopes require explicit resolution; local pointers are not assumed",vec![])?);
            } else if let Some(symbol) = self.definitions.get(reference).cloned() {
                parts.push(self.node(
                    SemanticType::Reference { symbol },
                    path,
                    vec![],
                    SemanticClassification::Direct,
                    "local indexed schema definition reference",
                )?);
            } else {
                parts.push(self.policy(
                    path,
                    "external or unindexed schema reference requires explicit resolution",
                    vec![],
                )?);
            }
        }
        if let Some(ty) = map.get("type") {
            let names: Vec<&str> = match ty {
                Value::String(s) => vec![s],
                Value::Array(a) if !a.is_empty() => a
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| SchemaError("type array members must be strings".into()))
                    })
                    .collect::<Result<_, _>>()?,
                _ => {
                    return Err(SchemaError(
                        "type must be a string or nonempty array".into(),
                    ))
                }
            };
            if names.iter().collect::<BTreeSet<_>>().len() != names.len() {
                return Err(SchemaError("type array members must be unique".into()));
            }
            let mut domains = vec![];
            for name in names {
                domains.push(match name {
                    "string" => self.primitive(PrimitiveDomain::String,path)?,
                    "boolean" => self.primitive(PrimitiveDomain::Boolean,path)?,
                    "null" => self.primitive(PrimitiveDomain::Null,path)?,
                    "integer"|"number" => self.policy(path,"JSON mathematical numeric domain needs explicit bounded integer/precision policy",vec![])?,
                    "object" => self.object(map,path,depth)?,
                    "array" => {
                        let Some(items) = map.get("items") else { return self.policy(path,"unconstrained array items require a JSON value boundary policy",vec![]); };
                        let element = self.schema(items,&format!("{path}/items"),depth+1)?;
                        self.node(SemanticType::Collection { element, mutable: false },path,vec![element],SemanticClassification::Direct,"JSON array is interpreted as an immutable decoded data snapshot")?
                    },
                    _ => return Err(SchemaError(format!("invalid JSON Schema type {name}"))),
                });
            }
            parts.push(self.compose(domains, true, path)?);
        } else if map.keys().any(|k| {
            matches!(
                k.as_str(),
                "properties" | "required" | "additionalProperties" | "items"
            )
        }) {
            parts.push(self.policy(
                path,
                "object/array applicators without a matching type also accept other JSON domains",
                vec![],
            )?);
        }
        if let Some(values) = map.get("enum") {
            let Some(values) = values.as_array().filter(|a| !a.is_empty()) else {
                return Err(SchemaError("enum requires a nonempty array".into()));
            };
            let mut literals = vec![];
            for (i, v) in values.iter().enumerate() {
                literals.push(self.literal(v, &format!("{path}/enum/{i}"))?);
            }
            parts.push(self.compose(literals, true, path)?);
        }
        if let Some(value) = map.get("const") {
            parts.push(self.literal(value, &format!("{path}/const"))?);
        }
        for (key, union) in [("anyOf", true), ("allOf", false)] {
            if let Some(branches) = map.get(key) {
                let Some(branches) = branches.as_array().filter(|a| !a.is_empty()) else {
                    return Err(SchemaError(format!("{key} requires nonempty schema array")));
                };
                let mut members = vec![];
                for (i, v) in branches.iter().enumerate() {
                    members.push(self.schema(v, &format!("{path}/{key}/{i}"), depth + 1)?);
                }
                parts.push(self.compose(members, union, path)?);
            }
        }
        if parts.is_empty() {
            return self.policy(path, "schema has no interpreted domain constraint", vec![]);
        }
        self.compose(parts, false, path)
    }
    fn object(
        &mut self,
        map: &Map<String, Value>,
        path: &str,
        depth: usize,
    ) -> Result<TypeId, SchemaError> {
        if map.get("additionalProperties") != Some(&Value::Bool(false)) {
            return self.policy(
                path,
                "open JSON object requires an explicit additional-property boundary policy",
                vec![],
            );
        }
        let props = match map.get("properties") {
            None => Map::new(),
            Some(Value::Object(p)) => p.clone(),
            Some(_) => return Err(SchemaError("properties must be an object".into())),
        };
        let required = match map.get("required") {
            None => BTreeSet::new(),
            Some(Value::Array(a)) => a
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| SchemaError("required members must be strings".into()))
                })
                .collect::<Result<BTreeSet<_>, _>>()?,
            Some(_) => return Err(SchemaError("required must be an array".into())),
        };
        if map
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|a| a.len() != required.len())
        {
            return Err(SchemaError("required members must be unique".into()));
        }
        if required.iter().any(|k| !props.contains_key(k)) {
            return self.primitive(PrimitiveDomain::Never, path);
        }
        let mut fields = BTreeMap::new();
        let mut inputs = vec![];
        for (name, value) in &props {
            let id = self.schema(
                value,
                &format!("{path}/properties/{}", pointer(name)),
                depth + 1,
            )?;
            inputs.push(id);
            fields.insert(
                name.clone(),
                ShapeField {
                    type_ref: id,
                    optional: !required.contains(name),
                    readonly: true,
                },
            );
        }
        self.node(
            SemanticType::Shape {
                identity: ShapeIdentity::ReadonlySnapshot,
                fields,
            },
            path,
            inputs,
            SemanticClassification::Direct,
            "closed JSON object is a readonly decoded data snapshot; optional does not imply null",
        )
    }
    fn literal(&mut self, value: &Value, path: &str) -> Result<TypeId, SchemaError> {
        let value = match value {
            Value::String(s) => LiteralValue::String(s.clone()),
            Value::Bool(b) => LiteralValue::Boolean(*b),
            Value::Number(n) if n.as_i64().is_some() => LiteralValue::Integer(n.as_i64().unwrap()),
            Value::Null => return self.primitive(PrimitiveDomain::Null,path),
            _ => return self.policy(path,"non-scalar or non-integral/out-of-range numeric literal requires explicit data/precision policy",vec![]),
        };
        self.node(
            SemanticType::Literal { value },
            path,
            vec![],
            SemanticClassification::Direct,
            "finite exact JSON scalar literal",
        )
    }
}

// serde_json::Value normally accepts repeated object keys with last-value wins.
// Declaration meaning must not depend on silently discarded source constraints.
struct StrictJson(Value);
impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictJson;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::Bool(v)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::Null))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictJson(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictJson(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| StrictJson(Value::Number(n)))
                    .ok_or_else(|| E::custom("invalid JSON number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::String(v.into())))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictJson(Value::String(v)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut v = vec![];
                while let Some(StrictJson(x)) = a.next_element()? {
                    v.push(x);
                }
                Ok(StrictJson(Value::Array(v)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut m = Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if m.contains_key(&k) {
                        return Err(de::Error::custom(format!("duplicate object key {k}")));
                    }
                    let StrictJson(v) = a.next_value()?;
                    m.insert(k, v);
                }
                Ok(StrictJson(Value::Object(m)))
            }
        }
        d.deserialize_any(StrictVisitor)
    }
}
