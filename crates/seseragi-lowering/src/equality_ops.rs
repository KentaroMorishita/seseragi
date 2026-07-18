use crate::{CoreCallEvidence, CoreInstanceEvidence, CoreType};
use seseragi_semantics::standard_equality_instance_by_identity;

/// Returns the TypeScript operator only when the selected standard Eq
/// evidence explicitly guarantees a strict-equality-compatible
/// representation. User dictionaries and future structural standard
/// instances remain on dictionary dispatch.
pub(crate) fn strict_equality_operator_with_evidence<'operator>(
    operator: &'operator str,
    evidence: &[CoreCallEvidence],
) -> Option<&'operator str> {
    let typescript_operator = match operator {
        "==" => "===",
        "!=" => "!==",
        _ => return None,
    };
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity } = &selected.evidence else {
        return None;
    };
    let [value] = selected.constraint.arguments.as_slice() else {
        return None;
    };
    let instance = standard_equality_instance_by_identity(identity)?;
    (instance.strict_equality_compatible
        && selected.constraint.name == "Eq"
        && is_named(value, instance.type_name))
    .then_some(typescript_operator)
}

fn is_named(type_ref: &CoreType, expected: &str) -> bool {
    matches!(type_ref, CoreType::Named { name, arguments }
        if name == expected && arguments.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoreCallEvidence;

    #[test]
    fn maps_only_registered_standard_eq_evidence() {
        for (type_name, identity) in [
            ("Int", "std/int::Eq"),
            ("Bool", "std/bool::Eq"),
            ("String", "std/string::Eq"),
        ] {
            let evidence = evidence(type_name, identity);
            assert_eq!(
                strict_equality_operator_with_evidence("==", &evidence),
                Some("===")
            );
            assert_eq!(
                strict_equality_operator_with_evidence("!=", &evidence),
                Some("!==")
            );
        }
        assert!(
            strict_equality_operator_with_evidence("==", &evidence("Int", "std/string::Eq"))
                .is_none()
        );
    }

    fn evidence(type_name: &str, identity: &str) -> Vec<CoreCallEvidence> {
        vec![CoreCallEvidence {
            constraint: crate::CoreInstanceConstraint {
                name: "Eq".to_owned(),
                arguments: vec![CoreType::Named {
                    name: type_name.to_owned(),
                    arguments: Vec::new(),
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: identity.to_owned(),
            },
        }]
    }
}
