use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedRecordField, TypedType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreType {
    Named {
        name: String,
        arguments: Vec<CoreType>,
    },
    Hole,
    Record {
        closed: bool,
        fields: Vec<CoreRecordField>,
    },
    Tuple {
        elements: Vec<CoreType>,
    },
    Function {
        parameter: Box<CoreType>,
        result: Box<CoreType>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreRecordField {
    pub name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub optional: bool,
    #[serde(rename = "type")]
    pub type_ref: CoreType,
}

pub(crate) fn lower_typed_type(type_ref: TypedType) -> CoreType {
    match type_ref {
        TypedType::Named { name, arguments } => CoreType::Named {
            name,
            arguments: arguments.into_iter().map(lower_typed_type).collect(),
        },
        TypedType::Hole => CoreType::Hole,
        TypedType::Record { closed, fields } => CoreType::Record {
            closed,
            fields: fields.into_iter().map(lower_record_field).collect(),
        },
        TypedType::Tuple { elements } => CoreType::Tuple {
            elements: elements.into_iter().map(lower_typed_type).collect(),
        },
        TypedType::Function { parameter, result } => CoreType::Function {
            parameter: Box::new(lower_typed_type(*parameter)),
            result: Box::new(lower_typed_type(*result)),
        },
    }
}

fn lower_record_field(field: TypedRecordField) -> CoreRecordField {
    CoreRecordField {
        name: field.name,
        optional: field.optional,
        type_ref: lower_typed_type(field.type_ref),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_nested_named_type_arguments() {
        let type_ref = lower_typed_type(TypedType::Named {
            name: "Maybe".to_owned(),
            arguments: vec![TypedType::Named {
                name: "String".to_owned(),
                arguments: Vec::new(),
            }],
        });

        assert_eq!(
            type_ref,
            CoreType::Named {
                name: "Maybe".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
            }
        );
    }
}
