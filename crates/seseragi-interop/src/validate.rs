use crate::*;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub subject: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ValidationError(pub Vec<ValidationIssue>);

impl ValidationError {
    pub(crate) fn serialization(error: serde_json::Error) -> Self {
        Self(vec![ValidationIssue {
            subject: "serialization".to_owned(),
            reason: error.to_string(),
        }])
    }
}
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for issue in &self.0 {
            writeln!(f, "{}: {}", issue.subject, issue.reason)?;
        }
        Ok(())
    }
}
impl std::error::Error for ValidationError {}

pub(crate) fn validate(graph: &SemanticGraph) -> Result<(), ValidationError> {
    let mut issues = Vec::new();
    let mut issue = |subject: String, reason: &str| {
        issues.push(ValidationIssue {
            subject,
            reason: reason.to_owned(),
        })
    };
    if graph.schema != SEMANTIC_SCHEMA || graph.kind != SEMANTIC_KIND {
        issue("graph".to_owned(), "unsupported semantic graph schema/kind");
    }
    for (id, node) in &graph.types {
        let subject = format!("type:{}", id.0);
        for reference in references(node) {
            if !graph.types.contains_key(&reference) {
                issue(subject.clone(), "dangling type reference");
            }
        }
        match node {
            SemanticType::Namespace { members } => {
                if members
                    .iter()
                    .any(|(name, target)| name.is_empty() || !graph.symbols.contains_key(target))
                {
                    issue(subject, "namespace contains an invalid member identity");
                }
            }
            SemanticType::Overload { signatures } => {
                if signatures.is_empty() || signatures.iter().any(|id| !callable(graph, *id)) {
                    issue(subject, "overload set requires callable signatures");
                }
            }
            SemanticType::Reference { symbol } if !graph.symbols.contains_key(symbol) => {
                issue(subject, "missing referenced symbol")
            }
            SemanticType::Parameter { binder, index } => match graph.types.get(binder) {
                Some(SemanticType::Generic { parameters, .. })
                    if (*index as usize) < parameters.len() => {}
                _ => issue(subject, "parameter has no matching generic binder"),
            },
            SemanticType::Generic { parameters, .. } => {
                let names = parameters
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<BTreeSet<_>>();
                if parameters.is_empty() || names.len() != parameters.len() || names.contains("") {
                    issue(
                        subject,
                        "generic parameter names must be nonempty and unique",
                    );
                }
            }
            SemanticType::Shape {
                identity: ShapeIdentity::ReadonlySnapshot,
                fields,
            } if fields.values().any(|field| !field.readonly) => {
                issue(subject, "readonly snapshot contains a mutable field")
            }
            SemanticType::Opaque { identity, .. } if identity.is_empty() => {
                issue(subject, "opaque identity is empty")
            }
            SemanticType::Unresolved { reason } if reason.trim().is_empty() => {
                issue(subject, "unresolved meaning requires a reason")
            }
            SemanticType::Projection { relation, key, .. }
                if matches!(relation, ProjectionRelation::Keys) != key.is_none() =>
            {
                issue(subject, "projection key does not match its relation")
            }
            SemanticType::ShapeTransform { mapper, .. } if !matches!(graph.types.get(mapper), Some(SemanticType::Generic { parameters, .. }) if parameters.len() == 1) => {
                issue(
                    subject,
                    "structural transform requires a single lexical key binder",
                )
            }
            SemanticType::Union { members } | SemanticType::Intersection { members }
                if members.is_empty() =>
            {
                issue(subject, "empty domain composition")
            }
            SemanticType::Callable { parameters, .. } => {
                let mut optional = false;
                for (index, parameter) in parameters.iter().enumerate() {
                    if parameter.name.is_empty()
                        || parameter
                            .origins
                            .iter()
                            .any(|origin| !valid_origin(graph, origin))
                    {
                        issue(
                            subject.clone(),
                            "invalid callable parameter name/provenance",
                        );
                    }
                    if parameter.rest && index + 1 != parameters.len() {
                        issue(subject.clone(), "rest parameter must be last");
                    }
                    if optional && !parameter.optional && !parameter.rest {
                        issue(
                            subject.clone(),
                            "required parameter follows an optional parameter",
                        );
                    }
                    optional |= parameter.optional;
                }
            }
            _ => {}
        }
    }
    for (id, symbol) in &graph.symbols {
        if id.is_empty() || symbol.public_name.is_empty() || symbol.declaration_name.is_empty() {
            issue(id.clone(), "symbol identity/name is empty");
        }
        if !graph.types.contains_key(&symbol.type_ref) {
            issue(id.clone(), "missing symbol type");
        }
        if symbol.kind == SymbolKind::Callable && !callable_set(graph, symbol.type_ref) {
            issue(id.clone(), "callable symbol has no callable signature");
        }
        if symbol.kind == SymbolKind::Namespace
            && !matches!(
                graph.types.get(&symbol.type_ref),
                Some(SemanticType::Namespace { .. })
            )
        {
            issue(id.clone(), "namespace symbol has no member ownership table");
        }
        for origin in &symbol.origins {
            match graph.sources.get(&origin.source) {
                Some(source)
                    if origin.start <= origin.end
                        && origin.end <= source.byte_length
                        && !origin.rule.is_empty() => {}
                _ => issue(id.clone(), "invalid source provenance"),
            }
        }
        if let Some(runtime) = &symbol.runtime {
            if runtime.exact_identity.is_empty()
                || runtime.export_path.iter().any(String::is_empty)
                || runtime.targets.iter().any(String::is_empty)
            {
                issue(id.clone(), "incomplete runtime identity");
            }
        }
    }
    // Ownership is a finite tree. Type references may be recursive, while
    // namespace containment must never cause recursive emission or aliasing.
    let mut indegree = graph
        .symbols
        .keys()
        .map(|key| (key.clone(), 0usize))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut edges = std::collections::BTreeMap::new();
    for (key, symbol) in &graph.symbols {
        if let Some(SemanticType::Namespace { members }) = graph.types.get(&symbol.type_ref) {
            for (name, target) in members {
                if let Some(child) = graph.symbols.get(target) {
                    if child.public_name != *name {
                        issue(
                            key.clone(),
                            "namespace member name differs from exported symbol",
                        );
                    }
                }
                if let Some(degree) = indegree.get_mut(target) {
                    *degree += 1;
                }
            }
            edges.insert(key.clone(), members.values().cloned().collect::<Vec<_>>());
        }
    }
    if indegree.values().any(|degree| *degree > 1) {
        issue(
            "ownership".to_owned(),
            "symbol has multiple namespace owners",
        );
    }
    let mut pending = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let mut count = 0;
    while let Some(key) = pending.pop() {
        count += 1;
        for target in edges.get(&key).into_iter().flatten() {
            if let Some(degree) = indegree.get_mut(target) {
                *degree -= 1;
                if *degree == 0 {
                    pending.push(target.clone());
                }
            }
        }
    }
    if count != graph.symbols.len() {
        issue("ownership".to_owned(), "cyclic namespace ownership");
    }
    for (id, source) in &graph.sources {
        if id.is_empty()
            || source.digest.len() != 64
            || !source.digest.bytes().all(|b| b.is_ascii_hexdigit())
        {
            issue(id.clone(), "source requires an identity and SHA-256 digest");
        }
    }
    for (id, origins) in &graph.type_origins {
        if !graph.types.contains_key(id) {
            issue(
                format!("type:{}", id.0),
                "provenance refers to a missing type",
            );
        }
        for origin in origins {
            if !valid_origin(graph, origin) {
                issue(format!("type:{}", id.0), "invalid type provenance");
            }
        }
    }
    for step in &graph.normalizations {
        if !graph.types.contains_key(&step.output)
            || step.inputs.iter().any(|id| !graph.types.contains_key(id))
            || step.rule.trim().is_empty()
            || step.reason.trim().is_empty()
            || step
                .policy
                .as_ref()
                .is_some_and(|policy| policy.name.is_empty() || policy.version.is_empty())
        {
            issue(
                format!("type:{}", step.output.0),
                "invalid interpretation step",
            );
        }
    }
    if let Some(identity) = &graph.identity {
        let sources = graph
            .sources
            .iter()
            .map(|(name, source)| (name.clone(), source.digest.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        if identity.frontend.name.is_empty()
            || identity.frontend.version.is_empty()
            || !digest(&identity.settings_digest)
            || identity.source_digests != sources
            || identity
                .runtime_contract_digest
                .as_ref()
                .is_some_and(|value| !digest(value))
        {
            issue("artifact".to_owned(), "invalid producer/input identity");
        }
    }
    // Validate lexical generic ownership from every exported root. Symbol
    // references start a new declaration scope; recursive symbols are legal.
    for (id, symbol) in &graph.symbols {
        scope(
            graph,
            symbol.type_ref,
            &BTreeSet::new(),
            &mut BTreeSet::new(),
            &mut issues,
            id,
        );
    }
    issues.sort_by(|a, b| (&a.subject, &a.reason).cmp(&(&b.subject, &b.reason)));
    issues.dedup();
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ValidationError(issues))
    }
}

fn callable(graph: &SemanticGraph, mut id: TypeId) -> bool {
    let mut seen = BTreeSet::new();
    while seen.insert(id) {
        match graph.types.get(&id) {
            Some(SemanticType::Generic { body, .. }) => id = *body,
            Some(SemanticType::Callable { .. }) => return true,
            _ => return false,
        }
    }
    false
}
fn callable_set(graph: &SemanticGraph, id: TypeId) -> bool {
    match graph.types.get(&id) {
        Some(SemanticType::Overload { signatures }) => {
            !signatures.is_empty() && signatures.iter().all(|id| callable(graph, *id))
        }
        _ => callable(graph, id),
    }
}

fn valid_origin(graph: &SemanticGraph, origin: &Origin) -> bool {
    graph.sources.get(&origin.source).is_some_and(|source| {
        origin.start <= origin.end && origin.end <= source.byte_length && !origin.rule.is_empty()
    })
}

fn digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn scope(
    graph: &SemanticGraph,
    id: TypeId,
    binders: &BTreeSet<TypeId>,
    visited: &mut BTreeSet<(TypeId, Vec<TypeId>)>,
    issues: &mut Vec<ValidationIssue>,
    symbol: &str,
) {
    let mut pending = vec![(id, binders.clone())];
    while let Some((id, binders)) = pending.pop() {
        if !visited.insert((id, binders.iter().copied().collect())) {
            continue;
        }
        let Some(node) = graph.types.get(&id) else {
            continue;
        };
        if let SemanticType::Parameter { binder, .. } = node {
            if !binders.contains(binder) {
                issues.push(ValidationIssue {
                    subject: symbol.to_owned(),
                    reason: "generic parameter escapes its lexical binder".to_owned(),
                });
            }
            continue;
        }
        if let SemanticType::Reference { symbol: target } = node {
            if let Some(target) = graph.symbols.get(target) {
                pending.push((target.type_ref, BTreeSet::new()));
            }
            continue;
        }
        let mut local = binders;
        if matches!(node, SemanticType::Generic { .. }) {
            local.insert(id);
        }
        for next in references(node) {
            pending.push((next, local.clone()));
        }
    }
}

fn references(node: &SemanticType) -> Vec<TypeId> {
    match node {
        SemanticType::Generic { parameters, body } => parameters
            .iter()
            .flat_map(|p| p.constraint.into_iter().chain(p.default))
            .chain([*body])
            .collect(),
        SemanticType::Apply {
            constructor,
            arguments,
        } => std::iter::once(*constructor)
            .chain(arguments.iter().copied())
            .collect(),
        SemanticType::Shape { fields, .. } => fields.values().map(|f| f.type_ref).collect(),
        SemanticType::Opaque { arguments, .. } => arguments.clone(),
        SemanticType::Tuple { elements } => elements.clone(),
        SemanticType::Overload { signatures } => signatures.clone(),
        SemanticType::Union { members } | SemanticType::Intersection { members } => members.clone(),
        SemanticType::Collection { element, .. } => vec![*element],
        SemanticType::Nullable { value }
        | SemanticType::Optional { value }
        | SemanticType::Async { value } => vec![*value],
        SemanticType::Callable {
            parameters,
            result,
            receiver,
        } => parameters
            .iter()
            .map(|p| p.type_ref)
            .chain([*result])
            .chain(*receiver)
            .collect(),
        SemanticType::Projection { subject, key, .. } => {
            std::iter::once(*subject).chain(*key).collect()
        }
        SemanticType::ShapeTransform { domain, mapper, .. } => vec![*domain, *mapper],
        _ => vec![],
    }
}
