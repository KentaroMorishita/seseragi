fn native_big_int_invalid_digit(offset: usize) -> V {
    native_left(V::adt(
        "std/big-int::InvalidBigIntDigit",
        V::Record(Shared::new(BTreeMap::from([
            (
                "offset".into(),
                checked_int(i64::try_from(offset).expect("BigInt offset fits Int")),
            ),
            ("radix".into(), V::Int(10)),
        ]))),
    ))
}

fn native_big_int_parse(text: &str) -> V {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return native_left(V::adt("std/big-int::EmptyBigInt", V::Unit));
    }
    let start = usize::from(matches!(bytes[0], b'+' | b'-'));
    if start == bytes.len() {
        return native_big_int_invalid_digit(start);
    }
    if bytes[start] == b'0' && start + 1 < bytes.len() {
        return native_big_int_invalid_digit(start + 1);
    }
    for (offset, byte) in bytes.iter().enumerate().skip(start) {
        if !byte.is_ascii_digit() {
            // The accepted prefix contains only ASCII, so the first rejected
            // byte is exactly the shared contract's UTF-8 scalar offset.
            return native_big_int_invalid_digit(offset);
        }
    }
    // A distinct opaque value stores canonical signed decimal digits exactly.
    // These operations require no host integer/floating-point conversion or
    // arbitrary-precision arithmetic dependency. Future arithmetic must retain
    // the same exact value contract; it is not admitted by these three APIs.
    let canonical = if &text[start..] == "0" {
        "0"
    } else if bytes[0] == b'+' {
        &text[1..]
    } else {
        text
    };
    native_right(V::BigInt(Shared::new(canonical.into())))
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
}
