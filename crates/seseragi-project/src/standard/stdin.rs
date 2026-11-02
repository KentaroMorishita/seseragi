use super::*;

pub(super) fn stdin_interface() -> ModuleInterface {
    let module = "std/stdin";
    let stdin = || prelude_type("Stdin");
    let stdin_error = || prelude_type("StdinError");
    let environment = || record([required("stdin", stdin())]);
    let bytes = || {
        external_type(
            "Bytes",
            "std/bytes::Bytes",
            "std/bytes",
            "Bytes",
            Vec::new(),
        )
    };
    let stream = |value| {
        external_type(
            "Stream",
            "std/stream::Stream",
            "std/stream",
            "Stream",
            vec![environment(), stdin_error(), value],
        )
    };
    let mut exports = vec![
        canonical_type_export(module, "Stdin", "std/prelude::Stdin", 0, "opaque-type"),
        opaque_adt_type_export(module, "StdinConfigError", []),
    ];
    for (constructor, payload) in [
        ("NonPositiveReadSize", named("Int")),
        ("ReadSizeTooLarge", named("Int")),
        ("NonPositiveLineLimit", named("Int")),
        ("LineLimitTooLarge", named("Int")),
    ] {
        exports.push(constructor_export(
            module,
            "StdinConfigError",
            constructor,
            [],
            Some(payload),
        ));
    }
    exports.extend([
        type_export(module, "ReadSize", 0, "opaque-type"),
        type_export(module, "LineLimit", 0, "opaque-type"),
        function_export(
            module,
            "readSize",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with("Either", vec![named("StdinConfigError"), named("ReadSize")]),
        ),
        function_export(
            module,
            "lineLimit",
            [],
            Vec::new(),
            vec![named("Int")],
            named_with(
                "Either",
                vec![named("StdinConfigError"), named("LineLimit")],
            ),
        ),
        function_export(
            module,
            "defaultReadSize",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("ReadSize"),
        ),
        function_export(
            module,
            "defaultLineLimit",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("LineLimit"),
        ),
        canonical_type_export(
            module,
            "StdinError",
            "std/prelude::StdinError",
            0,
            "opaque-type",
        ),
    ]);
    for (constructor, payload) in [
        ("StdinUnavailable", None),
        ("StdinReadFailure", None),
        ("ConcurrentStdinRead", None),
        (
            "InvalidStdinUtf8",
            Some(record([required("offset", named("Int"))])),
        ),
        (
            "StdinLineTooLong",
            Some(record([required("limitBytes", named("Int"))])),
        ),
        ("StdinPositionOverflow", None),
    ] {
        exports.push(canonical_constructor_export(
            module,
            "StdinError",
            "std/prelude::StdinError",
            constructor,
            payload,
        ));
    }
    exports.extend([
        effect_function_export(
            module,
            "readChunk",
            [],
            Vec::new(),
            vec![named("ReadSize")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![bytes()]),
            ),
        ),
        effect_function_export(
            module,
            "readLine",
            [],
            Vec::new(),
            vec![named("Unit")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![named("String")]),
            ),
        ),
        effect_function_export(
            module,
            "readLineWith",
            [],
            Vec::new(),
            vec![named("LineLimit")],
            effect(
                environment(),
                stdin_error(),
                named_with("Maybe", vec![named("String")]),
            ),
        ),
        function_export(
            module,
            "lines",
            [],
            Vec::new(),
            vec![named("LineLimit")],
            stream(named("String")),
        ),
    ]);
    standard_interface(module, exports)
}
