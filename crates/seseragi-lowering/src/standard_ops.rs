//! Shared evidence-aware, source-arity-preserving call path for standard APIs.
use crate::collection_ops::{
    runtime_collection_operation_for_feature, runtime_standard_collection_operation,
    RuntimeCollectionOperation,
};
use crate::sum_ops::{runtime_sum_operation, runtime_sum_operation_for_feature};

pub(crate) fn runtime_standard_operation(
    name: &str,
) -> Option<&'static RuntimeCollectionOperation> {
    runtime_standard_collection_operation(name).or_else(|| runtime_sum_operation(name))
}

pub(crate) fn runtime_standard_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    runtime_collection_operation_for_feature(feature)
        .or_else(|| runtime_sum_operation_for_feature(feature))
}
