use crate::{TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType};
use seseragi_syntax::SurfaceDecl;
use std::collections::BTreeMap;

use super::instances::{canonical_instance_head_identity, canonical_type_ref};
use super::semantic_types::semantic_values_are_compatible;
use super::TypedResolution;

/// Selects evidence for constraints attached to a saturated function call.
///
/// This is deliberately a small registry boundary. The first implemented
/// standard instances are registered here; local and imported instance
/// search can extend this module without teaching expression typing about
/// compiler-private standard names.
pub(crate) fn select_call_evidence(
    constraints: &[TypedConstraint],
) -> Result<Vec<TypedCallEvidence>, TypedConstraint> {
    constraints
        .iter()
        .cloned()
        .map(|constraint| {
            let identity =
                standard_instance_identity(&constraint).ok_or_else(|| constraint.clone())?;
            Ok(TypedCallEvidence {
                constraint,
                evidence: TypedInstanceEvidence::Standard { identity },
            })
        })
        .collect()
}

pub(crate) fn select_trait_call_evidence(
    constraints: &[TypedConstraint],
    trait_identity: &str,
    resolution: &TypedResolution<'_>,
) -> Result<Vec<TypedCallEvidence>, TypedConstraint> {
    constraints
        .iter()
        .cloned()
        .map(|constraint| {
            let evidence = if let Some(identity) =
                local_instance_identity(trait_identity, &constraint, resolution)
            {
                TypedInstanceEvidence::Local { identity }
            } else if let Some(identity) = standard_instance_identity(&constraint) {
                TypedInstanceEvidence::Standard {
                    identity: identity.to_owned(),
                }
            } else {
                return Err(constraint);
            };
            Ok(TypedCallEvidence {
                constraint,
                evidence,
            })
        })
        .collect()
}

fn local_instance_identity(
    trait_identity: &str,
    constraint: &TypedConstraint,
    resolution: &TypedResolution<'_>,
) -> Option<String> {
    let actual_arguments = constraint
        .arguments
        .iter()
        .map(|argument| resolution.semantic_value_from_typed_type(argument))
        .collect::<Vec<_>>();
    let binders = BTreeMap::new();
    let mut matches = resolution
        .resolved()
        .declarations
        .iter()
        .filter_map(|declaration| {
            let SurfaceDecl::Instance {
                type_parameters,
                trait_name_span,
                arguments,
                ..
            } = declaration
            else {
                return None;
            };
            if !type_parameters.is_empty() || arguments.len() != actual_arguments.len() {
                return None;
            }
            let target = resolution.target(*trait_name_span, crate::SymbolNamespace::Trait)?;
            let symbol = resolution.symbol(target)?;
            if symbol.canonical.as_deref() != Some(trait_identity) {
                return None;
            }
            let expected_arguments = arguments
                .iter()
                .map(|argument| resolution.semantic_value_from_type_ref(argument))
                .collect::<Vec<_>>();
            if !expected_arguments
                .iter()
                .zip(&actual_arguments)
                .all(|(expected, actual)| semantic_values_are_compatible(expected, actual))
            {
                return None;
            }
            let canonical_arguments = arguments
                .iter()
                .map(|argument| canonical_type_ref(argument, resolution, &binders))
                .collect::<Option<Vec<_>>>()?;
            Some(canonical_instance_head_identity(
                trait_identity,
                &canonical_arguments,
            ))
        });
    let selected = matches.next()?;
    matches.next().is_none().then_some(selected)
}

fn standard_instance_identity(constraint: &TypedConstraint) -> Option<String> {
    if let Some(identity) = arithmetic_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    let [collection, element] = constraint.arguments.as_slice() else {
        return None;
    };
    let TypedType::Named { name, arguments } = collection else {
        return None;
    };
    (constraint.name == "Reducible"
        && name == "Array"
        && matches!(arguments.as_slice(), [array_element] if array_element == element))
    .then(|| "std/array::Reducible".to_owned())
}

fn arithmetic_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [left, right, output] = constraint.arguments.as_slice() else {
        return None;
    };
    let all_int = [left, right, output]
        .iter()
        .all(|type_ref| matches!(type_ref, TypedType::Named { name, arguments } if name == "Int" && arguments.is_empty()));
    if !all_int {
        return None;
    }
    match constraint.name.as_str() {
        "Add" => Some("std/int::Add"),
        "Sub" => Some("std/int::Sub"),
        "Mul" => Some("std/int::Mul"),
        "Div" => Some("std/int::Div"),
        "Rem" => Some("std/int::Rem"),
        "Pow" => Some("std/int::Pow"),
        _ => None,
    }
}

pub(crate) fn select_arithmetic_evidence(
    operator: &str,
    left: TypedType,
    right: TypedType,
    output: TypedType,
) -> Vec<TypedCallEvidence> {
    let name = match operator {
        "+" => "Add",
        "-" => "Sub",
        "*" => "Mul",
        "/" => "Div",
        "%" => "Rem",
        "**" => "Pow",
        _ => return Vec::new(),
    };
    select_call_evidence(&[TypedConstraint {
        name: name.to_owned(),
        arguments: vec![left, right, output],
    }])
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> TypedType {
        TypedType::Named {
            name: name.to_owned(),
            arguments: Vec::new(),
        }
    }

    #[test]
    fn selects_the_standard_array_reducible_instance() {
        let int = named("Int");
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![
                TypedType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![int.clone()],
                },
                int,
            ],
        }])
        .expect("standard array instance");

        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/array::Reducible"
        ));
    }

    #[test]
    fn does_not_invent_evidence_for_an_unsupported_collection() {
        let int = named("Int");
        assert!(select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![named("Int"), int],
        }])
        .is_err());
    }

    #[test]
    fn selects_standard_int_add_evidence() {
        let evidence = select_arithmetic_evidence("+", named("Int"), named("Int"), named("Int"));
        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                constraint: TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Standard { identity },
            }] if name == "Add" && arguments.len() == 3 && identity == "std/int::Add"
        ));
    }
}
