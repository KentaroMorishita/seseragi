use super::*;

pub(super) fn big_int_interface() -> ModuleInterface {
    let module = "std/big-int";
    let big_int = external_type(
        "BigInt",
        "std/big-int::BigInt",
        module,
        "BigInt",
        Vec::new(),
    );
    let parse_error = external_type(
        "BigIntParseError",
        "std/big-int::BigIntParseError",
        module,
        "BigIntParseError",
        Vec::new(),
    );
    let division_error = external_type(
        "BigIntDivisionError",
        "std/big-int::BigIntDivisionError",
        module,
        "BigIntDivisionError",
        Vec::new(),
    );
    let power_error = external_type(
        "BigIntPowerError",
        "std/big-int::BigIntPowerError",
        module,
        "BigIntPowerError",
        Vec::new(),
    );
    let conversion_error = external_type(
        "BigIntConversionError",
        "std/big-int::BigIntConversionError",
        module,
        "BigIntConversionError",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "BigInt", []),
            opaque_adt_type_export(module, "BigIntParseError", []),
            constructor_export(module, "BigIntParseError", "EmptyBigInt", [], None),
            constructor_export(
                module,
                "BigIntParseError",
                "InvalidBigIntRadix",
                [],
                Some(named("Int")),
            ),
            constructor_export(
                module,
                "BigIntParseError",
                "InvalidBigIntDigit",
                [],
                Some(record([
                    required("offset", named("Int")),
                    required("radix", named("Int")),
                ])),
            ),
            opaque_adt_type_export(module, "BigIntDivisionError", []),
            constructor_export(
                module,
                "BigIntDivisionError",
                "BigIntDivisionByZero",
                [],
                None,
            ),
            opaque_adt_type_export(module, "BigIntPowerError", []),
            constructor_export(
                module,
                "BigIntPowerError",
                "NegativeBigIntExponent",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "BigIntConversionError", []),
            constructor_export(
                module,
                "BigIntConversionError",
                "BigIntOutsideIntRange",
                [],
                None,
            ),
            function_export(
                module,
                "parse",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Either", vec![parse_error.clone(), big_int.clone()]),
            ),
            function_export(
                module,
                "parseRadix",
                [],
                Vec::new(),
                vec![named("Int"), named("String")],
                named_with("Either", vec![parse_error.clone(), big_int.clone()]),
            ),
            function_export(
                module,
                "format",
                [],
                Vec::new(),
                vec![big_int.clone()],
                named("String"),
            ),
            function_export(
                module,
                "formatRadix",
                [],
                Vec::new(),
                vec![named("Int"), big_int.clone()],
                named_with("Either", vec![parse_error, named("String")]),
            ),
            function_export(
                module,
                "fromInt",
                [],
                Vec::new(),
                vec![named("Int")],
                big_int.clone(),
            ),
            function_export(
                module,
                "toInt",
                [],
                Vec::new(),
                vec![big_int.clone()],
                named_with("Either", vec![conversion_error, named("Int")]),
            ),
            function_export(
                module,
                "checkedDivide",
                [],
                Vec::new(),
                vec![big_int.clone(), big_int.clone()],
                named_with("Either", vec![division_error.clone(), big_int.clone()]),
            ),
            function_export(
                module,
                "checkedRemainder",
                [],
                Vec::new(),
                vec![big_int.clone(), big_int.clone()],
                named_with("Either", vec![division_error, big_int.clone()]),
            ),
            function_export(
                module,
                "checkedPower",
                [],
                Vec::new(),
                vec![named("Int"), big_int.clone()],
                named_with("Either", vec![power_error, big_int.clone()]),
            ),
            function_export(
                module,
                "abs",
                [],
                Vec::new(),
                vec![big_int.clone()],
                big_int.clone(),
            ),
            function_export(
                module,
                "sign",
                [],
                Vec::new(),
                vec![big_int.clone()],
                named("Int"),
            ),
        ],
    )
}
