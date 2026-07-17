use std::collections::{BTreeMap, BTreeSet};

use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};
use crate::{TypedPattern, TypedRecordPatternField};
use seseragi_syntax::{ByteSpan, SurfaceRecordPatternField};

use super::{invalid, type_pattern, CoveragePattern, PatternAnalysis};
use crate::typed::surface_expr::PureExpressionContext;

pub(super) fn type_struct_pattern(
    name: &str,
    name_span: ByteSpan,
    fields: &[SurfaceRecordPatternField],
    span: ByteSpan,
    expected: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> PatternAnalysis {
    let Some(owner) = context.type_target(name_span) else {
        return invalid(span, format!("struct pattern `{name}` is unresolved"));
    };
    let SemanticTypeKey::Struct {
        owner: expected_owner,
        arguments,
    } = &expected.key
    else {
        return invalid(span, format!("struct pattern `{name}` requires `{name}`"));
    };
    if owner != *expected_owner {
        return invalid(
            span,
            format!("struct pattern `{name}` does not match the scrutinee type"),
        );
    }
    let available = context
        .semantic_types()
        .instantiate_struct_fields(owner, arguments)
        .unwrap_or_default();

    let mut seen = BTreeSet::new();
    let mut children = Vec::with_capacity(fields.len());
    for field in fields {
        if !seen.insert(field.name.as_str()) {
            return invalid(
                field.name_span,
                format!("struct pattern repeats field {}", field.name),
            );
        }
        if field.optional {
            return invalid(
                field.span,
                "struct patterns do not support optional queries",
            );
        }
        let Some(available) = available.iter().find(|item| item.name == field.name) else {
            return invalid(
                field.name_span,
                format!("struct pattern field {} does not exist", field.name),
            );
        };
        children.push((
            field,
            type_pattern(&field.pattern, &available.type_ref, context),
        ));
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
                    optional: false,
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
