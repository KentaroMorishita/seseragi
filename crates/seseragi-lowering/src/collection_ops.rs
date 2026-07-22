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

const LIST_REDUCE: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.list.reduce",
    local_name: "_ssrg_list_reduce",
    module: "@seseragi/runtime/list",
    export_name: "reduce",
};

const COLLECTION_JOIN: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.collection.join",
    local_name: "_ssrg_collection_join",
    module: "@seseragi/runtime/collection",
    export_name: "join",
};

const COLLECTION_FOR_EACH: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "effect.collection.for-each",
    local_name: "_ssrg_collection_for_each",
    module: "@seseragi/runtime/collection",
    export_name: "forEach",
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

const LIST_COMPREHEND: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.list.comprehend",
    local_name: "_ssrg_list_comprehend",
    module: "@seseragi/runtime/list",
    export_name: "collectMap",
};

const LIST_COMPREHEND_FLAT: RuntimeCollectionOperation = RuntimeCollectionOperation {
    runtime_feature: "core.list.comprehend.flat",
    local_name: "_ssrg_list_comprehend_flat",
    module: "@seseragi/runtime/list",
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
        "std/list::Reducible" => Some(&LIST_REDUCE),
        _ => None,
    }
}

pub(crate) fn runtime_collection_join_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [selected] if selected.constraint.name == "Reducible"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::join")
    .map(|_| &COLLECTION_JOIN)
}

pub(crate) fn runtime_collection_for_each_operation(
    callee: &str,
    evidence: &[CoreCallEvidence],
) -> Option<&'static RuntimeCollectionOperation> {
    matches!(
        evidence,
        [selected] if selected.constraint.name == "Iterable"
    )
    .then_some(())
    .filter(|_| callee == "std/prelude::forEach")
    .map(|_| &COLLECTION_FOR_EACH)
}

pub(crate) fn runtime_collection_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    [
        ARRAY_REDUCE,
        RANGE_REDUCE,
        LIST_REDUCE,
        COLLECTION_JOIN,
        COLLECTION_FOR_EACH,
        ARRAY_COMPREHEND,
        ARRAY_COMPREHEND_FLAT,
        RANGE_COMPREHEND,
        RANGE_COMPREHEND_FLAT,
        LIST_COMPREHEND,
        LIST_COMPREHEND_FLAT,
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
        ("std/list::Iterable", false) => Some(&LIST_COMPREHEND),
        ("std/list::Iterable", true) => Some(&LIST_COMPREHEND_FLAT),
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
    fn resolves_generic_join_from_reducible_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Reducible".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "C".to_owned(),
                        arguments: Vec::new(),
                    },
                    CoreType::Named {
                        name: "String".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Parameter { index: 0 },
        }];

        assert_eq!(
            runtime_collection_join_operation("std/prelude::join", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.collection.join")
        );
        assert!(runtime_collection_join_operation("user::join", &evidence).is_none());
    }

    #[test]
    fn resolves_generic_for_each_from_iterable_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Iterable".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "C".to_owned(),
                        arguments: Vec::new(),
                    },
                    CoreType::Named {
                        name: "A".to_owned(),
                        arguments: Vec::new(),
                    },
                ],
            },
            evidence: CoreInstanceEvidence::Parameter { index: 0 },
        }];

        assert_eq!(
            runtime_collection_for_each_operation("std/prelude::forEach", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("effect.collection.for-each")
        );
    }

    #[test]
    fn resolves_list_reduce_with_selected_standard_evidence() {
        let evidence = [CoreCallEvidence {
            constraint: CoreInstanceConstraint {
                name: "Reducible".to_owned(),
                arguments: vec![
                    CoreType::Named {
                        name: "List".to_owned(),
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
                identity: "std/list::Reducible".to_owned(),
            },
        }];

        assert_eq!(
            runtime_collection_operation("std/prelude::reduce", &evidence)
                .map(|operation| operation.runtime_feature),
            Some("core.list.reduce")
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
