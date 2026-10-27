use serde::{Deserialize, Serialize};
use seseragi_semantics::{TypedRecordField, TypedType};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum CoreType {
    TypeFunction {
        parameter_kinds: Vec<u32>,
        body: Box<CoreType>,
    },
    TypeFunctionParameter {
        depth: u32,
        index: u32,
    },
    Named {
        name: String,
        arguments: Vec<CoreType>,
    },
    ExternalNamed {
        name: String,
        canonical: String,
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
    RequirementMerge {
        operands: Vec<CoreType>,
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

impl CoreType {
    pub fn contains_type_function(&self) -> bool {
        match self {
            Self::TypeFunction { .. } | Self::TypeFunctionParameter { .. } => true,
            Self::Named { arguments, .. } | Self::ExternalNamed { arguments, .. } => {
                arguments.iter().any(Self::contains_type_function)
            }
            Self::Record { fields, .. } => fields
                .iter()
                .any(|field| field.type_ref.contains_type_function()),
            Self::Tuple { elements } => elements.iter().any(Self::contains_type_function),
            Self::Function { parameter, result } => {
                parameter.contains_type_function() || result.contains_type_function()
            }
            Self::RequirementMerge { operands } => {
                operands.iter().any(Self::contains_type_function)
            }
            Self::Hole => false,
        }
    }

    /// Validate lexical slot scopes and discard display-only nominal names.
    /// Canonical leaves remain structured; this never resolves a type name.
    pub fn normalized_type_function(&self) -> Option<Self> {
        if !matches!(self, Self::TypeFunction { .. }) {
            return None;
        }
        fn normalized(value: &CoreType, scopes: &[Vec<u32>]) -> Option<CoreType> {
            Some(match value {
                CoreType::TypeFunction {
                    parameter_kinds,
                    body,
                } => {
                    if parameter_kinds.is_empty() || parameter_kinds.iter().any(|kind| *kind != 0) {
                        return None;
                    }
                    let mut scopes = scopes.to_vec();
                    scopes.push(parameter_kinds.clone());
                    CoreType::TypeFunction {
                        parameter_kinds: parameter_kinds.clone(),
                        body: Box::new(normalized(body, &scopes)?),
                    }
                }
                CoreType::TypeFunctionParameter { depth, index } => {
                    if scopes
                        .iter()
                        .rev()
                        .nth(*depth as usize)?
                        .get(*index as usize)?
                        != &0
                    {
                        return None;
                    }
                    value.clone()
                }
                CoreType::ExternalNamed {
                    canonical,
                    arguments,
                    ..
                } => CoreType::ExternalNamed {
                    name: canonical.clone(),
                    canonical: canonical.clone(),
                    arguments: arguments
                        .iter()
                        .map(|argument| normalized(argument, scopes))
                        .collect::<Option<_>>()?,
                },
                CoreType::Record { closed, fields } => CoreType::Record {
                    closed: *closed,
                    fields: fields
                        .iter()
                        .map(|field| {
                            Some(CoreRecordField {
                                name: field.name.clone(),
                                optional: field.optional,
                                type_ref: normalized(&field.type_ref, scopes)?,
                            })
                        })
                        .collect::<Option<_>>()?,
                },
                CoreType::Tuple { elements } => CoreType::Tuple {
                    elements: elements
                        .iter()
                        .map(|element| normalized(element, scopes))
                        .collect::<Option<_>>()?,
                },
                CoreType::Function { parameter, result } => CoreType::Function {
                    parameter: Box::new(normalized(parameter, scopes)?),
                    result: Box::new(normalized(result, scopes)?),
                },
                // Runtime/inference holes and unqualified/free names provide
                // no proof for this closed type-function representation.
                CoreType::Named { .. } | CoreType::Hole | CoreType::RequirementMerge { .. } => {
                    return None
                }
            })
        }
        normalized(self, &[])
    }
}

impl seseragi_syntax::type_functions::TypeFunctionTerm for CoreType {
    fn type_function(&self) -> Option<(&[u32], &Self)> {
        match self {
            Self::TypeFunction {
                parameter_kinds,
                body,
            } => Some((parameter_kinds, body)),
            _ => None,
        }
    }
    fn type_function_parameter(&self) -> Option<(u32, u32)> {
        match self {
            Self::TypeFunctionParameter { depth, index } => Some((*depth, *index)),
            _ => None,
        }
    }
    fn from_type_function(parameter_kinds: Vec<u32>, body: Self) -> Self {
        Self::TypeFunction {
            parameter_kinds,
            body: Box::new(body),
        }
    }
    fn from_type_function_parameter(depth: u32, index: u32) -> Self {
        Self::TypeFunctionParameter { depth, index }
    }
    fn is_type_hole(&self) -> bool {
        matches!(self, Self::Hole)
    }
    fn map_type_children(&self, map: &mut dyn FnMut(&Self) -> Option<Self>) -> Option<Self> {
        Some(match self {
            Self::Named { name, arguments } => Self::Named {
                name: name.clone(),
                arguments: arguments.iter().map(map).collect::<Option<_>>()?,
            },
            Self::ExternalNamed {
                name,
                canonical,
                arguments,
            } => Self::ExternalNamed {
                name: name.clone(),
                canonical: canonical.clone(),
                arguments: arguments.iter().map(map).collect::<Option<_>>()?,
            },
            Self::Record { closed, fields } => Self::Record {
                closed: *closed,
                fields: fields
                    .iter()
                    .map(|field| {
                        Some(CoreRecordField {
                            name: field.name.clone(),
                            optional: field.optional,
                            type_ref: map(&field.type_ref)?,
                        })
                    })
                    .collect::<Option<_>>()?,
            },
            Self::Tuple { elements } => Self::Tuple {
                elements: elements.iter().map(map).collect::<Option<_>>()?,
            },
            Self::Function { parameter, result } => Self::Function {
                parameter: Box::new(map(parameter)?),
                result: Box::new(map(result)?),
            },
            Self::RequirementMerge { operands } => Self::RequirementMerge {
                operands: operands.iter().map(map).collect::<Option<_>>()?,
            },
            leaf => leaf.clone(),
        })
    }
}

pub(crate) fn lower_typed_type(type_ref: TypedType) -> CoreType {
    match type_ref {
        TypedType::TypeFunction {
            parameter_kinds,
            body,
        } => CoreType::TypeFunction {
            parameter_kinds,
            body: Box::new(lower_typed_type(*body)),
        },
        TypedType::TypeFunctionParameter { depth, index } => {
            CoreType::TypeFunctionParameter { depth, index }
        }
        TypedType::Named { name, arguments } => CoreType::Named {
            name,
            arguments: arguments.into_iter().map(lower_typed_type).collect(),
        },
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => CoreType::ExternalNamed {
            name,
            canonical,
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
        TypedType::RequirementMerge { operands } => CoreType::RequirementMerge {
            operands: operands.into_iter().map(lower_typed_type).collect(),
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
    fn type_function_proofs_preserve_scope_and_canonical_owner() {
        let function = |body| CoreType::TypeFunction {
            parameter_kinds: vec![0],
            body: Box::new(body),
        };
        let canonical = CoreType::ExternalNamed {
            name: "LocalDisplay".into(),
            canonical: "provider/model::Pair".into(),
            arguments: vec![CoreType::TypeFunctionParameter { depth: 0, index: 0 }],
        };
        let normalized = function(canonical).normalized_type_function().unwrap();
        assert!(normalized.contains_type_function());
        let CoreType::TypeFunction { body, .. } = normalized else {
            unreachable!()
        };
        let CoreType::ExternalNamed {
            name, canonical, ..
        } = *body
        else {
            unreachable!()
        };
        assert_eq!(name, canonical);
        assert_eq!(canonical, "provider/model::Pair");
        for unproved in [
            CoreType::TypeFunctionParameter { depth: 1, index: 0 },
            CoreType::TypeFunctionParameter { depth: 0, index: 1 },
            CoreType::Hole,
            CoreType::Named {
                name: "Pair".into(),
                arguments: vec![],
            },
        ] {
            assert_eq!(function(unproved).normalized_type_function(), None);
        }
        assert_eq!(
            CoreType::TypeFunction {
                parameter_kinds: vec![1],
                body: Box::new(CoreType::TypeFunctionParameter { depth: 0, index: 0 })
            }
            .normalized_type_function(),
            None
        );
    }

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
