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
    let element_type = expected_element
        .map(|expected| refine_expected_element(expected, &inferred_element, context))
        .unwrap_or_else(|| inferred_element.clone());
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
            element_type.type_ref.clone()
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
            SemanticTypeKey::NamedGeneric {
                name: kind.name().to_owned(),
                arguments: vec![element_type],
            }
        },
    );
    result.array_issue = issue;
    for child in children {
        result.merge_issues_from(child);
    }
    result
}

fn refine_expected_element(
    expected: SemanticValueType,
    inferred: &SemanticValueType,
    context: &PureExpressionContext<'_>,
) -> SemanticValueType {
    let type_ref = fill_expected_holes(&expected.type_ref, &inferred.type_ref);
    if type_ref == expected.type_ref {
        expected
    } else {
        context.semantic_value_from_typed_type(&type_ref)
    }
}

fn fill_expected_holes(expected: &TypedType, inferred: &TypedType) -> TypedType {
    match (expected, inferred) {
        (TypedType::Hole, inferred) => inferred.clone(),
        (
            TypedType::Named {
                name: expected_name,
                arguments: expected_arguments,
            },
            TypedType::Named {
                name: inferred_name,
                arguments: inferred_arguments,
            },
        ) if expected_name == inferred_name
            && expected_arguments.len() == inferred_arguments.len() =>
        {
            TypedType::Named {
                name: expected_name.clone(),
                arguments: expected_arguments
                    .iter()
                    .zip(inferred_arguments)
                    .map(|(expected, inferred)| fill_expected_holes(expected, inferred))
                    .collect(),
            }
        }
        (
            TypedType::ExternalNamed {
                canonical,
                name,
                arguments: expected_arguments,
            },
            TypedType::ExternalNamed {
                canonical: inferred_canonical,
                arguments: inferred_arguments,
                ..
            },
        ) if canonical == inferred_canonical
            && expected_arguments.len() == inferred_arguments.len() =>
        {
            TypedType::ExternalNamed {
                canonical: canonical.clone(),
                name: name.clone(),
                arguments: expected_arguments
                    .iter()
                    .zip(inferred_arguments)
                    .map(|(expected, inferred)| fill_expected_holes(expected, inferred))
                    .collect(),
            }
        }
        (
            TypedType::Tuple {
                elements: expected_elements,
            },
            TypedType::Tuple {
                elements: inferred_elements,
            },
        ) if expected_elements.len() == inferred_elements.len() => TypedType::Tuple {
            elements: expected_elements
                .iter()
                .zip(inferred_elements)
                .map(|(expected, inferred)| fill_expected_holes(expected, inferred))
                .collect(),
        },
        (
            TypedType::Function {
                parameter: expected_parameter,
                result: expected_result,
            },
            TypedType::Function {
                parameter: inferred_parameter,
                result: inferred_result,
            },
        ) => TypedType::Function {
            parameter: Box::new(fill_expected_holes(expected_parameter, inferred_parameter)),
            result: Box::new(fill_expected_holes(expected_result, inferred_result)),
        },
        _ => expected.clone(),
    }
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
    let element = expected.unwrap_or(SemanticValueType {
        type_ref: TypedType::Hole,
        key: SemanticTypeKey::Invalid,
    });
    let mut result = SurfaceExpressionAnalysis::valid_with_semantic_type(
        kind.expression(
            Vec::new(),
            TypedType::Named {
                name: kind.name().to_owned(),
                arguments: vec![element.type_ref.clone()],
            },
            span,
        ),
        if issue.is_some() {
            SemanticTypeKey::Invalid
        } else {
            SemanticTypeKey::NamedGeneric {
                name: kind.name().to_owned(),
                arguments: vec![element],
            }
        },
    );
    result.array_issue = issue;
    result
}

#[cfg(test)]
mod tests {
    use super::fill_expected_holes;
    use crate::TypedType;

    fn named(name: &str, arguments: Vec<TypedType>) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments,
        }
    }

    #[test]
    fn refines_nested_expected_holes_from_the_first_element() {
        let expected = named(
            "Effect",
            vec![
                TypedType::Hole,
                named("FixedFailure", Vec::new()),
                TypedType::Hole,
            ],
        );
        let inferred = named(
            "Effect",
            vec![
                named("Environment", Vec::new()),
                named("DifferentFailure", Vec::new()),
                named("Int", Vec::new()),
            ],
        );

        assert_eq!(
            fill_expected_holes(&expected, &inferred),
            named(
                "Effect",
                vec![
                    named("Environment", Vec::new()),
                    named("FixedFailure", Vec::new()),
                    named("Int", Vec::new()),
                ],
            )
        );
    }
}
