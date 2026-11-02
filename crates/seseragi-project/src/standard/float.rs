use super::*;

pub(super) fn float_interface() -> ModuleInterface {
    let module = "std/float";
    let parse_error = external_type(
        "FloatParseError",
        "std/float::FloatParseError",
        module,
        "FloatParseError",
        Vec::new(),
    );
    let conversion_error = external_type(
        "FloatConversionError",
        "std/float::FloatConversionError",
        module,
        "FloatConversionError",
        Vec::new(),
    );
    let rounding_mode = external_type(
        "RoundingMode",
        "std/number::RoundingMode",
        "std/number",
        "RoundingMode",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "FloatParseError", []),
        constructor_export(module, "FloatParseError", "EmptyFloat", [], None),
        constructor_export(
            module,
            "FloatParseError",
            "InvalidFloat",
            [],
            Some(record([required("offset", named("Int"))])),
        ),
        constructor_export(module, "FloatParseError", "FloatParseOverflow", [], None),
        opaque_adt_type_export(module, "FloatConversionError", []),
        constructor_export(module, "FloatConversionError", "FloatNotFinite", [], None),
        constructor_export(
            module,
            "FloatConversionError",
            "FloatOutsideIntRange",
            [],
            None,
        ),
    ];
    for name in ["nan", "positiveInfinity", "negativeInfinity"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Float"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error, named("Float")]),
        ),
        function_export(
            module,
            "format",
            [],
            Vec::new(),
            vec![named("Float")],
            named("String"),
        ),
        function_export(
            module,
            "fromInt",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Float"),
        ),
        function_export(
            module,
            "toInt",
            [],
            Vec::new(),
            vec![rounding_mode.clone(), named("Float")],
            named_with("Either", vec![conversion_error, named("Int")]),
        ),
    ]);
    for name in ["isNaN", "isFinite", "isInfinite", "isNegativeZero"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float")],
            named("Bool"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "ieeeEq",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Bool"),
        ),
        function_export(
            module,
            "totalCompare",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Ordering"),
        ),
    ]);
    for name in ["minimumNumber", "maximumNumber"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Float"),
        ));
    }
    exports.extend([
        function_export(
            module,
            "clampNumber",
            [],
            Vec::new(),
            vec![named("Float"), named("Float"), named("Float")],
            named_with("Maybe", vec![named("Float")]),
        ),
        function_export(
            module,
            "abs",
            [],
            Vec::new(),
            vec![named("Float")],
            named("Float"),
        ),
        function_export(
            module,
            "sign",
            [],
            Vec::new(),
            vec![named("Float")],
            named_with("Maybe", vec![named("Int")]),
        ),
        function_export(
            module,
            "power",
            [],
            Vec::new(),
            vec![named("Float"), named("Float")],
            named("Float"),
        ),
        function_export(
            module,
            "roundIntegral",
            [],
            Vec::new(),
            vec![rounding_mode, named("Float")],
            named("Float"),
        ),
    ]);
    standard_interface(module, exports)
}
