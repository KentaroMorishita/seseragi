use std::collections::{BTreeMap, BTreeSet};

use crate::{TypedExpr, TypedRecordField, TypedRecordValueItem, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceRecordItem};

use super::{type_name, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::RecordIssue;
use crate::typed::semantic_types::SemanticTypeKey;
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_record(
    items: &[SurfaceRecordItem],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let expected = match context.expected().map(|value| &value.type_ref) {
        Some(TypedType::Record { fields, .. }) => fields
            .iter()
            .map(|field| (field.name.as_str(), field.type_ref.clone()))
            .collect::<BTreeMap<_, _>>(),
        _ => BTreeMap::new(),
    };
    let mut seen = BTreeSet::new();
    let mut typed_items = Vec::with_capacity(items.len());
    let mut field_types = BTreeMap::new();
    let mut children = Vec::with_capacity(items.len());
    let mut issue = None;

    for item in items {
        match item {
            SurfaceRecordItem::Field {
                name,
                name_span,
                value,
                span,
            } => {
                if !seen.insert(name.as_str()) {
                    issue.get_or_insert(RecordIssue::DuplicateField {
                        field: *name_span,
                        name: name.clone(),
                    });
                }
                let expected = expected
                    .get(name.as_str())
                    .map(|type_ref| context.semantic_value_from_typed_type(type_ref));
                let analysis = type_surface_expression(value, &context.with_expected(expected));
                let type_ref = inferred_type_from_expr(&analysis.value);
                typed_items.push(TypedRecordValueItem::Field {
                    name: name.clone(),
                    value: analysis.value.clone(),
                    origin: *span,
                });
                field_types.insert(
                    name.clone(),
                    TypedRecordField {
                        name: name.clone(),
                        optional: false,
                        type_ref,
                    },
                );
                children.push(analysis);
            }
            SurfaceRecordItem::Spread { value, span } => {
                let analysis = type_surface_expression(value, &context.without_expected());
                let spread_type = inferred_type_from_expr(&analysis.value);
                match &spread_type {
                    TypedType::Record { fields, .. } => {
                        for field in fields {
                            field_types.insert(field.name.clone(), field.clone());
                        }
                    }
                    TypedType::Hole => {}
                    actual => {
                        issue.get_or_insert(RecordIssue::SpreadOnNonRecord {
                            spread: *span,
                            actual: actual.clone(),
                        });
                    }
                }
                typed_items.push(TypedRecordValueItem::Spread {
                    value: analysis.value.clone(),
                    origin: *span,
                });
                children.push(analysis);
            }
        }
    }
    let type_ref = TypedType::Record {
        closed: true,
        fields: field_types.into_values().collect(),
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Record {
            items: typed_items,
            type_ref,
            origin: span,
        },
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.record_issue = issue;
    for child in children {
        result.merge_issues_from(child);
    }
    result
}

pub(super) fn type_member(
    receiver: &SurfaceExpr,
    field: &str,
    field_span: ByteSpan,
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    if context.target(span).is_some() {
        let receiver_name = match receiver {
            SurfaceExpr::Name { name, .. } => name.as_str(),
            _ => "",
        };
        return type_name(&format!("{receiver_name}.{field}"), span, context);
    }

    let receiver_analysis = type_surface_expression(receiver, &context.without_expected());
    let receiver_type = inferred_type_from_expr(&receiver_analysis.value);
    let (type_ref, optional, issue) = match &receiver_analysis.semantic_type {
        SemanticTypeKey::Struct { owner, arguments } => {
            let fields = context
                .semantic_types()
                .instantiate_struct_fields(*owner, arguments)
                .unwrap_or_default();
            let found = fields.iter().find(|item| item.name == field);
            match found {
                Some(found) => (found.type_ref.type_ref.clone(), false, None),
                None => {
                    if let Some(method) = super::application::type_inherent_method_member(
                        receiver, field, field_span, span, context,
                    ) {
                        return method;
                    }
                    (
                        TypedType::Hole,
                        false,
                        Some(RecordIssue::MissingField {
                            field: field_span,
                            name: field.to_owned(),
                            suggestion: closest_field_name(
                                field,
                                fields.iter().map(|item| item.name.as_str()),
                            ),
                        }),
                    )
                }
            }
        }
        SemanticTypeKey::Adt { .. } => {
            if let Some(method) = super::application::type_inherent_method_member(
                receiver, field, field_span, span, context,
            ) {
                return method;
            }
            (
                TypedType::Hole,
                false,
                Some(RecordIssue::AccessOnNonRecord {
                    receiver: receiver.span(),
                    actual: receiver_type.clone(),
                }),
            )
        }
        _ => match &receiver_type {
            TypedType::Record { fields, .. } => match fields.iter().find(|item| item.name == field)
            {
                Some(found) if found.optional => (
                    TypedType::Named {
                        name: "Maybe".to_owned(),
                        arguments: vec![found.type_ref.clone()],
                    },
                    true,
                    None,
                ),
                Some(found) => (found.type_ref.clone(), false, None),
                None => (
                    TypedType::Hole,
                    false,
                    Some(RecordIssue::MissingField {
                        field: field_span,
                        name: field.to_owned(),
                        suggestion: closest_field_name(
                            field,
                            fields.iter().map(|item| item.name.as_str()),
                        ),
                    }),
                ),
            },
            TypedType::Hole => (TypedType::Hole, false, None),
            actual => (
                TypedType::Hole,
                false,
                Some(RecordIssue::AccessOnNonRecord {
                    receiver: receiver.span(),
                    actual: actual.clone(),
                }),
            ),
        },
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        if optional {
            TypedExpr::OptionalFieldAccess {
                receiver: Box::new(receiver_analysis.value.clone()),
                field: field.to_owned(),
                type_ref: type_ref.clone(),
                origin: span,
            }
        } else {
            TypedExpr::FieldAccess {
                receiver: Box::new(receiver_analysis.value.clone()),
                field: field.to_owned(),
                type_ref: type_ref.clone(),
                origin: span,
            }
        },
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            context.semantic_value_from_typed_type(&type_ref).key
        },
    );
    result.record_issue = issue;
    result.merge_issues_from(receiver_analysis);
    result
}

pub(super) fn closest_field_name<'a>(
    requested: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<String> {
    let maximum_distance = if requested.chars().count() <= 2 { 1 } else { 2 };
    candidates
        .map(|candidate| (edit_distance(requested, candidate), candidate))
        .filter(|(distance, _)| *distance <= maximum_distance)
        .min_by(|left, right| left.cmp(right))
        .map(|(_, candidate)| candidate.to_owned())
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        current.push(left_index + 1);
        for (right_index, right_char) in right.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_char != *right_char);
            current.push(
                (previous[right_index + 1] + 1)
                    .min(current[right_index] + 1)
                    .min(substitution),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::closest_field_name;

    #[test]
    fn suggests_only_close_field_names() {
        let fields = ["name", "score", "displayName"];
        assert_eq!(
            closest_field_name("nmae", fields.into_iter()),
            Some("name".to_owned())
        );
        assert_eq!(closest_field_name("unrelated", fields.into_iter()), None);
    }
}
