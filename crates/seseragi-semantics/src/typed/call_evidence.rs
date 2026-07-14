use crate::{TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType};

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
