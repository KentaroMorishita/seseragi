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
    [ARRAY_REDUCE, RANGE_REDUCE]
        .into_iter()
        .find(|operation| operation.runtime_feature == feature)
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
}
