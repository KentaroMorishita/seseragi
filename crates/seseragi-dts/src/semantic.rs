//! The only boundary that reads the TypeScript declaration model.
//! Projection receives the common graph, never parser nodes or TypeKind.
use crate::model::{Declaration, Parameter, Scope, Span, TypeKind, TypeRef};
use crate::parser::{parse_declarations, ParseError};
use seseragi_interop::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum SemanticError {
    Parse {
        message: String,
        start: usize,
        end: usize,
    },
    Invalid(ValidationError),
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse {
                message,
                start,
                end,
            } => write!(f, "{start}..{end}: {message}"),
            Self::Invalid(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for SemanticError {}

/// Portable semantic interpretation. No host resolution or execution occurs.
/// Settings are part of the artifact identity even when projection policies
/// will be applied later; reusing a graph with changed settings fails closed.
pub fn interpret_declaration(
    source: &str,
    file: &str,
    settings: &str,
) -> Result<SemanticGraph, SemanticError> {
    interpret_with_digest(source, file, &digest(settings))
}

pub(crate) fn interpret_with_digest(
    source: &str,
    file: &str,
    settings_digest: &str,
) -> Result<SemanticGraph, SemanticError> {
    let scope = parse_declarations(source).map_err(|ParseError { message, span }| {
        SemanticError::Parse {
            message,
            start: span.start,
            end: span.end,
        }
    })?;
    interpret_scope(&scope, source, file, settings_digest)
}

pub(crate) fn interpret_scope(
    scope: &Scope,
    source: &str,
    file: &str,
    settings_digest: &str,
) -> Result<SemanticGraph, SemanticError> {
    let source_digest = digest(source);
    let mut builder = Builder {
        graph: SemanticGraph::default(),
        file: file.to_owned(),
        next: 0,
        names: BTreeMap::new(),
    };
    builder.graph.sources.insert(
        file.to_owned(),
        SourceIdentity {
            digest: source_digest.clone(),
            byte_length: source.len(),
        },
    );
    builder.graph.identity = Some(ArtifactIdentity {
        frontend: ProducerIdentity {
            name: "seseragi-dts".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        settings_digest: settings_digest.to_owned(),
        source_digests: BTreeMap::from([(file.to_owned(), source_digest)]),
        runtime_contract_digest: None,
    });
    builder.index(scope, "");
    builder.scope(scope, "");
    builder.graph.validate().map_err(SemanticError::Invalid)?;
    Ok(builder.graph)
}

fn digest(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}
fn path(namespace: &str, name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{namespace}.{name}")
    }
}

struct Builder {
    graph: SemanticGraph,
    file: String,
    next: u32,
    names: BTreeMap<String, String>,
}

impl Builder {
    fn reserve(&mut self) -> TypeId {
        let id = TypeId(self.next);
        self.next += 1;
        id
    }
    fn origin(&self, span: Span, rule: &str) -> Origin {
        Origin {
            source: self.file.clone(),
            start: span.start,
            end: span.end,
            rule: rule.to_owned(),
        }
    }
    fn node(&mut self, node: SemanticType, span: Span, rule: &str) -> TypeId {
        let id = self.reserve();
        self.graph.types.insert(id, node);
        self.graph
            .type_origins
            .insert(id, vec![self.origin(span, rule)]);
        id
    }
    fn decision(&mut self, output: TypeId, classification: SemanticClassification, reason: &str) {
        self.graph.normalizations.push(Normalization {
            output,
            inputs: vec![],
            rule: "declaration-interpretation".to_owned(),
            classification,
            reason: reason.to_owned(),
            policy: None,
        });
    }
    fn index(&mut self, scope: &Scope, namespace: &str) {
        for declaration in &scope.declarations {
            let public = public_name(declaration);
            self.names
                .entry(path(namespace, declaration.original_name()))
                .or_insert_with(|| path(namespace, public));
            self.names
                .insert(path(namespace, public), path(namespace, public));
            if let Declaration::Namespace(value) = declaration {
                self.index(&value.scope, &path(namespace, public));
            }
        }
    }
    fn scope(&mut self, scope: &Scope, namespace: &str) -> BTreeMap<String, String> {
        let mut members = BTreeMap::new();
        for declaration in &scope.declarations {
            let public = public_name(declaration);
            let key = path(namespace, public);
            let (span, name_span, relations, kind) = match declaration {
                Declaration::Function(value) => (
                    value.span,
                    value.name_span,
                    value.type_parameter_relations.as_slice(),
                    SymbolKind::Callable,
                ),
                Declaration::Interface(value) => (
                    value.span,
                    value.name_span,
                    value.type_parameter_relations.as_slice(),
                    SymbolKind::Type,
                ),
                Declaration::Class(value) | Declaration::Enum(value) => (
                    value.span,
                    value.name_span,
                    value.type_parameter_relations.as_slice(),
                    SymbolKind::Type,
                ),
                Declaration::TypeAlias(value) => (
                    value.span,
                    value.name_span,
                    value.type_parameter_relations.as_slice(),
                    SymbolKind::Type,
                ),
                Declaration::Namespace(value) => {
                    (value.span, value.name_span, &[][..], SymbolKind::Namespace)
                }
            };
            let binder = (!relations.is_empty()).then(|| self.reserve());
            let parameters = relations
                .iter()
                .enumerate()
                .map(|(index, p)| (p.name.clone(), (binder.unwrap(), index as u32)))
                .collect::<BTreeMap<_, _>>();
            let body = match declaration {
                Declaration::Function(value) => {
                    let arguments = value
                        .parameters
                        .iter()
                        .map(|p| self.parameter(p, namespace, &parameters))
                        .collect();
                    let result = self.ty(&value.result, namespace, &parameters);
                    self.node(
                        SemanticType::Callable {
                            parameters: arguments,
                            result,
                            receiver: None,
                        },
                        span,
                        "callable-signature",
                    )
                }
                Declaration::TypeAlias(value) => self.ty(&value.type_ref, namespace, &parameters),
                Declaration::Namespace(value) => {
                    let nested = self.scope(&value.scope, &key);
                    self.node(
                        SemanticType::Namespace { members: nested },
                        span,
                        "member-ownership",
                    )
                }
                _ => {
                    let arguments = relations
                        .iter()
                        .enumerate()
                        .map(|(index, _)| {
                            self.node(
                                SemanticType::Parameter {
                                    binder: binder.unwrap(),
                                    index: index as u32,
                                },
                                span,
                                "generic-parameter",
                            )
                        })
                        .collect();
                    let id = self.node(
                        SemanticType::Opaque {
                            identity: path(namespace, declaration.original_name()),
                            arguments,
                        },
                        span,
                        "opaque-identity",
                    );
                    self.decision(id, SemanticClassification::OpaquePreserved, "declaration identity is retained; structural and constructor interpretation requires a separate boundary policy");
                    id
                }
            };
            let root = if let Some(binder) = binder {
                let generic = relations
                    .iter()
                    .map(|p| GenericParameter {
                        name: p.name.clone(),
                        constraint: p
                            .constraint
                            .as_ref()
                            .map(|ty| self.ty(ty, namespace, &parameters)),
                        default: p
                            .default
                            .as_ref()
                            .map(|ty| self.ty(ty, namespace, &parameters)),
                    })
                    .collect();
                self.graph.types.insert(
                    binder,
                    SemanticType::Generic {
                        parameters: generic,
                        body,
                    },
                );
                self.graph
                    .type_origins
                    .insert(binder, vec![self.origin(span, "generic-binder")]);
                binder
            } else {
                body
            };
            let origins = vec![
                self.origin(span, "declaration"),
                self.origin(name_span, "public-name"),
            ];
            if let Some(previous) = self.graph.symbols.get(&key).cloned() {
                if kind == SymbolKind::Callable && previous.kind == SymbolKind::Callable {
                    let mut signatures = match self.graph.types.get(&previous.type_ref) {
                        Some(SemanticType::Overload { signatures }) => signatures.clone(),
                        _ => vec![previous.type_ref],
                    };
                    signatures.push(root);
                    let overload =
                        self.node(SemanticType::Overload { signatures }, span, "overload-set");
                    let symbol = self.graph.symbols.get_mut(&key).unwrap();
                    symbol.type_ref = overload;
                    symbol.origins.extend(origins);
                    self.decision(
                        overload,
                        SemanticClassification::PolicyRequired,
                        "overload selection must be explicit at the runtime binding boundary",
                    );
                } else if kind == SymbolKind::Namespace && previous.kind == SymbolKind::Namespace {
                    let SemanticType::Namespace { members: next } = self.graph.types[&root].clone()
                    else {
                        unreachable!()
                    };
                    let SemanticType::Namespace { members } =
                        self.graph.types.get_mut(&previous.type_ref).unwrap()
                    else {
                        unreachable!()
                    };
                    members.extend(next);
                    self.graph
                        .symbols
                        .get_mut(&key)
                        .unwrap()
                        .origins
                        .extend(origins);
                } else {
                    // Do not silently pretend declaration merging is understood.
                    self.graph
                        .symbols
                        .get_mut(&key)
                        .unwrap()
                        .origins
                        .extend(origins);
                    let merged_origin = self.origin(span, "merged-declaration");
                    self.graph
                        .type_origins
                        .entry(previous.type_ref)
                        .or_default()
                        .push(merged_origin);
                    self.graph.normalizations.push(Normalization {
                        output: previous.type_ref,
                        inputs: vec![root],
                        rule: "declaration-merge".to_owned(),
                        classification: SemanticClassification::Uninterpreted,
                        reason: "declaration identity and all source origins are retained; merged structural meaning still requires interpretation".to_owned(),
                        policy: None,
                    });
                }
            } else {
                self.graph.symbols.insert(
                    key.clone(),
                    SemanticSymbol {
                        public_name: public.to_owned(),
                        declaration_name: declaration.original_name().to_owned(),
                        kind,
                        type_ref: root,
                        origins,
                        runtime: None,
                    },
                );
            }
            members.insert(public.to_owned(), key);
        }
        members
    }
    fn parameter(
        &mut self,
        value: &Parameter,
        namespace: &str,
        parameters: &BTreeMap<String, (TypeId, u32)>,
    ) -> CallParameter {
        CallParameter {
            name: value.name.clone(),
            origins: vec![self.origin(value.name_span, "parameter-name")],
            type_ref: self.ty(&value.type_ref, namespace, parameters),
            optional: value.optional,
            rest: value.rest,
        }
    }
    fn ty(
        &mut self,
        value: &TypeRef,
        namespace: &str,
        parameters: &BTreeMap<String, (TypeId, u32)>,
    ) -> TypeId {
        let node = match &value.kind {
            TypeKind::Primitive(name) => match primitive(name) {
                Some(domain) => SemanticType::Primitive { domain },
                None => SemanticType::Unresolved {
                    reason: format!("unknown primitive domain: {name}"),
                },
            },
            TypeKind::Named(name) => return self.named(name, value.span, namespace, parameters),
            TypeKind::Generic { name, arguments } => {
                let args = arguments
                    .iter()
                    .map(|ty| self.ty(ty, namespace, parameters))
                    .collect::<Vec<_>>();
                match (name.as_str(), args.as_slice()) {
                    ("Promise" | "PromiseLike", [value]) => SemanticType::Async { value: *value },
                    ("ReadonlyArray", [element]) => SemanticType::Collection {
                        element: *element,
                        mutable: false,
                    },
                    ("Array", [element]) => SemanticType::Collection {
                        element: *element,
                        mutable: true,
                    },
                    ("Promise" | "PromiseLike" | "ReadonlyArray" | "Array", _) => {
                        SemanticType::Unresolved {
                            reason: format!("{name} requires exactly one argument"),
                        }
                    }
                    _ => {
                        let constructor = self.named(name, value.span, namespace, parameters);
                        SemanticType::Apply {
                            constructor,
                            arguments: args,
                        }
                    }
                }
            }
            TypeKind::ReadonlyArray(element) | TypeKind::MutableArray(element) => {
                SemanticType::Collection {
                    element: self.ty(element, namespace, parameters),
                    mutable: matches!(value.kind, TypeKind::MutableArray(_)),
                }
            }
            TypeKind::Tuple(elements) => SemanticType::Tuple {
                elements: elements
                    .iter()
                    .map(|ty| self.ty(ty, namespace, parameters))
                    .collect(),
            },
            TypeKind::Union(members) => SemanticType::Union {
                members: members
                    .iter()
                    .map(|ty| self.ty(ty, namespace, parameters))
                    .collect(),
            },
            TypeKind::Function {
                parameters: args,
                result,
            } => {
                let args = args
                    .iter()
                    .map(|p| self.parameter(p, namespace, parameters))
                    .collect();
                let result = self.ty(result, namespace, parameters);
                SemanticType::Callable {
                    parameters: args,
                    result,
                    receiver: None,
                }
            }
            TypeKind::Literal(raw) => {
                if let Some(domain) = primitive(raw) {
                    SemanticType::Primitive { domain }
                } else if let Ok(integer) = raw.parse::<i64>() {
                    SemanticType::Literal {
                        value: LiteralValue::Integer(integer),
                    }
                } else if raw == "true" || raw == "false" {
                    SemanticType::Literal {
                        value: LiteralValue::Boolean(raw == "true"),
                    }
                } else if let Ok(text) = serde_json::from_str::<String>(raw) {
                    SemanticType::Literal {
                        value: LiteralValue::String(text),
                    }
                } else {
                    SemanticType::Unresolved {
                        reason: "literal domain requires frontend normalization".to_owned(),
                    }
                }
            }
            TypeKind::Unsupported(_) => SemanticType::Unresolved {
                reason: "type expression has no implemented semantic interpretation".to_owned(),
            },
        };
        let id = self.node(node, value.span, "type-interpretation");
        match self.graph.types.get(&id) {
            Some(SemanticType::Unresolved { .. }) => self.decision(
                id,
                SemanticClassification::Uninterpreted,
                "type meaning is unresolved; no permissive fallback was applied",
            ),
            Some(SemanticType::Primitive {
                domain: PrimitiveDomain::Dynamic,
            }) => self.decision(
                id,
                SemanticClassification::PolicyRequired,
                "dynamic domain requires explicit unsafe boundary approval",
            ),
            _ => {}
        }
        id
    }
    fn named(
        &mut self,
        name: &str,
        span: Span,
        namespace: &str,
        parameters: &BTreeMap<String, (TypeId, u32)>,
    ) -> TypeId {
        if let Some((binder, index)) = parameters.get(name) {
            return self.node(
                SemanticType::Parameter {
                    binder: *binder,
                    index: *index,
                },
                span,
                "lexical-type-parameter",
            );
        }
        if name == "Uint8Array" {
            return self.node(
                SemanticType::Primitive {
                    domain: PrimitiveDomain::Binary,
                },
                span,
                "binary-domain",
            );
        }
        if let Some(domain) = primitive(name) {
            return self.node(SemanticType::Primitive { domain }, span, "primitive-domain");
        }
        let mut owner = namespace;
        loop {
            if let Some(symbol) = self.names.get(&path(owner, name)).cloned() {
                return self.node(
                    SemanticType::Reference { symbol },
                    span,
                    "declaration-reference",
                );
            }
            if owner.is_empty() {
                break;
            }
            owner = owner.rsplit_once('.').map_or("", |(parent, _)| parent);
        }
        let id = self.node(
            SemanticType::Opaque {
                identity: name.to_owned(),
                arguments: vec![],
            },
            span,
            "external-type-identity",
        );
        self.decision(id, SemanticClassification::RuntimeAbiUnresolved, "external type name retained as an opaque identity; declaration and runtime evidence are absent");
        id
    }
}

fn public_name(declaration: &Declaration) -> &str {
    match declaration {
        Declaration::Function(v) => &v.public_name,
        Declaration::Interface(v) => &v.public_name,
        Declaration::Class(v) | Declaration::Enum(v) => &v.public_name,
        Declaration::TypeAlias(v) => &v.public_name,
        Declaration::Namespace(v) => &v.public_name,
    }
}
fn primitive(name: &str) -> Option<PrimitiveDomain> {
    Some(match name {
        "boolean" => PrimitiveDomain::Boolean,
        "string" => PrimitiveDomain::String,
        "number" => PrimitiveDomain::Float64,
        "bigint" => PrimitiveDomain::BigInteger,
        "never" => PrimitiveDomain::Never,
        "void" => PrimitiveDomain::Unit,
        "unknown" => PrimitiveDomain::Unknown,
        "object" => PrimitiveDomain::Object,
        "null" => PrimitiveDomain::Null,
        "undefined" => PrimitiveDomain::Undefined,
        "any" => PrimitiveDomain::Dynamic,
        _ => return None,
    })
}
