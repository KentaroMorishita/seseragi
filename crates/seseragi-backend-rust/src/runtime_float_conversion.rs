#[derive(Clone, Copy)]
enum NativeRoundingMode {
    HalfEven,
    HalfUp,
    TowardZero,
    AwayFromZero,
    Floor,
    Ceiling,
}

fn native_rounding_mode(value: &V) -> NativeRoundingMode {
    use NativeRoundingMode::*;
    for (name, mode) in [
        ("std/number::HalfEven", HalfEven),
        ("std/number::HalfUp", HalfUp),
        ("std/number::TowardZero", TowardZero),
        ("std/number::AwayFromZero", AwayFromZero),
        ("std/number::Floor", Floor),
        ("std/number::Ceiling", Ceiling),
    ] {
        if value.tagged(name) {
            return mode;
        }
    }
    panic!("Rust ABI: expected canonical RoundingMode")
}

fn native_round_integral(mode: NativeRoundingMode, value: f64) -> f64 {
    // Preserve both zero signs and the original non-finite value.
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    use NativeRoundingMode::*;
    match mode {
        HalfEven => {
            let floor = value.floor();
            let fraction = value - floor;
            let rounded = if fraction < 0.5 {
                floor
            } else if fraction > 0.5 {
                floor + 1.0
            } else if floor % 2.0 == 0.0 {
                floor
            } else {
                floor + 1.0
            };
            if rounded == 0.0 && value < 0.0 {
                -0.0
            } else {
                rounded
            }
        }
        HalfUp => {
            let magnitude = value.abs();
            let integral = magnitude.floor();
            // Adding 0.5 first loses below-midpoint and large odd integers.
            let rounded = if magnitude - integral < 0.5 {
                integral
            } else {
                integral + 1.0
            };
            rounded.copysign(value)
        }
        TowardZero => value.trunc(),
        AwayFromZero => {
            if value < 0.0 {
                value.floor()
            } else {
                value.ceil()
            }
        }
        Floor => value.floor(),
        Ceiling => value.ceil(),
    }
}

fn native_float_to_int(mode: &V, value: f64) -> V {
    if !value.is_finite() {
        return native_left(V::adt("std/float::FloatNotFinite", V::Unit));
    }
    let rounded = native_round_integral(native_rounding_mode(mode), value);
    // Check the rounded Float before conversion: Rust's saturating cast is
    // never used to decide whether a value belongs to the Seseragi Int range.
    if !(-9_007_199_254_740_991.0..=9_007_199_254_740_991.0).contains(&rounded) {
        return native_left(V::adt("std/float::FloatOutsideIntRange", V::Unit));
    }
    native_right(V::Int(checked_int_value(rounded as i64)))
}

fn native_float_conversion_operation(name: &str, arguments: &[V]) -> Option<V> {
    Some(match (name, arguments) {
        ("std/float::toInt", [mode, value]) => native_float_to_int(mode, native_float(value)),
        ("std/float::roundIntegral", [mode, value]) => V::Float(native_round_integral(
            native_rounding_mode(mode),
            native_float(value),
        )),
        (
            "std/float::FloatNotFinite"
            | "std/float::FloatOutsideIntRange"
            | "std/number::HalfEven"
            | "std/number::HalfUp"
            | "std/number::TowardZero"
            | "std/number::AwayFromZero"
            | "std/number::Floor"
            | "std/number::Ceiling",
            [],
        ) => V::adt(name, V::Unit),
        _ => return None,
    })
}

#[cfg(test)]
mod float_conversion_tests {
    use super::*;
    const MODES: [NativeRoundingMode; 6] = [
        NativeRoundingMode::HalfEven,
        NativeRoundingMode::HalfUp,
        NativeRoundingMode::TowardZero,
        NativeRoundingMode::AwayFromZero,
        NativeRoundingMode::Floor,
        NativeRoundingMode::Ceiling,
    ];
    const NAMES: [&str; 6] = [
        "HalfEven",
        "HalfUp",
        "TowardZero",
        "AwayFromZero",
        "Floor",
        "Ceiling",
    ];

    #[test]
    fn six_modes_keep_adjacent_midpoints_subnormals_and_zero_signs() {
        let cases: &[(f64, [f64; 6])] = &[
            (0.49999999999999994, [0.0, 0.0, 0.0, 1.0, 0.0, 1.0]),
            (-0.49999999999999994, [-0.0, -0.0, -0.0, -1.0, -1.0, -0.0]),
            (0.5, [0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
            (-0.5, [-0.0, -1.0, -0.0, -1.0, -1.0, -0.0]),
            (0.5000000000000001, [1.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
            (-0.5000000000000001, [-1.0, -1.0, -0.0, -1.0, -1.0, -0.0]),
            (1.4999999999999998, [1.0, 1.0, 1.0, 2.0, 1.0, 2.0]),
            (-1.4999999999999998, [-1.0, -1.0, -1.0, -2.0, -2.0, -1.0]),
            (1.5, [2.0, 2.0, 1.0, 2.0, 1.0, 2.0]),
            (-1.5, [-2.0, -2.0, -1.0, -2.0, -2.0, -1.0]),
            (1.5000000000000002, [2.0, 2.0, 1.0, 2.0, 1.0, 2.0]),
            (-1.5000000000000002, [-2.0, -2.0, -1.0, -2.0, -2.0, -1.0]),
            (2.5, [2.0, 3.0, 2.0, 3.0, 2.0, 3.0]),
            (-2.5, [-2.0, -3.0, -2.0, -3.0, -3.0, -2.0]),
            (3.5, [4.0, 4.0, 3.0, 4.0, 3.0, 4.0]),
            (-3.5, [-4.0, -4.0, -3.0, -4.0, -4.0, -3.0]),
            (f64::from_bits(1), [0.0, 0.0, 0.0, 1.0, 0.0, 1.0]),
            (-f64::from_bits(1), [-0.0, -0.0, -0.0, -1.0, -1.0, -0.0]),
        ];
        for &(value, expected) in cases {
            for (index, mode) in MODES.iter().enumerate() {
                assert_eq!(
                    native_round_integral(*mode, value).to_bits(),
                    expected[index].to_bits(),
                    "{value} {}",
                    NAMES[index]
                );
            }
        }
        for mode in MODES {
            for value in [
                0.0,
                -0.0,
                f64::from_bits(0x7ff8_0000_0000_002a),
                f64::INFINITY,
                f64::NEG_INFINITY,
            ] {
                assert_eq!(
                    native_round_integral(mode, value).to_bits(),
                    value.to_bits()
                );
            }
            for magnitude in [
                1.0,
                2.0,
                2_f64.powi(51) + 1.0,
                2_f64.powi(52) - 1.0,
                2_f64.powi(52),
                2_f64.powi(52) + 1.0,
                9_007_199_254_740_991.0,
                2_f64.powi(53),
                f64::MAX,
            ] {
                for value in [magnitude, -magnitude] {
                    assert_eq!(
                        native_round_integral(mode, value).to_bits(),
                        value.to_bits()
                    );
                }
            }
        }
    }

    #[test]
    fn half_up_matches_integer_binary64_oracle_across_finite_exponents() {
        for exponent in 0_u64..2047 {
            for fraction in [
                0,
                1,
                (1 << 51) - 1,
                1 << 51,
                (1 << 51) + 1,
                (1 << 52) - 2,
                (1 << 52) - 1,
            ] {
                let value = f64::from_bits((exponent << 52) | fraction);
                let significand = if exponent == 0 {
                    fraction
                } else {
                    (1 << 52) | fraction
                };
                let shift = if exponent == 0 {
                    -1074
                } else {
                    exponent as i32 - 1023 - 52
                };
                let expected = if shift >= 0 {
                    value
                } else if shift <= -64 {
                    0.0
                } else {
                    let denominator = 1_u64 << -shift;
                    let whole = significand / denominator;
                    let remainder = significand % denominator;
                    (whole + u64::from(remainder >= denominator / 2)) as f64
                };
                assert_eq!(
                    native_round_integral(NativeRoundingMode::HalfUp, value).to_bits(),
                    expected.to_bits()
                );
                assert_eq!(
                    native_round_integral(NativeRoundingMode::HalfUp, -value).to_bits(),
                    (-expected).to_bits()
                );
            }
        }
    }

    #[test]
    fn conversion_checks_range_after_rounding_and_keeps_closed_error_owners() {
        clear_dictionaries();
        for name in NAMES {
            let mode = V::adt(&format!("std/number::{name}"), V::Unit);
            for value in [
                -9_007_199_254_740_991.0,
                -4_503_599_627_370_497.0,
                0.0,
                -0.0,
                4_503_599_627_370_497.0,
                9_007_199_254_740_991.0,
            ] {
                let result = native_float_to_int(&mode, value);
                assert!(result.tagged("std/prelude::Right"));
                assert!(matches!(result.payload(), V::Int(v) if v==value as i64));
            }
            for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let result = native_float_to_int(&mode, value);
                assert!(
                    result.tagged("std/prelude::Left")
                        && result.payload().tagged("std/float::FloatNotFinite")
                );
            }
            for value in [
                9_007_199_254_740_992.0,
                -9_007_199_254_740_992.0,
                f64::MAX,
                -f64::MAX,
            ] {
                let result = native_float_to_int(&mode, value);
                assert!(
                    result.tagged("std/prelude::Left")
                        && result.payload().tagged("std/float::FloatOutsideIntRange")
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
