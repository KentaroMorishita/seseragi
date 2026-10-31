use std::collections::{BTreeMap, BTreeSet};

use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::{TypedPattern, TypedRecordPatternField, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceRecordPatternField};

use super::{invalid, type_pattern, CoveragePattern, PatternAnalysis};
use crate::typed::surface_expr::PureExpressionContext;

pub(super) fn type_record_pattern(
    fields: &[SurfaceRecordPatternField],
    span: ByteSpan,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    let TypedType::Record {
        fields: available, ..
    } = &expected.type_ref
    else {
        return invalid(span, "record pattern requires a record scrutinee");
    };

    let mut seen = BTreeSet::new();
    let mut children = Vec::with_capacity(fields.len());
    for field in fields {
        if !seen.insert(field.name.as_str()) {
            return invalid(
                field.name_span,
                format!("record pattern repeats field {}", field.name),
            );
        }
        let Some(available) = available.iter().find(|item| item.name == field.name) else {
            return invalid(
                field.name_span,
                format!("record pattern field {} does not exist", field.name),
            );
        };
        if available.optional && !field.optional {
            return invalid(
                field.name_span,
                format!(
                    "optional record field {} requires an optional query pattern",
                    field.name
                ),
            );
        }
        let nested_expected = match &expected.key {
            SemanticTypeKey::Record(values) => values.get(&field.name).cloned(),
            _ => None,
        }
        .unwrap_or_else(|| context.semantic_value_from_typed_type(&available.type_ref));
        let query_expected = if field.optional {
            super::super::fallback::maybe_type(nested_expected, context)
        } else {
            nested_expected
        };
        let mut child = type_pattern(&field.pattern, &query_expected, context);
        if field.optional && !available.optional && !child.invalid {
            child.coverage = match child.coverage {
                CoveragePattern::Constructor {
                    argument: Some(argument),
                    ..
                } => *argument,
                CoveragePattern::Constructor { argument: None, .. } => CoveragePattern::Never,
                coverage => coverage,
            };
        }
        children.push((field, child));
    }

    let invalid = children.iter().any(|(_, child)| child.invalid);
    let mut locals = BTreeMap::new();
    let mut issues = Vec::new();
    for (_, child) in &children {
        locals.extend(child.locals.clone());
        issues.extend(child.issues.clone());
    }
    let coverage_fields = children
        .iter()
        .map(|(field, child)| (field.name.clone(), child.coverage.clone()))
        .collect::<Vec<_>>();
    let coverage = if coverage_fields
        .iter()
        .all(|(_, pattern)| matches!(pattern, CoveragePattern::Any))
    {
        CoveragePattern::Any
    } else {
        CoveragePattern::Record(coverage_fields)
    };

    PatternAnalysis {
        typed: TypedPattern::Record {
            fields: children
                .into_iter()
                .map(|(field, child)| TypedRecordPatternField {
                    name: field.name.clone(),
                    optional: field.optional,
                    pattern: child.typed,
                    origin: field.span,
                })
                .collect(),
            type_ref: expected.type_ref.clone(),
            origin: span,
        },
        coverage,
        locals,
        issues,
        invalid,
    }
}
