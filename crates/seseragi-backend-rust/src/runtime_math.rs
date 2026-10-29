fn native_math_operation(name: &str, value: V) -> V {
    let value = native_float(&value);
    V::Float(match name {
        "std/math::sqrt" => value.sqrt(),
        "std/math::sin" => value.sin(),
        "std/math::cos" => value.cos(),
        "std/math::log" => value.ln(),
        _ => panic!("Rust ABI: unsupported math operation"),
    })
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
}
