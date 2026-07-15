use super::TypedResolution;
use crate::{
    SymbolNamespace, TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType,
};
use seseragi_syntax::SurfaceConstraint;

mod imported;
mod local;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScopedCallEvidence {
    trait_identity: String,
    constraint: TypedConstraint,
    index: usize,
}

pub(crate) fn scoped_call_evidence(
    constraints: &[SurfaceConstraint],
    resolution: &TypedResolution<'_>,
) -> Vec<ScopedCallEvidence> {
    scoped_call_evidence_from(constraints, resolution, 0)
}

pub(crate) fn scoped_call_evidence_from(
    constraints: &[SurfaceConstraint],
    resolution: &TypedResolution<'_>,
    start_index: usize,
) -> Vec<ScopedCallEvidence> {
    constraints
        .iter()
        .enumerate()
        .filter_map(|(index, constraint)| {
            let target = resolution.target(constraint.name_span, SymbolNamespace::Trait)?;
            let trait_identity = resolution.symbol(target)?.canonical.clone()?;
            Some(ScopedCallEvidence {
                trait_identity,
                constraint: TypedConstraint {
                    name: constraint.name.clone(),
                    arguments: constraint
                        .arguments
                        .iter()
                        .map(super::type_ref::typed_type_from_type_ref)
                        .collect(),
                },
                index: start_index + index,
            })
        })
        .collect()
}

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

pub(crate) fn select_function_call_evidence(
    constraints: &[TypedConstraint],
    trait_identities: &[Option<String>],
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Result<Vec<TypedCallEvidence>, TypedConstraint> {
    constraints
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, constraint)| {
            let trait_identity = trait_identities.get(index).and_then(Option::as_deref);
            let evidence = match trait_identity {
                Some(trait_identity) => {
                    select_resolved_evidence(&constraint, trait_identity, resolution, scoped)
                }
                None => standard_instance_identity(&constraint)
                    .map(|identity| TypedInstanceEvidence::Standard { identity }),
            }
            .ok_or_else(|| constraint.clone())?;
            Ok(TypedCallEvidence {
                constraint,
                evidence,
            })
        })
        .collect()
}

fn select_resolved_evidence(
    constraint: &TypedConstraint,
    trait_identity: &str,
    resolution: &TypedResolution<'_>,
    scoped: &[ScopedCallEvidence],
) -> Option<TypedInstanceEvidence> {
    if let Some(parameter) = scoped.iter().find(|available| {
        available.trait_identity == trait_identity
            && available.constraint.arguments.len() == constraint.arguments.len()
            && available
                .constraint
                .arguments
                .iter()
                .zip(&constraint.arguments)
                .all(|(available, required)| {
                    super::semantic_types::semantic_values_are_compatible(
                        &resolution.semantic_value_from_typed_type(available),
                        &resolution.semantic_value_from_typed_type(required),
                    )
                })
    }) {
        return Some(TypedInstanceEvidence::Parameter {
            index: parameter.index,
        });
    }
    local::select_local_instance(trait_identity, constraint, resolution)
        .or_else(|| imported::select_imported_instance(trait_identity, constraint, resolution))
}

fn standard_instance_identity(constraint: &TypedConstraint) -> Option<String> {
    if let Some(identity) = arithmetic_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    if let Some(identity) = equality_instance_identity(constraint) {
        return Some(identity.to_owned());
    }
    let [collection, element] = constraint.arguments.as_slice() else {
        return None;
    };
    let TypedType::Named { name, arguments } = collection else {
        return None;
    };
    if !matches!(
        arguments.as_slice(),
        [collection_element] if collection_element == element
    ) {
        return None;
    }
    match (constraint.name.as_str(), name.as_str()) {
        ("Iterable", "Array") => Some("std/array::Iterable".to_owned()),
        ("Iterable", "Range") if named_type_is(element, "Int") => {
            Some("std/range::Iterable".to_owned())
        }
        ("Reducible", "Array") => Some("std/array::Reducible".to_owned()),
        ("Reducible", "Range") if named_type_is(element, "Int") => {
            Some("std/range::Reducible".to_owned())
        }
        _ => None,
    }
}

pub(crate) fn select_iterable_evidence(
    collection: TypedType,
    element: TypedType,
) -> Result<TypedCallEvidence, TypedConstraint> {
    let constraint = TypedConstraint {
        name: "Iterable".to_owned(),
        arguments: vec![collection, element],
    };
    select_call_evidence(std::slice::from_ref(&constraint))
        .map(|mut evidence| evidence.remove(0))
        .map_err(|_| constraint)
}

fn named_type_is(type_ref: &TypedType, expected: &str) -> bool {
    matches!(type_ref, TypedType::Named { name, arguments } if name == expected && arguments.is_empty())
}

fn equality_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [value] = constraint.arguments.as_slice() else {
        return None;
    };
    if constraint.name != "Eq" {
        return None;
    }
    let TypedType::Named { name, arguments } = value else {
        return None;
    };
    if !arguments.is_empty() {
        return None;
    }
    match name.as_str() {
        "Int" => Some("std/int::Eq"),
        "Bool" => Some("std/bool::Eq"),
        "String" => Some("std/string::Eq"),
        _ => None,
    }
}

fn arithmetic_instance_identity(constraint: &TypedConstraint) -> Option<&'static str> {
    let [left, right, output] = constraint.arguments.as_slice() else {
        return None;
    };
    let all_string = [left, right, output].iter().all(|type_ref| {
        matches!(type_ref, TypedType::Named { name, arguments } if name == "String" && arguments.is_empty())
    });
    if constraint.name == "Add" && all_string {
        return Some("std/string::Add");
    }
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

pub(crate) fn select_equality_evidence(
    operator: &str,
    left: TypedType,
    right: TypedType,
) -> Vec<TypedCallEvidence> {
    if !matches!(operator, "==" | "!=") || left != right {
        return Vec::new();
    }
    select_call_evidence(&[TypedConstraint {
        name: "Eq".to_owned(),
        arguments: vec![left],
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
    fn selects_the_standard_int_range_reducible_instance() {
        let int = named("Int");
        let evidence = select_call_evidence(&[TypedConstraint {
            name: "Reducible".to_owned(),
            arguments: vec![
                TypedType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![int.clone()],
                },
                int,
            ],
        }])
        .expect("standard range instance");

        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                evidence: TypedInstanceEvidence::Standard { identity },
                ..
            }] if identity == "std/range::Reducible"
        ));
    }

    #[test]
    fn selects_standard_iterable_instances_by_collection_type() {
        let int = named("Int");
        for (collection, expected) in [
            (
                TypedType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![int.clone()],
                },
                "std/array::Iterable",
            ),
            (
                TypedType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![int.clone()],
                },
                "std/range::Iterable",
            ),
        ] {
            let evidence = select_iterable_evidence(collection, int.clone())
                .expect("standard Iterable evidence");
            assert!(matches!(
                evidence.evidence,
                TypedInstanceEvidence::Standard { identity } if identity == expected
            ));
        }
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

    #[test]
    fn selects_standard_string_add_evidence() {
        let evidence =
            select_arithmetic_evidence("+", named("String"), named("String"), named("String"));
        assert!(matches!(
            evidence.as_slice(),
            [TypedCallEvidence {
                constraint: TypedConstraint { name, arguments },
                evidence: TypedInstanceEvidence::Standard { identity },
            }] if name == "Add" && arguments.len() == 3 && identity == "std/string::Add"
        ));
    }

    #[test]
    fn selects_standard_equality_evidence_for_primitive_types() {
        for (name, identity) in [
            ("Int", "std/int::Eq"),
            ("Bool", "std/bool::Eq"),
            ("String", "std/string::Eq"),
        ] {
            let evidence = select_equality_evidence("==", named(name), named(name));
            assert!(matches!(
                evidence.as_slice(),
                [TypedCallEvidence {
                    constraint: TypedConstraint { name, arguments },
                    evidence: TypedInstanceEvidence::Standard { identity: selected },
                }] if name == "Eq" && arguments.len() == 1 && selected == identity
            ));
        }
    }

    #[test]
    fn does_not_select_equality_evidence_for_mixed_types() {
        assert!(select_equality_evidence("==", named("Int"), named("String")).is_empty());
    }
}
