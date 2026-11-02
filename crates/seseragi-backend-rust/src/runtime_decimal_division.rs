fn native_decimal_integer(coefficient: &str) -> NativeBigInt {
    NativeBigInt::from_digits(
        coefficient.starts_with('-'),
        coefficient.trim_start_matches('-').as_bytes(),
        10,
    )
}

fn native_decimal_arithmetic_error(tag: &'static str) -> V {
    native_left(V::adt(tag, V::Unit))
}

fn native_decimal_integer_power(base: i64, mut exponent: usize) -> NativeBigInt {
    let mut result = NativeBigInt::from_i64(1);
    let mut factor = NativeBigInt::from_i64(base);
    while exponent != 0 {
        if exponent & 1 != 0 {
            result = result.multiply(&factor);
        }
        exponent >>= 1;
        if exponent != 0 {
            factor = factor.multiply(&factor);
        }
    }
    result
}

fn native_decimal_small_quotient(value: &NativeBigInt, divisor: u32) -> (NativeBigInt, u32) {
    let mut quotient = value.limbs.clone();
    let mut remainder = 0_u64;
    for word in quotient.iter_mut().rev() {
        let current = (remainder << 32) | u64::from(*word);
        *word = (current / u64::from(divisor)) as u32;
        remainder = current % u64::from(divisor);
    }
    (
        NativeBigInt::from_limbs(value.negative, quotient),
        remainder as u32,
    )
}

fn native_decimal_remove_factor(mut value: NativeBigInt, prime: u32) -> (NativeBigInt, usize) {
    let mut count = 0_usize;
    loop {
        let (quotient, remainder) = native_decimal_small_quotient(&value, prime);
        if remainder != 0 {
            return (value, count);
        }
        value = quotient;
        count = native_decimal_host_count(
            count.checked_add(1),
            "Decimal factor count exceeds host limits",
        );
    }
}

fn native_decimal_divide_exact(divisor: &NativeDecimal, dividend: &NativeDecimal) -> V {
    if divisor.coefficient == "0" {
        return native_decimal_arithmetic_error("std/decimal::DecimalDivisionByZero");
    }
    if dividend.coefficient == "0" {
        return native_right(V::Decimal(Shared::new(native_decimal_make(
            "0",
            "0".into(),
        ))));
    }
    let numerator = native_decimal_integer(&dividend.coefficient);
    let denominator = native_decimal_integer(&divisor.coefficient);
    let mut a = NativeBigInt::from_limbs(false, numerator.limbs.clone());
    let mut b = NativeBigInt::from_limbs(false, denominator.limbs.clone());
    while !b.is_zero() {
        let (_, remainder) = a.quotient_remainder(&b);
        a = b;
        b = remainder;
    }
    let mut numerator = numerator.quotient_remainder(&a).0;
    let mut denominator = denominator.quotient_remainder(&a).0;
    if denominator.negative {
        numerator = NativeBigInt::from_limbs(!numerator.negative, numerator.limbs);
        denominator = NativeBigInt::from_limbs(false, denominator.limbs);
    }
    let (denominator, twos) = native_decimal_remove_factor(denominator, 2);
    let (denominator, fives) = native_decimal_remove_factor(denominator, 5);
    if denominator.limbs.as_slice() != [1] {
        return native_decimal_arithmetic_error("std/decimal::NonTerminatingDecimal");
    }
    let places = twos.max(fives);
    let coefficient = numerator
        .multiply(&native_decimal_integer_power(2, places - twos))
        .multiply(&native_decimal_integer_power(5, places - fives));
    let scale = native_decimal_add(
        &native_decimal_add(&places.to_string(), &dividend.scale),
        &native_decimal_negate(&divisor.scale),
    );
    native_right(V::Decimal(Shared::new(native_decimal_make(
        coefficient.decimal(),
        scale,
    ))))
}

fn native_decimal_round_ratio(
    numerator: &NativeBigInt,
    denominator: &NativeBigInt,
    mode: NativeRoundingMode,
) -> NativeBigInt {
    assert!(
        !denominator.negative && !denominator.is_zero(),
        "Rust ABI: expected positive Decimal denominator"
    );
    let (quotient, remainder) = numerator.quotient_remainder(denominator);
    if remainder.is_zero() {
        return quotient;
    }
    let twice =
        NativeBigInt::from_limbs(false, remainder.limbs).multiply(&NativeBigInt::from_i64(2));
    let half = NativeBigInt::magnitude_order(&twice.limbs, &denominator.limbs);
    let increment = match mode {
        NativeRoundingMode::HalfEven => {
            half.is_gt() || (half.is_eq() && quotient.limbs[0] & 1 != 0)
        }
        NativeRoundingMode::HalfUp => !half.is_lt(),
        NativeRoundingMode::TowardZero => false,
        NativeRoundingMode::AwayFromZero => true,
        NativeRoundingMode::Floor => numerator.negative,
        NativeRoundingMode::Ceiling => !numerator.negative,
    };
    if increment {
        quotient.add(&NativeBigInt::from_i64(if numerator.negative {
            -1
        } else {
            1
        }))
    } else {
        quotient
    }
}

fn native_decimal_divide(
    context: &NativeDecimalContext,
    divisor: &NativeDecimal,
    dividend: &NativeDecimal,
) -> V {
    if divisor.coefficient == "0" {
        return native_decimal_arithmetic_error("std/decimal::DecimalDivisionByZero");
    }
    if dividend.coefficient == "0" {
        return native_right(V::Decimal(Shared::new(native_decimal_make(
            "0",
            "0".into(),
        ))));
    }
    let numerator_digits = dividend.coefficient.trim_start_matches('-');
    let denominator_digits = divisor.coefficient.trim_start_matches('-');
    let candidate = native_decimal_add(
        &numerator_digits.len().to_string(),
        &native_decimal_negate(&denominator_digits.len().to_string()),
    );
    // Compare the coefficient ratio with 10^candidate without expanding the
    // value's arbitrary scale. Only explicit result precision needs digits.
    let width = numerator_digits.len().max(denominator_digits.len());
    let at_least_candidate = numerator_digits
        .bytes()
        .chain(std::iter::repeat(b'0'))
        .take(width)
        .cmp(
            denominator_digits
                .bytes()
                .chain(std::iter::repeat(b'0'))
                .take(width),
        )
        .is_ge();
    let ratio_exponent = if at_least_candidate {
        candidate
    } else {
        native_decimal_add(&candidate, "-1")
    };
    let decimal_power = native_decimal_add(
        &(context.digits - 1).to_string(),
        &native_decimal_negate(&ratio_exponent),
    );
    let target_scale = native_decimal_add(
        &decimal_power,
        &native_decimal_add(&dividend.scale, &native_decimal_negate(&divisor.scale)),
    );
    let mut numerator = native_decimal_signed(
        dividend.coefficient.starts_with('-') != divisor.coefficient.starts_with('-'),
        numerator_digits,
    );
    let mut denominator = denominator_digits.to_owned();
    if decimal_power.starts_with('-') {
        native_decimal_zeros(&mut denominator, decimal_power.trim_start_matches('-'));
    } else {
        native_decimal_zeros(&mut numerator, &decimal_power);
    }
    let coefficient = native_decimal_round_ratio(
        &native_decimal_integer(&numerator),
        &native_decimal_integer(&denominator),
        context.rounding,
    );
    native_right(V::Decimal(Shared::new(native_decimal_make(
        coefficient.decimal(),
        target_scale,
    ))))
}

fn native_decimal_quantize(
    target_scale: i64,
    mode: NativeRoundingMode,
    value: &NativeDecimal,
) -> NativeDecimal {
    let target_scale = checked_int_value(target_scale).to_string();
    if value.coefficient == "0"
        || !native_decimal_signed_compare(&target_scale, &value.scale).is_lt()
    {
        return native_decimal_copy(value);
    }
    let difference = native_decimal_add(&value.scale, &native_decimal_negate(&target_scale));
    let negative = value.coefficient.starts_with('-');
    let digits = value.coefficient.trim_start_matches('-');
    let length = digits.len().to_string();
    let (kept, half, nonzero) = if native_decimal_signed_compare(&difference, &length).is_gt() {
        ("0", std::cmp::Ordering::Less, true)
    } else {
        // The string comparison proves that this count is bounded by an
        // already materialized coefficient. No arbitrary scale is expanded.
        let count = native_decimal_host_count(
            difference.parse::<usize>().ok(),
            "bounded Decimal quantize count",
        );
        let (kept, dropped) = digits.split_at(digits.len() - count);
        let half = dropped.as_bytes()[0].cmp(&b'5').then_with(|| {
            if dropped.as_bytes()[1..].iter().any(|digit| *digit != b'0') {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        });
        (
            if kept.is_empty() { "0" } else { kept },
            half,
            dropped.bytes().any(|digit| digit != b'0'),
        )
    };
    let increment = match mode {
        NativeRoundingMode::HalfEven => {
            half.is_gt() || (half.is_eq() && (kept.as_bytes()[kept.len() - 1] - b'0') & 1 != 0)
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
    native_decimal_make(&coefficient, target_scale)
}

#[cfg(test)]
mod decimal_division_tests {
    use super::*;
    fn value(text: &str) -> NativeDecimal {
        let parsed = native_decimal_parse(text).payload();
        let V::Decimal(value) = parsed else { panic!() };
        native_decimal_copy(value.get())
    }
    #[test]
    fn host_counts_keep_the_last_valid_value_and_overflow_is_a_defect() {
        let maximum = usize::MAX.to_string();
        let overflow = native_decimal_add(&maximum, "1");
        for message in [
            "Decimal factor count exceeds host limits",
            "bounded Decimal quantize count",
        ] {
            assert_eq!(
                native_decimal_host_count((usize::MAX - 1).checked_add(1), message),
                usize::MAX
            );
            assert_eq!(
                native_decimal_host_count(maximum.parse().ok(), message),
                usize::MAX
            );
            for count in [usize::MAX.checked_add(1), overflow.parse().ok()] {
                let result = catch_native(|| {
                    native_decimal_host_count(count, message);
                    Ok(V::Unit)
                });
                assert!(
                    matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &[message])
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn factor_counts_cross_limb_boundaries_without_losing_precision() {
        for prime in [2, 5] {
            for count in [31, 32, 33, 63, 64, 65, 127, 128, 129] {
                let input = native_decimal_integer_power(prime.into(), count)
                    .multiply(&NativeBigInt::from_i64(3));
                let (remainder, actual) = native_decimal_remove_factor(input, prime);
                assert_eq!(remainder.decimal(), "3", "{prime}^{count}");
                assert_eq!(actual, count, "{prime}^{count}");
            }
        }
    }

    #[test]
    fn quantize_scales_around_host_count_limits_do_not_materialize_a_power() {
        let maximum = usize::MAX.to_string();
        for scale in [
            native_decimal_add(&maximum, "-1"),
            maximum.clone(),
            native_decimal_add(&maximum, "1"),
        ] {
            for (coefficient, mode, expected) in [
                ("5", NativeRoundingMode::HalfEven, "0"),
                ("-5", NativeRoundingMode::HalfUp, "0"),
                ("5", NativeRoundingMode::Ceiling, "1"),
                ("-5", NativeRoundingMode::Floor, "-1"),
            ] {
                let input = native_decimal_make(coefficient, scale.clone());
                let result = native_decimal_quantize(0, mode, &input);
                assert_eq!(result.coefficient, expected);
                assert_eq!(result.scale, "0");
            }
        }
    }

    #[test]
    fn exact_division_reduces_factors_preserves_sign_scale_and_typed_errors() {
        for (divisor, dividend, expected) in [
            ("2", "5", "2.5"),
            ("-2", "5", "-2.5"),
            ("-2", "-5", "2.5"),
            ("0.0008", "-1.2", "-1500"),
            ("6", "9", "1.5"),
            ("3", "6", "2"),
            ("625", "1", "0.0016"),
            ("1024", "1", "0.0009765625"),
            ("-0.001", "0", "0"),
        ] {
            let result = native_decimal_divide_exact(&value(divisor), &value(dividend));
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(result.payload().render(), expected, "{dividend}/{divisor}");
        }
        for (divisor, dividend, tag) in [
            ("0", "0", "std/decimal::DecimalDivisionByZero"),
            ("0", "1", "std/decimal::DecimalDivisionByZero"),
            ("3", "1", "std/decimal::NonTerminatingDecimal"),
            ("-6", "-1", "std/decimal::NonTerminatingDecimal"),
            ("7", "100", "std/decimal::NonTerminatingDecimal"),
        ] {
            let result = native_decimal_divide_exact(&value(divisor), &value(dividend));
            assert!(result.tagged("std/prelude::Left"));
            assert!(result.payload().tagged(tag));
        }
        let exponent = "9".repeat(1000);
        let huge = NativeDecimal {
            coefficient: "1".into(),
            scale: format!("-{exponent}"),
        };
        assert_eq!(
            native_decimal_divide_exact(&huge, &huge).payload().render(),
            "1"
        );
        let denominator = native_decimal_integer_power(2, 1024);
        let result = native_decimal_divide_exact(
            &native_decimal_make(denominator.decimal(), "0".into()),
            &value("1"),
        );
        let V::Decimal(result) = result.payload() else {
            panic!()
        };
        assert_eq!(result.get().scale, "1024");
        assert_eq!(
            result.get().coefficient,
            native_decimal_integer_power(5, 1024).decimal()
        );
    }
    #[test]
    fn exact_small_rationals_match_independent_i128_factorization() {
        for numerator in -50_i128..=50 {
            for denominator in -40_i128..=40 {
                if denominator == 0 {
                    continue;
                }
                let (mut a, mut b) = (numerator.abs(), denominator.abs());
                while b != 0 {
                    let remainder = a % b;
                    a = b;
                    b = remainder;
                }
                let (mut coefficient, mut reduced) = (numerator / a, denominator / a);
                if reduced < 0 {
                    coefficient = -coefficient;
                    reduced = -reduced;
                }
                let (mut twos, mut fives) = (0_u32, 0_u32);
                while reduced % 2 == 0 {
                    reduced /= 2;
                    twos += 1;
                }
                while reduced % 5 == 0 {
                    reduced /= 5;
                    fives += 1;
                }
                let result = native_decimal_divide_exact(
                    &value(&denominator.to_string()),
                    &value(&numerator.to_string()),
                );
                if reduced != 1 {
                    assert!(result
                        .payload()
                        .tagged("std/decimal::NonTerminatingDecimal"));
                    continue;
                }
                let places = twos.max(fives);
                coefficient *= 2_i128.pow(places - twos) * 5_i128.pow(places - fives);
                let expected = native_decimal_make(&coefficient.to_string(), places.to_string());
                let V::Decimal(actual) = result.payload() else {
                    panic!()
                };
                assert_eq!(
                    (&actual.get().coefficient, &actual.get().scale),
                    (&expected.coefficient, &expected.scale),
                    "{numerator}/{denominator}"
                );
            }
        }
    }
    #[test]
    fn rounded_division_and_quantize_keep_all_modes_ties_signs_and_carries() {
        for (mode, positive, negative, third, negative_third) in [
            (NativeRoundingMode::HalfEven, "2", "-2", "0.33", "-0.33"),
            (NativeRoundingMode::HalfUp, "3", "-3", "0.33", "-0.33"),
            (NativeRoundingMode::TowardZero, "2", "-2", "0.33", "-0.33"),
            (NativeRoundingMode::AwayFromZero, "3", "-3", "0.34", "-0.34"),
            (NativeRoundingMode::Floor, "2", "-3", "0.33", "-0.34"),
            (NativeRoundingMode::Ceiling, "3", "-2", "0.34", "-0.33"),
        ] {
            let context = NativeDecimalContext {
                digits: 1,
                rounding: mode,
            };
            assert_eq!(
                native_decimal_divide(&context, &value("2"), &value("5"))
                    .payload()
                    .render(),
                positive
            );
            assert_eq!(
                native_decimal_divide(&context, &value("-2"), &value("5"))
                    .payload()
                    .render(),
                negative
            );
            assert_eq!(
                V::Decimal(Shared::new(native_decimal_quantize(0, mode, &value("2.5")))).render(),
                positive
            );
            assert_eq!(
                V::Decimal(Shared::new(native_decimal_quantize(
                    0,
                    mode,
                    &value("-2.5")
                )))
                .render(),
                negative
            );
            let context = NativeDecimalContext {
                digits: 2,
                rounding: mode,
            };
            assert_eq!(
                native_decimal_divide(&context, &value("3"), &value("1"))
                    .payload()
                    .render(),
                third
            );
            assert_eq!(
                native_decimal_divide(&context, &value("3"), &value("-1"))
                    .payload()
                    .render(),
                negative_third
            );
        }
        let half_even = NativeRoundingMode::HalfEven;
        assert_eq!(
            V::Decimal(Shared::new(native_decimal_quantize(
                2,
                half_even,
                &value("9.999")
            )))
            .render(),
            "10"
        );
        assert_eq!(
            V::Decimal(Shared::new(native_decimal_quantize(
                -2,
                half_even,
                &value("1250")
            )))
            .render(),
            "1200"
        );
        assert_eq!(
            V::Decimal(Shared::new(native_decimal_quantize(
                -2,
                half_even,
                &value("1350")
            )))
            .render(),
            "1400"
        );
    }
    #[test]
    fn scale_cancellation_and_quantize_do_not_expand_arbitrary_exponents() {
        let tiny = NativeDecimal {
            coefficient: "1".into(),
            scale: "9".repeat(1000),
        };
        let context = NativeDecimalContext {
            digits: 34,
            rounding: NativeRoundingMode::HalfEven,
        };
        assert_eq!(
            native_decimal_divide(&context, &tiny, &tiny)
                .payload()
                .render(),
            "1"
        );
        for (coefficient, mode, expected) in [
            ("1", NativeRoundingMode::HalfEven, "0"),
            ("-1", NativeRoundingMode::HalfUp, "0"),
            ("1", NativeRoundingMode::Ceiling, "1"),
            ("-1", NativeRoundingMode::Floor, "-1"),
        ] {
            let input = NativeDecimal {
                coefficient: coefficient.into(),
                scale: tiny.scale.clone(),
            };
            assert_eq!(
                V::Decimal(Shared::new(native_decimal_quantize(0, mode, &input))).render(),
                expected
            );
        }
        let context = NativeDecimalContext {
            digits: 9_007_199_254_740_991,
            rounding: NativeRoundingMode::HalfEven,
        };
        let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            native_decimal_divide(&context, &value("1000000000000000000001"), &value("1"))
        }));
        assert!(failure.is_err());
    }
}
