use crate::typed::pure_issues::ArrayIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::inferred_type_from_expr;
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};

pub(super) fn type_array(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    let expected_element = expected_element(context);
    if elements.is_empty() {
        return empty_array(span, expected_element);
    }

    let first_context = context.with_expected(expected_element.clone());
    let first = type_surface_expression(&elements[0], &first_context);
    let inferred_element = SemanticValueType {
        type_ref: inferred_type_from_expr(&first.value),
        key: first.semantic_type.clone(),
    };
    let element_type = expected_element.unwrap_or_else(|| inferred_element.clone());
    let mut children = vec![first];
    children.extend(elements[1..].iter().map(|element| {
        type_surface_expression(element, &context.with_expected(Some(element_type.clone())))
    }));

    let issue = children.iter().enumerate().find_map(|(index, child)| {
        let actual = SemanticValueType {
            type_ref: inferred_type_from_expr(&child.value),
            key: child.semantic_type.clone(),
        };
        (!semantic_values_are_compatible(&element_type, &actual)).then(|| {
            ArrayIssue::ElementTypeMismatch {
                element: elements[index].span(),
                index,
                expected: element_type.type_ref.clone(),
                actual: actual.type_ref,
            }
        })
    });
    let type_ref = TypedType::Named {
        name: "Array".to_owned(),
        arguments: vec![if issue.is_some() {
            TypedType::Hole
        } else {
            element_type.type_ref
        }],
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Array {
            elements: children.iter().map(|child| child.value.clone()).collect(),
            type_ref,
            origin: span,
        },
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.array_issue = issue;
    for child in children {
        result.merge_issues_from(child);
    }
    result
}

fn expected_element(context: &PureExpressionContext<'_>) -> Option<SemanticValueType> {
    let TypedType::Named { name, arguments } = &context.expected()?.type_ref else {
        return None;
    };
    (name == "Array" && arguments.len() == 1)
        .then(|| context.semantic_value_from_typed_type(&arguments[0]))
}

fn empty_array(span: ByteSpan, expected: Option<SemanticValueType>) -> SurfaceExpressionAnalysis {
    let issue = expected
        .is_none()
        .then_some(ArrayIssue::EmptyWithoutExpectedType { array: span });
    let element = expected
        .map(|expected| expected.type_ref)
        .unwrap_or(TypedType::Hole);
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        TypedExpr::Array {
            elements: Vec::new(),
            type_ref: TypedType::Named {
                name: "Array".to_owned(),
                arguments: vec![element],
            },
            origin: span,
        },
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.array_issue = issue;
    result
}
