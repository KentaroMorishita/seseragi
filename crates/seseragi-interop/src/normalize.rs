use crate::*;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
pub struct NormalizationBudget {
    pub steps: usize,
    pub domain_members: usize,
}
impl Default for NormalizationBudget {
    fn default() -> Self {
        Self {
            steps: 10_000,
            domain_members: 256,
        }
    }
}

pub(crate) fn normalize(graph: &mut SemanticGraph, mut budget: NormalizationBudget) {
    // Repeated passes allow forward aliases without imposing source ordering.
    // Every successful step removes one projection/composition from its live
    // position. Original relation snapshots stay in provenance, not the queue.
    let mut pending = graph.types.keys().copied().collect::<Vec<_>>();
    loop {
        let mut changed = false;
        let mut unresolved = Vec::new();
        for id in pending {
            if budget.steps == 0 {
                unresolved.push(id);
                continue;
            }
            budget.steps -= 1;
            let original = graph.types[&id].clone();
            let replacement = closed_keys(graph, id, &original, &mut budget)
                .or_else(|| closed(graph, &original, &mut budget));
            if let Some(replacement) = replacement {
                if replacement != original {
                    let snapshot = TypeId(
                        graph
                            .types
                            .keys()
                            .next_back()
                            .map_or(0, |id| id.0.saturating_add(1)),
                    );
                    if graph.types.contains_key(&snapshot) {
                        continue;
                    }
                    graph.types.insert(snapshot, original);
                    graph.types.insert(id, replacement);
                    for step in &mut graph.normalizations {
                        if step.output == id {
                            step.output = snapshot;
                        }
                    }
                    if let Some(origins) = graph.type_origins.get(&id).cloned() {
                        graph.type_origins.insert(snapshot, origins);
                    }
                    graph.normalizations.push(Normalization {
                        output: id, inputs: vec![snapshot], rule: "closed-semantic-relation".to_owned(),
                        classification: SemanticClassification::Normalized,
                        reason: "closed domain or structural evidence resolves this relation without widening".to_owned(), policy: None,
                    });
                    changed = true;
                }
            } else {
                unresolved.push(id);
            }
        }
        if budget.steps == 0 {
            for id in &unresolved {
                if matches!(
                    graph.types[id],
                    SemanticType::Projection { .. }
                        | SemanticType::Apply { .. }
                        | SemanticType::Union { .. }
                        | SemanticType::Intersection { .. }
                        | SemanticType::ShapeTransform { .. }
                ) {
                    graph.normalizations.push(Normalization { output: *id, inputs: vec![], rule: "normalization-budget".to_owned(), classification: SemanticClassification::PolicyRequired, reason: "finite semantic normalization work budget exhausted; relation retained without a fallback".to_owned(), policy: None });
                }
            }
        }
        if !changed || unresolved.is_empty() || budget.steps == 0 {
            break;
        }
        pending = unresolved;
    }
}

fn closed_keys(
    graph: &mut SemanticGraph,
    id: TypeId,
    node: &SemanticType,
    budget: &mut NormalizationBudget,
) -> Option<SemanticType> {
    let SemanticType::Projection {
        relation: ProjectionRelation::Keys,
        subject,
        ..
    } = node
    else {
        return None;
    };
    let (_, SemanticType::Shape { fields, .. }) = resolved(graph, *subject, budget)? else {
        return None;
    };
    if fields.len() > budget.domain_members {
        return None;
    }
    let names = fields.keys().cloned().collect::<Vec<_>>();
    if names.is_empty() {
        return Some(SemanticType::Primitive {
            domain: PrimitiveDomain::Never,
        });
    }
    let next = graph.types.keys().next_back()?.0.checked_add(1)?;
    next.checked_add(names.len() as u32)?;
    let mut members = Vec::new();
    for (offset, name) in names.into_iter().enumerate() {
        let key = TypeId(next + offset as u32);
        graph.types.insert(
            key,
            SemanticType::Literal {
                value: LiteralValue::String(name),
            },
        );
        if let Some(origins) = graph.type_origins.get(&id).cloned() {
            graph.type_origins.insert(key, origins);
        }
        members.push(key);
    }
    if members.len() == 1 {
        Some(graph.types[&members[0]].clone())
    } else {
        Some(SemanticType::Union { members })
    }
}

fn resolved<'a>(
    graph: &'a SemanticGraph,
    mut id: TypeId,
    budget: &mut NormalizationBudget,
) -> Option<(TypeId, &'a SemanticType)> {
    let mut seen = BTreeSet::new();
    loop {
        if budget.steps == 0 || !seen.insert(id) {
            return None;
        }
        budget.steps -= 1;
        match graph.types.get(&id)? {
            SemanticType::Reference { symbol } => id = graph.symbols.get(symbol)?.type_ref,
            node => return Some((id, node)),
        }
    }
}

fn closed(
    graph: &SemanticGraph,
    node: &SemanticType,
    budget: &mut NormalizationBudget,
) -> Option<SemanticType> {
    match node {
        SemanticType::ShapeTransform {
            domain,
            mapper,
            readonly,
            optional,
        } => {
            let (domain_id, domain) = resolved(graph, *domain, budget)?;
            // Homomorphic transforms preserve input optional/readonly flags.
            // Once Keys is reduced to literals, its trace is still authoritative.
            if (readonly.is_none() || optional.is_none())
                && graph.normalizations.iter().any(|step| {
                    step.output == domain_id
                        && step.inputs.iter().any(|id| {
                            matches!(
                                graph.types[id],
                                SemanticType::Projection {
                                    relation: ProjectionRelation::Keys,
                                    ..
                                }
                            )
                        })
                })
            {
                return None;
            }
            let members = match domain {
                SemanticType::Literal { .. } => vec![domain_id],
                SemanticType::Union { members } => members.clone(),
                _ => return None,
            };
            if members.len() > budget.domain_members {
                return None;
            }
            let (binder, SemanticType::Generic { parameters, body }) =
                resolved(graph, *mapper, budget)?
            else {
                return None;
            };
            if parameters.len() != 1
                || parameters[0].constraint.is_some()
                || parameters[0].default.is_some()
            {
                return None;
            }
            let (_, mapped) = resolved(graph, *body, budget)?;
            let mut fields = std::collections::BTreeMap::new();
            for member in members {
                let (
                    key,
                    SemanticType::Literal {
                        value: LiteralValue::String(name),
                    },
                ) = resolved(graph, member, budget)?
                else {
                    return None;
                };
                let value = match mapped {
                    SemanticType::Parameter {
                        binder: b,
                        index: 0,
                    } if *b == binder => key,
                    SemanticType::Primitive { .. } | SemanticType::Literal { .. } => *body,
                    _ => return None,
                };
                fields.insert(
                    name.clone(),
                    ShapeField {
                        type_ref: value,
                        readonly: readonly.unwrap_or(false),
                        optional: optional.unwrap_or(false),
                    },
                );
            }
            Some(SemanticType::Shape {
                identity: if readonly == &Some(true) {
                    ShapeIdentity::ReadonlySnapshot
                } else {
                    ShapeIdentity::MutableIdentity
                },
                fields,
            })
        }
        SemanticType::Projection {
            relation: ProjectionRelation::Member,
            subject,
            key: Some(key),
        } => {
            let (_, SemanticType::Shape { fields, .. }) = resolved(graph, *subject, budget)? else {
                return None;
            };
            let (
                _,
                SemanticType::Literal {
                    value: LiteralValue::String(key),
                },
            ) = resolved(graph, *key, budget)?
            else {
                return None;
            };
            let field = fields.get(key)?;
            if field.optional {
                Some(SemanticType::Optional {
                    value: field.type_ref,
                })
            } else {
                Some(graph.types[&field.type_ref].clone())
            }
        }
        SemanticType::Projection {
            relation: ProjectionRelation::Extract,
            subject,
            key: Some(key),
        } => {
            let (_, actual) = resolved(graph, *subject, budget)?;
            let (binder, SemanticType::Generic { parameters, body }) =
                resolved(graph, *key, budget)?
            else {
                return None;
            };
            if parameters.len() != 1
                || parameters[0].constraint.is_some()
                || parameters[0].default.is_some()
            {
                return None;
            }
            let (_, pattern) = resolved(graph, *body, budget)?;
            let (actual, parameter) = match (actual, pattern) {
                (
                    SemanticType::Collection {
                        element: a,
                        mutable: am,
                    },
                    SemanticType::Collection {
                        element: p,
                        mutable: pm,
                    },
                ) if am == pm => (*a, *p),
                (SemanticType::Async { value: a }, SemanticType::Async { value: p }) => (*a, *p),
                _ => return None,
            };
            if matches!(graph.types[&parameter], SemanticType::Parameter { binder: b, index: 0 } if b == binder)
            {
                Some(graph.types[&actual].clone())
            } else {
                None
            }
        }
        SemanticType::Union { members } if members.len() <= budget.domain_members => {
            let mut flat = Vec::new();
            for member in members {
                let (id, node) = resolved(graph, *member, budget)?;
                match node {
                    SemanticType::Union { members } => flat.extend(members.iter().copied()),
                    SemanticType::Primitive {
                        domain: PrimitiveDomain::Never,
                    } => {}
                    _ => flat.push(id),
                }
                if flat.len() > budget.domain_members {
                    return None;
                }
            }
            flat.sort();
            flat.dedup();
            // Structurally equal closed literals/primitives have distinct IDs.
            let mut unique = Vec::new();
            for id in flat {
                if !unique
                    .iter()
                    .any(|other| graph.types[other] == graph.types[&id])
                {
                    unique.push(id);
                }
            }
            match unique.as_slice() {
                [] => Some(SemanticType::Primitive {
                    domain: PrimitiveDomain::Never,
                }),
                [id] => Some(graph.types[id].clone()),
                _ => Some(SemanticType::Union { members: unique }),
            }
        }
        SemanticType::Intersection { members } if members.len() == 2 => {
            let (_, a) = resolved(graph, members[0], budget)?;
            let (_, b) = resolved(graph, members[1], budget)?;
            if a == b {
                return Some(a.clone());
            }
            match (a, b) {
                (SemanticType::Primitive { domain: a }, SemanticType::Primitive { domain: b })
                    if disjoint(*a, *b) =>
                {
                    Some(SemanticType::Primitive {
                        domain: PrimitiveDomain::Never,
                    })
                }
                (
                    SemanticType::Shape {
                        identity: ShapeIdentity::ReadonlySnapshot,
                        fields: a,
                    },
                    SemanticType::Shape {
                        identity: ShapeIdentity::ReadonlySnapshot,
                        fields: b,
                    },
                ) => {
                    let mut fields = a.clone();
                    for (name, field) in b {
                        if let Some(previous) = fields.get(name) {
                            if previous.optional != field.optional
                                || graph.types[&previous.type_ref] != graph.types[&field.type_ref]
                            {
                                return None;
                            }
                        }
                        fields.insert(name.clone(), field.clone());
                    }
                    Some(SemanticType::Shape {
                        identity: ShapeIdentity::ReadonlySnapshot,
                        fields,
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn disjoint(a: PrimitiveDomain, b: PrimitiveDomain) -> bool {
    use PrimitiveDomain::*;
    matches!(
        a,
        Boolean | String | Float64 | BigInteger | Null | Undefined
    ) && matches!(
        b,
        Boolean | String | Float64 | BigInteger | Null | Undefined
    ) && a != b
}
