use seseragi_syntax::{
    TypeDifference, TypeDifferenceEntry, TypeDifferenceKind, TypeDifferencePathSegment,
};

use crate::{TypeDocument, TypeDocumentField, TypeRenderOptions, TypedType};

pub(super) fn type_difference(expected: &TypedType, actual: &TypedType) -> Option<TypeDifference> {
    let expected = TypeDocument::from_typed_type(expected);
    let actual = TypeDocument::from_typed_type(actual);
    if contains_unknown(&expected) || contains_unknown(&actual) {
        return None;
    }

    let mut entries = Vec::new();
    collect_differences(&expected, &actual, &mut Vec::new(), &mut entries);
    if entries.is_empty() && expected != actual {
        entries.push(type_mismatch(&[], &expected, &actual));
    }
    if entries.is_empty() {
        return None;
    }

    Some(TypeDifference {
        expected_type: render(&expected),
        actual_type: render(&actual),
        entries,
    })
}

fn collect_differences(
    expected: &TypeDocument,
    actual: &TypeDocument,
    path: &mut Vec<TypeDifferencePathSegment>,
    entries: &mut Vec<TypeDifferenceEntry>,
) {
    match (expected, actual) {
        (
            TypeDocument::Named {
                name: expected_name,
                canonical: expected_canonical,
                arguments: expected_arguments,
            },
            TypeDocument::Named {
                name: actual_name,
                canonical: actual_canonical,
                arguments: actual_arguments,
            },
        ) if expected_name == actual_name
            && expected_canonical == actual_canonical
            && expected_arguments.len() == actual_arguments.len() =>
        {
            for (index, (expected, actual)) in
                expected_arguments.iter().zip(actual_arguments).enumerate()
            {
                path.push(TypeDifferencePathSegment::TypeArgument {
                    name: expected_name.clone(),
                    index,
                });
                collect_differences(expected, actual, path, entries);
                path.pop();
            }
        }
        (
            TypeDocument::Variable {
                name: expected_name,
                arity: expected_arity,
                arguments: expected_arguments,
            },
            TypeDocument::Variable {
                name: actual_name,
                arity: actual_arity,
                arguments: actual_arguments,
            },
        ) if expected_name == actual_name
            && expected_arity == actual_arity
            && expected_arguments.len() == actual_arguments.len() =>
        {
            for (index, (expected, actual)) in
                expected_arguments.iter().zip(actual_arguments).enumerate()
            {
                path.push(TypeDifferencePathSegment::TypeArgument {
                    name: expected_name.clone(),
                    index,
                });
                collect_differences(expected, actual, path, entries);
                path.pop();
            }
        }
        (
            TypeDocument::Function {
                parameters: expected_parameters,
                result: expected_result,
            },
            TypeDocument::Function {
                parameters: actual_parameters,
                result: actual_result,
            },
        ) => {
            for (index, (expected, actual)) in expected_parameters
                .iter()
                .zip(actual_parameters)
                .enumerate()
            {
                path.push(TypeDifferencePathSegment::FunctionParameter { index });
                collect_differences(expected, actual, path, entries);
                path.pop();
            }
            for (index, expected) in expected_parameters
                .iter()
                .enumerate()
                .skip(actual_parameters.len())
            {
                let mut entry_path = path.clone();
                entry_path.push(TypeDifferencePathSegment::FunctionParameter { index });
                entries.push(TypeDifferenceEntry {
                    message: format!(
                        "{} is missing; expected {}",
                        path_label(&entry_path),
                        render(expected)
                    ),
                    path: entry_path,
                    kind: TypeDifferenceKind::MissingFunctionParameter,
                    expected_type: Some(render(expected)),
                    actual_type: None,
                });
            }
            for (index, actual) in actual_parameters
                .iter()
                .enumerate()
                .skip(expected_parameters.len())
            {
                let mut entry_path = path.clone();
                entry_path.push(TypeDifferencePathSegment::FunctionParameter { index });
                entries.push(TypeDifferenceEntry {
                    message: format!(
                        "{} is extra; actual type is {}",
                        path_label(&entry_path),
                        render(actual)
                    ),
                    path: entry_path,
                    kind: TypeDifferenceKind::ExtraFunctionParameter,
                    expected_type: None,
                    actual_type: Some(render(actual)),
                });
            }
            path.push(TypeDifferencePathSegment::FunctionResult);
            collect_differences(expected_result, actual_result, path, entries);
            path.pop();
        }
        (
            TypeDocument::Record {
                fields: expected_fields,
                ..
            },
            TypeDocument::Record {
                fields: actual_fields,
                ..
            },
        ) => collect_record_differences(expected_fields, actual_fields, path, entries),
        (
            TypeDocument::Tuple {
                elements: expected_elements,
            },
            TypeDocument::Tuple {
                elements: actual_elements,
            },
        ) if expected_elements.len() == actual_elements.len() => {
            for (index, (expected, actual)) in
                expected_elements.iter().zip(actual_elements).enumerate()
            {
                path.push(TypeDifferencePathSegment::TupleElement { index });
                collect_differences(expected, actual, path, entries);
                path.pop();
            }
        }
        _ if expected == actual => {}
        _ => entries.push(type_mismatch(path, expected, actual)),
    }
}

fn collect_record_differences(
    expected_fields: &[TypeDocumentField],
    actual_fields: &[TypeDocumentField],
    path: &mut Vec<TypeDifferencePathSegment>,
    entries: &mut Vec<TypeDifferenceEntry>,
) {
    for expected in expected_fields {
        let Some(actual) = actual_fields
            .iter()
            .find(|actual| actual.name == expected.name)
        else {
            if !expected.optional {
                let mut entry_path = path.clone();
                entry_path.push(TypeDifferencePathSegment::RecordField {
                    name: expected.name.clone(),
                });
                entries.push(TypeDifferenceEntry {
                    message: format!(
                        "{} is missing; expected {}",
                        path_label(&entry_path),
                        render(&expected.type_ref)
                    ),
                    path: entry_path,
                    kind: TypeDifferenceKind::MissingRecordField,
                    expected_type: Some(render(&expected.type_ref)),
                    actual_type: None,
                });
            }
            continue;
        };

        path.push(TypeDifferencePathSegment::RecordField {
            name: expected.name.clone(),
        });
        if !expected.optional && actual.optional {
            entries.push(TypeDifferenceEntry {
                message: format!(
                    "{} is required, but the actual field is optional",
                    path_label(path)
                ),
                path: path.clone(),
                kind: TypeDifferenceKind::FieldOptionality,
                expected_type: Some(render(&expected.type_ref)),
                actual_type: Some(render(&actual.type_ref)),
            });
        }
        collect_differences(&expected.type_ref, &actual.type_ref, path, entries);
        path.pop();
    }

    for actual in actual_fields {
        if expected_fields
            .iter()
            .any(|expected| expected.name == actual.name)
        {
            continue;
        }
        let mut entry_path = path.clone();
        entry_path.push(TypeDifferencePathSegment::RecordField {
            name: actual.name.clone(),
        });
        entries.push(TypeDifferenceEntry {
            message: format!(
                "{} is extra; actual type is {}",
                path_label(&entry_path),
                render(&actual.type_ref)
            ),
            path: entry_path,
            kind: TypeDifferenceKind::ExtraRecordField,
            expected_type: None,
            actual_type: Some(render(&actual.type_ref)),
        });
    }
}

fn type_mismatch(
    path: &[TypeDifferencePathSegment],
    expected: &TypeDocument,
    actual: &TypeDocument,
) -> TypeDifferenceEntry {
    let expected_type = render(expected);
    let actual_type = render(actual);
    let location = path_label(path);
    TypeDifferenceEntry {
        message: if location == "type" {
            format!("expected {expected_type}, actual {actual_type}")
        } else {
            format!("{location}: expected {expected_type}, actual {actual_type}")
        },
        path: path.to_vec(),
        kind: TypeDifferenceKind::TypeMismatch,
        expected_type: Some(expected_type),
        actual_type: Some(actual_type),
    }
}

fn contains_unknown(document: &TypeDocument) -> bool {
    match document {
        TypeDocument::Unknown => true,
        TypeDocument::Named { arguments, .. } | TypeDocument::Variable { arguments, .. } => {
            arguments.iter().any(contains_unknown)
        }
        TypeDocument::Function { parameters, result } => {
            parameters.iter().any(contains_unknown) || contains_unknown(result)
        }
        TypeDocument::Tuple { elements } => elements.iter().any(contains_unknown),
        TypeDocument::Record { fields, .. } => {
            fields.iter().any(|field| contains_unknown(&field.type_ref))
        }
        TypeDocument::TypeConstructor { .. } => false,
    }
}

fn render(document: &TypeDocument) -> String {
    document.render(TypeRenderOptions::default())
}

fn path_label(path: &[TypeDifferencePathSegment]) -> String {
    let mut rendered = String::new();
    for segment in path {
        match segment {
            TypeDifferencePathSegment::RecordField { name } => {
                if !rendered.is_empty() {
                    rendered.push('.');
                }
                rendered.push_str(name);
            }
            TypeDifferencePathSegment::FunctionParameter { index } => {
                push_path_part(&mut rendered, &format!("parameter {}", index + 1));
            }
            TypeDifferencePathSegment::FunctionResult => {
                push_path_part(&mut rendered, "return type");
            }
            TypeDifferencePathSegment::TypeArgument { name, index } => {
                push_path_part(
                    &mut rendered,
                    &format!("{name} type argument {}", index + 1),
                );
            }
            TypeDifferencePathSegment::TupleElement { index } => {
                push_path_part(&mut rendered, &format!("tuple element {}", index + 1));
            }
        }
    }
    if rendered.is_empty() {
        "type".to_owned()
    } else {
        rendered
    }
}

fn push_path_part(rendered: &mut String, part: &str) {
    if !rendered.is_empty() {
        rendered.push_str(" > ");
    }
    rendered.push_str(part);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TypedRecordField;

    #[test]
    fn reports_nested_record_fields_in_source_order() {
        let expected = record(vec![
            field(
                "profile",
                record(vec![
                    field("name", named("String")),
                    field("score", named("Int")),
                ]),
            ),
            field("enabled", named("Bool")),
        ]);
        let actual = record(vec![
            field(
                "profile",
                record(vec![
                    field("name", named("String")),
                    field("extra", named("Bool")),
                ]),
            ),
            field("stale", named("Int")),
        ]);

        let difference = type_difference(&expected, &actual).unwrap();
        assert_eq!(
            difference
                .entries
                .iter()
                .map(|entry| entry.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "profile.score is missing; expected Int",
                "profile.extra is extra; actual type is Bool",
                "enabled is missing; expected Bool",
                "stale is extra; actual type is Int",
            ]
        );
    }

    #[test]
    fn reports_function_parameter_and_return_paths() {
        let expected = function(named("Int"), function(named("String"), named("Bool")));
        let actual = function(named("String"), function(named("String"), named("Int")));

        let difference = type_difference(&expected, &actual).unwrap();
        assert_eq!(difference.entries.len(), 2);
        assert_eq!(
            difference.entries[0].message,
            "parameter 1: expected Int, actual String"
        );
        assert_eq!(
            difference.entries[1].message,
            "return type: expected Bool, actual Int"
        );
    }

    #[test]
    fn reports_the_nested_generic_argument_path() {
        let expected = named_with("Array", vec![named_with("Maybe", vec![named("Int")])]);
        let actual = named_with("Array", vec![named_with("Maybe", vec![named("String")])]);

        let difference = type_difference(&expected, &actual).unwrap();
        assert_eq!(difference.entries.len(), 1);
        assert_eq!(
            difference.entries[0].message,
            "Array type argument 1 > Maybe type argument 1: expected Int, actual String"
        );
    }

    #[test]
    fn suppresses_differences_that_contain_recovery_holes() {
        assert!(type_difference(&named("Int"), &TypedType::Hole).is_none());
    }

    #[test]
    fn unresolved_arguments_do_not_cascade_into_type_differences() {
        let artifact = crate::semantic_diagnostics(
            "recovery.ssrg",
            concat!(
                "fn accept value: { count: Int } -> Int = value.count\n",
                "pub fn bad -> Int = accept missing\n",
            ),
        );

        assert_eq!(artifact.diagnostics.len(), 1, "{artifact:#?}");
        assert_eq!(artifact.diagnostics[0].code, "SES-N0001");
        assert!(artifact.diagnostics[0].type_difference.is_none());
    }

    fn named(name: &str) -> TypedType {
        named_with(name, Vec::new())
    }

    fn named_with(name: &str, arguments: Vec<TypedType>) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments,
        }
    }

    fn function(parameter: TypedType, result: TypedType) -> TypedType {
        TypedType::Function {
            parameter: Box::new(parameter),
            result: Box::new(result),
        }
    }

    fn record(fields: Vec<TypedRecordField>) -> TypedType {
        TypedType::Record {
            closed: true,
            fields,
        }
    }

    fn field(name: &str, type_ref: TypedType) -> TypedRecordField {
        TypedRecordField {
            name: name.to_owned(),
            optional: false,
            type_ref,
        }
    }
}
