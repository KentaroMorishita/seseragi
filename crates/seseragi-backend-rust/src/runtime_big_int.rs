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

const NATIVE_BIG_INT_LIMB_BASE: u64 = 1_000_000_000;

// Base-10^9 limbs keep each radix multiply/divide within u64. The complete
// value is never converted to a fixed-width integer or a Float.
fn native_big_int_radix_decimal(digits: &[u8], radix: u32) -> String {
    let mut limbs = vec![0_u32];
    for byte in digits {
        let mut carry = u64::from(native_big_int_digit(*byte).expect("validated BigInt digit"));
        for limb in &mut limbs {
            let value = u64::from(*limb) * u64::from(radix) + carry;
            *limb = (value % NATIVE_BIG_INT_LIMB_BASE) as u32;
            carry = value / NATIVE_BIG_INT_LIMB_BASE;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
    }
    let mut output = limbs.last().unwrap().to_string();
    for limb in limbs.iter().rev().skip(1) {
        output.push_str(&format!("{limb:09}"));
    }
    output
}

fn native_big_int_format_radix(value: &str, radix: u32) -> String {
    if radix == 10 || value == "0" {
        return value.into();
    }
    let negative = value.starts_with('-');
    let digits = value.trim_start_matches('-').as_bytes();
    let mut limbs: Vec<u32> = digits
        .rchunks(9)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0_u32, |value, digit| value * 10 + u32::from(digit - b'0'))
        })
        .collect();
    let mut output = Vec::new();
    while !(limbs.len() == 1 && limbs[0] == 0) {
        let mut remainder = 0_u64;
        for limb in limbs.iter_mut().rev() {
            let value = remainder * NATIVE_BIG_INT_LIMB_BASE + u64::from(*limb);
            *limb = (value / u64::from(radix)) as u32;
            remainder = value % u64::from(radix);
        }
        while limbs.len() > 1 && limbs.last() == Some(&0) {
            limbs.pop();
        }
        output.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[remainder as usize]);
    }
    if negative {
        output.push(b'-');
    }
    output.reverse();
    String::from_utf8(output).expect("ASCII BigInt radix output")
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
    // Preserve the linear decimal fast path, including very long inputs.
    let magnitude = if radix == 10 {
        let digits = text[start..].trim_start_matches('0');
        if digits.is_empty() {
            "0".into()
        } else {
            digits.to_owned()
        }
    } else {
        native_big_int_radix_decimal(&bytes[start..], radix as u32)
    };
    let canonical = if bytes[0] == b'-' && magnitude != "0" {
        format!("-{magnitude}")
    } else {
        magnitude
    };
    native_right(V::BigInt(Shared::new(canonical)))
}

fn native_big_int_operation(name: &str, value: V) -> V {
    match (name, value) {
        ("std/big-int::parse", V::Text(text)) => native_big_int_parse(text.get()),
        ("std/big-int::format", V::BigInt(value)) => V::text(value.get()),
        ("std/big-int::fromInt", V::Int(value)) => {
            V::BigInt(Shared::new(checked_int_value(value).to_string()))
        }
        _ => panic!("Rust ABI: unsupported BigInt operation or type"),
    }
}

fn native_big_int_standard_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/big-int::parseRadix", [V::Int(radix), text]) => {
            native_big_int_parse_radix(*radix, native_text(text), false)
        }
        ("std/big-int::formatRadix", [V::Int(radix), V::BigInt(value)]) => {
            if !(2..=36).contains(radix) {
                native_left(V::adt("std/big-int::InvalidBigIntRadix", V::Int(*radix)))
            } else {
                native_right(V::text(&native_big_int_format_radix(
                    value.get(),
                    *radix as u32,
                )))
            }
        }
        ("std/big-int::toInt", [V::BigInt(value)]) => {
            match value
                .get()
                .parse::<i64>()
                .ok()
                .filter(|n| (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(n))
            {
                Some(value) => native_right(V::Int(value)),
                None => native_left(V::adt("std/big-int::BigIntOutsideIntRange", V::Unit)),
            }
        }
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
