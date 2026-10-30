fn native_big_int_invalid_digit(offset: usize, radix: i64) -> V {
    native_left(V::adt(
        "std/big-int::InvalidBigIntDigit",
        V::Record(Shared::new(BTreeMap::from([
            (
                "offset".into(),
                checked_int(i64::try_from(offset).expect("BigInt offset fits Int")),
            ),
            ("radix".into(), V::Int(radix)),
        ]))),
    ))
}

fn native_big_int_parse(text: &str) -> V {
    native_big_int_parse_radix(10, text, true)
}

fn native_big_int_digit(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'z' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'Z' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

#[cfg(test)]
fn native_big_int_format_radix(value: &str, radix: u32) -> String {
    NativeBigInt::from_digits(
        value.starts_with('-'),
        value.trim_start_matches('-').as_bytes(),
        10,
    )
    .format_radix(radix)
}

fn native_big_int_parse_radix(radix: i64, text: &str, canonical_decimal: bool) -> V {
    if !(2..=36).contains(&radix) {
        return native_left(V::adt("std/big-int::InvalidBigIntRadix", V::Int(radix)));
    }
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return native_left(V::adt("std/big-int::EmptyBigInt", V::Unit));
    }
    let start = usize::from(matches!(bytes[0], b'+' | b'-'));
    if start == bytes.len() {
        return native_big_int_invalid_digit(start, radix);
    }
    if canonical_decimal && bytes[start] == b'0' && start + 1 < bytes.len() {
        return native_big_int_invalid_digit(start + 1, radix);
    }
    for (offset, byte) in bytes.iter().enumerate().skip(start) {
        if !native_big_int_digit(*byte).is_some_and(|digit| i64::from(digit) < radix) {
            // The accepted prefix contains only ASCII, so the first rejected
            // byte is exactly the shared contract's UTF-8 scalar offset.
            return native_big_int_invalid_digit(offset, radix);
        }
    }
    native_right(V::BigInt(Shared::new(NativeBigInt::from_digits(
        bytes[0] == b'-',
        &bytes[start..],
        radix as u32,
    ))))
}

fn native_big_int_operation(name: &str, value: V) -> V {
    match (name, value) {
        ("std/big-int::parse", V::Text(text)) => native_big_int_parse(text.get()),
        ("std/big-int::format", V::BigInt(value)) => V::text(value.get().decimal()),
        ("std/big-int::fromInt", V::Int(value)) => V::BigInt(Shared::new(NativeBigInt::from_i64(
            checked_int_value(value),
        ))),
        _ => panic!("Rust ABI: unsupported BigInt operation or type"),
    }
}

fn native_big_int_standard_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/big-int::abs", [V::BigInt(value)]) => {
            if value.get().negative {
                V::BigInt(Shared::new(NativeBigInt::from_limbs(
                    false,
                    value.get().limbs.clone(),
                )))
            } else {
                arguments[0].clone()
            }
        }
        ("std/big-int::sign", [V::BigInt(value)]) => V::Int(if value.get().is_zero() {
            0
        } else if value.get().negative {
            -1
        } else {
            1
        }),
        (
            "std/big-int::checkedDivide" | "std/big-int::checkedRemainder",
            [V::BigInt(divisor), V::BigInt(dividend)],
        ) => {
            if divisor.get().is_zero() {
                native_left(V::adt("std/big-int::BigIntDivisionByZero", V::Unit))
            } else {
                let (quotient, remainder) = dividend.get().quotient_remainder(divisor.get());
                native_right(V::BigInt(Shared::new(
                    if name == "std/big-int::checkedDivide" {
                        quotient
                    } else {
                        remainder
                    },
                )))
            }
        }
        ("std/big-int::checkedPower", [V::Int(exponent), V::BigInt(base)]) => {
            checked_int_value(*exponent);
            if *exponent < 0 {
                native_left(V::adt(
                    "std/big-int::NegativeBigIntExponent",
                    V::Int(*exponent),
                ))
            } else {
                native_right(V::BigInt(Shared::new(base.get().power(*exponent))))
            }
        }
        ("std/big-int::BigIntDivisionByZero", []) => {
            V::adt("std/big-int::BigIntDivisionByZero", V::Unit)
        }
        ("std/big-int::NegativeBigIntExponent", [V::Int(exponent)]) => V::adt(
            "std/big-int::NegativeBigIntExponent",
            checked_int(*exponent),
        ),
        ("std/big-int::parseRadix", [V::Int(radix), text]) => {
            native_big_int_parse_radix(*radix, native_text(text), false)
        }
        ("std/big-int::formatRadix", [V::Int(radix), V::BigInt(value)]) => {
            if !(2..=36).contains(radix) {
                native_left(V::adt("std/big-int::InvalidBigIntRadix", V::Int(*radix)))
            } else {
                native_right(V::text(&value.get().format_radix(*radix as u32)))
            }
        }
        ("std/big-int::toInt", [V::BigInt(value)]) => match value.get().to_safe_int() {
            Some(value) => native_right(V::Int(value)),
            None => native_left(V::adt("std/big-int::BigIntOutsideIntRange", V::Unit)),
        },
        ("std/big-int::EmptyBigInt", []) => V::adt("std/big-int::EmptyBigInt", V::Unit),
        ("std/big-int::BigIntOutsideIntRange", []) => {
            V::adt("std/big-int::BigIntOutsideIntRange", V::Unit)
        }
        ("std/big-int::InvalidBigIntRadix", [radix]) => {
            V::adt("std/big-int::InvalidBigIntRadix", radix.clone())
        }
        ("std/big-int::InvalidBigIntDigit", [detail]) => {
            V::adt("std/big-int::InvalidBigIntDigit", detail.clone())
        }
        (_, [value]) => native_big_int_operation(name, value.clone()),
        _ => panic!("Rust ABI: unsupported BigInt operation or arity"),
    }
}

#[cfg(test)]
mod big_int_tests {
    use super::*;

    #[test]
    fn decimal_parse_is_exact_unbounded_and_normalizes_only_sign_and_zero() {
        for (text, expected) in [
            ("0", "0"),
            ("+0", "0"),
            ("-0", "0"),
            ("+42", "42"),
            ("-42", "-42"),
            (
                "900719925474099200000000000000000001",
                "900719925474099200000000000000000001",
            ),
        ] {
            let parsed = native_big_int_parse(text);
            assert!(parsed.tagged("std/prelude::Right"));
            let value = parsed.payload();
            assert!(matches!(value, V::BigInt(_)));
            assert_eq!(
                native_big_int_operation("std/big-int::format", value).render(),
                expected
            );
        }
        let huge = format!("-{}", "1234567890".repeat(10_000));
        let parsed = native_big_int_parse(&huge);
        assert!(parsed.tagged("std/prelude::Right"));
        assert_eq!(
            native_big_int_operation("std/big-int::format", parsed.payload()).render(),
            huge
        );
        drop(parsed);
        for number in [-9_007_199_254_740_991, -1, 0, 1, 9_007_199_254_740_991] {
            let value = native_big_int_operation("std/big-int::fromInt", V::Int(number));
            assert_eq!(
                native_big_int_operation("std/big-int::format", value).render(),
                number.to_string()
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn invalid_syntax_keeps_the_first_byte_offset_after_arbitrarily_large_prefixes() {
        assert!(native_big_int_parse("")
            .payload()
            .tagged("std/big-int::EmptyBigInt"));
        for (text, offset) in [
            ("+", 1),
            ("-", 1),
            ("00", 1),
            ("-01", 2),
            ("0x10", 1),
            ("1_0", 1),
            ("1.0", 1),
            ("1e2", 1),
            (" 1", 0),
            ("1 ", 1),
            ("🙂", 0),
            ("1雪🙂", 1),
            ("90071992547409929999999999999x", 29),
        ] {
            let parsed = native_big_int_parse(text);
            assert!(parsed.tagged("std/prelude::Left"), "{text}");
            let error = parsed.payload();
            assert!(error.tagged("std/big-int::InvalidBigIntDigit"), "{text}");
            assert_eq!(
                error.payload().field("offset").render(),
                offset.to_string(),
                "{text}"
            );
            assert_eq!(error.payload().field("radix").render(), "10");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn opaque_big_values_keep_captures_alive_and_leave_no_storage_in_ref_cycles() {
        enable_cycle_collection();
        let value = native_big_int_parse("9007199254740992000000000001").payload();
        let cell = V::reference(value.clone());
        let format = V::fun(vec![cell.clone()], |captures, _| {
            native_big_int_operation("std/big-int::format", captures[0].read_reference())
        });
        assert_eq!(
            format.apply(V::Unit).render(),
            "9007199254740992000000000001"
        );
        cell.write_reference(V::Tuple(Shared::new(vec![
            cell.clone(),
            value.clone(),
            format.clone(),
        ])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((cell, value, format));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn arbitrary_radices_round_trip_exact_signs_zero_and_large_values() {
        for original in [
            "0".to_owned(),
            "-42".into(),
            "900719925474099200000000000000000001".into(),
            format!("-{}", "1234567890".repeat(100)),
        ] {
            for radix in 2..=36 {
                let encoded = native_big_int_format_radix(&original, radix);
                for spelling in [encoded.clone(), encoded.to_ascii_uppercase()] {
                    let decoded = native_big_int_parse_radix(i64::from(radix), &spelling, false);
                    assert!(decoded.tagged("std/prelude::Right"));
                    assert_eq!(
                        native_big_int_operation("std/big-int::format", decoded.payload()).render(),
                        original
                    );
                }
            }
        }
        for text in ["-0000", "+00000", "000000"] {
            assert_eq!(
                native_big_int_parse_radix(36, text, false)
                    .payload()
                    .render(),
                "0"
            );
        }
        assert_eq!(
            native_big_int_format_radix("340282366920938463463374607431768211455", 16),
            "ffffffffffffffffffffffffffffffff"
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn radix_failures_keep_validation_order_and_utf8_positions_after_unbounded_prefixes() {
        for (radix, text, tag, offset) in [
            (1, "", "InvalidBigIntRadix", None),
            (37, "🙂", "InvalidBigIntRadix", None),
            (16, "", "EmptyBigInt", None),
            (2, "+", "InvalidBigIntDigit", Some(1)),
            (16, "+0x10", "InvalidBigIntDigit", Some(2)),
            (36, "12雫🙂", "InvalidBigIntDigit", Some(2)),
            (2, "102", "InvalidBigIntDigit", Some(2)),
        ] {
            let parsed = native_big_int_parse_radix(radix, text, false);
            assert!(parsed.tagged("std/prelude::Left"));
            let error = parsed.payload();
            assert!(error.tagged(&format!("std/big-int::{tag}")));
            if let Some(offset) = offset {
                assert!(matches!(error.payload().field("offset"),V::Int(n) if n==offset));
                assert!(matches!(error.payload().field("radix"),V::Int(n) if n==radix));
            }
        }
        let invalid = format!("{}🙂", "9".repeat(100_000));
        let parsed = native_big_int_parse_radix(10, &invalid, false);
        assert!(matches!(
            parsed.payload().payload().field("offset"),
            V::Int(100_000)
        ));
        drop(parsed);
        assert!(native_big_int_parse("01").tagged("std/prelude::Left"));
        assert_eq!(
            native_big_int_parse_radix(10, "01", false)
                .payload()
                .render(),
            "1"
        );
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn to_int_keeps_safe_boundaries_without_rounding_or_overflowing() {
        for (text, expected) in [
            ("0", Some(0)),
            ("9007199254740991", Some(9_007_199_254_740_991)),
            ("-9007199254740991", Some(-9_007_199_254_740_991)),
            ("9007199254740992", None),
            ("-9007199254740992", None),
            ("9223372036854775807", None),
            ("9223372036854775808", None),
            ("184467440737095516160000000000000000001", None),
        ] {
            let value = native_big_int_parse(text).payload();
            let converted = native_big_int_standard_operation("std/big-int::toInt", vec![value]);
            if let Some(expected) = expected {
                assert!(converted.tagged("std/prelude::Right"));
                assert!(matches!(converted.payload(),V::Int(n) if n==expected));
            } else {
                assert!(
                    converted.tagged("std/prelude::Left")
                        && converted
                            .payload()
                            .tagged("std/big-int::BigIntOutsideIntRange")
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn radix_formatter_releases_captured_big_values_in_reference_cycles() {
        enable_cycle_collection();
        let value = native_big_int_parse("340282366920938463463374607431768211455").payload();
        let cell = V::reference(value.clone());
        let format = V::fun(vec![cell.clone()], |captures, _| {
            native_big_int_standard_operation(
                "std/big-int::formatRadix",
                vec![V::Int(16), captures[0].read_reference()],
            )
        });
        let result = format.apply(V::Unit);
        assert!(result.tagged("std/prelude::Right"));
        assert_eq!(
            result.payload().render(),
            "ffffffffffffffffffffffffffffffff"
        );
        drop(result);
        cell.write_reference(V::Tuple(Shared::new(vec![
            cell.clone(),
            value.clone(),
            format.clone(),
        ])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((cell, value, format));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
