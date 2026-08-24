#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeStreamOperation {
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! operation {
    ($name:literal, $feature:literal, $export:literal) => {
        RuntimeStreamOperation {
            runtime_feature: $feature,
            local_name: concat!("_ssrg_stream_", $name),
            module: "@seseragi/runtime/stream",
            export_name: $export,
            source_map_name: $export,
        }
    };
}

const OPERATIONS: &[(&str, RuntimeStreamOperation)] = &[
    (
        "std/stream::empty",
        operation!("empty", "stream.empty", "empty"),
    ),
    (
        "std/stream::singleton",
        operation!("singleton", "stream.singleton", "singleton"),
    ),
    (
        "std/stream::fromArray",
        operation!("fromArray", "stream.source.array", "fromArray"),
    ),
    (
        "std/stream::fromIterable",
        operation!("fromIterable", "stream.source.iterable", "fromIterable"),
    ),
    (
        "std/stream::fromEffect",
        operation!("fromEffect", "stream.source.effect", "fromEffect"),
    ),
    (
        "std/stream::unfold",
        operation!("unfold", "stream.source.unfold", "unfold"),
    ),
    ("std/stream::map", operation!("map", "stream.map", "map")),
    (
        "std/stream::filter",
        operation!("filter", "stream.filter", "filter"),
    ),
    (
        "std/stream::filterMap",
        operation!("filterMap", "stream.filter-map", "filterMap"),
    ),
    (
        "std/stream::mapError",
        operation!("mapError", "stream.map-error", "mapError"),
    ),
    (
        "std/stream::flatMap",
        operation!("flatMap", "stream.flat-map", "flatMap"),
    ),
    (
        "std/stream::take",
        operation!("take", "stream.take", "take"),
    ),
    (
        "std/stream::drop",
        operation!("drop", "stream.drop", "drop"),
    ),
    (
        "std/stream::concat",
        operation!("concat", "stream.concat", "concat"),
    ),
    ("std/stream::zip", operation!("zip", "stream.zip", "zip")),
    (
        "std/stream::merge",
        operation!("merge", "stream.merge", "merge"),
    ),
    (
        "std/stream::NonPositiveBufferCapacity",
        operation!(
            "NonPositiveBufferCapacity",
            "stream.buffer.capacity.non-positive",
            "NonPositiveBufferCapacity"
        ),
    ),
    (
        "std/stream::bufferCapacity",
        operation!("bufferCapacity", "stream.buffer.capacity", "bufferCapacity"),
    ),
    (
        "std/stream::buffer",
        operation!("buffer", "stream.buffer.lossless", "buffer"),
    ),
    (
        "std/stream::runCollect",
        operation!("runCollect", "stream.terminal.collect", "runCollect"),
    ),
    (
        "std/stream::runFold",
        operation!("runFold", "stream.terminal.fold", "runFold"),
    ),
    (
        "std/stream::runForEach",
        operation!("runForEach", "stream.terminal.for-each", "runForEach"),
    ),
];

pub(crate) fn runtime_stream_operation(canonical: &str) -> Option<RuntimeStreamOperation> {
    OPERATIONS
        .iter()
        .find(|(candidate, _)| *candidate == canonical)
        .map(|(_, operation)| *operation)
}

pub(crate) fn runtime_stream_operation_for_feature(
    feature: &str,
) -> Option<RuntimeStreamOperation> {
    OPERATIONS
        .iter()
        .map(|(_, operation)| *operation)
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_stream_surface_to_one_runtime_module() {
        let collect = runtime_stream_operation("std/stream::runCollect").unwrap();
        assert_eq!(collect.runtime_feature, "stream.terminal.collect");
        assert_eq!(collect.module, "@seseragi/runtime/stream");
        assert_eq!(collect.export_name, "runCollect");
        assert_eq!(
            runtime_stream_operation_for_feature("stream.terminal.collect"),
            Some(collect)
        );
    }
}
