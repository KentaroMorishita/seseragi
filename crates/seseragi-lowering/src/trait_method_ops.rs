use crate::int_ops::{runtime_int_operation_with_evidence, RuntimeIntOperation};
use crate::{CoreCallEvidence, CoreInstanceEvidence, CoreType};
use seseragi_semantics::{special_standard_instance_by_identity, PreludeSpecialInstanceDispatch};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuntimeTraitMethodOperation {
    Int(RuntimeIntOperation),
    Native(&'static str),
}

pub(crate) fn runtime_trait_method_operation(
    method: &str,
    evidence: &[CoreCallEvidence],
) -> Option<RuntimeTraitMethodOperation> {
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
        return None;
    };
    let instance = special_standard_instance_by_identity(identity)?;
    if instance.dispatch != PreludeSpecialInstanceDispatch::OperatorAbi
        || instance.trait_name != selected.constraint.name
        || !arguments_match(&selected.constraint.arguments, instance.arguments)
    {
        return None;
    }
    let operator = seseragi_syntax::standard_operators().find(|operator| {
        operator.method_name == method && operator.trait_name == selected.constraint.name
    })?;
    if let Some(operation) = runtime_int_operation_with_evidence(operator.spelling, evidence) {
        return Some(RuntimeTraitMethodOperation::Int(operation));
    }
    let operator = if instance.trait_name == "Eq" {
        "==="
    } else {
        operator.spelling
    };
    Some(RuntimeTraitMethodOperation::Native(operator))
}

fn arguments_match(arguments: &[CoreType], expected: &[&str]) -> bool {
    arguments.len() == expected.len()
        && arguments.iter().zip(expected).all(|(argument, expected)| {
            matches!(argument, CoreType::Named { name, arguments }
                if name == expected && arguments.is_empty())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoreInstanceConstraint;

    #[test]
    fn follows_the_canonical_operator_abi_dispatch_metadata() {
        assert!(matches!(
            runtime_trait_method_operation(
                "add",
                &evidence("Add", &["Int", "Int", "Int"], "std/int::Add")
            ),
            Some(RuntimeTraitMethodOperation::Int(operation)) if operation.operator == "+"
        ));
        assert_eq!(
            runtime_trait_method_operation("eq", &evidence("Eq", &["String"], "std/string::Eq")),
            Some(RuntimeTraitMethodOperation::Native("==="))
        );
        assert_eq!(
            runtime_trait_method_operation(
                "mul",
                &evidence("Mul", &["Float", "Float", "Float"], "std/float::Mul")
            ),
            Some(RuntimeTraitMethodOperation::Native("*"))
        );
    }

    #[test]
    fn rejects_dictionary_dispatch_and_mismatched_heads() {
        assert!(runtime_trait_method_operation(
            "zero",
            &evidence("Zero", &["Int"], "std/int::Zero")
        )
        .is_none());
        assert!(runtime_trait_method_operation(
            "add",
            &evidence("Add", &["Float", "Float", "Float"], "std/int::Add")
        )
        .is_none());
    }

    fn evidence(trait_name: &str, arguments: &[&str], identity: &str) -> Vec<CoreCallEvidence> {
        vec![CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                trait_identity: Some(format!("std/prelude::{trait_name}")),
                name: trait_name.to_owned(),
                arguments: arguments
                    .iter()
                    .map(|name| CoreType::Named {
                        name: (*name).to_owned(),
                        arguments: Vec::new(),
                    })
                    .collect(),
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: identity.to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
        }]
    }
}
