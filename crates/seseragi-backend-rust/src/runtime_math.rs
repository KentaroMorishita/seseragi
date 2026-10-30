fn native_math_operation(name: &str, value: V) -> V {
    let value = native_float(&value);
    V::Float(match name {
        "std/math::sqrt" => value.sqrt(),
        "std/math::sin" => value.sin(),
        "std/math::cos" => value.cos(),
        "std/math::log" => value.ln(),
        "std/math::tan" => value.tan(),
        "std/math::asin" => value.asin(),
        "std/math::acos" => value.acos(),
        "std/math::atan" => value.atan(),
        "std/math::exp" => value.exp(),
        "std/math::log2" => value.log2(),
        "std/math::log10" => value.log10(),
        "std/math::cbrt" => value.cbrt(),
        "std/math::sinh" => value.sinh(),
        "std/math::cosh" => value.cosh(),
        "std/math::tanh" => value.tanh(),
        _ => panic!("Rust ABI: unsupported math operation"),
    })
}

fn native_math_standard_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/math::pi", []) => V::Float(std::f64::consts::PI),
        ("std/math::e", []) => V::Float(std::f64::consts::E),
        ("std/math::tau", []) => V::Float(std::f64::consts::TAU),
        ("std/math::atan2", [y, x]) => V::Float(native_float(y).atan2(native_float(x))),
        ("std/math::hypot", [x, y]) => V::Float(native_float(x).hypot(native_float(y))),
        (_, [value]) => native_math_operation(name, value.clone()),
        _ => panic!("Rust ABI: unsupported math operation or arity"),
    }
}

#[cfg(test)]
mod math_tests {
    use super::*;

    fn evaluate(name: &str, input: f64) -> f64 {
        native_float(&native_math_operation(name, V::Float(input)))
    }

    #[test]
    fn native_basic_math_preserves_domain_special_values_and_signed_zero() {
        for name in [
            "std/math::sqrt",
            "std/math::sin",
            "std/math::cos",
            "std/math::log",
        ] {
            assert!(evaluate(name, f64::NAN).is_nan());
        }
        for name in ["std/math::sqrt", "std/math::sin"] {
            for zero in [0.0_f64, -0.0] {
                assert_eq!(evaluate(name, zero).to_bits(), zero.to_bits());
            }
        }
        assert_eq!(evaluate("std/math::cos", -0.0), 1.0);
        assert_eq!(evaluate("std/math::log", 1.0).to_bits(), 0.0_f64.to_bits());
        for zero in [0.0, -0.0] {
            assert_eq!(evaluate("std/math::log", zero), f64::NEG_INFINITY);
        }
        for name in ["std/math::sqrt", "std/math::log"] {
            assert!(evaluate(name, -1.0).is_nan());
            assert!(evaluate(name, f64::NEG_INFINITY).is_nan());
            assert_eq!(evaluate(name, f64::INFINITY), f64::INFINITY);
        }
        for name in ["std/math::sin", "std/math::cos"] {
            for infinity in [f64::INFINITY, f64::NEG_INFINITY] {
                assert!(evaluate(name, infinity).is_nan());
            }
        }
        assert!(evaluate("std/math::sqrt", f64::from_bits(1)) > 0.0);
    }

    #[test]
    fn native_basic_math_meets_portable_representative_tolerance() {
        for (name, input, expected) in [
            ("std/math::sqrt", 2.0, std::f64::consts::SQRT_2),
            ("std/math::sin", 0.5, 0.479425538604203),
            ("std/math::cos", 0.5, 0.8775825618903728),
            ("std/math::log", 2.0, std::f64::consts::LN_2),
        ] {
            assert!((evaluate(name, input) - expected).abs() <= 1e-14 * expected.abs().max(1.0));
        }
    }

    #[test]
    fn remaining_math_keeps_domain_infinities_and_negative_zero() {
        for name in [
            "tan", "asin", "acos", "atan", "exp", "log2", "log10", "cbrt", "sinh", "cosh", "tanh",
        ] {
            assert!(evaluate(&format!("std/math::{name}"), f64::NAN).is_nan());
        }
        for name in ["tan", "asin", "atan", "cbrt", "sinh", "tanh"] {
            for zero in [0.0_f64, -0.0] {
                assert_eq!(
                    evaluate(&format!("std/math::{name}"), zero).to_bits(),
                    zero.to_bits()
                );
            }
        }
        for name in ["log2", "log10"] {
            for zero in [0.0, -0.0] {
                assert_eq!(
                    evaluate(&format!("std/math::{name}"), zero),
                    f64::NEG_INFINITY
                );
            }
            assert!(evaluate(&format!("std/math::{name}"), -1.0).is_nan());
            assert_eq!(
                evaluate(&format!("std/math::{name}"), f64::INFINITY),
                f64::INFINITY
            );
        }
        for name in ["asin", "acos"] {
            assert!(evaluate(&format!("std/math::{name}"), 2.0).is_nan());
        }
        assert!(evaluate("std/math::tan", f64::INFINITY).is_nan());
        assert_eq!(
            evaluate("std/math::exp", f64::NEG_INFINITY).to_bits(),
            0.0_f64.to_bits()
        );
        assert_eq!(
            evaluate("std/math::sinh", f64::NEG_INFINITY),
            f64::NEG_INFINITY
        );
        assert_eq!(evaluate("std/math::cosh", f64::NEG_INFINITY), f64::INFINITY);
        assert_eq!(evaluate("std/math::tanh", f64::NEG_INFINITY), -1.0);
    }

    #[test]
    fn binary_math_preserves_operand_order_zero_quadrants_and_scaled_hypot() {
        let binary = |name, left, right| {
            native_float(&native_math_standard_operation(
                name,
                vec![V::Float(left), V::Float(right)],
            ))
        };
        assert_eq!(
            binary("std/math::atan2", -0.0, 0.0).to_bits(),
            (-0.0_f64).to_bits()
        );
        assert_eq!(binary("std/math::atan2", -0.0, -0.0), -std::f64::consts::PI);
        assert_eq!(binary("std/math::atan2", 0.0, -0.0), std::f64::consts::PI);
        assert_eq!(
            binary("std/math::atan2", 1.0, 0.0),
            std::f64::consts::FRAC_PI_2
        );
        assert_eq!(
            binary("std/math::hypot", f64::INFINITY, f64::NAN),
            f64::INFINITY
        );
        assert_eq!(
            binary("std/math::hypot", f64::NAN, f64::NEG_INFINITY),
            f64::INFINITY
        );
        assert_eq!(
            binary("std/math::hypot", -0.0, -0.0).to_bits(),
            0.0_f64.to_bits()
        );
        for value in [1e308, 1e-308, f64::from_bits(1)] {
            let actual = binary("std/math::hypot", value, value);
            assert!(actual.is_finite() && actual > 0.0);
            assert!(
                (actual / value - std::f64::consts::SQRT_2).abs()
                    <= if value == f64::from_bits(1) {
                        0.5
                    } else {
                        1e-14
                    }
            );
        }
        assert!(binary("std/math::atan2", f64::NAN, 1.0).is_nan());
    }

    #[test]
    fn remaining_math_constants_and_finite_results_meet_portable_tolerance() {
        for (name, expected) in [
            ("pi", std::f64::consts::PI),
            ("e", std::f64::consts::E),
            ("tau", std::f64::consts::TAU),
        ] {
            assert_eq!(
                native_float(&native_math_standard_operation(
                    &format!("std/math::{name}"),
                    vec![]
                ))
                .to_bits(),
                expected.to_bits()
            );
        }
        for (name, input, expected) in [
            ("tan", 0.5, 0.5463024898437905),
            ("asin", 0.5, std::f64::consts::FRAC_PI_6),
            ("acos", 0.5, std::f64::consts::FRAC_PI_3),
            ("atan", 1.0, std::f64::consts::FRAC_PI_4),
            ("exp", 1.0, std::f64::consts::E),
            ("log2", 8.0, 3.0),
            ("log10", 100.0, 2.0),
            ("cbrt", -27.0, -3.0),
            ("sinh", 0.5, 0.5210953054937474),
            ("cosh", 0.5, 1.1276259652063807),
            ("tanh", 0.5, 0.46211715726000974),
        ] {
            assert!(
                (evaluate(&format!("std/math::{name}"), input) - expected).abs()
                    <= 1e-14 * expected.abs().max(1.0)
            );
        }
    }
}
