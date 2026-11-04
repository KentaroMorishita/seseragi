const NATIVE_CHECKED_INT_BOUND: i128 = 9_007_199_254_740_991;

fn native_int_exact(value: i128) -> Option<i64> {
    (-NATIVE_CHECKED_INT_BOUND..=NATIVE_CHECKED_INT_BOUND)
        .contains(&value)
        .then_some(value as i64)
}

fn native_int_saturate(value: i128) -> i64 {
    value.clamp(-NATIVE_CHECKED_INT_BOUND, NATIVE_CHECKED_INT_BOUND) as i64
}

fn native_int_bounded_power(base: i64, exponent: i64) -> Option<i64> {
    assert!(exponent >= 0, "nonnegative checked Int exponent");
    if exponent == 0 {
        return Some(1);
    }
    if base == 0 || base == 1 {
        return Some(base);
    }
    if base == -1 {
        return Some(if exponent % 2 == 0 { 1 } else { -1 });
    }
    let mut result = 1_i64;
    let mut factor = base;
    let mut remaining = exponent;
    while remaining > 0 {
        if remaining & 1 == 1 {
            result = native_int_exact(i128::from(result) * i128::from(factor))?;
        }
        remaining >>= 1;
        if remaining > 0 {
            factor = native_int_exact(i128::from(factor) * i128::from(factor))?;
        }
    }
    Some(result)
}

fn native_int_checked_operation(name: &str, arguments: &[V]) -> Option<V> {
    Some(match (name, arguments) {
        (
            "std/int::checkedAdd"
            | "std/int::checkedSubtract"
            | "std/int::checkedMultiply"
            | "std/int::saturatingAdd"
            | "std/int::saturatingSubtract"
            | "std/int::saturatingMultiply",
            [V::Int(right), V::Int(left)],
        ) => {
            let left = i128::from(*left);
            let right = i128::from(*right);
            let value = match name {
                "std/int::checkedAdd" | "std/int::saturatingAdd" => left + right,
                "std/int::checkedSubtract" | "std/int::saturatingSubtract" => left - right,
                _ => left * right,
            };
            if name.starts_with("std/int::checked") {
                native_maybe(native_int_exact(value).map(V::Int))
            } else {
                V::Int(native_int_saturate(value))
            }
        }
        (
            "std/int::checkedDivide" | "std/int::checkedRemainder",
            [V::Int(divisor), V::Int(dividend)],
        ) => {
            if *divisor == 0 {
                native_left(V::adt("std/int::IntDivisionByZero", V::Unit))
            } else {
                let divisor = checked_int_value(*divisor);
                let dividend = checked_int_value(*dividend);
                native_right(V::Int(if name == "std/int::checkedDivide" {
                    dividend / divisor
                } else {
                    dividend % divisor
                }))
            }
        }
        (
            "std/int::checkedPower" | "std/int::saturatingPower",
            [V::Int(exponent), V::Int(base)],
        ) => {
            let exponent = checked_int_value(*exponent);
            let base = checked_int_value(*base);
            if exponent < 0 {
                native_left(V::adt("std/int::NegativeIntExponent", V::Int(exponent)))
            } else if let Some(value) = native_int_bounded_power(base, exponent) {
                native_right(V::Int(value))
            } else if name == "std/int::checkedPower" {
                native_left(V::adt("std/int::IntPowerOverflow", V::Unit))
            } else {
                native_right(V::Int(if base < 0 && exponent % 2 == 1 {
                    -NATIVE_CHECKED_INT_BOUND as i64
                } else {
                    NATIVE_CHECKED_INT_BOUND as i64
                }))
            }
        }
        ("std/int::minimum" | "std/int::maximum", [V::Int(right), V::Int(left)]) => {
            let right = checked_int_value(*right);
            let left = checked_int_value(*left);
            V::Int(if name == "std/int::minimum" {
                left.min(right)
            } else {
                left.max(right)
            })
        }
        ("std/int::abs" | "std/int::sign", [V::Int(value)]) => {
            let value = checked_int_value(*value);
            V::Int(if name == "std/int::abs" {
                value.abs()
            } else {
                value.signum()
            })
        }
        ("std/int::clamp", [V::Int(lower), V::Int(upper), V::Int(value)]) => {
            let lower = checked_int_value(*lower);
            let upper = checked_int_value(*upper);
            let value = checked_int_value(*value);
            assert!(
                lower <= upper,
                "Seseragi Int clamp lower bound exceeds upper bound"
            );
            V::Int(value.clamp(lower, upper))
        }
        ("std/int::IntDivisionByZero" | "std/int::IntPowerOverflow", []) => V::adt(
            if name == "std/int::IntDivisionByZero" {
                "std/int::IntDivisionByZero"
            } else {
                "std/int::IntPowerOverflow"
            },
            V::Unit,
        ),
        ("std/int::NegativeIntExponent", [V::Int(value)]) => {
            V::adt("std/int::NegativeIntExponent", V::Int(*value))
        }
        _ => return None,
    })
}

#[cfg(test)]
mod int_checked_tests {
    use super::*;

    #[test]
    fn checked_and_saturating_arithmetic_preserve_exact_results_and_operand_order() {
        clear_dictionaries();
        let bound = NATIVE_CHECKED_INT_BOUND as i64;
        for (name, right, left, expected) in [
            ("checkedAdd", 1, bound, None),
            ("checkedAdd", -1, 1, Some(0)),
            ("checkedSubtract", 2, 10, Some(8)),
            ("checkedSubtract", 1, -bound, None),
            ("checkedMultiply", 3, 7, Some(21)),
            ("checkedMultiply", bound, bound, None),
        ] {
            let value = native_int_checked_operation(
                &format!("std/int::{name}"),
                &[V::Int(right), V::Int(left)],
            )
            .unwrap();
            if let Some(expected) = expected {
                assert!(value.tagged("std/prelude::Just"));
                assert!(matches!(value.payload(), V::Int(n) if n == expected));
            } else {
                assert!(value.tagged("std/prelude::Nothing"));
            }
        }
        for (name, right, left, expected) in [
            ("saturatingAdd", 1, bound, bound),
            ("saturatingSubtract", 1, -bound, -bound),
            ("saturatingMultiply", bound, -bound, -bound),
            ("minimum", 3, -4, -4),
            ("maximum", 3, -4, 3),
        ] {
            assert!(
                matches!(native_int_checked_operation(&format!("std/int::{name}"), &[V::Int(right), V::Int(left)]).unwrap(), V::Int(n) if n == expected)
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn powers_are_bounded_without_expanding_the_safe_int_exponent() {
        clear_dictionaries();
        for (base, exponent, expected) in [
            (0, 0, Some(1)),
            (0, 9_007_199_254_740_991, Some(0)),
            (-1, 9_007_199_254_740_991, Some(-1)),
            (2, 52, Some(4_503_599_627_370_496)),
            (2, 53, None),
            (3, 33, Some(5_559_060_566_555_523)),
            (3, 34, None),
        ] {
            assert_eq!(native_int_bounded_power(base, exponent), expected);
        }
        for (name, exponent, base, tag, value) in [
            (
                "checkedPower",
                -2,
                3,
                "std/int::NegativeIntExponent",
                Some(-2),
            ),
            (
                "saturatingPower",
                -2,
                3,
                "std/int::NegativeIntExponent",
                Some(-2),
            ),
            ("checkedPower", 53, 2, "std/int::IntPowerOverflow", None),
            (
                "saturatingPower",
                53,
                -2,
                "std/prelude::Right",
                Some(-9_007_199_254_740_991),
            ),
            (
                "saturatingPower",
                54,
                -2,
                "std/prelude::Right",
                Some(9_007_199_254_740_991),
            ),
        ] {
            let result = native_int_checked_operation(
                &format!("std/int::{name}"),
                &[V::Int(exponent), V::Int(base)],
            )
            .unwrap();
            let selected = if result.tagged("std/prelude::Left") {
                result.payload()
            } else {
                result.clone()
            };
            assert!(selected.tagged(tag));
            if let Some(expected) = value {
                assert!(matches!(selected.payload(), V::Int(n) if n == expected));
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn division_clamp_abs_and_sign_keep_typed_failure_and_unwind() {
        clear_dictionaries();
        for (name, divisor, dividend, expected) in [
            ("checkedDivide", -3, 10, -3),
            ("checkedRemainder", 3, -10, -1),
        ] {
            let result = native_int_checked_operation(
                &format!("std/int::{name}"),
                &[V::Int(divisor), V::Int(dividend)],
            )
            .unwrap();
            assert!(result.tagged("std/prelude::Right"));
            assert!(matches!(result.payload(), V::Int(n) if n == expected));
            let zero =
                native_int_checked_operation(&format!("std/int::{name}"), &[V::Int(0), V::Int(42)])
                    .unwrap();
            assert!(
                zero.tagged("std/prelude::Left")
                    && zero.payload().tagged("std/int::IntDivisionByZero")
            );
        }
        for (name, input, expected) in [
            ("abs", -9_007_199_254_740_991, 9_007_199_254_740_991),
            ("sign", -42, -1),
            ("sign", 0, 0),
            ("sign", 42, 1),
        ] {
            assert!(
                matches!(native_int_checked_operation(&format!("std/int::{name}"), &[V::Int(input)]).unwrap(), V::Int(n) if n == expected)
            );
        }
        assert!(matches!(
            native_int_checked_operation("std/int::clamp", &[V::Int(-2), V::Int(3), V::Int(10)])
                .unwrap(),
            V::Int(3)
        ));
        assert!(std::panic::catch_unwind(|| native_int_checked_operation(
            "std/int::clamp",
            &[V::Int(3), V::Int(1), V::Int(0)]
        ))
        .is_err());
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
