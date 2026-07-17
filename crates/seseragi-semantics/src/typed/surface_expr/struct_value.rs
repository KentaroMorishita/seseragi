use std::collections::{BTreeMap, BTreeSet};

use crate::{TypedExpr, TypedRecordValueItem, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceRecordItem};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::pure_issues::RecordIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::inferred_type_from_expr;

pub(super) fn type_struct(
    name: &str,
    name_span: ByteSpan,
    items: &[SurfaceRecordItem],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let Some(owner) = context.type_target(name_span) else {
        return invalid_struct(name, items, span, context);
    };
    let Some(struct_type) = context.semantic_types().struct_type(owner) else {
        return invalid_struct(name, items, span, context);
    };

    let arguments = match context.expected().map(|expected| &expected.key) {
        Some(SemanticTypeKey::Struct {
            owner: expected_owner,
            arguments,
        }) if *expected_owner == owner => arguments.clone(),
        _ if struct_type.type_parameters.is_empty() => Vec::new(),
        _ => struct_type
            .type_parameter_names
            .iter()
            .map(|parameter| SemanticValueType {
                type_ref: TypedType::Named {
                    name: parameter.clone(),
                    arguments: Vec::new(),
                },
                key: SemanticTypeKey::Invalid,
            })
            .collect(),
    };
    let expected_type = TypedType::Named {
        name: struct_type.name.clone(),
        arguments: arguments
            .iter()
            .map(|argument| argument.type_ref.clone())
            .collect(),
    };
    let fields = context
        .semantic_types()
        .instantiate_struct_fields(owner, &arguments)
        .unwrap_or_default();
    let fields = fields
        .into_iter()
        .map(|field| (field.name, field.type_ref))
        .collect::<BTreeMap<_, _>>();

    let mut seen = BTreeSet::new();
    let mut typed_items = Vec::with_capacity(items.len());
    let mut children = Vec::with_capacity(items.len());
    let mut issue = None;
    let mut spread_count = 0;

    for (index, item) in items.iter().enumerate() {
        match item {
            SurfaceRecordItem::Field {
                name,
                name_span,
                value,
                span,
            } => {
                if !seen.insert(name.clone()) {
                    issue.get_or_insert(RecordIssue::DuplicateField {
                        field: *name_span,
                        name: name.clone(),
                    });
                }
                let expected = fields.get(name).cloned();
                if expected.is_none() {
                    issue.get_or_insert(RecordIssue::UnknownStructField {
                        field: *name_span,
                        name: name.clone(),
                        structure: struct_type.name.clone(),
                    });
                }
                let analysis =
                    type_surface_expression(value, &context.with_expected(expected.clone()));
                if let Some(expected) = expected {
                    let actual = SemanticValueType {
                        type_ref: inferred_type_from_expr(&analysis.value),
                        key: analysis.semantic_type.clone(),
                    };
                    if !semantic_values_are_compatible(&expected, &actual) {
                        issue.get_or_insert(RecordIssue::StructFieldType {
                            field: value.span(),
                            name: name.clone(),
                            expected: expected.type_ref,
                            actual: actual.type_ref,
                        });
                    }
                }
                typed_items.push(TypedRecordValueItem::Field {
                    name: name.clone(),
                    value: analysis.value.clone(),
                    origin: *span,
                });
                children.push(analysis);
            }
            SurfaceRecordItem::Spread { value, span } => {
                spread_count += 1;
                if index != 0 {
                    issue.get_or_insert(RecordIssue::StructSpreadPosition { spread: *span });
                }
                if spread_count > 1 {
                    issue.get_or_insert(RecordIssue::MultipleStructSpreads { spread: *span });
                }
                let analysis = type_surface_expression(value, &context.without_expected());
                let actual = SemanticValueType {
                    type_ref: inferred_type_from_expr(&analysis.value),
                    key: analysis.semantic_type.clone(),
                };
                let expected = SemanticValueType {
                    type_ref: expected_type.clone(),
                    key: SemanticTypeKey::Struct {
                        owner,
                        arguments: arguments.clone(),
                    },
                };
                if !semantic_values_are_compatible(&expected, &actual) {
                    issue.get_or_insert(RecordIssue::StructSpreadType {
                        spread: *span,
                        expected: expected.type_ref,
                        actual: actual.type_ref,
                    });
                }
                typed_items.push(TypedRecordValueItem::Spread {
                    value: analysis.value.clone(),
                    origin: *span,
                });
                children.push(analysis);
            }
        }
    }

    if spread_count == 0 {
        for field in fields.keys() {
            if !seen.contains(field) {
                issue.get_or_insert(RecordIssue::MissingStructField {
                    structure: name_span,
                    name: field.clone(),
                });
                break;
            }
        }
    }

    let semantic_type = if issue.is_some() {
        SemanticTypeKey::Invalid
    } else {
        SemanticTypeKey::Struct { owner, arguments }
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Record {
            items: typed_items,
            type_ref: expected_type,
            origin: span,
        },
        semantic_type,
    );
    result.record_issue = issue;
    for child in children {
        result.merge_issues_from(child);
    }
    result
}

fn invalid_struct(
    name: &str,
    items: &[SurfaceRecordItem],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let mut typed_items = Vec::with_capacity(items.len());
    let mut children = Vec::with_capacity(items.len());
    for item in items {
        let analysis = type_surface_expression(item.value(), &context.without_expected());
        typed_items.push(match item {
            SurfaceRecordItem::Field { name, span, .. } => TypedRecordValueItem::Field {
                name: name.clone(),
                value: analysis.value.clone(),
                origin: *span,
            },
            SurfaceRecordItem::Spread { span, .. } => TypedRecordValueItem::Spread {
                value: analysis.value.clone(),
                origin: *span,
            },
        });
        children.push(analysis);
    }
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Record {
            items: typed_items,
            type_ref: TypedType::Named {
                name: name.to_owned(),
                arguments: Vec::new(),
            },
            origin: span,
        },
        SemanticTypeKey::Invalid,
    );
    for child in children {
        result.merge_issues_from(child);
    }
    result
}
