fn native_float(value: &V) -> f64 {
    let V::Float(value) = value else {
        panic!("Rust ABI: expected Float")
    };
    *value
}

fn native_float_power(base: f64, exponent: f64) -> f64 {
    // https://tc39.es/ecma262/2026/#sec-numeric-types-number-exponentiate
    // The NaN/one rules differ from IEEE powf and are part of std/float's ABI.
    if exponent.is_nan() {
        return f64::NAN;
    }
    if exponent == 0.0 {
        return 1.0;
    }
    if base.is_nan() {
        return f64::NAN;
    }
    let odd = exponent.is_finite() && exponent.abs() % 2.0 == 1.0;
    let negative = base.is_sign_negative() && odd;
    if base.is_infinite() {
        return if exponent > 0.0 {
            if negative { f64::NEG_INFINITY } else { f64::INFINITY }
        } else if negative { -0.0 } else { 0.0 };
    }
    if base == 0.0 {
        return if exponent > 0.0 {
            if negative { -0.0 } else { 0.0 }
        } else if negative { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    if exponent.is_infinite() {
        if base.abs() == 1.0 {
            return f64::NAN;
        }
        return if (base.abs() > 1.0) == (exponent > 0.0) { f64::INFINITY } else { 0.0 };
    }
    if base < 0.0 && exponent.fract() != 0.0 {
        return f64::NAN;
    }
    base.powf(exponent)
}

fn native_float_binary(operator: &str, left: V, right: V) -> V {
    let (left, right) = (native_float(&left), native_float(&right));
    V::Float(match operator {
        "+" => left + right,
        "-" => left - right,
        "*" => left * right,
        "/" => left / right,
        "%" => left % right,
        "**" => native_float_power(left, right),
        _ => panic!("Rust ABI: unsupported Float operator"),
    })
}

fn native_float_dictionary(identity: &'static str) -> V {
    cached(identity, || {
        macro_rules! arithmetic {
            ($method:literal, $operator:literal) => {
                V::dict(vec![(
                    $method,
                    V::fun(vec![], |_, left| {
                        V::fun(vec![left], |c, right| {
                            native_float_binary($operator, c[0].clone(), right)
                        })
                    }),
                )])
            };
        }
        match identity {
            "std/float::Add" => arithmetic!("add", "+"),
            "std/float::Sub" => arithmetic!("sub", "-"),
            "std/float::Mul" => arithmetic!("mul", "*"),
            "std/float::Div" => arithmetic!("div", "/"),
            "std/float::Rem" => arithmetic!("rem", "%"),
            "std/float::Pow" => arithmetic!("pow", "**"),
            "Show<std/prelude::Float>" => V::dict(vec![(
                "show",
                V::fun(vec![], |_, v| {
                    native_display(|| V::text(&native_float_format(native_float(&v))))
                }),
            )]),
            "Debug<std/prelude::Float>" => V::dict(vec![(
                "debug",
                V::fun(vec![], |_, v| {
                    native_display(|| V::text(&native_float_format(native_float(&v))))
                }),
            )]),
            "std/float::Zero" => V::dict(vec![("zero", V::fun(vec![], |_, _| V::Float(0.0)))]),
            "std/float::One" => V::dict(vec![("one", V::fun(vec![], |_, _| V::Float(1.0)))]),
            _ => panic!("Rust ABI: unknown Float dictionary"),
        }
    })
}

fn native_float_compare(left: f64, right: f64) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if left.is_nan() {
        return if right.is_nan() { Equal } else { Greater };
    }
    if right.is_nan() {
        return Less;
    }
    if left == right {
        return if left == 0.0 {
            right.is_sign_negative().cmp(&left.is_sign_negative())
        } else {
            Equal
        };
    }
    left.partial_cmp(&right).expect("non-NaN Float comparison")
}

fn native_float_extreme(left: f64, right: f64, maximum: bool) -> f64 {
    if left.is_nan() {
        return right;
    }
    if right.is_nan() {
        return left;
    }
    let compare = native_float_compare(left, right);
    if (maximum && compare.is_gt()) || (!maximum && compare.is_lt()) {
        left
    } else {
        right
    }
}

fn native_float_operation(name: &str, args: Vec<V>) -> V {
    match (name, args.as_slice()) {
        ("std/float::nan", [V::Unit]) => V::Float(f64::NAN),
        ("std/float::positiveInfinity", [V::Unit]) => V::Float(f64::INFINITY),
        ("std/float::negativeInfinity", [V::Unit]) => V::Float(f64::NEG_INFINITY),
        ("std/float::fromInt", [V::Int(value)]) => V::Float(checked_int_value(*value) as f64),
        ("std/float::format", [value]) => V::text(&native_float_format(native_float(value))),
        ("std/float::isNaN", [value]) => V::Bool(native_float(value).is_nan()),
        ("std/float::isFinite", [value]) => V::Bool(native_float(value).is_finite()),
        ("std/float::isInfinite", [value]) => V::Bool(native_float(value).is_infinite()),
        ("std/float::isNegativeZero", [value]) => {
            let value = native_float(value);
            V::Bool(value == 0.0 && value.is_sign_negative())
        }
        ("std/float::ieeeEq", [left, right]) => V::Bool(native_float(left) == native_float(right)),
        ("std/float::totalCompare", [left, right]) => {
            let tag = match native_float_compare(native_float(left), native_float(right)) {
                std::cmp::Ordering::Less => "std/prelude::Less",
                std::cmp::Ordering::Equal => "std/prelude::Equal",
                std::cmp::Ordering::Greater => "std/prelude::Greater",
            };
            V::adt(tag, V::Unit)
        }
        ("std/float::minimumNumber" | "std/float::maximumNumber", [right, left]) => {
            V::Float(native_float_extreme(
                native_float(left),
                native_float(right),
                name == "std/float::maximumNumber",
            ))
        }
        ("std/float::clampNumber", [lower, upper, value]) => {
            let (lower, upper, value) = (
                native_float(lower),
                native_float(upper),
                native_float(value),
            );
            if lower.is_nan()
                || upper.is_nan()
                || value.is_nan()
                || native_float_compare(lower, upper).is_gt()
            {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                let result = if native_float_compare(value, lower).is_lt() {
                    lower
                } else if native_float_compare(value, upper).is_gt() {
                    upper
                } else {
                    value
                };
                V::adt("std/prelude::Just", V::Float(result))
            }
        }
        ("std/float::abs", [value]) => V::Float(native_float(value).abs()),
        ("std/float::power", [exponent, base]) => V::Float(native_float_power(native_float(base), native_float(exponent))),
        ("std/float::sign", [value]) => {
            let value = native_float(value);
            if value.is_nan() {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::adt(
                    "std/prelude::Just",
                    V::Int(if value < 0.0 {
                        -1
                    } else if value > 0.0 {
                        1
                    } else {
                        0
                    }),
                )
            }
        }
        _ => panic!("Rust ABI: unsupported Float operation or arity {name}"),
    }
}

#[cfg(test)]
mod float_tests {
    use super::*;

    #[test]
    fn float_power_preserves_the_ecmascript_special_value_table() {
        let nan = f64::NAN;
        let inf = f64::INFINITY;
        for (base, exponent, expected) in [
            (1.0, nan, nan), (-1.0, nan, nan), (0.0, nan, nan),
            (nan, 0.0, 1.0), (nan, -0.0, 1.0), (nan, 2.0, nan),
            (inf, 3.0, inf), (inf, -3.0, 0.0),
            (-inf, 3.0, -inf), (-inf, 2.0, inf),
            (-inf, -3.0, -0.0), (-inf, -2.0, 0.0),
            (0.0, 2.0, 0.0), (0.0, -2.0, inf),
            (-0.0, 3.0, -0.0), (-0.0, 2.0, 0.0),
            (-0.0, -3.0, -inf), (-0.0, -2.0, inf),
            (1.0, inf, nan), (-1.0, inf, nan),
            (1.0, -inf, nan), (-1.0, -inf, nan),
            (2.0, inf, inf), (2.0, -inf, 0.0),
            (0.5, inf, 0.0), (0.5, -inf, inf),
            (-2.0, 0.5, nan), (-2.0, 3.0, -8.0),
            (-2.0, -3.0, -0.125), (4.0, 0.5, 2.0), (4.0, -0.5, 0.5),
            (-0.0, 9_007_199_254_740_991.0, -0.0),
            (-0.0, 9_007_199_254_740_992.0, 0.0),
            (-inf, -9_007_199_254_740_991.0, -0.0),
            (2.0, 1024.0, inf), (-2.0, 1025.0, -inf),
            (2.0, -1074.0, f64::from_bits(1)), (-2.0, -1075.0, -0.0),
        ] {
            let actual = native_float_power(base, exponent);
            if expected.is_nan() {
                assert!(actual.is_nan(), "{base:?} ** {exponent:?}: {actual:?}");
            } else {
                assert_eq!(actual.to_bits(), expected.to_bits(), "{base:?} ** {exponent:?}");
            }
        }
    }

    #[test]
    fn float_power_preserves_method_and_exponent_first_function_stages() {
        clear_dictionaries();
        let method = native_float_dictionary("std/float::Pow").field("pow");
        let saved = method.apply(V::Float(2.0));
        assert_eq!(native_float(&saved.apply(V::Float(3.0))), 8.0);
        assert_eq!(native_float(&saved.apply(V::Float(4.0))), 16.0);
        assert_eq!(native_float(&native_float_operation(
            "std/float::power", vec![V::Float(3.0), V::Float(2.0)])), 8.0);
        drop((saved, method));
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn ieee_arithmetic_retains_negative_zero_and_non_finite_values() {
        assert!(native_float(&native_float_binary("/", V::Float(0.0), V::Float(0.0))).is_nan());
        assert_eq!(
            native_float(&native_float_binary("/", V::Float(1.0), V::Float(-0.0))),
            f64::NEG_INFINITY
        );
        assert_eq!(
            native_float(&native_float_binary("%", V::Float(-4.0), V::Float(2.0))).to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(
            native_float(&negate(V::Float(0.0))).to_bits(),
            (-0.0_f64).to_bits()
        );
    }

    #[test]
    fn total_order_and_extrema_preserve_zero_order_and_nan_policy() {
        use std::cmp::Ordering::*;
        let values = [
            f64::NEG_INFINITY,
            -1.0,
            -0.0,
            0.0,
            1.0,
            f64::INFINITY,
            f64::NAN,
        ];
        for (i, left) in values.iter().enumerate() {
            for (j, right) in values.iter().enumerate() {
                assert_eq!(native_float_compare(*left, *right), i.cmp(&j));
            }
        }
        assert_eq!(native_float_compare(f64::NAN, -f64::NAN), Equal);
        for (left, right) in [(0.0, -0.0), (-0.0, 0.0)] {
            assert_eq!(
                native_float_extreme(left, right, false).to_bits(),
                (-0.0_f64).to_bits()
            );
            assert_eq!(
                native_float_extreme(left, right, true).to_bits(),
                0.0_f64.to_bits()
            );
        }
        assert_eq!(native_float_extreme(f64::NAN, 42.0, false), 42.0);
        assert!(native_float_extreme(f64::NAN, f64::NAN, true).is_nan());
    }
}
