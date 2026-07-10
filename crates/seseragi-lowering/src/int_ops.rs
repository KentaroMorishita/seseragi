#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeIntOperation {
    pub(crate) operator: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const RUNTIME_INT_OPERATIONS: &[RuntimeIntOperation] = &[
    RuntimeIntOperation {
        operator: "+",
        runtime_feature: "core.int64.add",
        local_name: "_ssrg_int64_add",
        module: "@seseragi/runtime/int64",
        export_name: "add",
        source_map_name: "add",
    },
    RuntimeIntOperation {
        operator: "-",
        runtime_feature: "core.int64.subtract",
        local_name: "_ssrg_int64_subtract",
        module: "@seseragi/runtime/int64",
        export_name: "subtract",
        source_map_name: "subtract",
    },
    RuntimeIntOperation {
        operator: "*",
        runtime_feature: "core.int64.multiply",
        local_name: "_ssrg_int64_multiply",
        module: "@seseragi/runtime/int64",
        export_name: "multiply",
        source_map_name: "multiply",
    },
    RuntimeIntOperation {
        operator: "/",
        runtime_feature: "core.int64.divide",
        local_name: "_ssrg_int64_divide",
        module: "@seseragi/runtime/int64",
        export_name: "divide",
        source_map_name: "divide",
    },
    RuntimeIntOperation {
        operator: "%",
        runtime_feature: "core.int64.remainder",
        local_name: "_ssrg_int64_remainder",
        module: "@seseragi/runtime/int64",
        export_name: "remainder",
        source_map_name: "remainder",
    },
    RuntimeIntOperation {
        operator: "**",
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
}
