use std::collections::{BTreeMap, BTreeSet};

use crate::{TypedExpr, TypedRecordField, TypedRecordValueField, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr, SurfaceRecordField};

use super::{type_name, type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::RecordIssue;
use crate::typed::semantic_types::SemanticTypeKey;
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_record(
    fields: &[SurfaceRecordField],
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
    let mut typed_fields = Vec::with_capacity(fields.len());
    let mut field_types = Vec::with_capacity(fields.len());
    let mut children = Vec::with_capacity(fields.len());
    let mut issue = None;

    for field in fields {
        if !seen.insert(field.name.as_str()) {
            issue.get_or_insert(RecordIssue::DuplicateField {
                field: field.name_span,
                name: field.name.clone(),
            });
        }
        let expected = expected
            .get(field.name.as_str())
            .map(|type_ref| context.semantic_value_from_typed_type(type_ref));
        let analysis = type_surface_expression(&field.value, &context.with_expected(expected));
        let type_ref = inferred_type_from_expr(&analysis.value);
        typed_fields.push(TypedRecordValueField {
            name: field.name.clone(),
            value: analysis.value.clone(),
            origin: field.span,
        });
        field_types.push(TypedRecordField {
            name: field.name.clone(),
            optional: false,
            type_ref,
        });
        children.push(analysis);
    }
    field_types.sort_by(|left, right| left.name.cmp(&right.name));
    let type_ref = TypedType::Record {
        closed: true,
        fields: field_types,
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Record {
            fields: typed_fields,
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
