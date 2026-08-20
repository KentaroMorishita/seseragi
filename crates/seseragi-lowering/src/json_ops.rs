#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RuntimeJsonOperation {
    pub(crate) canonical: &'static str,
    pub(crate) runtime_feature: &'static str,
    pub(crate) local_name: &'static str,
    pub(crate) module: &'static str,
    pub(crate) export_name: &'static str,
    pub(crate) source_map_name: &'static str,
}

macro_rules! json_operation {
    ($name:literal, $feature:literal) => {
        RuntimeJsonOperation {
            canonical: concat!("std/json::", $name),
            runtime_feature: $feature,
            local_name: concat!("_ssrg_json_", $name),
            module: "@seseragi/runtime/json",
            export_name: $name,
            source_map_name: $name,
        }
    };
}

macro_rules! json_runtime_helper {
    ($feature:literal, $local:literal, $export:literal) => {
        RuntimeJsonOperation {
            canonical: concat!("@seseragi/internal::", $export),
            runtime_feature: $feature,
            local_name: $local,
            module: "@seseragi/runtime/json",
            export_name: $export,
            source_map_name: $export,
        }
    };
}

const OPERATIONS: &[RuntimeJsonOperation] = &[
    json_operation!("JsonNull", "json.constructor.null"),
    json_operation!("JsonBool", "json.constructor.bool"),
    json_operation!("JsonNumber", "json.constructor.number"),
    json_operation!("JsonString", "json.constructor.string"),
    json_operation!("JsonArray", "json.constructor.array"),
    json_operation!("JsonObject", "json.constructor.object"),
    json_operation!("JsonField", "json.path.field"),
    json_operation!("JsonIndex", "json.path.index"),
    json_operation!("ExpectedJsonType", "json.decode.expected-type"),
    json_operation!("MissingJsonField", "json.decode.missing-field"),
    json_operation!("UnknownJsonField", "json.decode.unknown-field"),
    json_operation!("UnknownJsonTag", "json.decode.unknown-tag"),
    json_operation!("InvalidJsonValue", "json.decode.invalid-value"),
    json_operation!("InvalidJsonSyntax", "json.parse.invalid-syntax"),
    json_operation!("DuplicateJsonField", "json.parse.duplicate-field"),
    json_operation!("JsonSyntaxFailure", "json.read.syntax-failure"),
    json_operation!("JsonDecodeFailure", "json.read.decode-failure"),
    json_operation!("parse", "json.parse"),
    json_operation!("stringify", "json.stringify"),
    json_operation!("encodeString", "json.encode-string"),
    json_operation!("decodeString", "json.decode-string"),
    json_operation!("field", "json.decoder.field"),
    json_operation!("optionalField", "json.decoder.optional-field"),
    json_operation!("index", "json.decoder.index"),
    json_operation!("array", "json.decoder.array"),
    json_operation!("record", "json.decoder.record"),
    json_operation!("oneOf", "json.decoder.one-of"),
    json_operation!("map", "json.decoder.map"),
    json_operation!("flatMap", "json.decoder.flat-map"),
    json_runtime_helper!(
        "json.derived-struct.encode",
        "_ssrg_json_derivedstruct_encode",
        "derivedStructJsonEncode"
    ),
    json_runtime_helper!(
        "json.derived-struct.decode",
        "_ssrg_json_derivedstruct_decode",
        "derivedStructJsonDecode"
    ),
    json_runtime_helper!(
        "json.derived-adt.encode",
        "_ssrg_json_derivedadt_encode",
        "derivedAdtJsonEncode"
    ),
    json_runtime_helper!(
        "json.derived-adt.decode",
        "_ssrg_json_derivedadt_decode",
        "derivedAdtJsonDecode"
    ),
    json_runtime_helper!(
        "json.derived-newtype.encode",
        "_ssrg_json_derivednewtype_encode",
        "derivedNewtypeJsonEncode"
    ),
    json_runtime_helper!(
        "json.derived-newtype.decode",
        "_ssrg_json_derivednewtype_decode",
        "derivedNewtypeJsonDecode"
    ),
];

pub(crate) fn runtime_json_operation(canonical: &str) -> Option<RuntimeJsonOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.canonical == canonical)
}

pub(crate) fn runtime_json_operation_for_feature(feature: &str) -> Option<RuntimeJsonOperation> {
    OPERATIONS
        .iter()
        .copied()
        .find(|operation| operation.runtime_feature == feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_json_operations_by_canonical_identity() {
        for (canonical, feature) in [
            ("std/json::parse", "json.parse"),
            ("std/json::encodeString", "json.encode-string"),
            ("std/json::optionalField", "json.decoder.optional-field"),
            ("std/json::JsonObject", "json.constructor.object"),
        ] {
            let operation = runtime_json_operation(canonical).unwrap();
            assert_eq!(operation.runtime_feature, feature);
            assert_eq!(runtime_json_operation_for_feature(feature), Some(operation));
        }
        assert!(runtime_json_operation("std/json::unknown").is_none());
    }
}
