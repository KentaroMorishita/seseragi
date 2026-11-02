use super::*;

pub(super) fn json_interface() -> ModuleInterface {
    let module = "std/json";
    let json = external_type("Json", "std/json::Json", module, "Json", Vec::new());
    let path_segment = external_type(
        "JsonPathSegment",
        "std/json::JsonPathSegment",
        module,
        "JsonPathSegment",
        Vec::new(),
    );
    let decode_error_kind = external_type(
        "DecodeErrorKind",
        "std/json::DecodeErrorKind",
        module,
        "DecodeErrorKind",
        Vec::new(),
    );
    let decode_error = external_type(
        "DecodeError",
        "std/json::DecodeError",
        module,
        "DecodeError",
        Vec::new(),
    );
    let parse_error = external_type(
        "JsonParseError",
        "std/json::JsonParseError",
        module,
        "JsonParseError",
        Vec::new(),
    );
    let read_error = external_type(
        "JsonReadError",
        "std/json::JsonReadError",
        module,
        "JsonReadError",
        Vec::new(),
    );
    let decoder = |value: InterfaceType| {
        function_type(
            vec![json.clone()],
            named_with("Either", vec![decode_error.clone(), value]),
        )
    };
    let encoder = |value: InterfaceType| function_type(vec![value], json.clone());
    let mut exports = vec![
        opaque_adt_type_export(module, "Json", []),
        constructor_export(module, "Json", "JsonNull", [], None),
        constructor_export(module, "Json", "JsonBool", [], Some(named("Bool"))),
        constructor_export(
            module,
            "Json",
            "JsonNumber",
            [],
            Some(external_type(
                "Decimal",
                "std/decimal::Decimal",
                "std/decimal",
                "Decimal",
                Vec::new(),
            )),
        ),
        constructor_export(module, "Json", "JsonString", [], Some(named("String"))),
        constructor_export(
            module,
            "Json",
            "JsonArray",
            [],
            Some(named_with("Array", vec![json.clone()])),
        ),
        constructor_export(
            module,
            "Json",
            "JsonObject",
            [],
            Some(external_type(
                "Map",
                "std/map::Map",
                "std/map",
                "Map",
                vec![named("String"), json.clone()],
            )),
        ),
        alias_type_export(module, "Decoder", ["A"], decoder(named("A"))),
        alias_type_export(module, "Encoder", ["A"], encoder(named("A"))),
        opaque_adt_type_export(module, "JsonPathSegment", []),
        constructor_export(
            module,
            "JsonPathSegment",
            "JsonField",
            [],
            Some(named("String")),
        ),
        constructor_export(
            module,
            "JsonPathSegment",
            "JsonIndex",
            [],
            Some(named("Int")),
        ),
        opaque_adt_type_export(module, "DecodeErrorKind", []),
    ];
    for name in [
        "ExpectedJsonType",
        "MissingJsonField",
        "UnknownJsonField",
        "UnknownJsonTag",
        "InvalidJsonValue",
    ] {
        exports.push(constructor_export(
            module,
            "DecodeErrorKind",
            name,
            [],
            Some(named("String")),
        ));
    }
    exports.extend([
        public_record_type_export(
            module,
            "DecodeError",
            [
                required("path", named_with("Array", vec![path_segment.clone()])),
                required("kind", decode_error_kind),
            ],
        ),
        opaque_adt_type_export(module, "JsonParseError", []),
        constructor_export(
            module,
            "JsonParseError",
            "InvalidJsonSyntax",
            [],
            Some(record([
                required("offset", named("Int")),
                required("message", named("String")),
            ])),
        ),
        constructor_export(
            module,
            "JsonParseError",
            "DuplicateJsonField",
            [],
            Some(record([
                required("path", named_with("Array", vec![path_segment])),
                required("field", named("String")),
            ])),
        ),
        opaque_adt_type_export(module, "JsonReadError", []),
        constructor_export(
            module,
            "JsonReadError",
            "JsonSyntaxFailure",
            [],
            Some(parse_error.clone()),
        ),
        constructor_export(
            module,
            "JsonReadError",
            "JsonDecodeFailure",
            [],
            Some(decode_error.clone()),
        ),
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error, json.clone()]),
        ),
        function_export(
            module,
            "stringify",
            [],
            Vec::new(),
            vec![json.clone()],
            named("String"),
        ),
        function_export(
            module,
            "encodeString",
            ["A"],
            vec![InterfaceConstraint {
                name: "JsonEncode".to_owned(),
                trait_identity: Some("std/prelude::JsonEncode".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![named("A")],
            named("String"),
        ),
        function_export(
            module,
            "decodeString",
            ["A"],
            vec![InterfaceConstraint {
                name: "JsonDecode".to_owned(),
                trait_identity: Some("std/prelude::JsonDecode".to_owned()),
                arguments: vec![named("A")],
            }],
            vec![named("String")],
            named_with("Either", vec![read_error, named("A")]),
        ),
        function_export(
            module,
            "field",
            ["A"],
            Vec::new(),
            vec![named("String"), decoder(named("A"))],
            decoder(named("A")),
        ),
        function_export(
            module,
            "optionalField",
            ["A"],
            Vec::new(),
            vec![named("String"), decoder(named("A"))],
            decoder(named_with("Maybe", vec![named("A")])),
        ),
        function_export(
            module,
            "index",
            ["A"],
            Vec::new(),
            vec![named("Int"), decoder(named("A"))],
            decoder(named("A")),
        ),
        function_export(
            module,
            "array",
            ["A"],
            Vec::new(),
            vec![decoder(named("A"))],
            decoder(named_with("Array", vec![named("A")])),
        ),
        function_export(
            module,
            "record",
            ["A"],
            Vec::new(),
            vec![named_with(
                "Array",
                vec![InterfaceType::Tuple {
                    elements: vec![named("String"), decoder(named("A"))],
                }],
            )],
            decoder(named_with(
                "Array",
                vec![InterfaceType::Tuple {
                    elements: vec![named("String"), named("A")],
                }],
            )),
        ),
        function_export(
            module,
            "oneOf",
            ["A"],
            Vec::new(),
            vec![named_with("Array", vec![decoder(named("A"))])],
            decoder(named("A")),
        ),
        function_export(
            module,
            "map",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], named("B")),
                decoder(named("A")),
            ],
            decoder(named("B")),
        ),
        function_export(
            module,
            "flatMap",
            ["A", "B"],
            Vec::new(),
            vec![
                function_type(vec![named("A")], decoder(named("B"))),
                decoder(named("A")),
            ],
            decoder(named("B")),
        ),
    ]);
    standard_interface(module, exports)
}
