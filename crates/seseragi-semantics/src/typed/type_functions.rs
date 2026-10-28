//! Capture-avoiding application of checked, compile-time type constructors.
//! A bound slot is distinct from a Hole, a nominal owner and a scheme binder.
use crate::{TypedRecordField, TypedType};

pub(crate) fn apply(constructor: &TypedType, arguments: Vec<TypedType>) -> Option<TypedType> {
    if arguments.is_empty() {
        return Some(constructor.clone());
    }
    match constructor {
        TypedType::TypeFunction { .. } => {
            seseragi_syntax::type_functions::apply_type_function(constructor, &arguments)
        }
        TypedType::Named {
            name,
            arguments: existing,
        } => Some(TypedType::Named {
            name: name.clone(),
            arguments: super::semantic_types::fill_constructor_holes(existing, arguments),
        }),
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments: existing,
        } => Some(TypedType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: super::semantic_types::fill_constructor_holes(existing, arguments),
        }),
        _ => None,
    }
}

fn map_children(
    value: &TypedType,
    mut map: impl FnMut(&TypedType) -> Option<TypedType>,
) -> Option<TypedType> {
    Some(match value {
        TypedType::Named { name, arguments } => TypedType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(&mut map).collect::<Option<_>>()?,
        },
        TypedType::ExternalNamed {
            name,
            canonical,
            arguments,
        } => TypedType::ExternalNamed {
            name: name.clone(),
            canonical: canonical.clone(),
            arguments: arguments.iter().map(&mut map).collect::<Option<_>>()?,
        },
        TypedType::Record { closed, fields } => TypedType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| {
                    Some(TypedRecordField {
                        name: field.name.clone(),
                        optional: field.optional,
                        type_ref: map(&field.type_ref)?,
                    })
                })
                .collect::<Option<_>>()?,
        },
        TypedType::Tuple { elements } => TypedType::Tuple {
            elements: elements.iter().map(&mut map).collect::<Option<_>>()?,
        },
        TypedType::Function { parameter, result } => TypedType::Function {
            parameter: Box::new(map(parameter)?),
            result: Box::new(map(result)?),
        },
        TypedType::RequirementMerge { operands } => TypedType::RequirementMerge {
            operands: operands.iter().map(&mut map).collect::<Option<_>>()?,
        },
        leaf => leaf.clone(),
    })
}

impl seseragi_syntax::type_functions::TypeFunctionTerm for TypedType {
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
        map_children(self, map)
    }
}

// Substitution must retain the selected semantic owners and free scheme/
// declaration binder identities alongside the display type. Reconstructing
// those keys from names after beta application would lose the original scope.
impl seseragi_syntax::type_functions::TypeFunctionTerm
    for super::semantic_types::SemanticValueType
{
    fn type_function(&self) -> Option<(&[u32], &Self)> {
        match (&self.type_ref, &self.key) {
            (
                TypedType::TypeFunction {
                    parameter_kinds, ..
                },
                super::semantic_types::SemanticTypeKey::TypeFunction {
                    parameter_kinds: checked,
                    body,
                },
            ) if parameter_kinds == checked => Some((parameter_kinds, body)),
            _ => None,
        }
    }
    fn type_function_parameter(&self) -> Option<(u32, u32)> {
        match self.type_ref {
            TypedType::TypeFunctionParameter { depth, index } => Some((depth, index)),
            _ => None,
        }
    }
    fn from_type_function(parameter_kinds: Vec<u32>, body: Self) -> Self {
        Self {
            type_ref: TypedType::TypeFunction {
                parameter_kinds: parameter_kinds.clone(),
                body: Box::new(body.type_ref.clone()),
            },
            key: super::semantic_types::SemanticTypeKey::TypeFunction {
                parameter_kinds,
                body: Box::new(body),
            },
        }
    }
    fn from_type_function_parameter(depth: u32, index: u32) -> Self {
        Self {
            type_ref: TypedType::TypeFunctionParameter { depth, index },
            key: super::semantic_types::SemanticTypeKey::Other,
        }
    }
    fn is_type_hole(&self) -> bool {
        matches!(self.type_ref, TypedType::Hole)
    }
    fn map_type_children(&self, map: &mut dyn FnMut(&Self) -> Option<Self>) -> Option<Self> {
        use super::semantic_types::{SemanticTypeKey as Key, SemanticValueType as Value};
        let mut key = self.key.clone();
        let type_ref = match (&self.type_ref, &mut key) {
            (
                TypedType::Named { name, .. },
                Key::Adt { arguments, .. }
                | Key::Struct { arguments, .. }
                | Key::NamedGeneric { arguments, .. },
            ) => {
                *arguments = arguments.iter().map(map).collect::<Option<_>>()?;
                TypedType::Named {
                    name: name.clone(),
                    arguments: arguments
                        .iter()
                        .map(|value| value.type_ref.clone())
                        .collect(),
                }
            }
            (
                TypedType::ExternalNamed {
                    name, canonical, ..
                },
                Key::Adt { arguments, .. }
                | Key::Struct { arguments, .. }
                | Key::ExternalNominal { arguments, .. },
            ) => {
                *arguments = arguments.iter().map(map).collect::<Option<_>>()?;
                TypedType::ExternalNamed {
                    name: name.clone(),
                    canonical: canonical.clone(),
                    arguments: arguments
                        .iter()
                        .map(|value| value.type_ref.clone())
                        .collect(),
                }
            }
            (TypedType::Tuple { elements }, Key::Tuple(keys)) if elements.len() == keys.len() => {
                let values = elements
                    .iter()
                    .zip(keys.iter())
                    .map(|(type_ref, key)| {
                        map(&Value {
                            type_ref: type_ref.clone(),
                            key: key.clone(),
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                *keys = values.iter().map(|value| value.key.clone()).collect();
                TypedType::Tuple {
                    elements: values.into_iter().map(|value| value.type_ref).collect(),
                }
            }
            (TypedType::Record { closed, fields }, Key::Record(values)) => {
                let fields = fields
                    .iter()
                    .map(|field| {
                        let value = map(values.get(&field.name)?)?;
                        values.insert(field.name.clone(), value.clone());
                        Some(TypedRecordField {
                            name: field.name.clone(),
                            optional: field.optional,
                            type_ref: value.type_ref,
                        })
                    })
                    .collect::<Option<_>>()?;
                TypedType::Record {
                    closed: *closed,
                    fields,
                }
            }
            (TypedType::Function { .. }, Key::Function { parameter, result }) => {
                **parameter = map(parameter)?;
                **result = map(result)?;
                TypedType::Function {
                    parameter: Box::new(parameter.type_ref.clone()),
                    result: Box::new(result.type_ref.clone()),
                }
            }
            // These carry no children. Unknown composite keys cannot supply a
            // proof for a constructor application and are not guessed here.
            (TypedType::TypeFunctionParameter { .. }, Key::Other)
            | (TypedType::Named { .. }, Key::TypeParameter(_) | Key::SchemeParameter(_)) => {
                self.type_ref.clone()
            }
            (TypedType::Named { arguments, .. }, Key::Other) if arguments.is_empty() => {
                self.type_ref.clone()
            }
            _ => return None,
        };
        Some(Value { type_ref, key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.into(),
            arguments: vec![],
        }
    }

    fn slot(depth: u32, index: u32) -> TypedType {
        TypedType::TypeFunctionParameter { depth, index }
    }

    #[test]
    fn substitutes_permuted_repeated_and_unused_bound_parameters() {
        for (arity, slots, expected) in [
            (
                2,
                vec![slot(0, 1), slot(0, 0)],
                vec![named("Bool"), named("Int")],
            ),
            (
                1,
                vec![slot(0, 0), slot(0, 0)],
                vec![named("Int"), named("Int")],
            ),
            (1, vec![], vec![]),
        ] {
            let value = TypedType::TypeFunction {
                parameter_kinds: vec![0; arity],
                body: Box::new(TypedType::Named {
                    name: "Pair".into(),
                    arguments: slots,
                }),
            };
            let arguments = vec![named("Int"), named("Bool")][..arity].to_vec();
            assert_eq!(
                apply(&value, arguments),
                Some(TypedType::Named {
                    name: "Pair".into(),
                    arguments: expected
                })
            );
        }
    }

    #[test]
    fn partial_application_keeps_the_remaining_binder_and_does_not_capture() {
        let value = TypedType::TypeFunction {
            parameter_kinds: vec![0, 0],
            body: Box::new(TypedType::Tuple {
                elements: vec![slot(0, 0), slot(0, 1)],
            }),
        };
        let partial = apply(&value, vec![slot(0, 0)]).unwrap();
        assert_eq!(
            partial,
            TypedType::TypeFunction {
                parameter_kinds: vec![0],
                body: Box::new(TypedType::Tuple {
                    elements: vec![slot(1, 0), slot(0, 0)]
                }),
            }
        );
    }

    #[test]
    fn application_preserves_nested_scopes_and_removes_only_its_own_binder() {
        let nested = TypedType::TypeFunction {
            parameter_kinds: vec![0],
            body: Box::new(TypedType::TypeFunction {
                parameter_kinds: vec![0],
                body: Box::new(TypedType::Tuple {
                    elements: vec![slot(0, 0), slot(1, 0), slot(2, 0)],
                }),
            }),
        };
        assert_eq!(
            apply(&nested, vec![slot(0, 1)]),
            Some(TypedType::TypeFunction {
                parameter_kinds: vec![0],
                body: Box::new(TypedType::Tuple {
                    elements: vec![slot(0, 0), slot(1, 1), slot(1, 0)]
                }),
            })
        );
    }

    #[test]
    fn semantic_application_preserves_nominal_owner_and_free_binder_identity() {
        use super::super::semantic_types::{SemanticTypeKey as Key, SemanticValueType as Value};
        use crate::SymbolId;
        use seseragi_syntax::type_functions::{apply_type_function, TypeFunctionTerm};
        let parameter = Value::from_type_function_parameter(0, 0);
        let body = Value {
            type_ref: TypedType::ExternalNamed {
                name: "Pair".into(),
                canonical: "provider/model::Pair".into(),
                arguments: vec![parameter.type_ref.clone(), parameter.type_ref.clone()],
            },
            key: Key::Adt {
                owner: SymbolId(11),
                arguments: vec![parameter.clone(), parameter],
            },
        };
        let constructor = Value::from_type_function(vec![0], body);
        let argument = Value {
            type_ref: named("T"),
            key: Key::TypeParameter(SymbolId(7)),
        };
        let result = apply_type_function(&constructor, &[argument.clone()]).unwrap();
        let Key::Adt { owner, arguments } = result.key else {
            panic!("{result:?}")
        };
        assert_eq!(owner, SymbolId(11));
        assert_eq!(arguments, vec![argument.clone(), argument]);
        let same_spelling_other_binder = Value {
            type_ref: named("T"),
            key: Key::TypeParameter(SymbolId(8)),
        };
        let other =
            apply_type_function(&constructor, &[same_spelling_other_binder.clone()]).unwrap();
        let Key::Adt { owner, arguments } = other.key else {
            panic!("{other:?}")
        };
        assert_eq!(owner, SymbolId(11));
        assert_eq!(
            arguments,
            vec![
                same_spelling_other_binder.clone(),
                same_spelling_other_binder
            ]
        );
    }

    #[test]
    fn an_unused_bound_parameter_keeps_its_kind_but_holes_and_overapplication_fail() {
        let constant = TypedType::TypeFunction {
            parameter_kinds: vec![0],
            body: Box::new(named("Int")),
        };
        assert_eq!(apply(&constant, vec![]), Some(constant.clone()));
        assert_eq!(apply(&constant, vec![named("Bool")]), Some(named("Int")));
        assert_eq!(apply(&constant, vec![TypedType::Hole]), None);
        assert_eq!(apply(&constant, vec![named("Bool"), named("Int")]), None);
    }
}
