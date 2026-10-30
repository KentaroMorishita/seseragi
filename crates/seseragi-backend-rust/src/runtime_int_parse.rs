fn native_int_parse_error(tag: &'static str, value: V) -> V {
    V::adt(tag, value)
}

fn native_int_invalid_digit(offset: usize, radix: i64) -> V {
    let offset = checked_int_value(i64::try_from(offset).expect("Int parse offset overflow"));
    native_int_parse_error(
        "std/int::InvalidIntDigit",
        V::Record(Shared::new(BTreeMap::from([
            ("offset".into(), V::Int(offset)),
            ("radix".into(), V::Int(radix)),
        ]))),
    )
}

fn native_decimal_int(text: &str) -> V {
    native_parse_int(10, text, true)
}

fn native_parse_int(radix: i64, text: &str, canonical_decimal: bool) -> V {
    if !(2..=36).contains(&radix) {
        return native_left(native_int_parse_error(
            "std/int::InvalidIntRadix",
            V::Int(radix),
        ));
    }
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return native_left(native_int_parse_error("std/int::EmptyInt", V::Unit));
    }
    let negative = bytes[0] == b'-';
    let start = usize::from(matches!(bytes[0], b'+' | b'-'));
    if start == bytes.len() {
        return native_left(native_int_invalid_digit(start, radix));
    }
    if canonical_decimal && bytes[start] == b'0' && start + 1 < bytes.len() {
        return native_left(native_int_invalid_digit(start + 1, radix));
    }
    let mut value = 0_i64;
    for (offset, byte) in bytes.iter().enumerate().skip(start) {
        let digit = match byte {
            b'0'..=b'9' => i64::from(byte - b'0'),
            b'a'..=b'z' => i64::from(byte - b'a') + 10,
            b'A'..=b'Z' => i64::from(byte - b'A') + 10,
            _ => -1,
        };
        if digit < 0 || digit >= radix {
            // Every accepted prefix byte is ASCII. The first rejected byte
            // therefore starts the same UTF-8 scalar as the TS parser's offset.
            return native_left(native_int_invalid_digit(offset, radix));
        }
        let next = value
            .checked_mul(radix)
            .and_then(|v| v.checked_add(digit))
            .filter(|v| *v <= 9_007_199_254_740_991);
        let Some(next) = next else {
            return native_left(native_int_parse_error("std/int::IntOutsideRange", V::Unit));
        };
        value = next;
    }
    native_right(V::Int(if negative { -value } else { value }))
}

fn native_format_int(value: i64, radix: i64) -> String {
    let value = checked_int_value(value);
    assert!(
        (2..=36).contains(&radix),
        "Rust ABI: invalid Int format radix"
    );
    if value == 0 {
        return "0".into();
    }
    let mut magnitude = value.unsigned_abs();
    let radix = radix as u64;
    let mut digits = Vec::new();
    while magnitude != 0 {
        digits.push(b"0123456789abcdefghijklmnopqrstuvwxyz"[(magnitude % radix) as usize]);
        magnitude /= radix;
    }
    if value < 0 {
        digits.push(b'-');
    }
    digits.reverse();
    String::from_utf8(digits).expect("ASCII Int digits")
}

fn native_int_parse_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/int::parse", [text]) => native_decimal_int(native_text(text)),
        ("std/int::parseRadix", [V::Int(radix), text]) => {
            native_parse_int(*radix, native_text(text), false)
        }
        ("std/int::minValue", [V::Unit]) => V::Int(-9_007_199_254_740_991),
        ("std/int::maxValue", [V::Unit]) => V::Int(9_007_199_254_740_991),
        ("std/int::format", [V::Int(value)]) => V::text(&native_format_int(*value, 10)),
        ("std/int::formatRadix", [V::Int(radix), V::Int(value)]) => {
            if !(2..=36).contains(radix) {
                native_left(native_int_parse_error(
                    "std/int::InvalidIntRadix",
                    V::Int(*radix),
                ))
            } else {
                native_right(V::text(&native_format_int(*value, *radix)))
            }
        }
        ("std/int::EmptyInt", []) => native_int_parse_error("std/int::EmptyInt", V::Unit),
        ("std/int::IntOutsideRange", []) => {
            native_int_parse_error("std/int::IntOutsideRange", V::Unit)
        }
        ("std/int::InvalidIntRadix", [value]) => {
            native_int_parse_error("std/int::InvalidIntRadix", value.clone())
        }
        ("std/int::InvalidIntDigit", [value]) => {
            native_int_parse_error("std/int::InvalidIntDigit", value.clone())
        }
        _ => panic!("Rust ABI: unsupported Int parse operation or arity"),
    }
}

fn native_int_parse_error_show(value: &V) -> String {
    let tag = native_standard_tag(value, "std/int::");
    match tag.as_str() {
        "EmptyInt" | "IntOutsideRange" => tag,
        "InvalidIntRadix" => format!("{tag} {}", value.payload().render()),
        "InvalidIntDigit" => format!(
            "{tag} {{ offset: {}, radix: {} }}",
            value.payload().field("offset").render(),
            value.payload().field("radix").render()
        ),
        _ => panic!("Rust ABI: IntParseError Show variant"),
    }
}

fn native_int_parse_error_show_dictionary() -> V {
    V::dict(vec![(
        "show",
        V::fun(vec![], |_, value| {
            native_display(|| V::text(&native_int_parse_error_show(&value)))
        }),
    )])
}

#[cfg(test)]
mod int_parse_tests {
    use super::*;

    #[test]
    fn decimal_values_and_safe_boundaries_match_the_standard_contract() {
        clear_dictionaries();
        for (text, expected) in [
            ("0", 0),
            ("+0", 0),
            ("-0", 0),
            ("42", 42),
            ("+42", 42),
            ("-42", -42),
            ("9007199254740991", 9_007_199_254_740_991),
            ("-9007199254740991", -9_007_199_254_740_991),
        ] {
            let parsed = native_decimal_int(text);
            assert!(parsed.tagged("std/prelude::Right"), "{text}");
            assert!(matches!(parsed.payload(), V::Int(value) if value == expected));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn decimal_failures_keep_canonical_tags_byte_offsets_and_first_failure() {
        clear_dictionaries();
        for (text, name, offset) in [
            ("", "EmptyInt", None),
            ("+", "InvalidIntDigit", Some(1)),
            ("-", "InvalidIntDigit", Some(1)),
            ("01", "InvalidIntDigit", Some(1)),
            ("-01", "InvalidIntDigit", Some(2)),
            ("0🙂", "InvalidIntDigit", Some(1)),
            ("1雫", "InvalidIntDigit", Some(1)),
            ("🙂", "InvalidIntDigit", Some(0)),
            (" 1", "InvalidIntDigit", Some(0)),
            ("1_0", "InvalidIntDigit", Some(1)),
            ("1.0", "InvalidIntDigit", Some(1)),
            ("0x10", "InvalidIntDigit", Some(1)),
            ("9007199254740992", "IntOutsideRange", None),
            ("-9007199254740992", "IntOutsideRange", None),
            ("9007199254740992x", "IntOutsideRange", None),
            ("9x9999999999999999", "InvalidIntDigit", Some(1)),
        ] {
            let parsed = native_decimal_int(text);
            assert!(parsed.tagged("std/prelude::Left"), "{text}");
            let error = parsed.payload();
            assert!(error.tagged(&format!("std/int::{name}")), "{text}");
            if let Some(offset) = offset {
                let record = error.payload();
                assert!(matches!(record.field("offset"), V::Int(value) if value == offset));
                assert!(matches!(record.field("radix"), V::Int(10)));
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn radix_round_trips_keep_safe_limits_signs_and_lowercase_digits() {
        clear_dictionaries();
        for radix in 2..=36 {
            for value in [
                -9_007_199_254_740_991,
                -42,
                -1,
                0,
                1,
                42,
                9_007_199_254_740_991,
            ] {
                let text = native_format_int(value, radix);
                assert!(!text.bytes().any(|b| b.is_ascii_uppercase()));
                for spelling in [text.clone(), text.to_ascii_uppercase()] {
                    let parsed = native_parse_int(radix, &spelling, false);
                    assert!(parsed.tagged("std/prelude::Right"), "{radix}/{spelling}");
                    assert!(matches!(parsed.payload(), V::Int(actual) if actual == value));
                }
            }
        }
        assert_eq!(native_format_int(9_007_199_254_740_991, 36), "2gosa7pa2gv");
        assert_eq!(native_format_int(-42, 16), "-2a");
        for text in ["+00042", "-00042", "+00000", "-00000"] {
            assert!(native_parse_int(10, text, false).tagged("std/prelude::Right"));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn radix_failures_preserve_validation_order_offsets_and_first_overflow() {
        clear_dictionaries();
        for (radix, text, name, offset) in [
            (1, "", "InvalidIntRadix", None),
            (37, "🙂", "InvalidIntRadix", None),
            (-2, "42", "InvalidIntRadix", None),
            (2, "", "EmptyInt", None),
            (16, "+", "InvalidIntDigit", Some(1)),
            (36, "-", "InvalidIntDigit", Some(1)),
            (16, "+0x2a", "InvalidIntDigit", Some(2)),
            (16, "12🙂", "InvalidIntDigit", Some(2)),
            (2, "102", "InvalidIntDigit", Some(2)),
            (36, "z_z", "InvalidIntDigit", Some(1)),
            (10, "9007199254740992z", "IntOutsideRange", None),
            (36, "2gosa7pa2gw!", "IntOutsideRange", None),
            (36, "2go!sa7pa2gw", "InvalidIntDigit", Some(3)),
        ] {
            let parsed = native_parse_int(radix, text, false);
            assert!(parsed.tagged("std/prelude::Left"), "{radix}/{text}");
            let error = parsed.payload();
            assert!(error.tagged(&format!("std/int::{name}")), "{radix}/{text}");
            if let Some(offset) = offset {
                assert!(matches!(error.payload().field("offset"), V::Int(n) if n == offset));
                assert!(matches!(error.payload().field("radix"), V::Int(n) if n == radix));
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn format_validation_and_unit_boundaries_keep_canonical_values() {
        clear_dictionaries();
        for (name, expected) in [
            ("std/int::minValue", -9_007_199_254_740_991),
            ("std/int::maxValue", 9_007_199_254_740_991),
        ] {
            assert!(
                matches!(native_int_parse_operation(name, vec![V::Unit]), V::Int(n) if n == expected)
            );
        }
        let invalid =
            native_int_parse_operation("std/int::formatRadix", vec![V::Int(1), V::Int(42)]);
        assert!(invalid.tagged("std/prelude::Left"));
        assert!(invalid.payload().tagged("std/int::InvalidIntRadix"));
        drop(invalid);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
