fn native_char(value: &V) -> char {
    match value {
        V::Char(value) => *value,
        _ => panic!("Rust ABI: expected Char"),
    }
}

fn native_char_code_point(value: V) -> V {
    checked_int(i64::from(u32::from(native_char(&value))))
}

fn native_char_from_code_point(value: V) -> V {
    let V::Int(value) = value else {
        panic!("Rust ABI: expected Int")
    };
    match u32::try_from(value).ok().and_then(char::from_u32) {
        Some(value) => V::adt("std/prelude::Just", V::Char(value)),
        None => V::adt("std/prelude::Nothing", V::Unit),
    }
}

fn native_char_to_string(value: V) -> V {
    V::text(&native_char(&value).to_string())
}

fn native_char_debug(value: V) -> V {
    let value = native_char(&value);
    // Match the language's Char Debug dictionary, including C1 controls.
    // Rust's escape_debug has a different policy for quotes and controls.
    let escaped = match value {
        '\\' => "\\\\".to_owned(),
        '\0' => "\\0".to_owned(),
        '\u{8}' => "\\b".to_owned(),
        '\t' => "\\t".to_owned(),
        '\n' => "\\n".to_owned(),
        '\u{c}' => "\\f".to_owned(),
        '\r' => "\\r".to_owned(),
        '\'' => "\\'".to_owned(),
        value if u32::from(value) < 0x20 || (0x7f..=0x9f).contains(&u32::from(value)) => {
            format!("\\u{{{:X}}}", u32::from(value))
        }
        value => value.to_string(),
    };
    V::text(&format!("'{escaped}'"))
}

fn native_char_binary(operator: &str, left: V, right: V) -> V {
    let left = native_char(&left);
    let right = native_char(&right);
    V::Bool(match operator {
        "==" => left == right,
        "!=" => left != right,
        "<" => left < right,
        "<=" => left <= right,
        ">" => left > right,
        ">=" => left >= right,
        _ => panic!("Rust ABI: unsupported Char comparison"),
    })
}

fn native_char_dictionary(identity: &'static str) -> V {
    cached(identity, || match identity {
        "Show<std/prelude::Char>" => V::dict(vec![(
            "show",
            V::fun(vec![], |_, value| {
                native_display(|| native_char_to_string(value))
            }),
        )]),
        "Debug<std/prelude::Char>" => V::dict(vec![(
            "debug",
            V::fun(vec![], |_, value| {
                native_display(|| native_char_debug(value))
            }),
        )]),
        "std/char::Eq" | "std/char::Ord" => {
            let eq = V::fun(vec![], |_, left| {
                V::fun(vec![left], |capture, right| {
                    native_char_binary("==", capture[0].clone(), right)
                })
            });
            let mut methods = vec![("eq", eq)];
            if identity == "std/char::Ord" {
                methods.push((
                    "compare",
                    V::fun(vec![], |_, left| {
                        V::fun(vec![left], |capture, right| {
                            let order = native_char(&capture[0]).cmp(&native_char(&right));
                            let tag = match order {
                                std::cmp::Ordering::Less => "std/prelude::Less",
                                std::cmp::Ordering::Equal => "std/prelude::Equal",
                                std::cmp::Ordering::Greater => "std/prelude::Greater",
                            };
                            V::adt(tag, V::Unit)
                        })
                    }),
                ));
            }
            V::dict(methods)
        }
        _ => panic!("Rust ABI: unsupported Char dictionary"),
    })
}

fn native_dictionary_comparison(operator: &str, dictionary: V, left: V, right: V) -> V {
    if matches!(operator, "==" | "!=") {
        let equal = dictionary.field("eq").apply(left).apply(right).boolean();
        return V::Bool(if operator == "==" { equal } else { !equal });
    }
    let order = dictionary.field("compare").apply(left).apply(right);
    let less = order.tagged("std/prelude::Less");
    let equal = order.tagged("std/prelude::Equal");
    let greater = order.tagged("std/prelude::Greater");
    assert!(less || equal || greater, "Rust ABI: expected Ordering");
    V::Bool(match operator {
        "<" => less,
        "<=" => less || equal,
        ">" => greater,
        ">=" => greater || equal,
        _ => panic!("Rust ABI: unsupported dictionary comparison"),
    })
}

fn native_textual_literal_matches(value: &V, literal: &str) -> bool {
    // Checked CoreIr represents both Char and String pattern tests as String.
    // Values retain their distinct ABI variants throughout calls and storage.
    match value {
        V::Char(value) => literal.chars().eq(std::iter::once(*value)),
        V::Text(value) => value.get() == literal,
        _ => false,
    }
}

#[cfg(test)]
mod character_tests {
    use super::*;

    #[test]
    fn code_point_conversion_covers_the_entire_unicode_scalar_domain() {
        for code_point in 0..=0x110000_u32 {
            let converted = native_char_from_code_point(checked_int(i64::from(code_point)));
            let valid = code_point <= 0x10ffff && !(0xd800..=0xdfff).contains(&code_point);
            assert_eq!(
                converted.tagged("std/prelude::Just"),
                valid,
                "{code_point:X}"
            );
            if valid {
                let scalar = converted.payload();
                let V::Int(roundtrip) = native_char_code_point(scalar.clone()) else {
                    panic!("codePoint did not return Int")
                };
                assert_eq!(roundtrip, i64::from(code_point));
                let text = native_char_to_string(scalar);
                let V::Text(text) = text else {
                    panic!("toString did not return String")
                };
                assert_eq!(text.get().chars().count(), 1);
                assert_eq!(text.get().chars().next().unwrap() as u32, code_point);
            } else {
                assert!(converted.tagged("std/prelude::Nothing"));
            }
        }
        for value in [-9_007_199_254_740_991, -1, 9_007_199_254_740_991] {
            assert!(native_char_from_code_point(checked_int(value)).tagged("std/prelude::Nothing"));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn character_display_order_and_literal_tests_preserve_scalar_identity() {
        for (value, expected) in [
            ('\0', "'\\0'"),
            ('\u{8}', "'\\b'"),
            ('\t', "'\\t'"),
            ('\n', "'\\n'"),
            ('\u{c}', "'\\f'"),
            ('\r', "'\\r'"),
            ('\'', "'\\''"),
            ('\\', "'\\\\'"),
            ('\u{1f}', "'\\u{1F}'"),
            ('\u{7f}', "'\\u{7F}'"),
            ('\u{9f}', "'\\u{9F}'"),
            ('λ', "'λ'"),
            ('\u{301}', "'\u{301}'"),
            ('🙂', "'🙂'"),
            ('\u{10ffff}', "'\u{10ffff}'"),
        ] {
            assert_eq!(native_char_debug(V::Char(value)).render(), expected);
            assert_eq!(
                native_char_dictionary("Show<std/prelude::Char>")
                    .field("show")
                    .apply(V::Char(value))
                    .render(),
                value.to_string()
            );
            assert!(native_textual_literal_matches(
                &V::Char(value),
                &value.to_string()
            ));
        }
        assert!(!native_textual_literal_matches(&V::Char('e'), "e\u{301}"));
        assert!(native_textual_literal_matches(
            &V::text("e\u{301}"),
            "e\u{301}"
        ));
        assert!(native_char_binary("<", V::Char('\u{ffff}'), V::Char('\u{10000}')).boolean());
        assert!(native_dictionary_comparison(
            "<",
            native_char_dictionary("std/char::Ord"),
            V::Char('\u{ffff}'),
            V::Char('\u{10000}')
        )
        .boolean());
        clear_dictionaries();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
