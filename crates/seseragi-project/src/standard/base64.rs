use super::*;

pub(super) fn base64_interface() -> ModuleInterface {
    let module = "std/bytes/base64";
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let error = external_type(
        "Base64DecodeError",
        "std/bytes/base64::Base64DecodeError",
        module,
        "Base64DecodeError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "Base64DecodeError", []),
            constructor_export(
                module,
                "Base64DecodeError",
                "InvalidBase64Length",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "Base64DecodeError",
                "InvalidBase64Digit",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "Base64DecodeError",
                "InvalidBase64Padding",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            constructor_export(
                module,
                "Base64DecodeError",
                "NonCanonicalBase64Bits",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            function_export(
                module,
                "encode",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named("String"),
            ),
            function_export(
                module,
                "decode",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Either", vec![error.clone(), bytes.clone()]),
            ),
            function_export(
                module,
                "encodeUrl",
                [],
                Vec::new(),
                vec![bytes.clone()],
                named("String"),
            ),
            function_export(
                module,
                "decodeUrl",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Either", vec![error, bytes]),
            ),
        ],
    )
}
