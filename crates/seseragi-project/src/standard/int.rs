use super::*;

pub(super) fn int_interface() -> ModuleInterface {
    let module = "std/int";
    let parse_error = external_type(
        "IntParseError",
        "std/int::IntParseError",
        module,
        "IntParseError",
        Vec::new(),
    );
    let division_error = external_type(
        "IntDivisionError",
        "std/int::IntDivisionError",
        module,
        "IntDivisionError",
        Vec::new(),
    );
    let power_error = external_type(
        "IntPowerError",
        "std/int::IntPowerError",
        module,
        "IntPowerError",
        Vec::new(),
    );
    let mut exports = vec![
        opaque_adt_type_export(module, "IntParseError", []),
        constructor_export(module, "IntParseError", "EmptyInt", [], None),
        constructor_export(
            module,
            "IntParseError",
            "InvalidIntRadix",
            [],
            Some(named("Int")),
        ),
        constructor_export(
            module,
            "IntParseError",
            "InvalidIntDigit",
            [],
            Some(record([
                required("offset", named("Int")),
                required("radix", named("Int")),
            ])),
        ),
        constructor_export(module, "IntParseError", "IntOutsideRange", [], None),
        opaque_adt_type_export(module, "IntDivisionError", []),
        constructor_export(module, "IntDivisionError", "IntDivisionByZero", [], None),
        opaque_adt_type_export(module, "IntPowerError", []),
        constructor_export(
            module,
            "IntPowerError",
            "NegativeIntExponent",
            [],
            Some(named("Int")),
        ),
        constructor_export(module, "IntPowerError", "IntPowerOverflow", [], None),
        function_export(
            module,
            "minValue",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Int"),
        ),
        function_export(
            module,
            "maxValue",
            [],
            Vec::new(),
            vec![named("Unit")],
            named("Int"),
        ),
        function_export(
            module,
            "parse",
            [],
            Vec::new(),
            vec![named("String")],
            named_with("Either", vec![parse_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "parseRadix",
            [],
            Vec::new(),
            vec![named("Int"), named("String")],
            named_with("Either", vec![parse_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "format",
            [],
            Vec::new(),
            vec![named("Int")],
            named("String"),
        ),
        function_export(
            module,
            "formatRadix",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![parse_error, named("String")]),
        ),
    ];

    for name in ["checkedAdd", "checkedSubtract", "checkedMultiply"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Maybe", vec![named("Int")]),
        ));
    }
    for name in [
        "saturatingAdd",
        "saturatingSubtract",
        "saturatingMultiply",
        "minimum",
        "maximum",
    ] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named("Int"),
        ));
    }
    for name in ["checkedDivide", "checkedRemainder"] {
        exports.push(function_export(
            module,
            name,
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![division_error.clone(), named("Int")]),
        ));
    }
    exports.extend([
        function_export(
            module,
            "checkedPower",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![power_error.clone(), named("Int")]),
        ),
        function_export(
            module,
            "saturatingPower",
            [],
            Vec::new(),
            vec![named("Int"), named("Int")],
            named_with("Either", vec![power_error, named("Int")]),
        ),
        function_export(
            module,
            "abs",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Int"),
        ),
        function_export(
            module,
            "clamp",
            [],
            Vec::new(),
            vec![named("Int"), named("Int"), named("Int")],
            named("Int"),
        ),
        function_export(
            module,
            "sign",
            [],
            Vec::new(),
            vec![named("Int")],
            named("Int"),
        ),
    ]);
    standard_interface(module, exports)
}
