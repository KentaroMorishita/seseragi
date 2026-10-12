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
    builder.index(scope, "", false);
    builder.scope(scope, "");
    builder.graph.validate().map_err(SemanticError::Invalid)?;
    builder
        .graph
        .normalize_closed(NormalizationBudget::default())
        .map_err(SemanticError::Invalid)?;
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
    fn index(&mut self, scope: &Scope, namespace: &str, internal: bool) {
        for (declaration, private) in scope
            .declarations
            .iter()
            .map(|d| (d, internal))
            .chain(scope.private_declarations.iter().map(|d| (d, true)))
        {
            let public = public_name(declaration);
            if private {
                self.graph.private_symbols.insert(path(namespace, public));
            }
            self.names
                .entry(path(namespace, declaration.original_name()))
                .or_insert_with(|| path(namespace, public));
            self.names
                .insert(path(namespace, public), path(namespace, public));
            if let Declaration::Namespace(value) = declaration {
                self.index(&value.scope, &path(namespace, public), private);
            }
        }
    }
    fn scope(&mut self, scope: &Scope, namespace: &str) -> BTreeMap<String, String> {
        let mut members = BTreeMap::new();
        for declaration in scope.declarations.iter().chain(&scope.private_declarations) {
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
                    let receiver = value
                        .parameters
                        .iter()
                        .find(|p| p.name == "this")
                        .map(|p| self.ty(&p.type_ref, namespace, &parameters));
                    let arguments = value
                        .parameters
                        .iter()
                        .filter(|p| p.name != "this")
                        .map(|p| self.parameter(p, namespace, &parameters))
                        .collect();
                    let result = self.ty(&value.result, namespace, &parameters);
                    self.node(
                        SemanticType::Callable {
                            parameters: arguments,
                            result,
                            receiver,
                        },
                        span,
                        "callable-signature",
                    )
                }
                Declaration::TypeAlias(value) => self.ty(&value.type_ref, namespace, &parameters),
                Declaration::Interface(value) => self.ty(&value.body, namespace, &parameters),
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
            self.graph
                .type_origins
                .entry(root)
                .or_default()
                .push(origins[0].clone());
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
                    let old = self.graph.types[&previous.type_ref].clone();
                    let combined = merge_shapes(&self.graph, &old, &self.graph.types[&root]);
                    let normalized = combined.is_some();
                    let inputs = if let Some(combined) = combined {
                        let snapshot = self.reserve();
                        self.graph.types.insert(snapshot, old);
                        self.graph.types.insert(previous.type_ref, combined);
                        vec![snapshot, root]
                    } else if matches!(old, SemanticType::Shape { .. })
                        && matches!(self.graph.types[&root], SemanticType::Shape { .. })
                    {
                        let snapshot = self.reserve();
                        self.graph.types.insert(snapshot, old);
                        self.graph.types.insert(previous.type_ref, SemanticType::Unresolved { reason: "merged property types or modifiers conflict; no source-order winner was selected".to_owned() });
                        vec![snapshot, root]
                    } else {
                        vec![root]
                    };
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
                        inputs,
                        rule: "declaration-merge".to_owned(),
                        classification: if normalized { SemanticClassification::Normalized } else { SemanticClassification::PolicyRequired },
                        reason: if normalized { "compatible structural declarations merge all fields and preserve readonly/mutable identity" } else { "declaration identity and all source origins are retained; conflicting or generic merged meaning requires explicit policy" }.to_owned(),
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
            if !self.graph.private_symbols.contains(&key) {
                members.insert(public.to_owned(), key);
            }
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
            TypeKind::Intersection(members) => SemanticType::Intersection {
                members: members
                    .iter()
                    .map(|ty| self.ty(ty, namespace, parameters))
                    .collect(),
            },
            TypeKind::Keys(subject) => SemanticType::Projection {
                relation: ProjectionRelation::Keys,
                subject: self.ty(subject, namespace, parameters),
                key: None,
            },
            TypeKind::Member { subject, key } => SemanticType::Projection {
                relation: ProjectionRelation::Member,
                subject: self.ty(subject, namespace, parameters),
                key: Some(self.ty(key, namespace, parameters)),
            },
            TypeKind::Shape(properties) => {
                let mut fields = BTreeMap::new();
                for property in properties {
                    if fields.contains_key(&property.name) {
                        let id = self.node(SemanticType::Unresolved { reason: "duplicate structural property requires declaration compatibility checking".to_owned() }, value.span, "structural-conflict");
                        self.decision(
                            id,
                            SemanticClassification::PolicyRequired,
                            "duplicate property declarations cannot be selected by source order",
                        );
                        return id;
                    }
                    fields.insert(
                        property.name.clone(),
                        ShapeField {
                            type_ref: self.ty(&property.type_ref, namespace, parameters),
                            optional: property.optional,
                            readonly: property.readonly,
                        },
                    );
                }
                SemanticType::Shape {
                    identity: if properties.iter().all(|p| p.readonly) {
                        ShapeIdentity::ReadonlySnapshot
                    } else {
                        ShapeIdentity::MutableIdentity
                    },
                    fields,
                }
            }
            TypeKind::Mapped {
                parameter,
                domain,
                value: body,
                readonly,
                optional,
            } => {
                let domain = self.ty(domain, namespace, parameters);
                let binder = self.reserve();
                let mut local = parameters.clone();
                local.insert(parameter.clone(), (binder, 0));
                let body = self.ty(body, namespace, &local);
                self.graph.types.insert(
                    binder,
                    SemanticType::Generic {
                        parameters: vec![GenericParameter {
                            name: parameter.clone(),
                            constraint: None,
                            default: None,
                        }],
                        body,
                    },
                );
                self.graph.type_origins.insert(
                    binder,
                    vec![self.origin(value.span, "structural-key-binder")],
                );
                SemanticType::ShapeTransform {
                    domain,
                    mapper: binder,
                    readonly: *readonly,
                    optional: *optional,
                }
            }
            TypeKind::Template(parts) => {
                let mut texts = vec![String::new()];
                let mut inputs = Vec::new();
                for part in parts {
                    let fragments = match part {
                        crate::model::TemplatePart::Text(text) => vec![text.clone()],
                        crate::model::TemplatePart::Domain(ty) => {
                            let id = self.ty(ty, namespace, parameters);
                            inputs.push(id);
                            match finite_literal_texts(&self.graph, id, 256) {
                                Some(texts) => texts,
                                None => {
                                    let id = self.node(
                                        SemanticType::Unresolved {
                                            reason:
                                                "template relation has an open or unresolved domain"
                                                    .to_owned(),
                                        },
                                        value.span,
                                        "template-domain",
                                    );
                                    self.graph.normalizations.push(Normalization { output: id, inputs, rule: "template-domain".to_owned(), classification: SemanticClassification::PolicyRequired, reason: "finite template expansion is not proven; no String fallback was applied".to_owned(), policy: None });
                                    return id;
                                }
                            }
                        }
                    };
                    if texts.len().saturating_mul(fragments.len()) > 256
                        || texts.iter().any(|text| {
                            fragments
                                .iter()
                                .any(|part| text.len().saturating_add(part.len()) > 16_384)
                        })
                    {
                        let id = self.node(
                            SemanticType::Unresolved {
                                reason: "finite template domain exceeds the expansion budget"
                                    .to_owned(),
                            },
                            value.span,
                            "template-budget",
                        );
                        self.graph.normalizations.push(Normalization { output: id, inputs, rule: "template-budget".to_owned(), classification: SemanticClassification::PolicyRequired, reason: "template expansion is bounded to 256 members and 16384 bytes per member".to_owned(), policy: None });
                        return id;
                    }
                    texts = texts
                        .iter()
                        .flat_map(|prefix| {
                            fragments
                                .iter()
                                .map(move |suffix| format!("{prefix}{suffix}"))
                        })
                        .collect();
                }
                texts.sort();
                texts.dedup();
                let members = texts
                    .into_iter()
                    .map(|text| {
                        self.node(
                            SemanticType::Literal {
                                value: LiteralValue::String(text),
                            },
                            value.span,
                            "finite-template-member",
                        )
                    })
                    .collect::<Vec<_>>();
                let node = match members.as_slice() {
                    [] => SemanticType::Primitive {
                        domain: PrimitiveDomain::Never,
                    },
                    [id] => self.graph.types[id].clone(),
                    _ => SemanticType::Union { members },
                };
                let id = self.node(node, value.span, "finite-template-domain");
                self.graph.normalizations.push(Normalization {
                    output: id,
                    inputs,
                    rule: "finite-template-domain".to_owned(),
                    classification: SemanticClassification::Normalized,
                    reason: "closed literal domains produce a bounded exact template domain"
                        .to_owned(),
                    policy: None,
                });
                return id;
            }
            TypeKind::Infer { name, .. } => SemanticType::Unresolved {
                reason: format!(
                    "inferred parameter {name} is outside a supported extraction relation"
                ),
            },
            TypeKind::Conditional {
                subject,
                constraint,
                matched,
                unmatched,
            } => {
                let subject_id = self.ty(subject, namespace, parameters);
                let mut inferred = Vec::new();
                collect_inferred(constraint, &mut inferred);
                if !inferred.is_empty()
                    && matches!(&unmatched.kind, TypeKind::Primitive(name) if name == "never")
                    && matches!(&matched.kind, TypeKind::Named(name) if inferred.len() == 1 && name == &inferred[0].0)
                {
                    let binder = self.reserve();
                    let mut local = parameters.clone();
                    for (index, (name, _)) in inferred.iter().enumerate() {
                        local.insert(name.clone(), (binder, index as u32));
                    }
                    let pattern = self.inference_pattern(constraint, namespace, &local);
                    let generic_parameters = inferred
                        .iter()
                        .map(|(name, constraint)| GenericParameter {
                            name: name.clone(),
                            constraint: constraint
                                .as_ref()
                                .map(|ty| self.ty(ty, namespace, &local)),
                            default: None,
                        })
                        .collect();
                    self.graph.types.insert(
                        binder,
                        SemanticType::Generic {
                            parameters: generic_parameters,
                            body: pattern,
                        },
                    );
                    self.graph
                        .type_origins
                        .insert(binder, vec![self.origin(value.span, "extraction-binder")]);
                    SemanticType::Projection {
                        relation: ProjectionRelation::Extract,
                        subject: subject_id,
                        key: Some(binder),
                    }
                } else {
                    let constraint_id = self.ty(constraint, namespace, parameters);
                    let matched_id = self.ty(matched, namespace, parameters);
                    let unmatched_id = self.ty(unmatched, namespace, parameters);
                    if let Some(accepted) =
                        closed_assignability(&self.graph, subject_id, constraint_id)
                    {
                        let selected = if accepted { matched_id } else { unmatched_id };
                        let id = self.node(
                            self.graph.types[&selected].clone(),
                            value.span,
                            "closed-domain-selection",
                        );
                        self.graph.normalizations.push(Normalization { output: id, inputs: vec![subject_id, constraint_id, matched_id, unmatched_id], rule: "closed-domain-selection".to_owned(), classification: SemanticClassification::Normalized, reason: "closed literal/primitive domain proves the selected result without checker heuristics".to_owned(), policy: None });
                        return id;
                    }
                    let id = self.node(SemanticType::Unresolved { reason: if inferred.is_empty() { "open conditional domain requires explicit semantic boundary policy" } else { "higher-rank or multi-result inference requires explicit extraction policy" }.to_owned() }, value.span, "open-domain-selection");
                    self.graph.normalizations.push(Normalization { output: id, inputs: vec![subject_id, constraint_id, matched_id, unmatched_id], rule: "open-domain-selection".to_owned(), classification: SemanticClassification::PolicyRequired, reason: "domain selection cannot be proven from closed semantic evidence; operand relations remain available".to_owned(), policy: None });
                    return id;
                }
            }
            TypeKind::Function {
                generics,
                parameters: args,
                result,
            } => {
                if !generics.is_empty() {
                    let binder = self.reserve();
                    let mut local = parameters.clone();
                    for (index, parameter) in generics.iter().enumerate() {
                        local.insert(parameter.name.clone(), (binder, index as u32));
                    }
                    let callable = TypeRef {
                        span: value.span,
                        kind: TypeKind::Function {
                            generics: vec![],
                            parameters: args.clone(),
                            result: result.clone(),
                        },
                    };
                    let body = self.ty(&callable, namespace, &local);
                    let generic_parameters = generics
                        .iter()
                        .map(|p| GenericParameter {
                            name: p.name.clone(),
                            constraint: p
                                .constraint
                                .as_ref()
                                .map(|ty| self.ty(ty, namespace, &local)),
                            default: p.default.as_ref().map(|ty| self.ty(ty, namespace, &local)),
                        })
                        .collect();
                    self.graph.types.insert(
                        binder,
                        SemanticType::Generic {
                            parameters: generic_parameters,
                            body,
                        },
                    );
                    self.graph.type_origins.insert(
                        binder,
                        vec![self.origin(value.span, "callback-generic-binder")],
                    );
                    self.decision(binder, SemanticClassification::PolicyRequired, "generic callback quantification is preserved; projection requires an explicit rank and ABI policy");
                    return binder;
                }
                let receiver = args
                    .iter()
                    .find(|p| p.name == "this")
                    .map(|p| self.ty(&p.type_ref, namespace, parameters));
                let args = args
                    .iter()
                    .filter(|p| p.name != "this")
                    .map(|p| self.parameter(p, namespace, parameters))
                    .collect();
                let result = self.ty(result, namespace, parameters);
                SemanticType::Callable {
                    parameters: args,
                    result,
                    receiver,
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
                } else if let Some(text) = literal_string(raw) {
                    SemanticType::Literal {
                        value: LiteralValue::String(text),
                    }
                } else {
                    SemanticType::Unresolved {
                        reason: "literal domain requires frontend normalization".to_owned(),
                    }
                }
            }
            TypeKind::Unsupported(reason) => SemanticType::Unresolved {
                reason: format!("type expression requires semantic interpretation: {reason}"),
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
            Some(SemanticType::Projection { .. } | SemanticType::ShapeTransform { .. }) => self.decision(id, SemanticClassification::PolicyRequired, "relation is preserved without widening; closed normalization or explicit boundary policy is required"),
            _ => {}
        }
        id
    }
    fn inference_pattern(
        &mut self,
        value: &TypeRef,
        namespace: &str,
        parameters: &BTreeMap<String, (TypeId, u32)>,
    ) -> TypeId {
        let converted = replace_inferred(value);
        self.ty(&converted, namespace, parameters)
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

fn merge_shapes(graph: &SemanticGraph, a: &SemanticType, b: &SemanticType) -> Option<SemanticType> {
    let (
        SemanticType::Shape {
            identity: ai,
            fields: a,
        },
        SemanticType::Shape {
            identity: bi,
            fields: b,
        },
    ) = (a, b)
    else {
        return None;
    };
    let mut fields = a.clone();
    for (name, field) in b {
        if let Some(previous) = fields.get(name) {
            if previous.optional != field.optional
                || previous.readonly != field.readonly
                || graph.types[&previous.type_ref] != graph.types[&field.type_ref]
            {
                return None;
            }
        }
        fields.insert(name.clone(), field.clone());
    }
    Some(SemanticType::Shape {
        identity: if *ai == ShapeIdentity::ReadonlySnapshot
            && *bi == ShapeIdentity::ReadonlySnapshot
        {
            ShapeIdentity::ReadonlySnapshot
        } else {
            ShapeIdentity::MutableIdentity
        },
        fields,
    })
}

fn literal_string(raw: &str) -> Option<String> {
    if raw.starts_with('"') {
        return serde_json::from_str(raw).ok();
    }
    let inner = raw.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut json = String::from("\"");
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next()? {
                '\'' => json.push('\''),
                escaped => {
                    json.push('\\');
                    json.push(escaped);
                }
            },
            '"' => json.push_str("\\\""),
            '\'' => return None,
            other => json.push(other),
        }
    }
    json.push('"');
    serde_json::from_str(&json).ok()
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

fn collect_inferred(value: &TypeRef, output: &mut Vec<(String, Option<TypeRef>)>) {
    match &value.kind {
        TypeKind::Infer { name, constraint } => {
            output.push((name.clone(), constraint.as_deref().cloned()))
        }
        TypeKind::Generic { arguments, .. }
        | TypeKind::Tuple(arguments)
        | TypeKind::Union(arguments)
        | TypeKind::Intersection(arguments) => {
            for argument in arguments {
                collect_inferred(argument, output);
            }
        }
        TypeKind::ReadonlyArray(element) | TypeKind::MutableArray(element) => {
            collect_inferred(element, output)
        }
        _ => {}
    }
}
fn replace_inferred(value: &TypeRef) -> TypeRef {
    let kind = match &value.kind {
        TypeKind::Infer { name, .. } => TypeKind::Named(name.clone()),
        TypeKind::Generic { name, arguments } => TypeKind::Generic {
            name: name.clone(),
            arguments: arguments.iter().map(replace_inferred).collect(),
        },
        TypeKind::ReadonlyArray(element) => {
            TypeKind::ReadonlyArray(Box::new(replace_inferred(element)))
        }
        TypeKind::MutableArray(element) => {
            TypeKind::MutableArray(Box::new(replace_inferred(element)))
        }
        TypeKind::Tuple(elements) => {
            TypeKind::Tuple(elements.iter().map(replace_inferred).collect())
        }
        _ => value.kind.clone(),
    };
    TypeRef {
        kind,
        span: value.span,
    }
}
fn closed_assignability(
    graph: &SemanticGraph,
    subject: TypeId,
    constraint: TypeId,
) -> Option<bool> {
    match (&graph.types[&subject], &graph.types[&constraint]) {
        (SemanticType::Primitive { domain: a }, SemanticType::Primitive { domain: b })
            if !matches!(
                a,
                PrimitiveDomain::Dynamic | PrimitiveDomain::Unknown | PrimitiveDomain::Object
            ) && !matches!(b, PrimitiveDomain::Dynamic | PrimitiveDomain::Object) =>
        {
            Some(a == b || *b == PrimitiveDomain::Unknown || *a == PrimitiveDomain::Never)
        }
        (SemanticType::Literal { value: a }, SemanticType::Literal { value: b }) => Some(a == b),
        (SemanticType::Literal { value }, SemanticType::Primitive { domain }) => Some(
            *domain == PrimitiveDomain::Unknown
                || matches!(
                    (value, domain),
                    (LiteralValue::String(_), PrimitiveDomain::String)
                        | (LiteralValue::Boolean(_), PrimitiveDomain::Boolean)
                        | (
                            LiteralValue::Integer(_),
                            PrimitiveDomain::Integer | PrimitiveDomain::Float64
                        )
                ),
        ),
        _ => None,
    }
}

fn finite_literal_texts(graph: &SemanticGraph, root: TypeId, limit: usize) -> Option<Vec<String>> {
    let mut pending = vec![root];
    let mut seen = std::collections::BTreeSet::new();
    let mut values = Vec::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if seen.len() > limit * 2 {
            return None;
        }
        match &graph.types[&id] {
            SemanticType::Literal {
                value: LiteralValue::String(text),
            } => values.push(text.clone()),
            SemanticType::Literal {
                value: LiteralValue::Integer(value),
            } => values.push(value.to_string()),
            SemanticType::Literal {
                value: LiteralValue::Boolean(value),
            } => values.push(value.to_string()),
            SemanticType::Union { members } => pending.extend(members),
            SemanticType::Primitive {
                domain: PrimitiveDomain::Never,
            } => {}
            _ => return None,
        }
        if values.len() > limit {
            return None;
        }
    }
    values.sort();
    values.dedup();
    Some(values)
}
