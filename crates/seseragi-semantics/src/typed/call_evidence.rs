use crate::{TypedCallEvidence, TypedConstraint, TypedInstanceEvidence, TypedType};

/// Selects evidence for constraints attached to a saturated function call.
///
/// This is deliberately a small registry boundary. The first implemented
/// standard instance is `Reducible<Array<A>, A>`; local and imported instance
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
}
