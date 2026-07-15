use crate::{CoreCallEvidence, CoreInstanceEvidence};

#[derive(Clone, Copy)]
pub(crate) struct RuntimeCollectionOperation {
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
}

const ARRAY_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.array.reduce",
    local_name: "_ssrg_array_reduce",
    module: "@seseragi/runtime/array",
    export_name: "reduce",
};

const RANGE_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.range.reduce",
    local_name: "_ssrg_range_reduce",
    module: "@seseragi/runtime/range",
    export_name: "reduce",
};

const ARRAY_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.array.comprehend",
    local_name: "_ssrg_array_comprehend",
    module: "@seseragi/runtime/array",
    export_name: "collectMap",
};

const ARRAY_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.array.comprehend.flat",
    local_name: "_ssrg_array_comprehend_flat",
    module: "@seseragi/runtime/array",
    export_name: "collectFlatMap",
};

const RANGE_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.range.comprehend",
    local_name: "_ssrg_range_comprehend",
    module: "@seseragi/runtime/range",
    export_name: "collectMap",
};

const RANGE_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.range.comprehend.flat",
    local_name: "_ssrg_range_comprehend_flat",
    module: "@seseragi/runtime/range",
    export_name: "collectFlatMap",
};

pub(crate) fn runtime_collection_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    let [selected] = evidence else {
        return None;
    };
    let CoreInstanceEvidence::Standard { identity } = &selected.evidence else {
        return None;
    };
    if callee != "std/prelude::reduce" || selected.constraint.name != "Reducible" {
        return None;
    }
    match identity.as_str() {
        "std/array::Reducible" => Some(&ARRAY_REDUCE),
        "std/range::Reducible" => Some(&RANGE_REDUCE),
        _ => None,
    }
}

pub(crate) fn runtime_collection_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    [
        ARRAY_REDUCE,
        RANGE_REDUCE,
        ARRAY_COMPREHEND,
        ARRAY_COMPREHEND_FLAT,
        RANGE_COMPREHEND,
        RANGE_COMPREHEND_FLAT,
    ]
    .into_iter()
    .find(|operation| operation.runtime_feature == feature)
}

pub(crate) fn runtime_iterable_operation(
    evidence: &CoreCallEvidence,
    flatten: bool,
) -> Option<&'static RuntimeCollectionOperation> {
    if evidence.constraint.name != "Iterable" {
        return None;
    }
    let CoreInstanceEvidence::Standard { identity } = &evidence.evidence else {
        return None;
    };
    match (identity.as_str(), flatten) {
        ("std/array::Iterable", false) => Some(&ARRAY_COMPREHEND),
        ("std/array::Iterable", true) => Some(&ARRAY_COMPREHEND_FLAT),
        ("std/range::Iterable", false) => Some(&RANGE_COMPREHEND),
        ("std/range::Iterable", true) => Some(&RANGE_COMPREHEND_FLAT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreInstanceConstraint, CoreType};

    #[test]
    fn resolves_array_reduce_only_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Reducible".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "Array".to_owned(),
                    arguments: vec![CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }],
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/array::Reducible".to_owned(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.array.reduce")
        );
        assert!(runtime_collection_operation("user::reduce", &evidence).is_none());
    }

    #[test]
    fn resolves_range_reduce_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Reducible".to_owned(),
                arguments: vec![CoreType::Named {
                    name: "Range".to_owned(),
                    arguments: vec![CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    }],
                }],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/range::Reducible".to_owned(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.range.reduce")
        );
    }

    #[test]
    fn resolves_iterable_runtime_by_evidence_and_nesting() {
        let range = CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Iterable".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "Range".to_owned(),
                        arguments: vec![CoreType::Named {
                            name: "Int".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    CoreType::Named {
                        name: "Int".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Standard {
                identity: "std/range::Iterable".to_owned(),
            },
        };

        assert_eq!(
            runtime_iterable_operation(&range, false).map(|operation| operation.runtime_feature),
            Some("core.range.comprehend")
        );
        assert_eq!(
            runtime_iterable_operation(&range, true).map(|operation| operation.runtime_feature),
            Some("core.range.comprehend.flat")
        );
    }
}
