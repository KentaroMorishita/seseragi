fn native_decimal_copy(value: &NativeDecimal) -> NativeDecimal {
    NativeDecimal {
        coefficient: value.coefficient.clone(),
        scale: value.scale.clone(),
    }
}

fn native_decimal_host_count(count: Option<usize>, message: &'static str) -> usize {
    count.unwrap_or_else(|| native_defect_at(message, None))
}

fn native_decimal_product_buffer(left_digits: usize, right_digits: usize) -> Vec<u8> {
    const MESSAGE: &str = "Decimal coefficient exceeds host limits";
    let length = native_decimal_host_count(left_digits.checked_add(right_digits), MESSAGE);
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .unwrap_or_else(|_| native_defect_at(MESSAGE, None));
    output.resize(length, 0);
    output
}

fn native_decimal_coefficient_multiply(left: &str, right: &str) -> String {
    if left == "0" || right == "0" {
        return "0".into();
    }
    let negative = left.starts_with('-') != right.starts_with('-');
    let left = left.trim_start_matches('-').as_bytes();
    let right = right.trim_start_matches('-').as_bytes();
    let mut output = native_decimal_product_buffer(left.len(), right.len());
    // Decimal words avoid a quadratic radix conversion when one operand is
    // much shorter. This is O(n*m) digit operations and O(n+m) storage.
    for i in (0..left.len()).rev() {
        let mut carry = 0_u16;
        for j in (0..right.len()).rev() {
            let value = u16::from(left[i] - b'0') * u16::from(right[j] - b'0')
                + u16::from(output[i + j + 1])
                + carry;
            output[i + j + 1] = (value % 10) as u8;
            carry = value / 10;
        }
        output[i] = carry as u8;
    }
    for digit in &mut output {
        *digit += b'0';
    }
    native_decimal_signed(
        negative,
        std::str::from_utf8(&output).expect("ASCII decimal product"),
    )
}

fn native_decimal_add_values(
    left: &NativeDecimal,
    right: &NativeDecimal,
    subtract: bool,
) -> NativeDecimal {
    if right.coefficient == "0" {
        return native_decimal_copy(left);
    }
    if left.coefficient == "0" {
        return native_decimal_make(
            &if subtract {
                native_decimal_negate(&right.coefficient)
            } else {
                right.coefficient.clone()
            },
            right.scale.clone(),
        );
    }
    let scale = if native_decimal_signed_compare(&left.scale, &right.scale).is_gt() {
        &left.scale
    } else {
        &right.scale
    };
    let mut left_coefficient = left.coefficient.clone();
    let mut right_coefficient = if subtract {
        native_decimal_negate(&right.coefficient)
    } else {
        right.coefficient.clone()
    };
    native_decimal_zeros(
        &mut left_coefficient,
        &native_decimal_add(scale, &native_decimal_negate(&left.scale)),
    );
    native_decimal_zeros(
        &mut right_coefficient,
        &native_decimal_add(scale, &native_decimal_negate(&right.scale)),
    );
    native_decimal_make(
        &native_decimal_add(&left_coefficient, &right_coefficient),
        scale.clone(),
    )
}

fn native_decimal_arithmetic(operator: &str, left: V, right: V) -> V {
    let (V::Decimal(left), V::Decimal(right)) = (left, right) else {
        panic!("Rust ABI: expected Decimal arithmetic operands")
    };
    let (left, right) = (left.get(), right.get());
    let result = match operator {
        "+" => native_decimal_add_values(left, right, false),
        "-" => native_decimal_add_values(left, right, true),
        "*" => native_decimal_make(
            &native_decimal_coefficient_multiply(&left.coefficient, &right.coefficient),
            native_decimal_add(&left.scale, &right.scale),
        ),
        _ => panic!("Rust ABI: unsupported Decimal arithmetic operator"),
    };
    V::Decimal(Shared::new(result))
}

fn native_decimal_arithmetic_dictionary(identity: &'static str) -> V {
    let (method, operator) = match identity {
        "std/decimal::Add" => ("add", "+"),
        "std/decimal::Sub" => ("sub", "-"),
        "std/decimal::Mul" => ("mul", "*"),
        _ => panic!("Rust ABI: unsupported Decimal arithmetic dictionary"),
    };
    V::dict(vec![(
        method,
        V::fun(vec![V::text(operator)], |captures, left| {
            V::fun(vec![captures[0].clone(), left], |captures, right| {
                native_decimal_arithmetic(native_text(&captures[0]), captures[1].clone(), right)
            })
        }),
    )])
}

fn native_decimal_constant_dictionary(one: bool) -> V {
    V::dict(vec![(
        if one { "one" } else { "zero" },
        V::fun(vec![V::Bool(one)], |captures, unit| {
            assert!(
                matches!(unit, V::Unit),
                "Rust ABI: expected Decimal constant Unit"
            );
            V::Decimal(Shared::new(native_decimal_make(
                if captures[0].boolean() { "1" } else { "0" },
                "0".into(),
            )))
        }),
    )])
}

fn native_decimal_display_dictionary(debug: bool) -> V {
    V::dict(vec![(
        if debug { "debug" } else { "show" },
        V::fun(vec![], |_, value| native_decimal_display(value)),
    )])
}

#[cfg(test)]
mod decimal_arithmetic_tests {
    use super::*;
    #[test]
    fn product_storage_limits_are_defects_before_allocation_or_digit_work() {
        for (left, right) in [
            (usize::MAX, 1),
            (usize::MAX - 1, 1),
            (isize::MAX as usize, 1),
        ] {
            let result = catch_native(|| {
                native_decimal_product_buffer(left, right);
                Ok(V::Unit)
            });
            assert!(
                matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["Decimal coefficient exceeds host limits"]),
                "{left} + {right}"
            );
        }
        let buffer = native_decimal_product_buffer(4095, 1);
        assert_eq!(buffer.len(), 4096);
        assert!(buffer.iter().all(|digit| *digit == 0));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn signed_scaled_arithmetic_matches_independent_i128_coefficients() {
        for left in [
            -9_007_199_254_740_991_i64,
            -99,
            -1,
            0,
            1,
            99,
            9_007_199_254_740_991,
        ] {
            for right in [-4_294_967_296_i64, -1, 0, 1, 4_294_967_296] {
                for a in [-3_i32, 0, 3] {
                    for b in [-3_i32, 0, 3] {
                        let first = native_decimal_make(&left.to_string(), a.to_string());
                        let second = native_decimal_make(&right.to_string(), b.to_string());
                        let scale = a.max(b);
                        let x = i128::from(left) * 10_i128.pow((scale - a) as u32);
                        let y = i128::from(right) * 10_i128.pow((scale - b) as u32);
                        for subtract in [false, true] {
                            let expected = native_decimal_make(
                                &(if subtract { x - y } else { x + y }).to_string(),
                                scale.to_string(),
                            );
                            let actual = native_decimal_add_values(&first, &second, subtract);
                            assert_eq!(
                                (actual.coefficient, actual.scale),
                                (expected.coefficient, expected.scale)
                            );
                        }
                        let expected = native_decimal_make(
                            &(i128::from(left) * i128::from(right)).to_string(),
                            (a + b).to_string(),
                        );
                        let actual = native_decimal_make(
                            &native_decimal_coefficient_multiply(
                                &first.coefficient,
                                &second.coefficient,
                            ),
                            native_decimal_add(&first.scale, &second.scale),
                        );
                        assert_eq!(
                            (actual.coefficient, actual.scale),
                            (expected.coefficient, expected.scale)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn arithmetic_keeps_arbitrary_scale_and_normalizes_exact_cancellation() {
        let exponent = "9".repeat(1000);
        let first = NativeDecimal {
            coefficient: "1".into(),
            scale: format!("-{exponent}"),
        };
        let second = NativeDecimal {
            coefficient: "9".into(),
            scale: format!("-{exponent}"),
        };
        let actual = native_decimal_add_values(&first, &second, false);
        assert_eq!(actual.coefficient, "1");
        assert_eq!(actual.scale, format!("-1{}", "0".repeat(1000)));
        let zero = native_decimal_add_values(&first, &first, true);
        assert_eq!((zero.coefficient, zero.scale), ("0".into(), "0".into()));
        let copied =
            native_decimal_add_values(&first, &native_decimal_make("0", "0".into()), false);
        assert_eq!(
            (copied.coefficient, copied.scale),
            (first.coefficient, first.scale)
        );
        assert_eq!(
            native_decimal_coefficient_multiply(&"9".repeat(1000), "9"),
            format!("8{}1", "9".repeat(999))
        );
    }
    #[test]
    fn saved_methods_constants_and_debug_keep_managed_values_and_collect_cycles() {
        enable_cycle_collection();
        let value = native_decimal_parse("1.25").payload();
        let cell = V::reference(value.clone());
        let plus = native_decimal_arithmetic_dictionary("std/decimal::Add")
            .field("add")
            .apply(value.clone());
        assert_eq!(plus.apply(value.clone()).render(), "2.5");
        assert_eq!(
            native_decimal_display_dictionary(true)
                .field("debug")
                .apply(value.clone())
                .render(),
            "1.25"
        );
        assert_eq!(
            native_decimal_constant_dictionary(false)
                .field("zero")
                .apply(V::Unit)
                .render(),
            "0"
        );
        assert_eq!(
            native_decimal_constant_dictionary(true)
                .field("one")
                .apply(V::Unit)
                .render(),
            "1"
        );
        let capture = V::fun(vec![cell.clone(), plus.clone()], |captures, _| {
            captures[1].apply(captures[0].read_reference())
        });
        cell.write_reference(V::Tuple(Shared::new(vec![cell.clone(), capture.clone()])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((value, cell, plus, capture));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
