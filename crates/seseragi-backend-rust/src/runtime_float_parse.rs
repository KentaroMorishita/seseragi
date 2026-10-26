fn native_float_parse_error(name: &str, value: V) -> V {
    V::adt(&format!("std/float::{name}"), value)
}

fn native_float_invalid(offset: usize) -> V {
    let offset = checked_int_value(i64::try_from(offset).expect("Float parse offset overflow"));
    native_float_parse_error(
        "InvalidFloat",
        V::Record(Shared::new(BTreeMap::from([(
            "offset".into(),
            V::Int(offset),
        )]))),
    )
}

fn native_float_syntax_error(bytes: &[u8]) -> Option<usize> {
    let digit = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_digit);
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        if !digit(index) {
            return Some(index);
        }
        while digit(index) {
            index += 1;
        }
    } else if digit(index) {
        while digit(index) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'.') {
            index += 1;
            while digit(index) {
                index += 1;
            }
        }
    } else {
        return Some(index);
    }
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        if !digit(index) {
            return Some(index);
        }
        while digit(index) {
            index += 1;
        }
    }
    // All accepted prefix bytes are ASCII. A rejected non-ASCII byte starts
    // its UTF-8 scalar, so this is also the standard parser's UTF-8 offset.
    (index != bytes.len()).then_some(index)
}

fn native_parse_float(text: &str) -> V {
    if text.is_empty() {
        return native_left(native_float_parse_error("EmptyFloat", V::Unit));
    }
    let special = match text {
        "NaN" => Some(f64::NAN),
        "Infinity" => Some(f64::INFINITY),
        "-Infinity" => Some(f64::NEG_INFINITY),
        _ => None,
    };
    if let Some(value) = special {
        return native_right(V::Float(value));
    }
    if let Some(offset) = native_float_syntax_error(text.as_bytes()) {
        // Syntax is checked before conversion, including after an overflowing
        // exponent. This preserves the TS parser's first syntax error.
        return native_left(native_float_invalid(offset));
    }
    let value = text.parse::<f64>().expect("validated decimal Float syntax");
    if value.is_finite() {
        native_right(V::Float(value))
    } else {
        native_left(native_float_parse_error("FloatParseOverflow", V::Unit))
    }
}

fn native_number_parse_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/float::parse", [text]) => native_parse_float(native_text(text)),
        ("std/float::EmptyFloat", []) => native_float_parse_error("EmptyFloat", V::Unit),
        ("std/float::FloatParseOverflow", []) => {
            native_float_parse_error("FloatParseOverflow", V::Unit)
        }
        ("std/float::InvalidFloat", [value]) => {
            native_float_parse_error("InvalidFloat", value.clone())
        }
        _ => native_int_parse_operation(name, arguments),
    }
}

#[cfg(test)]
mod float_parse_tests {
    use super::*;

    #[test]
    fn decimal_exponents_special_values_and_signed_underflow_keep_float_bits() {
        clear_dictionaries();
        for (text, expected) in [
            ("0", 0.0_f64),
            ("-0", -0.0),
            ("+0", 0.0),
            ("01", 1.0),
            ("2.5", 2.5),
            ("+.5", 0.5),
            ("1.", 1.0),
            ("1.e2", 100.0),
            (".1e-2", 0.001),
            ("5e-324", f64::from_bits(1)),
            ("-1e-999", -0.0),
            ("1.7976931348623157e308", f64::MAX),
            ("Infinity", f64::INFINITY),
            ("-Infinity", f64::NEG_INFINITY),
        ] {
            let parsed = native_parse_float(text);
            assert!(parsed.tagged("std/prelude::Right"), "{text}");
            let V::Float(value) = parsed.payload() else {
                panic!("Float value for {text}")
            };
            assert_eq!(value.to_bits(), expected.to_bits(), "{text}");
        }
        let parsed = native_parse_float("NaN");
        assert!(parsed.tagged("std/prelude::Right"));
        assert!(matches!(parsed.payload(), V::Float(value) if value.is_nan()));
        drop(parsed);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn syntax_failure_precedes_overflow_and_keeps_canonical_byte_offsets() {
        clear_dictionaries();
        for (text, name, offset) in [
            ("", "EmptyFloat", None),
            ("+", "InvalidFloat", Some(1)),
            (".", "InvalidFloat", Some(1)),
            ("1e+", "InvalidFloat", Some(3)),
            ("2.5🙂", "InvalidFloat", Some(3)),
            ("🙂", "InvalidFloat", Some(0)),
            ("-雫", "InvalidFloat", Some(1)),
            ("1_0", "InvalidFloat", Some(1)),
            ("0x10", "InvalidFloat", Some(1)),
            (" 1", "InvalidFloat", Some(0)),
            ("1 ", "InvalidFloat", Some(1)),
            ("nan", "InvalidFloat", Some(0)),
            ("+Infinity", "InvalidFloat", Some(1)),
            ("-NaN", "InvalidFloat", Some(1)),
            ("1e999x", "InvalidFloat", Some(5)),
            ("1e309", "FloatParseOverflow", None),
            ("-1e309", "FloatParseOverflow", None),
        ] {
            let parsed = native_parse_float(text);
            assert!(parsed.tagged("std/prelude::Left"), "{text}");
            let error = parsed.payload();
            assert!(error.tagged(&format!("std/float::{name}")), "{text}");
            if let Some(offset) = offset {
                assert!(
                    matches!(error.payload().field("offset"), V::Int(value) if value == offset)
                );
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
