#[derive(Clone, Copy)]
pub(crate) struct RuntimeRangeOperation {
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const EXCLUSIVE: RuntimeRangeOperation = RuntimeRangeOperation {
    runtime_feature: "core.range.exclusive",
    local_name: "_ssrg_range_exclusive",
    module: "@seseragi/runtime/range",
    export_name: "exclusive",
    source_map_name: "exclusive",
};

const INCLUSIVE: RuntimeRangeOperation = RuntimeRangeOperation {
    runtime_feature: "core.range.inclusive",
    local_name: "_ssrg_range_inclusive",
    module: "@seseragi/runtime/range",
    export_name: "inclusive",
    source_map_name: "inclusive",
};

pub(crate) fn runtime_range_operation(operator: &str) -> Option<RuntimeRangeOperation> {
    match operator {
        ".." => Some(EXCLUSIVE),
        "..=" => Some(INCLUSIVE),
        _ => None,
    }
}

pub(crate) fn runtime_range_operation_for_feature(feature: &str) -> Option<RuntimeRangeOperation> {
    [EXCLUSIVE, INCLUSIVE]
        .into_iter()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_both_range_operators_to_distinct_runtime_operations() {
        assert_eq!(
            runtime_range_operation("..").map(|operation| operation.runtime_feature),
            Some("core.range.exclusive")
        );
        assert_eq!(
            runtime_range_operation("..=").map(|operation| operation.runtime_feature),
            Some("core.range.inclusive")
        );
    }
}
