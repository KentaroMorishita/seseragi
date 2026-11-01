use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::{SymbolId, TypedType};

use super::super::pattern::{CoveragePattern, LiteralPattern};
use super::super::PureExpressionContext;

// Bound source complexity, not the cardinality of a type's value domain.
const MAX_STEPS: usize = 100_000;
const MAX_DEPTH: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Constructor {
    Literal(LiteralPattern),
    Adt(SymbolId),
    Tuple,
    Record(Vec<(String, bool)>),
    Empty(bool),
    Cons(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Pattern {
    Any,
    Never,
    Constructor(Constructor, Vec<Pattern>),
}

struct Signature {
    constructor: Constructor,
    fields: Vec<SemanticValueType>,
}

pub(super) struct CoverageResult {
    pub(super) unreachable: Vec<usize>,
    pub(super) missing: Vec<String>,
    pub(super) limited: bool,
}

pub(super) fn analyze_coverage(
    value: &SemanticValueType,
    context: &PureExpressionContext<'_>,
    arms: &[(CoveragePattern, bool)],
) -> CoverageResult {
    let mut matrix = Vec::new();
    let mut unreachable = Vec::new();
    let mut steps = MAX_STEPS;
    for (index, (pattern, guarded)) in arms.iter().enumerate() {
        let Ok(pattern) = normalize(pattern, value, context, 0) else {
            return CoverageResult {
                unreachable,
                missing: Vec::new(),
                limited: true,
            };
        };
        match useful(
            &matrix,
            &[pattern.clone()],
            std::slice::from_ref(value),
            context,
            &mut steps,
            0,
        ) {
            Ok(None) => unreachable.push(index),
            Ok(Some(_)) => {
                if !guarded {
                    matrix.push(vec![pattern]);
                }
            }
            Err(()) => {
                return CoverageResult {
                    unreachable,
                    missing: Vec::new(),
                    limited: true,
                }
            }
        }
    }
    match useful(
        &matrix,
        &[Pattern::Any],
        std::slice::from_ref(value),
        context,
        &mut steps,
        0,
    ) {
        Ok(witness) => CoverageResult {
            unreachable,
            missing: witness
                .map(|patterns| vec![render(&patterns[0], context)])
                .unwrap_or_default(),
            limited: false,
        },
        Err(()) => CoverageResult {
            unreachable,
            missing: Vec::new(),
            limited: true,
        },
    }
}

// Constructor specialization consumes finite source-pattern structure. Recursive
// type payloads are inspected only when a pattern demands them; no recursive type
// or Cartesian product is eagerly enumerated.
fn useful(
    matrix: &[Vec<Pattern>],
    candidate: &[Pattern],
    types: &[SemanticValueType],
    context: &PureExpressionContext<'_>,
    steps: &mut usize,
    depth: usize,
) -> Result<Option<Vec<Pattern>>, ()> {
    if candidate.iter().any(contains_never) {
        return Ok(None);
    }
    if matrix
        .iter()
        .any(|row| row.iter().all(|pattern| *pattern == Pattern::Any))
    {
        return Ok(None);
    }
    if matrix.is_empty() {
        return Ok(Some(candidate.to_vec()));
    }
    if candidate.is_empty() {
        return Ok(Some(Vec::new()));
    }
    if depth >= MAX_DEPTH || *steps == 0 {
        return Err(());
    }
    *steps -= 1;
    let signatures = signatures(&types[0], context);
    if let Pattern::Constructor(constructor, fields) = &candidate[0] {
        let field_types = signatures
            .iter()
            .find(|signature| signature.constructor == *constructor)
            .map(|signature| signature.fields.clone())
            .unwrap_or_default();
        let mut specialized_candidate = fields.clone();
        specialized_candidate.extend_from_slice(&candidate[1..]);
        let mut specialized_types = field_types;
        specialized_types.extend_from_slice(&types[1..]);
        let specialized = specialize(matrix, constructor, fields.len());
        return useful(
            &specialized,
            &specialized_candidate,
            &specialized_types,
            context,
            steps,
            depth + 1,
        )
        .map(|witness| {
            witness.map(|mut witness| {
                let tail = witness.split_off(fields.len());
                let mut result = vec![Pattern::Constructor(constructor.clone(), witness)];
                result.extend(tail);
                result
            })
        });
    }
    let complete = !signatures.is_empty() && signatures.iter().all(|signature| {
        matrix.iter().any(|row| matches!(&row[0], Pattern::Constructor(constructor, _) if *constructor == signature.constructor))
    });
    if complete {
        for signature in signatures {
            let arity = signature.fields.len();
            let specialized = specialize(matrix, &signature.constructor, signature.fields.len());
            let mut specialized_candidate = vec![Pattern::Any; signature.fields.len()];
            specialized_candidate.extend_from_slice(&candidate[1..]);
            let mut specialized_types = signature.fields;
            specialized_types.extend_from_slice(&types[1..]);
            if let Some(mut witness) = useful(
                &specialized,
                &specialized_candidate,
                &specialized_types,
                context,
                steps,
                depth + 1,
            )? {
                let tail = witness.split_off(arity);
                let mut result = vec![Pattern::Constructor(signature.constructor, witness)];
                result.extend(tail);
                return Ok(Some(result));
            }
        }
        Ok(None)
    } else {
        let default = matrix
            .iter()
            .filter(|row| row[0] == Pattern::Any)
            .map(|row| row[1..].to_vec())
            .collect::<Vec<_>>();
        let witness = useful(
            &default,
            &candidate[1..],
            &types[1..],
            context,
            steps,
            depth + 1,
        )?;
        Ok(witness.map(|mut tail| {
            let missing = signatures.iter().find(|signature| {
                !matrix.iter().any(|row| matches!(&row[0], Pattern::Constructor(constructor, _) if *constructor == signature.constructor))
            });
            let first = missing.map_or(Pattern::Any, |signature| {
                Pattern::Constructor(signature.constructor.clone(), vec![Pattern::Any; signature.fields.len()])
            });
            tail.insert(0, first);
            tail
        }))
    }
}

fn contains_never(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Never => true,
        Pattern::Constructor(_, fields) => fields.iter().any(contains_never),
        Pattern::Any => false,
    }
}

fn specialize(
    matrix: &[Vec<Pattern>],
    constructor: &Constructor,
    arity: usize,
) -> Vec<Vec<Pattern>> {
    matrix
        .iter()
        .filter_map(|row| {
            let mut fields = match &row[0] {
                Pattern::Any => vec![Pattern::Any; arity],
                Pattern::Constructor(found, fields) if found == constructor => fields.clone(),
                _ => return None,
            };
            fields.extend_from_slice(&row[1..]);
            Some(fields)
        })
        .collect()
}

fn signatures(value: &SemanticValueType, context: &PureExpressionContext<'_>) -> Vec<Signature> {
    match &value.key {
        SemanticTypeKey::NamedGeneric { name, arguments }
            if (name == "Array" || name == "List") && arguments.len() == 1 =>
        {
            let is_list = name == "List";
            return vec![
                Signature {
                    constructor: Constructor::Empty(is_list),
                    fields: Vec::new(),
                },
                Signature {
                    constructor: Constructor::Cons(is_list),
                    fields: vec![arguments[0].clone(), value.clone()],
                },
            ];
        }
        SemanticTypeKey::Adt { owner, arguments } => {
            return context
                .semantic_types()
                .adt(*owner)
                .map(|adt| {
                    adt.variants
                        .iter()
                        .map(|variant| Signature {
                            constructor: Constructor::Adt(variant.constructor),
                            fields: variant
                                .payload
                                .iter()
                                .map(|payload| {
                                    context
                                        .semantic_types()
                                        .instantiate_payload(*owner, arguments, payload)
                                })
                                .collect(),
                        })
                        .collect()
                })
                .unwrap_or_default();
        }
        SemanticTypeKey::Struct { owner, arguments } => {
            let fields = context
                .semantic_types()
                .instantiate_struct_fields(*owner, arguments)
                .unwrap_or_default();
            return vec![Signature {
                constructor: Constructor::Record(
                    fields
                        .iter()
                        .map(|field| (field.name.clone(), false))
                        .collect(),
                ),
                fields: fields.into_iter().map(|field| field.type_ref).collect(),
            }];
        }
        _ => {}
    }
    match &value.type_ref {
        TypedType::Tuple { elements } if matches!(&value.key, SemanticTypeKey::Tuple(keys) if keys.len() != elements.len()) => {
            Vec::new()
        }
        TypedType::Tuple { elements } => vec![Signature {
            constructor: Constructor::Tuple,
            fields: elements
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let mut field = context.semantic_value_from_typed_type(field);
                    if let SemanticTypeKey::Tuple(keys) = &value.key {
                        if let Some(key) = keys.get(index) {
                            field.key = key.clone();
                        }
                    }
                    field
                })
                .collect(),
        }],
        TypedType::Record { fields, .. } => vec![Signature {
            constructor: Constructor::Record(
                fields
                    .iter()
                    .map(|field| (field.name.clone(), field.optional))
                    .collect(),
            ),
            fields: fields
                .iter()
                .map(|field| {
                    let value = match &value.key {
                        SemanticTypeKey::Record(values) => values.get(&field.name).cloned(),
                        _ => None,
                    }
                    .unwrap_or_else(|| context.semantic_value_from_typed_type(&field.type_ref));
                    if field.optional {
                        super::super::fallback::maybe_type(value, context)
                    } else {
                        value
                    }
                })
                .collect(),
        }],
        TypedType::Named { name, arguments }
            if name == "Bool" && arguments.is_empty() && value.key == SemanticTypeKey::Other =>
        {
            [false, true]
                .into_iter()
                .map(|value| Signature {
                    constructor: Constructor::Literal(LiteralPattern::Boolean(value)),
                    fields: Vec::new(),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn normalize(
    pattern: &CoveragePattern,
    value: &SemanticValueType,
    context: &PureExpressionContext<'_>,
    depth: usize,
) -> Result<Pattern, ()> {
    if depth >= MAX_DEPTH {
        return Err(());
    }
    if matches!(pattern, CoveragePattern::Any) {
        return Ok(Pattern::Any);
    }
    let heads = signatures(value, context);
    let (constructor, fields) = match pattern {
        CoveragePattern::Any => return Ok(Pattern::Any),
        CoveragePattern::Never => return Ok(Pattern::Never),
        CoveragePattern::Invalid => return Err(()),
        CoveragePattern::Literal(literal) => {
            return Ok(Pattern::Constructor(
                Constructor::Literal(literal.clone()),
                Vec::new(),
            ))
        }
        CoveragePattern::Constructor {
            constructor,
            argument,
            ..
        } => (
            Constructor::Adt(*constructor),
            argument
                .iter()
                .map(|argument| argument.as_ref().clone())
                .collect::<Vec<_>>(),
        ),
        CoveragePattern::Tuple(fields) => (Constructor::Tuple, fields.clone()),
        CoveragePattern::Record(fields) => {
            let Some(head) = heads.first() else {
                return Err(());
            };
            let Constructor::Record(names) = &head.constructor else {
                return Err(());
            };
            (
                head.constructor.clone(),
                names
                    .iter()
                    .map(|(name, _)| {
                        fields
                            .iter()
                            .find(|(field, _)| field == name)
                            .map_or(CoveragePattern::Any, |(_, pattern)| pattern.clone())
                    })
                    .collect(),
            )
        }
        CoveragePattern::Array { elements, has_rest }
        | CoveragePattern::List { elements, has_rest } => {
            // A flat surface Vec becomes a recursive Cons tree. Bound the
            // resulting tree *before* allocation/clone/drop, not only when
            // usefulness later traverses it. Nested element depth includes
            // its position along that prefix.
            if depth.saturating_add(elements.len()) >= MAX_DEPTH {
                return Err(());
            }
            let is_list = matches!(pattern, CoveragePattern::List { .. });
            let Some(head) = heads
                .iter()
                .find(|head| head.constructor == Constructor::Cons(is_list))
            else {
                return Err(());
            };
            let mut tail = if *has_rest {
                Pattern::Any
            } else {
                Pattern::Constructor(Constructor::Empty(is_list), Vec::new())
            };
            for (index, element) in elements.iter().enumerate().rev() {
                tail = Pattern::Constructor(
                    Constructor::Cons(is_list),
                    vec![
                        normalize(element, &head.fields[0], context, depth + index + 1)?,
                        tail,
                    ],
                );
            }
            return Ok(tail);
        }
    };
    let Some(head) = heads.iter().find(|head| head.constructor == constructor) else {
        return Err(());
    };
    if head.fields.len() != fields.len() {
        return Err(());
    }
    Ok(Pattern::Constructor(
        constructor,
        fields
            .iter()
            .zip(&head.fields)
            .map(|(pattern, field)| normalize(pattern, field, context, depth + 1))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

fn render(pattern: &Pattern, context: &PureExpressionContext<'_>) -> String {
    let Pattern::Constructor(constructor, fields) = pattern else {
        return "_".to_owned();
    };
    let values = fields
        .iter()
        .map(|field| render(field, context))
        .collect::<Vec<_>>();
    match constructor {
        Constructor::Literal(literal) => match literal {
            LiteralPattern::Integer(value) => value.clone(),
            LiteralPattern::String(value) => format!("{value:?}"),
            LiteralPattern::Char(value) => format!(
                "'{}'",
                value
                    .chars()
                    .flat_map(char::escape_default)
                    .collect::<String>()
            ),
            LiteralPattern::Boolean(value) => if *value { "True" } else { "False" }.to_owned(),
        },
        Constructor::Adt(symbol) => {
            let name = context
                .semantic_types()
                .constructor(*symbol)
                .map(|(_, variant)| variant.spelling.as_str())
                .unwrap_or("_");
            if values.is_empty() {
                name.to_owned()
            } else {
                format!("{name} {}", values.join(" "))
            }
        }
        Constructor::Tuple => format!("({})", values.join(", ")),
        Constructor::Record(names) => format!(
            "{{ {} }}",
            names
                .iter()
                .zip(values)
                .map(|((name, optional), value)| format!(
                    "{name}{}: {value}",
                    if *optional { "?" } else { "" }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Constructor::Empty(is_list) | Constructor::Cons(is_list) => {
            let mut elements = Vec::new();
            let mut tail = pattern;
            while let Pattern::Constructor(Constructor::Cons(_), fields) = tail {
                elements.push(render(&fields[0], context));
                tail = &fields[1];
            }
            if *tail == Pattern::Any {
                elements.push("..._".to_owned());
            }
            format!(
                "{}[{}]",
                if *is_list { "`" } else { "" },
                elements.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed::TypedResolution;

    #[test]
    fn exhausted_work_budget_is_unknown_and_never_a_coverage_proof() {
        let resolved = crate::resolve_module("budget.ssrg", "");
        let resolution = TypedResolution::new(&resolved);
        let context = PureExpressionContext::new(&[], &resolution);
        let boolean = SemanticValueType {
            type_ref: TypedType::Named {
                name: "Bool".to_owned(),
                arguments: Vec::new(),
            },
            key: SemanticTypeKey::Other,
        };
        let matrix = vec![vec![Pattern::Constructor(
            Constructor::Literal(LiteralPattern::Boolean(true)),
            Vec::new(),
        )]];
        assert!(useful(
            &matrix,
            &[Pattern::Any],
            std::slice::from_ref(&boolean),
            &context,
            &mut 0,
            0
        )
        .is_err());
        // A direct wildcard row is already a proof; it does not need further
        // analysis even when all of the available work has been consumed.
        assert_eq!(
            useful(
                &[vec![Pattern::Any]],
                &[Pattern::Any],
                &[boolean],
                &context,
                &mut 0,
                0
            ),
            Ok(None)
        );
    }
    #[test]
    fn inconsistent_tuple_key_arity_is_rejected_without_indexing_past_the_key() {
        let resolved = crate::resolve_module("tuple-key.ssrg", "");
        let resolution = TypedResolution::new(&resolved);
        let context = PureExpressionContext::new(&[], &resolution);
        let value = SemanticValueType {
            type_ref: TypedType::Tuple {
                elements: vec![TypedType::Named {
                    name: "Bool".to_owned(),
                    arguments: Vec::new(),
                }],
            },
            key: SemanticTypeKey::Tuple(Vec::new()),
        };
        assert!(normalize(
            &CoveragePattern::Tuple(vec![CoveragePattern::Any]),
            &value,
            &context,
            0
        )
        .is_err());
    }
}
