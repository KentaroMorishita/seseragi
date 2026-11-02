fn native_decimal_conversion_error(tag: &'static str) -> V {
    native_left(V::adt(tag, V::Unit))
}

fn native_decimal_to_int_exact(value: &NativeDecimal) -> V {
    if native_decimal_signed_compare(&value.scale, "0").is_gt() {
        return native_decimal_conversion_error("std/decimal::DecimalNotIntegral");
    }
    let digits = value.coefficient.trim_start_matches('-').len();
    let integer_digits =
        native_decimal_add(&digits.to_string(), &native_decimal_negate(&value.scale));
    if native_decimal_signed_compare(&integer_digits, "16").is_gt() {
        return native_decimal_conversion_error("std/decimal::DecimalOutsideIntRange");
    }
    // The digit bound proves that expansion fits at most 16 decimal digits.
    let zeros = native_decimal_negate(&value.scale)
        .parse::<usize>()
        .expect("bounded integer scale");
    let mut integer = value.coefficient.clone();
    integer.extend(std::iter::repeat_n('0', zeros));
    let integer = integer
        .parse::<i64>()
        .expect("bounded canonical decimal integer");
    if !(-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&integer) {
        native_decimal_conversion_error("std/decimal::DecimalOutsideIntRange")
    } else {
        native_right(V::Int(integer))
    }
}

fn native_decimal_to_float(value: &NativeDecimal) -> V {
    if value.coefficient == "0" {
        return native_right(V::Float(0.0));
    }
    let negative = value.coefficient.starts_with('-');
    let coefficient = value.coefficient.trim_start_matches('-');
    let exponent = native_decimal_add(
        &(coefficient.len() - 1).to_string(),
        &native_decimal_negate(&value.scale),
    );
    if native_decimal_signed_compare(&exponent, "308").is_gt() {
        return native_decimal_conversion_error("std/decimal::DecimalOutsideFloatRange");
    }
    if native_decimal_signed_compare(&exponent, "-324").is_lt() {
        return native_right(V::Float(if negative { -0.0 } else { 0.0 }));
    }
    // Check against the exact largest Float before binary rounding. A decimal
    // one unit above it must fail even if parsing would round back to MAX.
    let maximum = NativeBigInt::from_i64(9_007_199_254_740_991)
        .multiply(&NativeBigInt::from_i64(2).power(971));
    let maximum = native_decimal_make(maximum.decimal(), "0".into());
    let magnitude = NativeDecimal {
        coefficient: coefficient.into(),
        scale: value.scale.clone(),
    };
    if native_decimal_compare_values(&magnitude, &maximum).is_gt() {
        return native_decimal_conversion_error("std/decimal::DecimalOutsideFloatRange");
    }
    // Scientific parsing avoids the native display path's expansion bound.
    let scientific = format!(
        "{}e{}",
        value.coefficient,
        native_decimal_negate(&value.scale)
    );
    let converted = scientific
        .parse::<f64>()
        .expect("canonical scientific decimal");
    if converted.is_finite() {
        native_right(V::Float(converted))
    } else {
        native_decimal_conversion_error("std/decimal::DecimalOutsideFloatRange")
    }
}

fn native_decimal_round_significant(
    value: &NativeDecimal,
    context: &NativeDecimalContext,
) -> NativeDecimal {
    assert!(
        context.digits > 0,
        "Rust ABI: nonpositive DecimalContext precision"
    );
    let copy = || NativeDecimal {
        coefficient: value.coefficient.clone(),
        scale: value.scale.clone(),
    };
    let Ok(precision) = usize::try_from(context.digits) else {
        return copy();
    };
    let negative = value.coefficient.starts_with('-');
    let digits = value.coefficient.trim_start_matches('-');
    if precision >= digits.len() {
        return copy();
    }
    let (kept, discarded) = digits.split_at(precision);
    let nonzero = discarded.bytes().any(|digit| digit != b'0');
    let half = discarded.as_bytes()[0].cmp(&b'5').then_with(|| {
        if discarded.as_bytes()[1..].iter().any(|digit| *digit != b'0') {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });
    let increment = match context.rounding {
        NativeRoundingMode::HalfEven => {
            half.is_gt() || (half.is_eq() && (kept.as_bytes()[precision - 1] - b'0') % 2 != 0)
        }
        NativeRoundingMode::HalfUp => !half.is_lt(),
        NativeRoundingMode::TowardZero => false,
        NativeRoundingMode::AwayFromZero => nonzero,
        NativeRoundingMode::Floor => negative && nonzero,
        NativeRoundingMode::Ceiling => !negative && nonzero,
    };
    let coefficient = native_decimal_signed(negative, kept);
    let coefficient = if increment {
        native_decimal_add(&coefficient, if negative { "-1" } else { "1" })
    } else {
        coefficient
    };
    let dropped = (digits.len() - precision).to_string();
    let scale = native_decimal_add(&value.scale, &native_decimal_negate(&dropped));
    native_decimal_make(&coefficient, scale)
}

fn native_decimal_from_float(context: &NativeDecimalContext, value: f64) -> V {
    if !value.is_finite() {
        return native_decimal_conversion_error("std/decimal::FloatNotFinite");
    }
    if value == 0.0 {
        return native_right(V::Decimal(Shared::new(native_decimal_make(
            "0",
            "0".into(),
        ))));
    }
    let bits = value.to_bits();
    let negative = bits >> 63 != 0;
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = bits & 0x000f_ffff_ffff_ffff;
    let significand = if exponent == 0 {
        fraction
    } else {
        (1_u64 << 52) | fraction
    };
    let binary_exponent = if exponent == 0 {
        -1074
    } else {
        exponent - 1023 - 52
    };
    let factor = NativeBigInt::from_i64(if binary_exponent >= 0 { 2 } else { 5 })
        .power(binary_exponent.abs());
    let coefficient = NativeBigInt::from_i64(significand as i64).multiply(&factor);
    let scale = if binary_exponent >= 0 {
        "0".into()
    } else {
        (-binary_exponent).to_string()
    };
    let exact = native_decimal_make(
        &native_decimal_signed(negative, coefficient.decimal()),
        scale,
    );
    native_right(V::Decimal(Shared::new(native_decimal_round_significant(
        &exact, context,
    ))))
}

#[cfg(test)]
mod decimal_conversion_tests {
    use super::*;
    fn decimal(text: &str) -> V {
        let value = native_decimal_parse(text);
        assert!(value.tagged("std/prelude::Right"));
        value.payload()
    }
    fn error(value: V, tag: &str) {
        assert!(value.tagged("std/prelude::Left"));
        assert!(value.payload().tagged(tag));
    }
    #[test]
    fn exact_integer_conversion_bounds_scale_before_expanding_it() {
        for (input, expected) in [
            ("0", 0),
            ("-0e100000", 0),
            ("42.000", 42),
            ("-42.0", -42),
            ("9.007199254740991e15", 9_007_199_254_740_991),
            ("-9.007199254740991e15", -9_007_199_254_740_991),
        ] {
            let result =
                native_decimal_standard_operation("std/decimal::toIntExact", vec![decimal(input)]);
            assert!(result.tagged("std/prelude::Right"));
            assert!(matches!(result.payload(),V::Int(value) if value==expected));
        }
        for input in ["0.1", "-1e-100000000000000000000000000000000000000000"] {
            error(
                native_decimal_standard_operation("std/decimal::toIntExact", vec![decimal(input)]),
                "std/decimal::DecimalNotIntegral",
            );
        }
        for input in [
            "9007199254740992",
            "-9007199254740992",
            "1e100000000000000000000000000000000000000000",
        ] {
            error(
                native_decimal_standard_operation("std/decimal::toIntExact", vec![decimal(input)]),
                "std/decimal::DecimalOutsideIntRange",
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn float_conversion_keeps_signed_underflow_and_the_exact_maximum_boundary() {
        for (input, expected) in [
            (
                "-1e-100000000000000000000000000000000000000000",
                (-0.0_f64).to_bits(),
            ),
            ("1e-325", 0_f64.to_bits()),
            ("4.9406564584124654e-324", 1),
            ("0.1", 0.1_f64.to_bits()),
        ] {
            let result =
                native_decimal_standard_operation("std/decimal::toFloat", vec![decimal(input)]);
            assert!(result.tagged("std/prelude::Right"));
            assert!(matches!(result.payload(),V::Float(value) if value.to_bits()==expected));
        }
        let maximum = NativeBigInt::from_i64(9_007_199_254_740_991)
            .multiply(&NativeBigInt::from_i64(2).power(971));
        let next = maximum.add(&NativeBigInt::from_i64(1));
        {
            let value = native_decimal_make(maximum.decimal(), "0".into());
            assert!(
                matches!(native_decimal_to_float(&value).payload(),V::Float(value) if value.to_bits()==f64::MAX.to_bits())
            );
            let value = native_decimal_make(next.decimal(), "0".into());
            error(
                native_decimal_to_float(&value),
                "std/decimal::DecimalOutsideFloatRange",
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn float_inputs_use_their_exact_binary_value_and_all_shared_rounding_modes() {
        for (mode, positive, negative) in [
            (NativeRoundingMode::HalfEven, "2", "-2"),
            (NativeRoundingMode::HalfUp, "3", "-3"),
            (NativeRoundingMode::TowardZero, "2", "-2"),
            (NativeRoundingMode::AwayFromZero, "3", "-3"),
            (NativeRoundingMode::Floor, "2", "-3"),
            (NativeRoundingMode::Ceiling, "3", "-2"),
        ] {
            let context = NativeDecimalContext {
                digits: 1,
                rounding: mode,
            };
            assert_eq!(
                native_decimal_from_float(&context, 2.5).payload().render(),
                positive
            );
            assert_eq!(
                native_decimal_from_float(&context, -2.5).payload().render(),
                negative
            );
        }
        let context = NativeDecimalContext {
            digits: 1,
            rounding: NativeRoundingMode::Ceiling,
        };
        assert_eq!(
            native_decimal_from_float(&context, 0.1).payload().render(),
            "0.2"
        );
        let context = NativeDecimalContext {
            digits: 9_007_199_254_740_991,
            rounding: NativeRoundingMode::HalfEven,
        };
        assert_eq!(
            native_decimal_from_float(&context, 0.1).payload().render(),
            "0.1000000000000000055511151231257827021181583404541015625"
        );
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            error(
                native_decimal_from_float(&context, value),
                "std/decimal::FloatNotFinite",
            );
        }
        assert_eq!(
            native_decimal_from_float(&context, -0.0).payload().render(),
            "0"
        );
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn seventeen_digits_round_trip_original_finite_float_bits() {
        let context = NativeDecimalContext {
            digits: 17,
            rounding: NativeRoundingMode::HalfEven,
        };
        let mut bits = 0x5eed_b17d_0123_4567_u64;
        for _ in 0..256 {
            bits = bits.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let value = f64::from_bits(bits);
            if !value.is_finite() {
                continue;
            }
            let parsed = native_decimal_from_float(&context, value).payload();
            let converted = native_decimal_standard_operation("std/decimal::toFloat", vec![parsed]);
            assert!(converted.tagged("std/prelude::Right"));
            let expected = if value == 0.0 { 0 } else { bits };
            assert!(
                matches!(converted.payload(),V::Float(value) if value.to_bits()==expected),
                "{bits:016x}"
            );
        }
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
