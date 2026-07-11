use crate::typed::type_ref::inferred_type_from_expr;
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::semantic_types::{SemanticTypeKey, SemanticValueType};

pub(super) fn type_tuple(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let expected_elements = match context.expected() {
        Some(SemanticValueType {
            type_ref: TypedType::Tuple { elements: types },
            key: SemanticTypeKey::Tuple(keys),
        }) if types.len() == elements.len() && keys.len() == elements.len() => Some(
            types
                .iter()
                .cloned()
                .zip(keys.iter().cloned())
                .map(|(type_ref, key)| SemanticValueType { type_ref, key })
                .collect::<Vec<_>>(),
        ),
        _ => None,
    };
    let children = elements
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let element_context = context.with_expected(
                expected_elements
                    .as_ref()
                    .and_then(|elements| elements.get(index))
                    .cloned(),
            );
            type_surface_expression(element, &element_context)
        })
        .collect::<Vec<_>>();
    let type_ref = TypedType::Tuple {
        elements: children
            .iter()
            .map(|child| inferred_type_from_expr(&child.value))
            .collect(),
    };
    let semantic_type = SemanticTypeKey::Tuple(
        children
            .iter()
            .map(|child| child.semantic_type.clone())
            .collect(),
    );
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Tuple {
            elements: children.iter().map(|child| child.value.clone()).collect(),
            type_ref,
            origin: span,
        },
        semantic_type,
    );
    for child in children {
        result.merge_issues_from(child);
    }
    result
}
