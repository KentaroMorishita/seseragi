use crate::{CoreExpr, CoreParameter, CoreType};
use std::collections::BTreeMap;

use super::names::{local_name, safe_identifier};
use super::{TypeScriptParameter, TypeScriptType};

pub(super) fn type_ref_from_core_expr(
    expr: &CoreExpr,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptType {
    match expr {
        CoreExpr::Unit { .. } => TypeScriptType::Undefined,
        CoreExpr::Int64 { .. } => TypeScriptType::Bigint,
        CoreExpr::String { .. } => TypeScriptType::String,
        CoreExpr::Template { .. } => TypeScriptType::String,
        CoreExpr::Boolean { .. } => TypeScriptType::Boolean,
        CoreExpr::Variable { type_ref, .. }
        | CoreExpr::Call { type_ref, .. }
        | CoreExpr::Lambda { type_ref, .. }
        | CoreExpr::Tuple { type_ref, .. }
        | CoreExpr::FieldAccess { type_ref, .. }
        | CoreExpr::OptionalFieldAccess { type_ref, .. }
        | CoreExpr::Record { type_ref, .. }
        | CoreExpr::Array { type_ref, .. }
        | CoreExpr::List { type_ref, .. }
        | CoreExpr::ArrayComprehension { type_ref, .. }
        | CoreExpr::ListComprehension { type_ref, .. }
        | CoreExpr::Binary { type_ref, .. }
        | CoreExpr::If { type_ref, .. }
        | CoreExpr::Decision { type_ref, .. }
        | CoreExpr::MonadDo { type_ref, .. } => type_ref_from_core_type(type_ref, imported_types),
        CoreExpr::EffectOperation { success, .. } | CoreExpr::EffectInvoke { success, .. } => {
            type_ref_from_core_type(success, imported_types)
        }
        CoreExpr::Sequence { result, .. } => type_ref_from_core_expr(result, imported_types),
    }
}

pub(super) fn lower_core_parameter_to_typescript(
    parameter: CoreParameter,
    imported_types: &BTreeMap<String, String>,
    type_constructor_parameters: &[String],
) -> TypeScriptParameter {
    TypeScriptParameter {
        name: if parameter.kind == "implicit" {
            "_unit".to_owned()
        } else {
            safe_identifier(&parameter.id)
        },
        type_name: render_typescript_type(&type_ref_from_core_type_with_erasure(
            &parameter.type_ref,
            imported_types,
            type_constructor_parameters,
        )),
        implicit: parameter.kind == "implicit",
    }
}

pub(super) fn type_ref_from_core_type(
    type_ref: &CoreType,
    imported_types: &BTreeMap<String, String>,
) -> TypeScriptType {
    type_ref_from_core_type_with_erasure(type_ref, imported_types, &[])
}

pub(super) fn type_ref_from_core_type_with_erasure(
    type_ref: &CoreType,
    imported_types: &BTreeMap<String, String>,
    type_constructor_parameters: &[String],
) -> TypeScriptType {
    match type_ref {
        CoreType::Named { name, arguments }
            if !arguments.is_empty()
                && type_constructor_parameters
                    .iter()
                    .any(|parameter| parameter == name) =>
        {
            TypeScriptType::Unknown
        }
        CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty() => {
            TypeScriptType::Bigint
        }
        CoreType::Named { name, arguments } if name == "String" && arguments.is_empty() => {
            TypeScriptType::String
        }
        CoreType::Named { name, arguments } if name == "Bool" && arguments.is_empty() => {
            TypeScriptType::Boolean
        }
        CoreType::Named { name, arguments } if name == "Unit" && arguments.is_empty() => {
            TypeScriptType::Undefined
        }
        CoreType::Named { name, arguments } if name == "Never" && arguments.is_empty() => {
            TypeScriptType::Never
        }
        CoreType::Named { name, arguments } if name == "Maybe" && arguments.len() == 1 => {
            TypeScriptType::Maybe {
                element: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "Either" && arguments.len() == 2 => {
            TypeScriptType::Either {
                error: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
                value: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[1],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "Array" && arguments.len() == 1 => {
            TypeScriptType::Array {
                element: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments } if name == "List" && arguments.len() == 1 => {
            TypeScriptType::List {
                element: Box::new(type_ref_from_core_type_with_erasure(
                    &arguments[0],
                    imported_types,
                    type_constructor_parameters,
                )),
            }
        }
        CoreType::Named { name, arguments }
            if name == "Range"
                && matches!(
                    arguments.as_slice(),
                    [CoreType::Named { name, arguments }] if name == "Int" && arguments.is_empty()
                ) =>
        {
            TypeScriptType::Range
        }
        CoreType::Named { name, arguments } => TypeScriptType::Reference {
            name: local_name(name),
            arguments: arguments
                .iter()
                .map(|argument| {
                    type_ref_from_core_type_with_erasure(
                        argument,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypeScriptType::Reference {
            name: imported_types
                .get(canonical)
                .cloned()
                .unwrap_or_else(|| {
                    panic!(
                        "external type {canonical} ({name}) was not validated by module import planning"
                    )
                }),
            arguments: arguments
                .iter()
                .map(|argument| {
                    type_ref_from_core_type_with_erasure(
                        argument,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::Tuple { elements } => TypeScriptType::Tuple {
            elements: elements
                .iter()
                .map(|element| {
                    type_ref_from_core_type_with_erasure(
                        element,
                        imported_types,
                        type_constructor_parameters,
                    )
                })
                .collect(),
        },
        CoreType::Record { fields, .. } => TypeScriptType::Record {
            fields: fields
                .iter()
                .map(|field| super::TypeScriptRecordTypeField {
                    name: field.name.clone(),
                    optional: field.optional,
                    type_ref: type_ref_from_core_type_with_erasure(
                        &field.type_ref,
                        imported_types,
                        type_constructor_parameters,
                    ),
                })
                .collect(),
        },
        CoreType::Function { parameter, result } => TypeScriptType::Function {
            parameter: Box::new(type_ref_from_core_type_with_erasure(
                parameter,
                imported_types,
                type_constructor_parameters,
            )),
            result: Box::new(type_ref_from_core_type_with_erasure(
                result,
                imported_types,
                type_constructor_parameters,
            )),
        },
        CoreType::Hole => TypeScriptType::Unknown,
    }
}

pub(crate) fn render_typescript_type(type_ref: &TypeScriptType) -> String {
    match type_ref {
        TypeScriptType::Bigint => "bigint".to_owned(),
        TypeScriptType::Boolean => "boolean".to_owned(),
        TypeScriptType::String => "string".to_owned(),
        TypeScriptType::Undefined => "undefined".to_owned(),
        TypeScriptType::Never => "never".to_owned(),
        TypeScriptType::Unknown => "unknown".to_owned(),
        TypeScriptType::Reference { name, arguments } if arguments.is_empty() => name.clone(),
        TypeScriptType::Reference { name, arguments } => format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptType::Maybe { element } => {
            format!(
                "{{ readonly tag: \"Nothing\" }} | {{ readonly tag: \"Just\"; readonly value: {} }}",
                render_typescript_type(element)
            )
        }
        TypeScriptType::Either { error, value } => {
            format!(
                "{{ readonly tag: \"Left\"; readonly value: {} }} | {{ readonly tag: \"Right\"; readonly value: {} }}",
                render_typescript_type(error),
                render_typescript_type(value)
            )
        }
        TypeScriptType::Tuple { elements } => format!(
            "readonly [{}]",
            elements
                .iter()
                .map(render_typescript_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeScriptType::Record { fields } => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|field| format!(
                    "readonly {}{}: {}",
                    format!("{:?}", field.name),
                    if field.optional { "?" } else { "" },
                    render_typescript_type(&field.type_ref)
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ),
        TypeScriptType::Array { element } => {
            format!("ReadonlyArray<{}>", render_typescript_type(element))
        }
        TypeScriptType::List { element } => {
            format!("List<{}>", render_typescript_type(element))
        }
        TypeScriptType::Range => {
            "Readonly<{ start: bigint; end: bigint; inclusive: boolean }>".to_owned()
        }
        TypeScriptType::Function { parameter, result } => format!(
            "(argument: {}) => {}",
            render_typescript_type(parameter),
            render_typescript_type(result)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        lower_core_parameter_to_typescript, render_typescript_type, type_ref_from_core_type,
        CoreParameter, CoreType,
    };
    use crate::TypeScriptType;

    #[test]
    fn renders_qualified_nominal_types_with_local_backend_names() {
        let type_ref = CoreType::Named {
            name: "artifact/domain::Hand".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "Hand"
        );
    }

    #[test]
    fn renders_maybe_as_a_tagged_union() {
        let type_ref = CoreType::Named {
            name: "Maybe".to_owned(),
            arguments: vec![CoreType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }],
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Maybe {
                element: Box::new(TypeScriptType::String),
            }
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "{ readonly tag: \"Nothing\" } | { readonly tag: \"Just\"; readonly value: string }"
        );
    }

    #[test]
    fn renders_either_as_a_tagged_union() {
        let type_ref = CoreType::Named {
            name: "Either".to_owned(),
            arguments: vec![
                CoreType::Named {
                    name: "InputError".to_owned(),
                    arguments: Vec::new(),
                },
                CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                },
            ],
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Either {
                error: Box::new(TypeScriptType::Reference {
                    name: "InputError".to_owned(),
                    arguments: Vec::new(),
                }),
                value: Box::new(TypeScriptType::String),
            }
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "{ readonly tag: \"Left\"; readonly value: InputError } | { readonly tag: \"Right\"; readonly value: string }"
        );
    }

    #[test]
    fn renders_never_as_the_typescript_bottom_type() {
        let type_ref = CoreType::Named {
            name: "Never".to_owned(),
            arguments: Vec::new(),
        };

        assert_eq!(
            type_ref_from_core_type(&type_ref, &Default::default()),
            TypeScriptType::Never
        );
        assert_eq!(
            render_typescript_type(&type_ref_from_core_type(&type_ref, &Default::default())),
            "never"
        );
    }

    #[test]
    fn erases_higher_kinded_parameter_applications_at_the_typescript_boundary() {
        let parameter = CoreParameter {
            id: "value".to_owned(),
            kind: "named".to_owned(),
            type_ref: CoreType::Named {
                name: "F".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                }],
            },
        };

        let lowered =
            lower_core_parameter_to_typescript(parameter, &Default::default(), &["F".to_owned()]);

        assert_eq!(lowered.type_name, "unknown");
    }
}
