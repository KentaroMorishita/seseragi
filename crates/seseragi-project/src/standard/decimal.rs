use super::*;

pub(super) fn decimal_interface() -> ModuleInterface {
    let module = "std/decimal";
    let decimal = external_type(
        "Decimal",
        "std/decimal::Decimal",
        module,
        "Decimal",
        Vec::new(),
    );
    let decimal_context = external_type(
        "DecimalContext",
        "std/decimal::DecimalContext",
        module,
        "DecimalContext",
        Vec::new(),
    );
    let parse_error = external_type(
        "DecimalParseError",
        "std/decimal::DecimalParseError",
        module,
        "DecimalParseError",
        Vec::new(),
    );
    let context_error = external_type(
        "DecimalContextError",
        "std/decimal::DecimalContextError",
        module,
        "DecimalContextError",
        Vec::new(),
    );
    let arithmetic_error = external_type(
        "DecimalArithmeticError",
        "std/decimal::DecimalArithmeticError",
        module,
        "DecimalArithmeticError",
        Vec::new(),
    );
    let conversion_error = external_type(
        "DecimalConversionError",
        "std/decimal::DecimalConversionError",
        module,
        "DecimalConversionError",
        Vec::new(),
    );
    let rounding_mode = external_type(
        "RoundingMode",
        "std/number::RoundingMode",
        "std/number",
        "RoundingMode",
        Vec::new(),
    );
    standard_interface(
        module,
        vec![
            opaque_adt_type_export(module, "Decimal", []),
            opaque_adt_type_export(module, "DecimalContext", []),
            opaque_adt_type_export(module, "DecimalParseError", []),
            constructor_export(
                module,
                "DecimalParseError",
                "InvalidDecimal",
                [],
                Some(record([required("offset", named("Int"))])),
            ),
            opaque_adt_type_export(module, "DecimalContextError", []),
            constructor_export(
                module,
                "DecimalContextError",
                "NonPositiveDecimalPrecision",
                [],
                Some(named("Int")),
            ),
            opaque_adt_type_export(module, "DecimalArithmeticError", []),
            constructor_export(
                module,
                "DecimalArithmeticError",
                "DecimalDivisionByZero",
                [],
                None,
            ),
            constructor_export(
                module,
                "DecimalArithmeticError",
                "NonTerminatingDecimal",
                [],
                None,
            ),
            opaque_adt_type_export(module, "DecimalConversionError", []),
            constructor_export(
                module,
                "DecimalConversionError",
                "DecimalNotIntegral",
                [],
                None,
            ),
            constructor_export(
                module,
                "DecimalConversionError",
                "DecimalOutsideIntRange",
                [],
                None,
            ),
            constructor_export(
                module,
                "DecimalConversionError",
                "DecimalOutsideFloatRange",
                [],
                None,
            ),
            constructor_export(module, "DecimalConversionError", "FloatNotFinite", [], None),
            function_export(
                module,
                "parse",
                [],
                Vec::new(),
                vec![named("String")],
                named_with("Either", vec![parse_error, decimal.clone()]),
            ),
            function_export(
                module,
                "fromInt",
                [],
                Vec::new(),
                vec![named("Int")],
                decimal.clone(),
            ),
            function_export(
                module,
                "toIntExact",
                [],
                Vec::new(),
                vec![decimal.clone()],
                named_with("Either", vec![conversion_error.clone(), named("Int")]),
            ),
            function_export(
                module,
                "fromFloat",
                [],
                Vec::new(),
                vec![decimal_context.clone(), named("Float")],
                named_with("Either", vec![conversion_error.clone(), decimal.clone()]),
            ),
            function_export(
                module,
                "toFloat",
                [],
                Vec::new(),
                vec![decimal.clone()],
                named_with("Either", vec![conversion_error, named("Float")]),
            ),
            function_export(
                module,
                "context",
                [],
                Vec::new(),
                vec![named("Int"), rounding_mode.clone()],
                named_with("Either", vec![context_error, decimal_context.clone()]),
            ),
            function_export(
                module,
                "precision",
                [],
                Vec::new(),
                vec![decimal_context.clone()],
                named("Int"),
            ),
            function_export(
                module,
                "rounding",
                [],
                Vec::new(),
                vec![decimal_context.clone()],
                rounding_mode.clone(),
            ),
            function_export(
                module,
                "divideExact",
                [],
                Vec::new(),
                vec![decimal.clone(), decimal.clone()],
                named_with("Either", vec![arithmetic_error.clone(), decimal.clone()]),
            ),
            function_export(
                module,
                "divide",
                [],
                Vec::new(),
                vec![decimal_context, decimal.clone(), decimal.clone()],
                named_with("Either", vec![arithmetic_error, decimal.clone()]),
            ),
            function_export(
                module,
                "quantize",
                [],
                Vec::new(),
                vec![named("Int"), rounding_mode, decimal.clone()],
                decimal,
            ),
        ],
    )
}
