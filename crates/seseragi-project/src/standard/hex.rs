use super::*;

pub(super) fn hex_interface() -> ModuleInterface {
    let module = "std/bytes/hex";
    let bytes = external_type(
        "Bytes",
        "std/bytes::Bytes",
        "std/bytes",
        "Bytes",
        Vec::new(),
    );
    let error = external_type(
        "HexDecodeError",
        "std/bytes/hex::HexDecodeError",
        module,
        "HexDecodeError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "HexDecodeError", []),
            constructor_export(
                module,
                "HexDecodeError",
                "OddHexLength",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "HexDecodeError",
                "InvalidHexDigit",
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
                named_with("Either", vec![error, bytes]),
            ),
        ],
    )
}
