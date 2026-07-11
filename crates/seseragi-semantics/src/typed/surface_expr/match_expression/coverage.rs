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
    let witnesses = if key == &SemanticTypeKey::Other {
        scalar_witnesses(arms)
    } else {
        witnesses(key, catalog, &mut BTreeSet::new())?
    };
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

fn scalar_witnesses(arms: &[(CoveragePattern, bool)]) -> Vec<Witness> {
    let literals = arms
        .iter()
        .filter_map(|(pattern, _)| match pattern {
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
) -> Option<Vec<Witness>> {
    match key {
        SemanticTypeKey::Invalid => None,
        SemanticTypeKey::Other | SemanticTypeKey::TypeParameter(_) => Some(vec![Witness {
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
                    for child in witnesses(&payload.key, catalog, visiting)? {
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
            for element in elements {
                let children = witnesses(element, catalog, visiting)?;
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
        _ => false,
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
