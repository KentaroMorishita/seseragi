use crate::typed::type_ref::inferred_type_from_expr;
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};
use crate::typed::semantic_types::SemanticTypeKey;

pub(super) fn type_tuple(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let children = elements
        .iter()
        .map(|element| type_surface_expression(element, context))
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
