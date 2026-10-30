fn native_json_decode_error(kind: &'static str, message: &str) -> V {
    native_left(V::Record(Shared::new(BTreeMap::from([
        ("path".into(), V::Array(Shared::new(vec![]))),
        ("kind".into(), V::adt(kind, V::text(message))),
    ]))))
}

fn native_json_int(value: &NativeDecimal) -> Result<i64, &'static str> {
    if value.coefficient == "0" {
        return Ok(0);
    }
    // Decimal is normalized: a nonzero coefficient has no trailing zeros.
    // A positive scale therefore denotes a genuinely fractional value.
    if value.scale != "0" && !value.scale.starts_with('-') {
        return Err("expected integer");
    }
    let digits = value.coefficient.trim_start_matches('-');
    let zeros = value.scale.strip_prefix('-').unwrap_or("0");
    // Bound the exponent before converting it to a host integer. Never expand
    // an arbitrary exponent or pass this exact value through binary Float.
    if zeros.len() > 2 || (zeros.len() == 2 && zeros > "16") {
        return Err("integer is outside the safe Int range");
    }
    let zeros: usize = zeros.parse().expect("normalized Decimal scale");
    if digits.len() + zeros > 16 {
        return Err("integer is outside the safe Int range");
    }
    let mut magnitude: u64 = digits.parse().expect("at most sixteen Decimal digits");
    for _ in 0..zeros {
        magnitude *= 10;
    }
    if magnitude > 9_007_199_254_740_991 {
        return Err("integer is outside the safe Int range");
    }
    Ok(if value.coefficient.starts_with('-') {
        -(magnitude as i64)
    } else {
        magnitude as i64
    })
}

fn native_json_encode_primitive(kind: &'static str, value: V) -> V {
    match (kind, value) {
        ("Bool", value @ V::Bool(_)) => V::adt("std/json::JsonBool", value),
        ("String", value @ V::Text(_)) => V::adt("std/json::JsonString", value),
        ("Int", V::Int(value)) => V::adt(
            "std/json::JsonNumber",
            V::Decimal(Shared::new(native_decimal_make(
                &checked_int_value(value).to_string(),
                "0".into(),
            ))),
        ),
        ("Decimal", value @ V::Decimal(_)) => V::adt("std/json::JsonNumber", value),
        ("Unit", V::Unit) => V::adt("std/json::JsonNull", V::Unit),
        ("Json", value) => value,
        _ => panic!("Rust ABI: primitive JsonEncode type"),
    }
}

fn native_json_decode_primitive(kind: &'static str, value: V) -> V {
    let (tag, expected) = match kind {
        "Bool" => ("std/json::JsonBool", "JsonBool"),
        "String" => ("std/json::JsonString", "JsonString"),
        "Int" | "Decimal" => ("std/json::JsonNumber", "number"),
        "Unit" => ("std/json::JsonNull", "null"),
        "Json" => return native_right(value),
        _ => panic!("Rust ABI: primitive JsonDecode type"),
    };
    if !value.tagged(tag) {
        return native_json_decode_error("std/json::ExpectedJsonType", expected);
    }
    let payload = value.payload();
    if kind != "Int" {
        return native_right(payload);
    }
    let V::Decimal(decimal) = payload else {
        panic!("Rust ABI: JsonNumber Decimal payload")
    };
    match native_json_int(decimal.get()) {
        Ok(value) => native_right(checked_int(value)),
        Err(message) => native_json_decode_error("std/json::InvalidJsonValue", message),
    }
}

fn native_json_codec_dictionary(kind: &'static str, encode: bool) -> V {
    // These are closed standard methods. The function pointer has no hidden
    // managed captures; user dictionaries keep using the traced V ABI.
    macro_rules! dictionary {
        ($kind:literal) => {
            if encode {
                V::dict(vec![(
                    "encodeJson",
                    V::fun(vec![], |_, value| {
                        native_json_encode_primitive($kind, value)
                    }),
                )])
            } else {
                V::dict(vec![(
                    "decodeJson",
                    V::fun(vec![], |_, value| {
                        native_json_decode_primitive($kind, value)
                    }),
                )])
            }
        };
    }
    match kind {
        "Bool" => dictionary!("Bool"),
        "String" => dictionary!("String"),
        "Int" => dictionary!("Int"),
        "Decimal" => dictionary!("Decimal"),
        "Unit" => dictionary!("Unit"),
        "Json" => dictionary!("Json"),
        _ => panic!("Rust ABI: primitive JSON dictionary"),
    }
}

fn native_json_codec_string(encode: bool, value: V, dictionary: V) -> V {
    if encode {
        return V::text(&native_json_stringify(
            dictionary.field("encodeJson").apply(value),
        ));
    }
    let V::Text(text) = value else {
        panic!("Rust ABI: decodeString text")
    };
    let parsed = native_json_parse(text.get());
    if parsed.tagged("std/prelude::Left") {
        return native_left(V::adt("std/json::JsonSyntaxFailure", parsed.payload()));
    }
    let decoded = dictionary.field("decodeJson").apply(parsed.payload());
    if decoded.tagged("std/prelude::Left") {
        native_left(V::adt("std/json::JsonDecodeFailure", decoded.payload()))
    } else {
        decoded
    }
}

#[cfg(test)]
mod json_codec_tests {
    use super::*;
    fn as_text(value: &V) -> &str {
        let V::Text(value) = value else {
            panic!("expected text")
        };
        value.get()
    }

    #[test]
    fn primitive_dictionaries_round_trip_exact_values_and_identity_json() {
        for (kind, text) in [
            ("Bool", "true"),
            ("String", "\"雪🙂\\n\""),
            ("Int", "-9007199254740991"),
            ("Int", "9007199254740991"),
            ("Decimal", "12345678901234567890.001"),
            ("Unit", "null"),
            ("Json", "{\"z\":[1,true,null],\"a\":\"雪🙂\"}"),
        ] {
            let decoded = native_json_codec_string(
                false,
                V::text(text),
                native_json_codec_dictionary(kind, false),
            );
            assert!(decoded.tagged("std/prelude::Right"), "{kind}:{text}");
            let encoded = native_json_codec_string(
                true,
                decoded.payload(),
                native_json_codec_dictionary(kind, true),
            );
            assert_eq!(as_text(&encoded), text);
        }
    }

    #[test]
    fn int_decoding_checks_normalized_decimal_without_expanding_exponents() {
        for (source, expected) in [
            ("0e99999999999999999999999999999999999999", Ok(0)),
            ("-0e-99999999999999999999999999999999999999", Ok(0)),
            ("100.00e-2", Ok(1)),
            ("-900719925474099100e-2", Ok(-9_007_199_254_740_991)),
            ("9007199254740991.0000", Ok(9_007_199_254_740_991)),
            ("1.01", Err("expected integer")),
            (
                "1e-99999999999999999999999999999999",
                Err("expected integer"),
            ),
            (
                "9007199254740992",
                Err("integer is outside the safe Int range"),
            ),
            (
                "-9007199254740992",
                Err("integer is outside the safe Int range"),
            ),
            (
                "1e99999999999999999999999999999999",
                Err("integer is outside the safe Int range"),
            ),
        ] {
            let parsed = native_decimal_parse(source).payload();
            let V::Decimal(value) = parsed else { panic!() };
            assert_eq!(native_json_int(value.get()), expected, "{source}");
        }
    }

    #[test]
    fn decode_errors_preserve_root_path_kind_and_standard_messages() {
        for (kind, source, expected) in [
            ("Bool", "null", "JsonBool"),
            ("String", "false", "JsonString"),
            ("Int", "\"42\"", "number"),
            ("Decimal", "true", "number"),
            ("Unit", "0", "null"),
        ] {
            let decoded = native_json_decode_primitive(kind, native_json_parse(source).payload());
            assert!(decoded.tagged("std/prelude::Left"));
            let error = decoded.payload();
            assert!(matches!(error.field("path"), V::Array(path) if path.get().is_empty()));
            let kind = error.field("kind");
            assert!(kind.tagged("std/json::ExpectedJsonType"));
            assert_eq!(as_text(&kind.payload()), expected);
        }
    }

    #[test]
    fn read_errors_keep_syntax_and_domain_failures_distinct() {
        for (source, tag, inner) in [
            (
                "[",
                "std/json::JsonSyntaxFailure",
                "std/json::InvalidJsonSyntax",
            ),
            (
                "{\"x\":1,\"x\":2}",
                "std/json::JsonSyntaxFailure",
                "std/json::DuplicateJsonField",
            ),
            (
                "false",
                "std/json::JsonDecodeFailure",
                "std/json::ExpectedJsonType",
            ),
            (
                "1.5",
                "std/json::JsonDecodeFailure",
                "std/json::InvalidJsonValue",
            ),
        ] {
            let result = native_json_codec_string(
                false,
                V::text(source),
                native_json_codec_dictionary("Int", false),
            );
            assert!(result.tagged("std/prelude::Left"));
            let error = result.payload();
            assert!(error.tagged(tag));
            let payload = error.payload();
            assert!(if tag.ends_with("JsonDecodeFailure") {
                payload.field("kind").tagged(inner)
            } else {
                payload.tagged(inner)
            });
        }
    }
}
