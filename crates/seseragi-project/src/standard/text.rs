use super::*;

pub(super) fn text_interface() -> ModuleInterface {
    let module = "std/text";
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let error = external_type(
        "Utf8DecodeError",
        "std/text::Utf8DecodeError",
        module,
        "Utf8DecodeError",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "Utf8DecodeError", []),
        constructor_export(
            module,
            "Utf8DecodeError",
            "InvalidUtf8",
            [],
            Some(record([required("offset", named("Int"))])),
        ),
        function_export(
            module,
            "encodeUtf8",
            [],
            Vec::new(),
            vec![named("String")],
            bytes.clone(),
        ),
        function_export(
            module,
            "decodeUtf8",
            [],
            Vec::new(),
            vec![bytes.clone()],
            named_with("Either", vec![error, named("String")]),
        ),
        function_export(
            module,
            "decodeUtf8Lossy",
            [],
            Vec::new(),
            vec![bytes],
            named("String"),
        ),
    ];
    exports.extend([
        opaque_adt_type_export(module, "TextSliceError", []),
        constructor_export(
            module,
            "TextSliceError",
            "InvalidScalarRange",
            [],
            Some(text_range_payload()),
        ),
        function_export(
            module,
            "isEmpty",
            [],
            Vec::new(),
            vec![named("String")],
            named("Bool"),
        ),
        function_export(
            module,
            "concat",
            [],
            Vec::new(),
            vec![named_with("Array", vec![named("String")])],
            named("String"),
        ),
        function_export(
            module,
            "join",
            [],
            Vec::new(),
            vec![named("String"), named_with("Array", vec![named("String")])],
            named("String"),
        ),
        function_export(
            module,
            "split",
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named_with("Array", vec![named("String")]),
        ),
        function_export(
            module,
            "scalarAt",
            [],
            Vec::new(),
            vec![named("Int"), named("String")],
            named_with("Maybe", vec![named("Char")]),
        ),
        function_export(
            module,
            "sliceScalars",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("String")],
            named_with(
                "Either",
                vec![
                    external_type(
                        "TextSliceError",
                        "std/text::TextSliceError",
                        module,
                        "TextSliceError",
                        Vec::new(),
                    ),
                    named("String"),
                ],
            ),
        ),
    ]);
    for name in ["lengthScalars", "lengthBytes"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named("Int"),
        ));
    }
    for name in ["lines", "words"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Array", vec![named("String")]),
        ));
    }
    for name in [
        "trim",
        "trimStart",
        "trimEnd",
        "toLower",
        "toUpper",
        "caseFold",
    ] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String")],
            named("String"),
        ));
    }
    for name in ["startsWith", "endsWith", "contains"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String"), named("String")],
            named("Bool"),
        ));
    }
    for name in ["replace", "replaceAll"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("String"), named("String"), named("String")],
            named("String"),
        ));
    }
    standard_interface(module, exports)
}
