// Exact coefficient * 10^-scale. Both signed integers are canonical decimal
// strings: parsing an exponent never converts it to a host integer or Float.
struct NativeDecimal {
    coefficient: String,
    scale: String,
}
impl TraceValue for NativeDecimal {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}

fn native_decimal_signed(negative: bool, digits: &str) -> String {
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        "0".into()
    } else if negative {
        format!("-{digits}")
    } else {
        digits.into()
    }
}
fn native_decimal_negate(value: &str) -> String {
    if value == "0" {
        "0".into()
    } else if let Some(digits) = value.strip_prefix('-') {
        digits.into()
    } else {
        format!("-{value}")
    }
}
// Only scale normalization needs arithmetic. Work is linear in input digits,
// even when the exponent's value is much larger than any machine integer.
fn native_decimal_add(left: &str, right: &str) -> String {
    let a_negative = left.starts_with('-');
    let b_negative = right.starts_with('-');
    let mut a = left.trim_start_matches('-');
    let mut b = right.trim_start_matches('-');
    let adding = a_negative == b_negative;
    let mut negative = a_negative;
    if !adding && (a.len(), a) < (b.len(), b) {
        std::mem::swap(&mut a, &mut b);
        negative = b_negative;
    }
    let mut a = a.bytes().rev();
    let mut b = b.bytes().rev();
    let mut output = Vec::new();
    let mut carry: i16 = 0;
    loop {
        let x = a.next();
        let y = b.next();
        if x.is_none() && y.is_none() {
            if adding && carry != 0 {
                output.push(b'0' + carry as u8);
            }
            break;
        }
        let x = x.map_or(0, |digit| i16::from(digit - b'0'));
        let y = y.map_or(0, |digit| i16::from(digit - b'0'));
        let digit = if adding { x + y + carry } else { x - y - carry };
        if adding {
            output.push(b'0' + (digit % 10) as u8);
            carry = digit / 10;
        } else {
            carry = i16::from(digit < 0);
            output.push(b'0' + digit.rem_euclid(10) as u8);
        }
    }
    output.reverse();
    native_decimal_signed(negative, std::str::from_utf8(&output).unwrap())
}
fn native_decimal_make(coefficient: &str, scale: String) -> NativeDecimal {
    let negative = coefficient.starts_with('-');
    let digits = coefficient.trim_start_matches('-').trim_start_matches('0');
    if digits.is_empty() {
        return NativeDecimal {
            coefficient: "0".into(),
            scale: "0".into(),
        };
    }
    let canonical = digits.trim_end_matches('0');
    let trailing = digits.len() - canonical.len();
    NativeDecimal {
        coefficient: native_decimal_signed(negative, canonical),
        scale: native_decimal_add(&scale, &native_decimal_negate(&trailing.to_string())),
    }
}
fn native_decimal_invalid(offset: usize) -> V {
    native_left(V::adt(
        "std/decimal::InvalidDecimal",
        V::Record(Shared::new(BTreeMap::from([(
            "offset".into(),
            checked_int(i64::try_from(offset).expect("Decimal offset fits Int")),
        )]))),
    ))
}
fn native_decimal_parse(text: &str) -> V {
    let bytes = text.as_bytes();
    let mut index = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let negative = bytes.first() == Some(&b'-');
    let integer_start = index;
    match bytes.get(index) {
        Some(b'0') => {
            index += 1;
            if bytes.get(index).is_some_and(u8::is_ascii_digit) {
                return native_decimal_invalid(index);
            }
        }
        Some(b'1'..=b'9') => {
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
        }
        _ => return native_decimal_invalid(index),
    }
    let integer = &text[integer_start..index];
    let mut fraction = "";
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        if !bytes.get(index).is_some_and(u8::is_ascii_digit) {
            return native_decimal_invalid(index);
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        fraction = &text[start..index];
    }
    let mut exponent = "0".to_owned();
    if matches!(bytes.get(index), Some(b'e' | b'E')) {
        index += 1;
        let negative = bytes.get(index) == Some(&b'-');
        if matches!(bytes.get(index), Some(b'+' | b'-')) {
            index += 1;
        }
        let start = index;
        if !bytes.get(index).is_some_and(u8::is_ascii_digit) {
            return native_decimal_invalid(index);
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        exponent = native_decimal_signed(negative, &text[start..index]);
    }
    if index != bytes.len() {
        return native_decimal_invalid(index);
    }
    let coefficient = format!("{}{integer}{fraction}", if negative { "-" } else { "" });
    let scale = native_decimal_add(
        &fraction.len().to_string(),
        &native_decimal_negate(&exponent),
    );
    native_right(V::Decimal(Shared::new(native_decimal_make(
        &coefficient,
        scale,
    ))))
}
fn native_decimal_zeros(output: &mut String, count: &str) {
    const MAX_SAFE: &str = "9007199254740991";
    assert!(
        (count.len(), count) <= (MAX_SAFE.len(), MAX_SAFE),
        "Decimal canonical spelling exceeds host limits"
    );
    let count = count
        .parse::<usize>()
        .expect("Decimal canonical spelling exceeds host limits");
    output
        .try_reserve(count)
        .expect("Decimal canonical spelling exceeds host limits");
    output.extend(std::iter::repeat_n('0', count));
}
fn native_decimal_format(value: &NativeDecimal) -> String {
    if value.coefficient == "0" {
        return "0".into();
    }
    let digits = value.coefficient.trim_start_matches('-');
    let mut output = if value.coefficient.starts_with('-') {
        "-".to_owned()
    } else {
        String::new()
    };
    if value.scale.starts_with('-') || value.scale == "0" {
        output.push_str(digits);
        native_decimal_zeros(&mut output, value.scale.trim_start_matches('-'));
    } else {
        let length = digits.len().to_string();
        if (value.scale.len(), value.scale.as_str()) >= (length.len(), length.as_str()) {
            output.push_str("0.");
            let zeros = native_decimal_add(&value.scale, &native_decimal_negate(&length));
            native_decimal_zeros(&mut output, &zeros);
            output.push_str(digits);
        } else {
            let point = digits.len() - value.scale.parse::<usize>().expect("bounded Decimal point");
            output.push_str(&digits[..point]);
            output.push('.');
            output.push_str(&digits[point..]);
        }
    }
    output
}
fn native_decimal_display(value: V) -> V {
    let V::Decimal(value) = value else {
        panic!("Rust ABI: expected Decimal for Show")
    };
    V::text(&native_decimal_format(value.get()))
}

fn native_decimal_show_dictionary() -> V {
    V::dict(vec![(
        "show",
        V::fun(vec![], |_, value| native_decimal_display(value)),
    )])
}

fn native_decimal_operation(name: &str, value: V) -> V {
    match (name, value) {
        ("std/decimal::parse", V::Text(text)) => native_decimal_parse(text.get()),
        ("std/decimal::fromInt", V::Int(value)) => V::Decimal(Shared::new(native_decimal_make(
            &checked_int_value(value).to_string(),
            "0".into(),
        ))),
        _ => panic!("Rust ABI: unsupported Decimal operation or type"),
    }
}

#[cfg(test)]
mod decimal_tests {
    use super::*;

    #[test]
    fn exact_decimal_grammar_and_canonical_spelling_do_not_use_binary_floats() {
        for (source, expected) in [
            ("0", "0"),
            ("-0.000e99999999999999999999999999999999", "0"),
            ("+12.3400", "12.34"),
            ("1.25e3", "1250"),
            ("1.25E-3", "0.00125"),
            ("-1000e-3", "-1"),
            ("1e+0000", "1"),
            ("0.000000000000000001", "0.000000000000000001"),
            (
                "123456789012345678901234567890.0001",
                "123456789012345678901234567890.0001",
            ),
        ] {
            let parsed = native_decimal_parse(source);
            assert!(parsed.tagged("std/prelude::Right"), "{source}");
            assert_eq!(
                native_decimal_display(parsed.payload()).render(),
                expected,
                "{source}"
            );
        }
        for value in [
            -9_007_199_254_740_991,
            -1000,
            -1,
            0,
            1,
            1000,
            9_007_199_254_740_991,
        ] {
            let decimal = native_decimal_operation("std/decimal::fromInt", V::Int(value));
            assert_eq!(native_decimal_display(decimal).render(), value.to_string());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn first_invalid_byte_including_missing_digits_and_unicode_is_preserved() {
        for (source, offset) in [
            ("", 0),
            ("+", 1),
            ("-", 1),
            ("01", 1),
            ("-01", 2),
            ("1.", 2),
            ("1e+", 3),
            ("1e-", 3),
            ("12_3", 2),
            ("1.2猫🙂", 3),
            ("🙂", 0),
            (" 1", 0),
            ("1 ", 1),
            ("NaN", 0),
            ("Infinity", 0),
            ("1e1.2", 3),
        ] {
            let result = native_decimal_parse(source);
            assert!(result.tagged("std/prelude::Left"), "{source}");
            let error = result.payload();
            assert!(error.tagged("std/decimal::InvalidDecimal"));
            assert_eq!(
                error.payload().field("offset").render(),
                offset.to_string(),
                "{source}"
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn signed_scale_arithmetic_keeps_carries_borrows_and_unbounded_exponents_exact() {
        for left in (-1234_i64..=1234).step_by(37) {
            for right in (-1234_i64..=1234).step_by(41) {
                assert_eq!(
                    native_decimal_add(&left.to_string(), &right.to_string()),
                    (left + right).to_string()
                );
            }
        }
        let nines = "9".repeat(10_000);
        let power = format!("1{}", "0".repeat(10_000));
        assert_eq!(native_decimal_add(&nines, "1"), power);
        assert_eq!(native_decimal_add(&power, "-1"), nines);
        for exponent in [&power, &format!("-{power}")] {
            let parsed = native_decimal_parse(&format!("1.200e{exponent}"));
            let V::Decimal(value) = parsed.payload() else {
                panic!()
            };
            assert_eq!(value.get().coefficient, "12");
            assert_eq!(
                value.get().scale,
                native_decimal_add("1", &native_decimal_negate(exponent))
            );
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn huge_coefficients_remain_exact_and_unformattable_exponents_fail_before_allocation() {
        let text = format!(
            "-{}.{}",
            "1234567890".repeat(10_000),
            "0".repeat(1000) + "1"
        );
        let parsed = native_decimal_parse(&text);
        assert_eq!(native_decimal_display(parsed.payload()).render(), text);
        drop(parsed);
        for source in [
            "1e9007199254740992",
            "1e-99999999999999999999999999999999999999",
        ] {
            let parsed = native_decimal_parse(source);
            assert!(parsed.tagged("std/prelude::Right"));
            let value = parsed.payload();
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_decimal_display(value)
            }))
            .is_err());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn decimal_saved_formatter_survives_its_owner_and_releases_reference_cycles() {
        enable_cycle_collection();
        let value = native_decimal_parse("9007199254740992000000000001.001").payload();
        let reference = V::reference(value.clone());
        let formatter = V::fun(vec![reference.clone()], |captures, _| {
            native_decimal_display(captures[0].read_reference())
        });
        assert_eq!(
            formatter.apply(V::Unit).render(),
            "9007199254740992000000000001.001"
        );
        reference.write_reference(V::Tuple(Shared::new(vec![
            reference.clone(),
            value.clone(),
            formatter.clone(),
        ])));
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((reference, value, formatter));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
