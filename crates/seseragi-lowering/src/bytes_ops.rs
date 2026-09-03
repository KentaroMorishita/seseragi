use crate::runtime_modules::RuntimeDataOperation as RuntimeBytesOperation;

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

macro_rules! codec_operation {
    ($module:literal, $runtime_module:literal, $prefix:literal, $name:literal, $feature:literal) => {
        RuntimeBytesOperation {
            canonical: concat!($module, "::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_", $prefix, "_", $name),
            module: $runtime_module,
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
    codec_operation!(
        "std/bytes/hex",
        "@seseragi/runtime/bytes-hex",
        "hex",
        "OddHexLength",
        "core.bytes.hex.error.odd-length"
    ),
    codec_operation!(
        "std/bytes/hex",
        "@seseragi/runtime/bytes-hex",
        "hex",
        "InvalidHexDigit",
        "core.bytes.hex.error.invalid-digit"
    ),
    codec_operation!(
        "std/bytes/hex",
        "@seseragi/runtime/bytes-hex",
        "hex",
        "encode",
        "core.bytes.hex.encode"
    ),
    codec_operation!(
        "std/bytes/hex",
        "@seseragi/runtime/bytes-hex",
        "hex",
        "decode",
        "core.bytes.hex.decode"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "InvalidBase64Length",
        "core.bytes.base64.error.invalid-length"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "InvalidBase64Digit",
        "core.bytes.base64.error.invalid-digit"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "InvalidBase64Padding",
        "core.bytes.base64.error.invalid-padding"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "NonCanonicalBase64Bits",
        "core.bytes.base64.error.non-canonical-bits"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "encode",
        "core.bytes.base64.encode"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "decode",
        "core.bytes.base64.decode"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "encodeUrl",
        "core.bytes.base64.encode-url"
    ),
    codec_operation!(
        "std/bytes/base64",
        "@seseragi/runtime/bytes-base64",
        "base64",
        "decodeUrl",
        "core.bytes.base64.decode-url"
    ),
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
    fn maps_bytes_operations_by_canonical_identity() {
        for operation in OPERATIONS {
            assert_eq!(
                runtime_bytes_operation(operation.canonical),
                Some(*operation)
            );
            assert_eq!(
                runtime_bytes_operation_for_feature(operation.runtime_feature),
                Some(*operation)
            );
        }
        assert!(runtime_bytes_operation("app/bytes/hex::encode").is_none());
    }
}
