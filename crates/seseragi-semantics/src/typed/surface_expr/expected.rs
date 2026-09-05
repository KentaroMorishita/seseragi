use super::PureExpressionContext;
use crate::typed::semantic_types::SemanticValueType;
use crate::TypedType;

pub(super) fn refine_expected_type(
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
            TypedType::Record { closed, fields },
            TypedType::Record {
                fields: inferred_fields,
                ..
            },
        ) => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| {
                    let type_ref = inferred_fields
                        .iter()
                        .find(|inferred| inferred.name == field.name)
                        .map(|inferred| fill_expected_holes(&field.type_ref, &inferred.type_ref))
                        .unwrap_or_else(|| field.type_ref.clone());
                    crate::TypedRecordField {
                        name: field.name.clone(),
                        optional: field.optional,
                        type_ref,
                    }
                })
                .collect(),
        },
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
