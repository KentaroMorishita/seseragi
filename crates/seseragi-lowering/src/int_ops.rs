use crate::{CoreCallEvidence, CoreInstanceEvidence, CoreType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeIntOperation {
    pub(crate) operator: &'static str,
    pub(crate) trait_name: &'static str,
    pub(crate) standard_instance: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_INT_OPERATIONS: &[RuntimeIntOperation] = &[
    RuntimeIntOperation {
        operator: "+",
        trait_name: "Add",
        standard_instance: "std/int::Add",
        runtime_feature: "core.int64.add",
        local_name: "_ssrg_int64_add",
        module: "@seseragi/runtime/int64",
        export_name: "add",
        source_map_name: "add",
    },
    RuntimeIntOperation {
        operator: "-",
        trait_name: "Sub",
        standard_instance: "std/int::Sub",
        runtime_feature: "core.int64.subtract",
        local_name: "_ssrg_int64_subtract",
        module: "@seseragi/runtime/int64",
        export_name: "subtract",
        source_map_name: "subtract",
    },
    RuntimeIntOperation {
        operator: "*",
        trait_name: "Mul",
        standard_instance: "std/int::Mul",
        runtime_feature: "core.int64.multiply",
        local_name: "_ssrg_int64_multiply",
        module: "@seseragi/runtime/int64",
        export_name: "multiply",
        source_map_name: "multiply",
    },
    RuntimeIntOperation {
        operator: "/",
        trait_name: "Div",
        standard_instance: "std/int::Div",
        runtime_feature: "core.int64.divide",
        local_name: "_ssrg_int64_divide",
        module: "@seseragi/runtime/int64",
        export_name: "divide",
        source_map_name: "divide",
    },
    RuntimeIntOperation {
        operator: "%",
        trait_name: "Rem",
        standard_instance: "std/int::Rem",
        runtime_feature: "core.int64.remainder",
        local_name: "_ssrg_int64_remainder",
        module: "@seseragi/runtime/int64",
        export_name: "remainder",
        source_map_name: "remainder",
    },
    RuntimeIntOperation {
        operator: "**",
        trait_name: "Pow",
        standard_instance: "std/int::Pow",
        runtime_feature: "core.int64.power",
        local_name: "_ssrg_int64_power",
        module: "@seseragi/runtime/int64",
        export_name: "power",
        source_map_name: "power",
    },
];

pub(crate) fn runtime_int_operation(operator: &str) -> Option<RuntimeIntOperation> {
    RUNTIME_INT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.operator == operator)
}

pub(crate) fn runtime_int_operation_with_evidence(
    operator: &str,
    evidence: &[CoreCallEvidence],
) -> Option<RuntimeIntOperation> {
    let operation = runtime_int_operation(operator)?;
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity } = &selected.evidence else {
        return None;
    };
    let arguments_are_int = matches!(selected.constraint.arguments.as_slice(), [left, right, output]
        if [left, right, output].iter().all(|type_ref| is_int(type_ref)));
    (selected.constraint.name == operation.trait_name
        && identity == operation.standard_instance
        && arguments_are_int)
        .then_some(operation)
}

fn is_int(type_ref: &CoreType) -> bool {
    matches!(type_ref, CoreType::Named { name, arguments } if name == "Int" && arguments.is_empty())
}

pub(crate) fn runtime_int_operation_for_feature(feature: &str) -> Option<RuntimeIntOperation> {
    RUNTIME_INT_OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_all_int_arithmetic_operators() {
        for operator in ["+", "-", "*", "/", "%", "**"] {
            assert!(runtime_int_operation(operator).is_some());
        }
    }

    #[test]
    fn leaves_comparisons_as_operators() {
        assert!(runtime_int_operation("<").is_none());
        assert!(runtime_int_operation("==").is_none());
    }

    #[test]
    fn requires_matching_selected_evidence() {
        let int = CoreType::Named {
            name: "Int".to_owned(),
            arguments: Vec::new(),
        };
        let evidence = [CoreCallEvidence {
            constraint: crate::CoreInstanceConstraint {
                name: "Add".to_owned(),
                arguments: vec![int.clone(), int.clone(), int],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/int::Add".to_owned(),
            },
        }];
        assert!(runtime_int_operation_with_evidence("+", &evidence).is_some());
        assert!(runtime_int_operation_with_evidence("+", &[]).is_none());
        assert!(runtime_int_operation_with_evidence("-", &evidence).is_none());
    }
}
