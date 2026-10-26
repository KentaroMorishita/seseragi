fn native_int_parse_error(name: &str, value: V) -> V {
    V::adt(&format!("std/int::{name}"), value)
}

fn native_int_invalid_digit(offset: usize) -> V {
    let offset = checked_int_value(i64::try_from(offset).expect("Int parse offset overflow"));
    native_int_parse_error(
        "InvalidIntDigit",
        V::Record(Shared::new(BTreeMap::from([
            ("offset".into(), V::Int(offset)),
            ("radix".into(), V::Int(10)),
        ]))),
    )
}

fn native_decimal_int(text: &str) -> V {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return native_left(native_int_parse_error("EmptyInt", V::Unit));
    }
    let negative = bytes[0] == b'-';
    let start = usize::from(matches!(bytes[0], b'+' | b'-'));
    if start == bytes.len() {
        return native_left(native_int_invalid_digit(start));
    }
    if bytes[start] == b'0' && start + 1 < bytes.len() {
        return native_left(native_int_invalid_digit(start + 1));
    }
    let mut value = 0_i64;
    for (offset, byte) in bytes.iter().enumerate().skip(start) {
        if !byte.is_ascii_digit() {
            // Every accepted prefix byte is ASCII. The first rejected byte
            // therefore starts the same UTF-8 scalar as the TS parser's offset.
            return native_left(native_int_invalid_digit(offset));
        }
        let next = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(i64::from(byte - b'0')))
            .filter(|v| *v <= 9_007_199_254_740_991);
        let Some(next) = next else {
            return native_left(native_int_parse_error("IntOutsideRange", V::Unit));
        };
        value = next;
    }
    native_right(V::Int(if negative { -value } else { value }))
}

fn native_int_parse_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/int::parse", [text]) => native_decimal_int(native_text(text)),
        ("std/int::EmptyInt", []) => native_int_parse_error("EmptyInt", V::Unit),
        ("std/int::IntOutsideRange", []) => native_int_parse_error("IntOutsideRange", V::Unit),
        ("std/int::InvalidIntRadix", [value]) => {
            native_int_parse_error("InvalidIntRadix", value.clone())
        }
        ("std/int::InvalidIntDigit", [value]) => {
            native_int_parse_error("InvalidIntDigit", value.clone())
        }
        _ => panic!("Rust ABI: unsupported Int parse operation or arity"),
    }
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
}
