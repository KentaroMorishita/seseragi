#[derive(Clone, Copy)]
pub(crate) struct RuntimeListOperation {
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const FROM_ARRAY: RuntimeListOperation = RuntimeListOperation {
    runtime_feature: "core.list.from-array",
    local_name: "_ssrg_list_from_array",
    module: "@seseragi/runtime/list",
    export_name: "fromArray",
    source_map_name: "fromArray",
};

const CONS: RuntimeListOperation = RuntimeListOperation {
    runtime_feature: "core.list.cons",
    local_name: "_ssrg_list_cons",
    module: "@seseragi/runtime/list",
    export_name: "Cons",
    source_map_name: "Cons",
};

pub(crate) fn runtime_list_cons_operation() -> RuntimeListOperation {
    CONS
}

pub(crate) fn runtime_list_literal_operation() -> RuntimeListOperation {
    FROM_ARRAY
}

pub(crate) fn runtime_list_operation_for_feature(feature: &str) -> Option<RuntimeListOperation> {
    [FROM_ARRAY, CONS]
        .into_iter()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_list_literals_to_the_persistent_runtime_constructor() {
        assert_eq!(
            runtime_list_literal_operation().runtime_feature,
            "core.list.from-array"
        );
    }
}
