//! Checked application adapters generated from common meaning, never source AST.
//! These adapters consume std/json::Json. They do not cast arbitrary raw host
//! objects or acquire resources. Raw ABI binding generation is a separate route.
use serde::{Deserialize, Serialize};
use seseragi_interop::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticAdapters {
    pub schema: u32,
    pub input_boundary: String,
    pub graph_digest: String,
    pub semantic_source: String,
    pub applied_policies: Vec<AppliedProjectionPolicy>,
}

pub fn generate_adapters(
    graph: &SemanticGraph,
    policies: &ProjectionPolicySet,
) -> Result<SemanticAdapters, String> {
    let applied = validate_projection_policies(graph, policies)?;
    let mut generator = Generator {
        graph,
        work: 4096,
        helpers: vec![],
        helper_bytes: 0,
    };
    let mut source = String::from("// Generated checked semantic API. Raw foreign bindings are separate.\nimport * as json from \"std/json\"\n\n");
    for policy in &applied {
        let (target, decoder) = match &policy.mapping {
            AdapterMapping::JsonSnapshot | AdapterMapping::JsonNullable => {
                generator.codec(policy.type_ref, &BTreeSet::new())?
            }
            AdapterMapping::JsonTaggedUnion { discriminator } => {
                let (declaration, target, decoder) =
                    generator.adt(policy.type_ref, discriminator, &policy.function_name)?;
                source.push_str(&declaration);
                source.push('\n');
                generator.checked(target, decoder)?
            }
        };
        source.push_str(&format!("pub fn {} value: json.Json -> Either<json.DecodeError, {target}> =\n  ({decoder}) value\n\n",policy.function_name));
        if source.len() > 131_072 {
            return Err("semantic adapter source byte budget exceeded".into());
        }
    }
    for policy in &applied {
        if policy.function_name.starts_with("interopCodec__")
            || policy.function_name == "interopReject__"
        {
            return Err("adapter name conflicts with generated private helper namespace".into());
        }
    }
    source.push_str("fn interopReject__<A> value: json.Json -> Either<json.DecodeError, A> = { let choices: Array<json.Decoder<A>> = []; let reader: json.Decoder<A> = json.oneOf<A> choices; reader value }\n\n");
    source.push_str(&generator.helpers.join("\n\n"));
    if source.len() > 131_072 {
        return Err("semantic adapter source byte budget exceeded".into());
    }
    Ok(SemanticAdapters {
        schema: 1,
        input_boundary: "std/json::Json".into(),
        graph_digest: graph.digest().map_err(|e| e.to_string())?,
        semantic_source: source,
        applied_policies: applied,
    })
}
struct Generator<'a> {
    graph: &'a SemanticGraph,
    work: usize,
    helpers: Vec<String>,
    helper_bytes: usize,
}
impl Generator<'_> {
    fn checked(&mut self, ty: String, expression: String) -> Result<(String, String), String> {
        let (parameter, body) = if let Some(body) =
            expression.strip_prefix("\\__input: json.Json -> ")
        {
            ("__input", body.to_owned())
        } else if let Some(body) = expression.strip_prefix("\\__object: json.Json -> ") {
            ("__object", body.to_owned())
        } else {
            (
                "__input",
                format!("{{ let reader: json.Decoder<{ty}> = ({expression}); reader __input }}"),
            )
        };
        let name = format!("interopCodec__{}", self.helpers.len());
        let helper = format!(
            "fn {name} {parameter}: json.Json -> Either<json.DecodeError, {ty}> =\n  {body}"
        );
        self.helper_bytes += helper.len();
        if self.helper_bytes > 131_072 {
            return Err("semantic helper source byte budget exceeded".into());
        }
        self.helpers.push(helper);
        Ok((ty, name))
    }
    fn codec(&mut self, id: TypeId, seen: &BTreeSet<TypeId>) -> Result<(String, String), String> {
        let (ty, expression) = self.expression(id, seen)?;
        self.checked(ty, expression)
    }
    fn expression(
        &mut self,
        id: TypeId,
        seen: &BTreeSet<TypeId>,
    ) -> Result<(String, String), String> {
        if self.work == 0 || seen.len() >= 32 {
            return Err("semantic codec work/depth budget exceeded".into());
        }
        self.work -= 1;
        let (id, node) = projection_node(self.graph, id)
            .ok_or("cyclic/missing alias has no finite JSON codec")?;
        if seen.contains(&id) {
            return Err("recursive shape requires an explicit recursive codec contract".into());
        }
        let mut seen = seen.clone();
        seen.insert(id);
        match node {
            SemanticType::Primitive { domain: PrimitiveDomain::String } => Ok(("String".into(),"decodeJson<String>".into())),
            SemanticType::Primitive { domain: PrimitiveDomain::Boolean } => Ok(("Bool".into(),"decodeJson<Bool>".into())),
            SemanticType::Primitive { domain: PrimitiveDomain::Integer } => Ok(("Int".into(),"decodeJson<Int>".into())),
            SemanticType::Literal { value } => {
                let (ty,lit) = match value {
                    LiteralValue::String(s) => ("String",quote(s)?),
                    LiteralValue::Boolean(b) => ("Bool",if *b { "True" } else { "False" }.into()),
                    LiteralValue::Integer(n) if n.unsigned_abs() <= 9_007_199_254_740_991 => ("Int",n.to_string()),
                    _ => return Err("exact literal is outside the native Int codec domain".into()),
                };
                Ok((ty.into(),format!("\\__input: json.Json -> match (decodeJson<{ty}>) __input {{ Left __error -> Left __error; Right __decoded -> if __decoded == {lit} then Right __decoded else {} }}",invalid(ty,"__input"))))
            },
            SemanticType::Shape { identity: ShapeIdentity::ReadonlySnapshot, fields } => self.shape(fields,&seen),
            SemanticType::Nullable { value } => self.nullable(*value,&seen),
            SemanticType::Collection { element, mutable: false } => {
                let (ty,decoder) = self.codec(*element,&seen)?;
                Ok((format!("Array<{ty}>"),format!("json.array<{ty}> ({decoder})")))
            },
            SemanticType::Union { members } => {
                let mut nullable = None; let mut null_count = 0;
                for member in members {
                    match projection_node(self.graph,*member) {
                        Some((_,SemanticType::Primitive { domain: PrimitiveDomain::Null })) => null_count += 1,
                        _ if nullable.is_none() => nullable = Some(*member),
                        _ => { nullable = None; break; },
                    }
                }
                if members.len()==2 && null_count==1 { if let Some(value) = nullable { return self.nullable(value,&seen); } }
                let mut values = vec![];
                for member in members {
                    let Some((_,SemanticType::Literal { value: LiteralValue::String(s) })) = projection_node(self.graph,*member) else { return Err("union needs finite homogeneous literals or explicit tagged ADT policy".into()); };
                    values.push(quote(s)?);
                }
                if values.is_empty() || values.len()>64 { return Err("finite enum member budget exceeded".into()); }
                let condition = values.iter().map(|v|format!("__decoded == {v}")).collect::<Vec<_>>().join(" || ");
                Ok(("String".into(),format!("\\__input: json.Json -> match (decodeJson<String>) __input {{ Left __error -> Left __error; Right __decoded -> if {condition} then Right __decoded else {} }}",invalid("String","__input"))))
            },
            SemanticType::Optional { .. } => Err("JSON cannot preserve explicit undefined; optional foreign value needs another boundary adapter".into()),
            _ => Err("nominal/mutable/opaque/generic/async/resource meaning has no implicit JSON data codec".into()),
        }
    }
    fn nullable(
        &mut self,
        id: TypeId,
        seen: &BTreeSet<TypeId>,
    ) -> Result<(String, String), String> {
        let (ty, decoder) = self.codec(id, seen)?;
        Ok((format!("Maybe<{ty}>"),format!("\\__input: json.Json -> match __input {{ json.JsonNull -> Right Nothing; _ -> match ({decoder}) __input {{ Left __error -> Left __error; Right __decoded -> Right (Just __decoded) }} }}")))
    }
    fn shape(
        &mut self,
        fields: &BTreeMap<String, ShapeField>,
        seen: &BTreeSet<TypeId>,
    ) -> Result<(String, String), String> {
        if fields.is_empty() {
            return Err(
                "empty snapshot requires a native empty-record value codec contract".into(),
            );
        }
        if fields.len() > 32 {
            return Err("snapshot field budget exceeded".into());
        }
        let mut types = vec![];
        let mut inputs = vec![];
        let mut values = vec![];
        for (index, (name, field)) in fields.iter().enumerate() {
            if !safe_field(name) {
                return Err("record field is not representable as a safe native identifier".into());
            }
            let (inner, decoder) = self.codec(field.type_ref, seen)?;
            let ty = if field.optional {
                format!("Maybe<{inner}>")
            } else {
                inner.clone()
            };
            let getter = if field.optional {
                "optionalField"
            } else {
                "field"
            };
            types.push(format!("{name}: {ty}"));
            values.push(format!("{name}: __field{index}"));
            inputs.push((
                format!("json.{getter}<{inner}> {} ({decoder})", quote(name)?),
                ty,
            ));
        }
        let ty = format!("{{ {} }}", types.join(", "));
        let mut body = format!("Right ({{ {} }})", values.join(", "));
        for (index, (decoder, ty)) in inputs.iter().enumerate().rev() {
            body = format!("{{ let reader{index}: json.Decoder<{ty}> = ({decoder}); match reader{index} __object {{ Left __error -> Left __error; Right __field{index} -> {body} }} }}");
        }
        Ok((ty.clone(),format!("\\__object: json.Json -> match __object {{ json.JsonObject _ -> {body}; _ -> {} }}",invalid(&ty,"__object"))))
    }
    fn adt(
        &mut self,
        id: TypeId,
        tag: &str,
        function: &str,
    ) -> Result<(String, String, String), String> {
        let Some((_, SemanticType::Union { members })) = projection_node(self.graph, id) else {
            return Err("ADT policy requires a union".into());
        };
        let mut chars = function.chars();
        let target = format!(
            "{}{}Value",
            chars.next().unwrap().to_ascii_uppercase(),
            chars.as_str()
        );
        let mut declarations = vec![];
        let mut branches = vec![];
        for (index, member) in members.iter().enumerate() {
            let Some((
                _,
                SemanticType::Shape {
                    identity: ShapeIdentity::ReadonlySnapshot,
                    fields,
                },
            )) = projection_node(self.graph, *member)
            else {
                return Err("ADT variant is not a readonly snapshot".into());
            };
            let Some((
                _,
                SemanticType::Literal {
                    value: LiteralValue::String(value),
                },
            )) = projection_node(self.graph, fields[tag].type_ref)
            else {
                return Err("ADT tag is not a literal string".into());
            };
            let mut payload = fields.clone();
            payload.remove(tag);
            let variant = format!("{target}Case{index}");
            if payload.is_empty() {
                declarations.push(format!("  | {variant}"));
                branches.push((quote(value)?, format!("Right {variant}")));
                continue;
            }
            let (ty, expression) = self.shape(&payload, &BTreeSet::from([*member]))?;
            let (ty, decoder) = self.checked(ty, expression)?;
            declarations.push(format!("  | {variant} {ty}"));
            branches.push((quote(value)?,format!("match ({decoder}) __input {{ Left __error -> Left __error; Right __payload -> Right ({variant} __payload) }}")));
        }
        let mut branch = invalid(&target, "__input");
        for (tag_value, success) in branches.into_iter().rev() {
            branch = format!("if __tag == {tag_value} then {success} else {branch}");
        }
        let declaration = format!("pub type {target} =\n{}\n", declarations.join("\n"));
        let decoder = format!("\\__input: json.Json -> {{ let tagReader: json.Decoder<String> = json.field<String> {} decodeJson<String>; match tagReader __input {{ Left __error -> Left __error; Right __tag -> {branch} }} }}",quote(tag)?);
        Ok((declaration, target, decoder))
    }
}
fn invalid(target: &str, parameter: &str) -> String {
    format!("interopReject__<{target}> {parameter}")
}
fn quote(value: &str) -> Result<String, String> {
    let mut output = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\0' => output.push_str("\\0"),
            c if c.is_control() => output.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => output.push(c),
        }
    }
    output.push('"');
    Ok(output)
}
fn safe_field(name: &str) -> bool {
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
        )
}
