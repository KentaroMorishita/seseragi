use crate::typed::semantic_types::{SemanticTypeCatalog, SemanticTypeKey};
use crate::SymbolId;
use std::collections::BTreeSet;

use super::pattern::{CoveragePattern, LiteralPattern};

const MAX_WITNESSES: usize = 4096;

#[derive(Clone, Debug)]
struct Witness {
    pattern: CoveragePattern,
    label: String,
}

pub(super) struct CoverageResult {
    pub(super) unreachable: Vec<usize>,
    pub(super) missing: Vec<String>,
}

pub(super) fn analyze_coverage(
    key: &SemanticTypeKey,
    catalog: &SemanticTypeCatalog,
    arms: &[(CoveragePattern, bool)],
) -> Option<CoverageResult> {
    let hints = arms.iter().map(|(pattern, _)| pattern).collect::<Vec<_>>();
    let witnesses = witnesses(key, catalog, &mut BTreeSet::new(), &hints)?;
    Some(analyze_witnesses(&witnesses, arms))
}

fn analyze_witnesses(witnesses: &[Witness], arms: &[(CoveragePattern, bool)]) -> CoverageResult {
    let mut covered = Vec::<CoveragePattern>::new();
    let mut unreachable = Vec::new();
    for (index, (pattern, guarded)) in arms.iter().enumerate() {
        let useful = witnesses.iter().any(|witness| {
            pattern_matches(pattern, &witness.pattern)
                && !covered
                    .iter()
                    .any(|previous| pattern_matches(previous, &witness.pattern))
        });
        if !useful {
            unreachable.push(index);
        }
        if !guarded && useful {
            covered.push(pattern.clone());
        }
    }
    let missing = witnesses
        .iter()
        .filter(|witness| {
            !covered
                .iter()
                .any(|pattern| pattern_matches(pattern, &witness.pattern))
        })
        .map(|witness| witness.label.clone())
        .collect();
    CoverageResult {
        unreachable,
        missing,
    }
}

fn scalar_witnesses(patterns: &[&CoveragePattern]) -> Vec<Witness> {
    let literals = patterns
        .iter()
        .filter_map(|pattern| match pattern {
            CoveragePattern::Literal(literal) => Some(literal.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if literals
        .iter()
        .all(|literal| matches!(literal, LiteralPattern::Boolean(_)))
        && !literals.is_empty()
    {
        return [false, true]
            .into_iter()
            .map(|value| Witness {
                pattern: CoveragePattern::Literal(LiteralPattern::Boolean(value)),
                label: if value { "True" } else { "False" }.to_owned(),
            })
            .collect();
    }
    literals
        .into_iter()
        .map(|literal| Witness {
            label: literal_label(&literal),
            pattern: CoveragePattern::Literal(literal),
        })
        .chain(std::iter::once(Witness {
            pattern: CoveragePattern::Any,
            label: "_".to_owned(),
        }))
        .collect()
}

fn literal_label(literal: &LiteralPattern) -> String {
    match literal {
        LiteralPattern::Integer(value) => value.clone(),
        LiteralPattern::String(value) => format!("{value:?}"),
        LiteralPattern::Boolean(value) => if *value { "True" } else { "False" }.to_owned(),
    }
}

fn witnesses(
    key: &SemanticTypeKey,
    catalog: &SemanticTypeCatalog,
    visiting: &mut BTreeSet<SymbolId>,
    hints: &[&CoveragePattern],
) -> Option<Vec<Witness>> {
    match key {
        SemanticTypeKey::Invalid => None,
        SemanticTypeKey::Other => Some(
            if hints
                .iter()
                .any(|hint| matches!(hint, CoveragePattern::Record(_)))
            {
                record_witnesses(hints)?
            } else {
                scalar_witnesses(hints)
            },
        ),
        SemanticTypeKey::TypeParameter(_)
        | SemanticTypeKey::SchemeParameter(_)
        | SemanticTypeKey::ExternalNominal { .. } => Some(vec![Witness {
            pattern: CoveragePattern::Any,
            label: "_".to_owned(),
        }]),
        SemanticTypeKey::Adt { owner, arguments } => {
            let adt = catalog.adt(*owner)?;
            if !visiting.insert(*owner) {
                return None;
            }
            let mut result = Vec::new();
            for variant in &adt.variants {
                if let Some(payload) = &variant.payload {
                    let payload = catalog.instantiate_payload(*owner, arguments, payload);
                    let payload_hints = hints
                        .iter()
                        .filter_map(|hint| match hint {
                            CoveragePattern::Constructor {
                                constructor,
                                argument: Some(argument),
                            } if *constructor == variant.constructor => Some(argument.as_ref()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    for child in witnesses(&payload.key, catalog, visiting, &payload_hints)? {
                        result.push(Witness {
                            pattern: CoveragePattern::Constructor {
                                constructor: variant.constructor,
                                argument: Some(Box::new(child.pattern)),
                            },
                            label: format!("{} {}", variant.spelling, child.label),
                        });
                        if result.len() > MAX_WITNESSES {
                            visiting.remove(owner);
                            return None;
                        }
                    }
                } else {
                    result.push(Witness {
                        pattern: CoveragePattern::Constructor {
                            constructor: variant.constructor,
                            argument: None,
                        },
                        label: variant.spelling.clone(),
                    });
                }
            }
            visiting.remove(owner);
            Some(result)
        }
        SemanticTypeKey::Tuple(elements) => {
            let mut products = vec![(Vec::<CoveragePattern>::new(), Vec::<String>::new())];
            for (index, element) in elements.iter().enumerate() {
                let element_hints = hints
                    .iter()
                    .filter_map(|hint| match hint {
                        CoveragePattern::Tuple(elements) => elements.get(index),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                let children = witnesses(element, catalog, visiting, &element_hints)?;
                let mut next = Vec::new();
                for (patterns, labels) in &products {
                    for child in &children {
                        let mut product_patterns = patterns.clone();
                        product_patterns.push(child.pattern.clone());
                        let mut product_labels = labels.clone();
                        product_labels.push(child.label.clone());
                        next.push((product_patterns, product_labels));
                        if next.len() > MAX_WITNESSES {
                            return None;
                        }
                    }
                }
                products = next;
            }
            Some(
                products
                    .into_iter()
                    .map(|(patterns, labels)| Witness {
                        pattern: CoveragePattern::Tuple(patterns),
                        label: format!("({})", labels.join(", ")),
                    })
                    .collect(),
            )
        }
    }
}

fn pattern_matches(pattern: &CoveragePattern, witness: &CoveragePattern) -> bool {
    match (pattern, witness) {
        (CoveragePattern::Any, _) => true,
        (CoveragePattern::Literal(pattern), CoveragePattern::Literal(witness)) => {
            pattern == witness
        }
        (
            CoveragePattern::Constructor {
                constructor: pattern_constructor,
                argument: pattern_argument,
            },
            CoveragePattern::Constructor {
                constructor: witness_constructor,
                argument: witness_argument,
            },
        ) if pattern_constructor == witness_constructor => {
            match (pattern_argument, witness_argument) {
                (None, None) => true,
                (Some(pattern), Some(witness)) => pattern_matches(pattern, witness),
                _ => false,
            }
        }
        (CoveragePattern::Tuple(patterns), CoveragePattern::Tuple(witnesses))
            if patterns.len() == witnesses.len() =>
        {
            patterns
                .iter()
                .zip(witnesses)
                .all(|(pattern, witness)| pattern_matches(pattern, witness))
        }
        (CoveragePattern::Record(patterns), CoveragePattern::Record(witnesses)) => {
            patterns.iter().all(|(name, pattern)| {
                witnesses
                    .iter()
                    .find(|(witness_name, _)| witness_name == name)
                    .is_some_and(|(_, witness)| pattern_matches(pattern, witness))
            })
        }
        _ => false,
    }
}

fn record_witnesses(hints: &[&CoveragePattern]) -> Option<Vec<Witness>> {
    let field_names = hints
        .iter()
        .filter_map(|hint| match hint {
            CoveragePattern::Record(fields) => Some(fields),
            _ => None,
        })
        .flat_map(|fields| fields.iter().map(|(name, _)| name.clone()))
        .collect::<BTreeSet<_>>();
    let mut products = vec![(
        Vec::<(String, CoveragePattern)>::new(),
        Vec::<String>::new(),
    )];
    for name in field_names {
        let field_hints = hints
            .iter()
            .filter_map(|hint| match hint {
                CoveragePattern::Record(fields) => fields
                    .iter()
                    .find(|(field_name, _)| field_name == &name)
                    .map(|(_, pattern)| pattern),
                _ => None,
            })
            .collect::<Vec<_>>();
        let children = hinted_witnesses(&field_hints)?;
        let mut next = Vec::new();
        for (patterns, labels) in &products {
            for child in &children {
                let mut product_patterns = patterns.clone();
                product_patterns.push((name.clone(), child.pattern.clone()));
                let mut product_labels = labels.clone();
                product_labels.push(format!("{name}: {}", child.label));
                next.push((product_patterns, product_labels));
                if next.len() > MAX_WITNESSES {
                    return None;
                }
            }
        }
        products = next;
    }
    Some(
        products
            .into_iter()
            .map(|(fields, labels)| Witness {
                pattern: CoveragePattern::Record(fields),
                label: format!("{{ {} }}", labels.join(", ")),
            })
            .collect(),
    )
}

fn hinted_witnesses(hints: &[&CoveragePattern]) -> Option<Vec<Witness>> {
    if hints
        .iter()
        .any(|hint| matches!(hint, CoveragePattern::Record(_)))
    {
        record_witnesses(hints)
    } else {
        Some(scalar_witnesses(hints))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matches_a_tuple_witness() {
        assert!(pattern_matches(
            &CoveragePattern::Any,
            &CoveragePattern::Tuple(vec![CoveragePattern::Any])
        ));
    }
}
