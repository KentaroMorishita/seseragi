use crate::typed::pure_issues::ArrayIssue;
use crate::typed::semantic_types::{
    semantic_values_are_compatible, SemanticTypeKey, SemanticValueType,
};
use crate::typed::type_ref::inferred_type_from_expr;
use crate::{TypedExpr, TypedType};
use seseragi_syntax::{ByteSpan, SurfaceExpr};

use super::{type_surface_expression, PureExpressionContext, SurfaceExpressionAnalysis};

#[derive(Clone, Copy)]
enum CollectionKind {
    Array,
    List,
}

impl CollectionKind {
    fn name(self) -> &'static str {
        match self {
            Self::Array => "Array",
            Self::List => "List",
        }
    }

    fn expression(
        self,
        elements: Vec<TypedExpr>,
        type_ref: TypedType,
        origin: ByteSpan,
    ) -> TypedExpr {
        match self {
            Self::Array => TypedExpr::Array {
                elements,
                type_ref,
                origin,
            },
            Self::List => TypedExpr::List {
                elements,
                type_ref,
                origin,
            },
        }
    }
}

pub(super) fn type_array(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_collection(elements, span, context, CollectionKind::Array)
}

pub(super) fn type_list(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
) -> SurfaceExpressionAnalysis {
    type_collection(elements, span, context, CollectionKind::List)
}

fn type_collection(
    elements: &[SurfaceExpr],
    span: ByteSpan,
    context: &PureExpressionContext<'_>,
    kind: CollectionKind,
) -> SurfaceExpressionAnalysis {
    let expected_element = expected_element(context, kind);
    if elements.is_empty() {
        return empty_collection(span, expected_element, kind);
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
                collection: kind.name(),
                element: elements[index].span(),
                index,
                expected: element_type.type_ref.clone(),
                actual: actual.type_ref,
            }
        })
    });
    let type_ref = TypedType::Named {
        name: kind.name().to_owned(),
        arguments: vec![if issue.is_some() {
            TypedType::Hole
        } else {
            element_type.type_ref
        }],
    };
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        kind.expression(
            children.iter().map(|child| child.value.clone()).collect(),
            type_ref,
            span,
        ),
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

fn expected_element(
    context: &PureExpressionContext<'_>,
    kind: CollectionKind,
) -> Option<SemanticValueType> {
    let TypedType::Named { name, arguments } = &context.expected()?.type_ref else {
        return None;
    };
    (name == kind.name() && arguments.len() == 1)
        .then(|| context.semantic_value_from_typed_type(&arguments[0]))
}

fn empty_collection(
    span: ByteSpan,
    expected: Option<SemanticValueType>,
    kind: CollectionKind,
) -> SurfaceExpressionAnalysis {
    let issue = expected
        .is_none()
        .then_some(ArrayIssue::EmptyWithoutExpectedType {
            collection: kind.name(),
            literal: span,
        });
    let element = expected
        .map(|expected| expected.type_ref)
        .unwrap_or(TypedType::Hole);
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        kind.expression(
            Vec::new(),
            TypedType::Named {
                name: kind.name().to_owned(),
                arguments: vec![element],
            },
            span,
        ),
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::Other
        },
    );
    result.array_issue = issue;
    result
}
