thread_local! {
    static JSON_DECODER_DEPTH: Cell<usize> = const { Cell::new(0) };
}
struct NativeJsonDecoderFrame(usize);
impl NativeJsonDecoderFrame {
    fn enter() -> Self {
        JSON_DECODER_DEPTH.with(|depth| {
            let previous = depth.get();
            if previous >= 128 {
                panic!("JSON decoder nesting exceeds host limits");
            }
            depth.set(previous + 1);
            Self(previous)
        })
    }
}
impl Drop for NativeJsonDecoderFrame {
    fn drop(&mut self) {
        JSON_DECODER_DEPTH.with(|depth| depth.set(self.0));
    }
}

fn native_json_prepend_path(segment: V, failure: V) -> V {
    let error = failure.payload();
    let V::Array(path) = error.field("path") else {
        panic!("Rust ABI: JSON DecodeError path")
    };
    let mut result = Vec::with_capacity(path.get().len() + 1);
    result.push(segment);
    result.extend(path.get().iter().cloned());
    native_left(V::Record(Shared::new(BTreeMap::from([
        ("path".into(), V::Array(Shared::new(result))),
        ("kind".into(), error.field("kind")),
    ]))))
}
fn native_json_field_value(value: &V, name: &str) -> Option<V> {
    let payload = value.payload();
    let map = native_map_value(&payload);
    let mut id = map.first;
    while let Some(current) = id {
        let entry = native_map_entry(map, current);
        let V::Text(key) = &entry.key else {
            panic!("Rust ABI: JSON field name")
        };
        if key.get() == name {
            return Some(entry.value.clone());
        }
        id = entry.next;
    }
    None
}
fn native_json_field_decoder(captures: &[V], value: V, optional: bool) -> V {
    if !value.tagged("std/json::JsonObject") {
        return native_json_decode_error("std/json::ExpectedJsonType", "object");
    }
    let V::Text(name) = &captures[0] else {
        panic!("Rust ABI: JSON field name")
    };
    let selected = native_json_field_value(&value, name.get());
    // The returned value and decoder never retain unrelated object fields.
    drop(value);
    let Some(selected) = selected else {
        return if optional {
            native_right(V::adt("std/prelude::Nothing", V::Unit))
        } else {
            native_json_decode_error("std/json::MissingJsonField", name.get())
        };
    };
    let result = captures[1].apply(selected);
    if result.tagged("std/prelude::Left") {
        native_json_prepend_path(V::adt("std/json::JsonField", captures[0].clone()), result)
    } else if optional {
        native_right(V::adt("std/prelude::Just", result.payload()))
    } else {
        result
    }
}

fn native_json_decoder_run(name: &'static str, captures: &[V], value: V) -> V {
    let _frame = NativeJsonDecoderFrame::enter();
    match name {
        "std/json::field" => native_json_field_decoder(captures, value, false),
        "std/json::optionalField" => native_json_field_decoder(captures, value, true),
        "std/json::index" => {
            if !value.tagged("std/json::JsonArray") {
                return native_json_decode_error("std/json::ExpectedJsonType", "array");
            }
            let V::Int(position) = captures[0] else {
                panic!("Rust ABI: JSON index")
            };
            let position = checked_int_value(position);
            let V::Array(values) = value.payload() else {
                panic!("Rust ABI: JsonArray")
            };
            let selected = usize::try_from(position)
                .ok()
                .and_then(|index| values.get().get(index))
                .cloned();
            drop(values);
            drop(value);
            let Some(selected) = selected else {
                return native_json_decode_error(
                    "std/json::InvalidJsonValue",
                    &format!("missing array index {position}"),
                );
            };
            let result = captures[1].apply(selected);
            if result.tagged("std/prelude::Left") {
                native_json_prepend_path(
                    V::adt("std/json::JsonIndex", checked_int(position)),
                    result,
                )
            } else {
                result
            }
        }
        "std/json::array" => {
            if !value.tagged("std/json::JsonArray") {
                return native_json_decode_error("std/json::ExpectedJsonType", "array");
            }
            let V::Array(values) = value.payload() else {
                panic!("Rust ABI: JsonArray")
            };
            let mut output = Vec::with_capacity(values.get().len());
            for (index, value) in values.get().iter().enumerate() {
                let decoded = captures[0].apply(value.clone());
                if decoded.tagged("std/prelude::Left") {
                    return native_json_prepend_path(
                        V::adt(
                            "std/json::JsonIndex",
                            checked_int(i64::try_from(index).expect("JSON array index fits Int")),
                        ),
                        decoded,
                    );
                }
                output.push(decoded.payload());
            }
            native_right(V::Array(Shared::new(output)))
        }
        "std/json::record" => {
            if !value.tagged("std/json::JsonObject") {
                return native_json_decode_error("std/json::ExpectedJsonType", "object");
            }
            let V::Array(fields) = &captures[0] else {
                panic!("Rust ABI: JSON record fields")
            };
            let expected = fields
                .get()
                .iter()
                .map(|field| {
                    let V::Text(name) = field.index(0) else {
                        panic!("Rust ABI: JSON record field name")
                    };
                    name.get().clone()
                })
                .collect::<std::collections::BTreeSet<_>>();
            let payload = value.payload();
            let map = native_map_value(&payload);
            let mut id = map.first;
            while let Some(current) = id {
                let entry = native_map_entry(map, current);
                let V::Text(name) = &entry.key else {
                    panic!("Rust ABI: JSON object field name")
                };
                if !expected.contains(name.get()) {
                    return native_json_decode_error("std/json::UnknownJsonField", name.get());
                }
                id = entry.next;
            }
            let mut output = Vec::with_capacity(fields.get().len());
            for field in fields.get() {
                let name = field.index(0);
                let decoded = native_json_field_decoder(
                    &[name.clone(), field.index(1)],
                    value.clone(),
                    false,
                );
                if decoded.tagged("std/prelude::Left") {
                    return decoded;
                }
                output.push(V::Tuple(Shared::new(vec![name, decoded.payload()])));
            }
            native_right(V::Array(Shared::new(output)))
        }
        "std/json::oneOf" => {
            let V::Array(decoders) = &captures[0] else {
                panic!("Rust ABI: JSON alternatives")
            };
            let mut last = None;
            for decoder in decoders.get() {
                let result = decoder.apply(value.clone());
                if result.tagged("std/prelude::Right") {
                    return result;
                }
                last = Some(result);
            }
            last.unwrap_or_else(|| {
                native_json_decode_error("std/json::InvalidJsonValue", "no JSON decoder matched")
            })
        }
        "std/json::map" => {
            let result = captures[1].apply(value);
            if result.tagged("std/prelude::Left") {
                result
            } else {
                native_right(captures[0].apply(result.payload()))
            }
        }
        "std/json::flatMap" => {
            let result = captures[1].apply(value.clone());
            if result.tagged("std/prelude::Left") {
                result
            } else {
                captures[0].apply(result.payload()).apply(value)
            }
        }
        _ => panic!("Rust ABI: unsupported JSON decoder"),
    }
}

fn native_json_decoder_operation(name: &'static str, arguments: Vec<V>) -> V {
    macro_rules! decoder {
        ($name:literal) => {
            V::fun(arguments, |captures, value| {
                native_json_decoder_run($name, captures, value)
            })
        };
    }
    match name {
        "std/json::field" => decoder!("std/json::field"),
        "std/json::optionalField" => decoder!("std/json::optionalField"),
        "std/json::index" => decoder!("std/json::index"),
        "std/json::array" => decoder!("std/json::array"),
        "std/json::record" => decoder!("std/json::record"),
        "std/json::oneOf" => decoder!("std/json::oneOf"),
        "std/json::map" => decoder!("std/json::map"),
        "std/json::flatMap" => decoder!("std/json::flatMap"),
        _ => panic!("Rust ABI: unsupported JSON decoder constructor"),
    }
}

#[cfg(test)]
mod json_decoder_tests {
    use super::*;
    fn json(source: &str) -> V {
        let value = native_json_parse(source);
        assert!(value.tagged("std/prelude::Right"));
        value.payload()
    }
    fn int() -> V {
        native_json_codec_dictionary("Int", false).field("decodeJson")
    }
    fn decode(name: &'static str, args: Vec<V>, source: &str) -> V {
        native_json_decoder_operation(name, args).apply(json(source))
    }
    fn error(value: &V, tag: &str, message: &str, path: &[(&str, &str)]) {
        assert!(value.tagged("std/prelude::Left"));
        let detail = value.payload();
        let kind = detail.field("kind");
        assert!(kind.tagged(tag));
        let V::Text(text) = kind.payload() else {
            panic!()
        };
        assert_eq!(text.get(), message);
        let V::Array(parts) = detail.field("path") else {
            panic!()
        };
        assert_eq!(parts.get().len(), path.len());
        for (actual, (tag, payload)) in parts.get().iter().zip(path) {
            assert!(actual.tagged(tag));
            assert_eq!(&actual.payload().render(), payload);
        }
    }
    #[test]
    fn fields_optional_null_and_nested_paths_preserve_standard_failures() {
        assert!(
            decode(
                "std/json::field",
                vec![V::text("雪"), int()],
                r#"{"雪":42}"#
            )
            .payload()
            .render()
                == "42"
        );
        error(
            &decode("std/json::field", vec![V::text("雪"), int()], "{}"),
            "std/json::MissingJsonField",
            "雪",
            &[],
        );
        error(
            &decode("std/json::field", vec![V::text("雪"), int()], "[]"),
            "std/json::ExpectedJsonType",
            "object",
            &[],
        );
        let missing = decode("std/json::optionalField", vec![V::text("雪"), int()], "{}");
        assert!(missing.payload().tagged("std/prelude::Nothing"));
        let present = decode(
            "std/json::optionalField",
            vec![V::text("雪"), int()],
            r#"{"雪":42}"#,
        );
        assert!(present.payload().tagged("std/prelude::Just"));
        assert_eq!(present.payload().payload().render(), "42");
        let inner =
            native_json_decoder_operation("std/json::field", vec![V::text("missing"), int()]);
        error(
            &decode(
                "std/json::field",
                vec![V::text("outer"), inner],
                r#"{"outer":{}}"#,
            ),
            "std/json::MissingJsonField",
            "missing",
            &[("std/json::JsonField", "outer")],
        );
        error(
            &decode(
                "std/json::optionalField",
                vec![V::text("雪"), int()],
                r#"{"雪":null}"#,
            ),
            "std/json::ExpectedJsonType",
            "number",
            &[("std/json::JsonField", "雪")],
        );
    }
    #[test]
    fn index_and_array_keep_bounds_path_order_and_stop_on_first_failure() {
        for position in [-1, 2, 9_007_199_254_740_991] {
            error(
                &decode(
                    "std/json::index",
                    vec![checked_int(position), int()],
                    "[1,2]",
                ),
                "std/json::InvalidJsonValue",
                &format!("missing array index {position}"),
                &[],
            );
        }
        error(
            &decode("std/json::index", vec![checked_int(0), int()], "{}"),
            "std/json::ExpectedJsonType",
            "array",
            &[],
        );
        let guarded = V::fun(vec![], |_, value| {
            let decoded = native_json_decode_primitive("Int", value);
            assert!(
                !decoded.tagged("std/prelude::Right") || decoded.payload().render() != "3",
                "late callback must not run"
            );
            decoded
        });
        error(
            &decode("std/json::array", vec![guarded], "[1,false,3]"),
            "std/json::ExpectedJsonType",
            "number",
            &[("std/json::JsonIndex", "1")],
        );
        let empty = decode("std/json::array", vec![int()], "[]");
        assert!(matches!(empty.payload(),V::Array(values) if values.get().is_empty()));
    }
    #[test]
    fn record_checks_unknown_fields_before_callbacks_and_keeps_duplicate_spec_order() {
        let pair = |name: &str, decoder: V| V::Tuple(Shared::new(vec![V::text(name), decoder]));
        let fields = V::Array(Shared::new(vec![
            pair("a", int()),
            pair("b", int()),
            pair("a", int()),
        ]));
        let result = decode("std/json::record", vec![fields], r#"{"b":2,"a":1}"#);
        assert_eq!(result.payload().index(0).index(0).render(), "a");
        assert_eq!(result.payload().index(1).index(1).render(), "2");
        assert_eq!(result.payload().index(2).index(0).render(), "a");
        let never = V::fun(vec![], |_, _| {
            panic!("unknown fields must be checked before any callback")
        });
        let fields = V::Array(Shared::new(vec![pair("required", never)]));
        error(
            &decode("std/json::record", vec![fields], r#"{"z":0,"a":1}"#),
            "std/json::UnknownJsonField",
            "z",
            &[],
        );
        error(
            &decode(
                "std/json::record",
                vec![V::Array(Shared::new(vec![]))],
                "[]",
            ),
            "std/json::ExpectedJsonType",
            "object",
            &[],
        );
    }
    #[test]
    fn alternatives_keep_last_failure_and_composition_uses_original_json() {
        let field = |name: &str| {
            native_json_decoder_operation("std/json::field", vec![V::text(name), int()])
        };
        error(
            &decode(
                "std/json::oneOf",
                vec![V::Array(Shared::new(vec![field("first"), field("second")]))],
                r#"{"second":false}"#,
            ),
            "std/json::ExpectedJsonType",
            "number",
            &[("std/json::JsonField", "second")],
        );
        error(
            &decode(
                "std/json::oneOf",
                vec![V::Array(Shared::new(vec![]))],
                "null",
            ),
            "std/json::InvalidJsonValue",
            "no JSON decoder matched",
            &[],
        );
        let never = V::fun(vec![], |_, _| panic!("failure must skip transform"));
        error(
            &decode("std/json::map", vec![never.clone(), int()], "false"),
            "std/json::ExpectedJsonType",
            "number",
            &[],
        );
        error(
            &decode("std/json::flatMap", vec![never, int()], "false"),
            "std/json::ExpectedJsonType",
            "number",
            &[],
        );
        let flag = native_json_decoder_operation(
            "std/json::field",
            vec![
                V::text("flag"),
                native_json_codec_dictionary("Bool", false).field("decodeJson"),
            ],
        );
        let choose = V::fun(vec![], |_, flag| {
            native_json_decoder_operation(
                "std/json::field",
                vec![
                    V::text(if flag.boolean() { "a" } else { "b" }),
                    native_json_codec_dictionary("Int", false).field("decodeJson"),
                ],
            )
        });
        let result = decode(
            "std/json::flatMap",
            vec![choose, flag],
            r#"{"flag":true,"a":7,"b":9}"#,
        );
        assert_eq!(result.payload().render(), "7");
    }
    #[test]
    fn deep_dynamic_composition_is_a_bounded_defect_and_restores_decoder_frames() {
        let mut decoder = int();
        for _ in 0..2048 {
            decoder = native_json_decoder_operation(
                "std/json::map",
                vec![V::fun(vec![], |_, value| value), decoder],
            );
        }
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || decoder.apply(json("42"))
        ))
        .is_err());
        assert_eq!(JSON_DECODER_DEPTH.with(Cell::get), 0);
        drop(decoder);
        assert_eq!(int().apply(json("42")).payload().render(), "42");
    }
}
