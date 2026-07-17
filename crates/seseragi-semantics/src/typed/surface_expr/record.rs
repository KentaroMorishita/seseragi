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
    let (type_ref, optional, issue) = match &receiver_type {
        TypedType::Record { fields, .. } => match fields.iter().find(|item| item.name == field) {
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
