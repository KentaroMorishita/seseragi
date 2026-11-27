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
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
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

/// Returns the TypeScript comparison operator only for the standard Int Ord
/// evidence. Int values are safe JavaScript integers, so the operator gives
/// the same answer as `compare`. Float, String and Char keep dictionary
/// dispatch because NaN and code-point order differ from JavaScript's `<`.
pub(crate) fn int_comparison_operator_with_evidence<'operator>(
    operator: &'operator str,
    evidence: &[CoreCallEvidence],
) -> Option<&'operator str> {
    if !matches!(operator, "<" | "<=" | ">" | ">=") {
        return None;
    }
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
        return None;
    };
    let [value] = selected.constraint.arguments.as_slice() else {
        return None;
    };
    (identity == "std/int::Ord" && selected.constraint.name == "Ord" && is_named(value, "Int"))
        .then_some(operator)
}

/// Returns the JavaScript operator that replaces a standard comparison, so
/// lowering and import planning skip the dictionary together.
pub(crate) fn javascript_operator_with_evidence<'operator>(
    operator: &'operator str,
    evidence: &[CoreCallEvidence],
) -> Option<&'operator str> {
    strict_equality_operator_with_evidence(operator, evidence)
        .or_else(|| int_comparison_operator_with_evidence(operator, evidence))
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

    #[test]
    fn maps_only_standard_int_ord_evidence() {
        let int = ord_evidence("Int", "std/int::Ord");
        for operator in ["<", "<=", ">", ">="] {
            assert_eq!(
                int_comparison_operator_with_evidence(operator, &int),
                Some(operator)
            );
        }
        assert!(int_comparison_operator_with_evidence("==", &int).is_none());
        for (type_name, identity) in [
            ("Float", "std/float::Ord"),
            ("String", "std/string::Ord"),
            ("Char", "std/char::Ord"),
            ("Int", "std/string::Ord"),
        ] {
            assert!(
                int_comparison_operator_with_evidence("<", &ord_evidence(type_name, identity))
                    .is_none()
            );
        }
        assert!(
            int_comparison_operator_with_evidence("<", &evidence("Int", "std/int::Ord")).is_none()
        );
    }

    fn evidence(type_name: &str, identity: &str) -> Vec<CoreCallEvidence> {
        evidence_for("Eq", type_name, identity)
    }

    fn ord_evidence(type_name: &str, identity: &str) -> Vec<CoreCallEvidence> {
        evidence_for("Ord", type_name, identity)
    }

    fn evidence_for(trait_name: &str, type_name: &str, identity: &str) -> Vec<CoreCallEvidence> {
        vec![CoreCallEvidence {
            constraint: crate::CoreInstanceConstraint {
                trait_identity: None,
                name: trait_name.to_owned(),
                arguments: vec![CoreType::Named {
                    name: type_name.to_owned(),
                    arguments: Vec::new(),
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: identity.to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        }]
    }
}
