//! Shared evidence-aware, source-arity-preserving call path for standard APIs.
use crate::collection_ops::{
    runtime_collection_operation_for_feature, runtime_standard_collection_operation,
    RuntimeCollectionOperation,
};
use crate::sum_ops::{runtime_sum_operation, runtime_sum_operation_for_feature};

pub(crate) fn runtime_standard_operation(
    name: &str,
) -> Option<&'static RuntimeCollectionOperation> {
    runtime_standard_collection_operation(name)
        .or_else(|| runtime_sum_operation(name))
        .or_else(|| {
            TRANSFORMER_OPERATIONS
                .iter()
                .find(|(canonical, _)| *canonical == name)
                .map(|(_, operation)| operation)
        })
}

pub(crate) fn runtime_standard_operation_for_feature(
    feature: &str,
) -> Option<RuntimeCollectionOperation> {
    runtime_collection_operation_for_feature(feature)
        .or_else(|| runtime_sum_operation_for_feature(feature))
        .or_else(|| {
            TRANSFORMER_OPERATIONS
                .iter()
                .find(|(_, operation)| operation.runtime_feature == feature)
                .map(|(_, operation)| *operation)
        })
}

const TRANSFORMER_OPERATIONS: &[(&str, RuntimeCollectionOperation)] = &[
    (
        "std/transformer/maybe::run",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.maybe.run",
            local_name: "_ssrg_maybeTRun",
            module: "@seseragi/runtime/transformer",
            export_name: "maybeTRun",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/maybe::fromMaybe",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.maybe.fromMaybe",
            local_name: "_ssrg_maybeTFromMaybe",
            module: "@seseragi/runtime/transformer",
            export_name: "maybeTFromMaybe",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/maybe::lift",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.maybe.lift",
            local_name: "_ssrg_maybeTLift",
            module: "@seseragi/runtime/transformer",
            export_name: "maybeTLift",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/either::run",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.either.run",
            local_name: "_ssrg_eitherTRun",
            module: "@seseragi/runtime/transformer",
            export_name: "eitherTRun",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/either::fromEither",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.either.fromEither",
            local_name: "_ssrg_eitherTFromEither",
            module: "@seseragi/runtime/transformer",
            export_name: "eitherTFromEither",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/either::lift",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.either.lift",
            local_name: "_ssrg_eitherTLift",
            module: "@seseragi/runtime/transformer",
            export_name: "eitherTLift",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/reader::run",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.reader.run",
            local_name: "_ssrg_readerTRun",
            module: "@seseragi/runtime/transformer",
            export_name: "readerTRun",
            source_arity: 2,
        },
    ),
    (
        "std/transformer/reader::ask",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.reader.ask",
            local_name: "_ssrg_readerTAsk",
            module: "@seseragi/runtime/transformer",
            export_name: "readerTAsk",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/reader::asks",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.reader.asks",
            local_name: "_ssrg_readerTAsks",
            module: "@seseragi/runtime/transformer",
            export_name: "readerTAsks",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/reader::local",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.reader.local",
            local_name: "_ssrg_readerTLocal",
            module: "@seseragi/runtime/transformer",
            export_name: "readerTLocal",
            source_arity: 2,
        },
    ),
    (
        "std/transformer/reader::lift",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.reader.lift",
            local_name: "_ssrg_readerTLift",
            module: "@seseragi/runtime/transformer",
            export_name: "readerTLift",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/state::run",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.state.run",
            local_name: "_ssrg_stateTRun",
            module: "@seseragi/runtime/transformer",
            export_name: "stateTRun",
            source_arity: 2,
        },
    ),
    (
        "std/transformer/state::get",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.state.get",
            local_name: "_ssrg_stateTGet",
            module: "@seseragi/runtime/transformer",
            export_name: "stateTGet",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/state::put",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.state.put",
            local_name: "_ssrg_stateTPut",
            module: "@seseragi/runtime/transformer",
            export_name: "stateTPut",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/state::modify",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.state.modify",
            local_name: "_ssrg_stateTModify",
            module: "@seseragi/runtime/transformer",
            export_name: "stateTModify",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/state::lift",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.state.lift",
            local_name: "_ssrg_stateTLift",
            module: "@seseragi/runtime/transformer",
            export_name: "stateTLift",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/writer::run",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.writer.run",
            local_name: "_ssrg_writerTRun",
            module: "@seseragi/runtime/transformer",
            export_name: "writerTRun",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/writer::tell",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.writer.tell",
            local_name: "_ssrg_writerTTell",
            module: "@seseragi/runtime/transformer",
            export_name: "writerTTell",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/writer::listen",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.writer.listen",
            local_name: "_ssrg_writerTListen",
            module: "@seseragi/runtime/transformer",
            export_name: "writerTListen",
            source_arity: 1,
        },
    ),
    (
        "std/transformer/writer::lift",
        RuntimeCollectionOperation {
            result_erased: true,
            runtime_feature: "core.transformer.writer.lift",
            local_name: "_ssrg_writerTLift",
            module: "@seseragi/runtime/transformer",
            export_name: "writerTLift",
            source_arity: 1,
        },
    ),
];
