#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeBytesOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! bytes_operation {
    ($name:literal, $feature:literal) => {
        RuntimeBytesOperation {
            canonical: concat!("std/bytes::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_bytes_", $name),
            module: "@seseragi/runtime/bytes",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

macro_rules! text_operation {
    ($name:literal, $feature:literal) => {
        RuntimeBytesOperation {
            canonical: concat!("std/text::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_text_", $name),
            module: "@seseragi/runtime/text",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

const OPERATIONS: &[RuntimeBytesOperation] = &[
    bytes_operation!("ByteOutOfRange", "core.bytes.error.out-of-range"),
    bytes_operation!("InvalidByteRange", "core.bytes.error.invalid-range"),
    bytes_operation!("byte", "core.bytes.byte"),
    bytes_operation!("toInt", "core.bytes.to-int"),
    bytes_operation!("empty", "core.bytes.empty"),
    bytes_operation!("singleton", "core.bytes.singleton"),
    bytes_operation!("fromArray", "core.bytes.from-array"),
    bytes_operation!("fromInts", "core.bytes.from-ints"),
    bytes_operation!("toArray", "core.bytes.to-array"),
    bytes_operation!("toInts", "core.bytes.to-ints"),
    bytes_operation!("length", "core.bytes.length"),
    bytes_operation!("isEmpty", "core.bytes.is-empty"),
    bytes_operation!("get", "core.bytes.get"),
    bytes_operation!("slice", "core.bytes.slice"),
    bytes_operation!("copy", "core.bytes.copy"),
    bytes_operation!("append", "core.bytes.append"),
    bytes_operation!("concat", "core.bytes.concat"),
    text_operation!("InvalidUtf8", "core.text.invalid-utf8"),
    text_operation!("encodeUtf8", "core.text.encode-utf8"),
    text_operation!("decodeUtf8", "core.text.decode-utf8"),
    text_operation!("decodeUtf8Lossy", "core.text.decode-utf8-lossy"),
];

pub(crate) fn runtime_bytes_operation(canonical: &str) -> Option<RuntimeBytesOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_bytes_operation_for_feature(feature: &str) -> Option<RuntimeBytesOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_bytes_and_utf8_operations_by_canonical_identity() {
        for (canonical, feature) in [
            ("std/bytes::fromInts", "core.bytes.from-ints"),
            ("std/bytes::slice", "core.bytes.slice"),
            ("std/text::encodeUtf8", "core.text.encode-utf8"),
            ("std/text::decodeUtf8", "core.text.decode-utf8"),
        ] {
            let operation = runtime_bytes_operation(canonical).unwrap();
            assert_eq!(operation.runtime_feature, feature);
            assert_eq!(
                runtime_bytes_operation_for_feature(feature),
                Some(operation)
            );
        }
        assert!(runtime_bytes_operation("std/bytes/hex::encode").is_none());
    }
}
