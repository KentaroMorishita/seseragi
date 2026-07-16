#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeIteratorOperation {
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

const ITERATOR_UNFOLD: RuntimeIteratorOperation = RuntimeIteratorOperation {
    runtime_feature: "core.iterator.unfold",
    local_name: "_ssrg_iterator_unfold",
    module: "@seseragi/runtime/iterator",
    export_name: "unfold",
    source_map_name: "unfold",
};

const ITERATOR_NEXT: RuntimeIteratorOperation = RuntimeIteratorOperation {
    runtime_feature: "core.iterator.next",
    local_name: "_ssrg_iterator_next",
    module: "@seseragi/runtime/iterator",
    export_name: "next",
    source_map_name: "next",
};

const ITERATOR_COMPREHEND: RuntimeIteratorOperation = RuntimeIteratorOperation {
    runtime_feature: "core.iterator.comprehend",
    local_name: "_ssrg_iterator_comprehend",
    module: "@seseragi/runtime/iterator",
    export_name: "collectMap",
    source_map_name: "collectMap",
};

const ITERATOR_COMPREHEND_FLAT: RuntimeIteratorOperation = RuntimeIteratorOperation {
    runtime_feature: "core.iterator.comprehend.flat",
    local_name: "_ssrg_iterator_comprehend_flat",
    module: "@seseragi/runtime/iterator",
    export_name: "collectFlatMap",
    source_map_name: "collectFlatMap",
};

pub(crate) fn runtime_iterator_operation(
    callee: &str,
) -> Option<&'static RuntimeIteratorOperation> {
    match callee {
        "std/prelude::unfold" => Some(&ITERATOR_UNFOLD),
        "std/prelude::next" => Some(&ITERATOR_NEXT),
        _ => None,
    }
}

pub(crate) fn runtime_iterator_operation_for_feature(
    feature: &str,
) -> Option<RuntimeIteratorOperation> {
    [
        ITERATOR_UNFOLD,
        ITERATOR_NEXT,
        ITERATOR_COMPREHEND,
        ITERATOR_COMPREHEND_FLAT,
    ]
    .into_iter()
    .find(|operation| operation.runtime_feature == feature)
}

pub(crate) fn runtime_iterator_comprehension_operation(
    flatten: bool,
) -> &'static RuntimeIteratorOperation {
    if flatten {
        &ITERATOR_COMPREHEND_FLAT
    } else {
        &ITERATOR_COMPREHEND
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_standard_iterator_calls_to_the_runtime_abi() {
        assert_eq!(
            runtime_iterator_operation("std/prelude::unfold")
                .map(|operation| operation.runtime_feature),
            Some("core.iterator.unfold")
        );
        assert_eq!(
            runtime_iterator_operation("std/prelude::next").map(|operation| operation.export_name),
            Some("next")
        );
        assert!(runtime_iterator_operation("user::next").is_none());
    }
}
