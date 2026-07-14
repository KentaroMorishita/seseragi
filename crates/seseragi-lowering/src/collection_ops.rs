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
    (callee == "std/prelude::reduce"
        && selected.constraint.name == "Reducible"
        && identity == "std/array::Reducible")
        .then_some(&ARRAY_REDUCE)
}

pub(crate) fn runtime_collection_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    (feature == ARRAY_REDUCE.runtime_feature).then_some(ARRAY_REDUCE)
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
}
