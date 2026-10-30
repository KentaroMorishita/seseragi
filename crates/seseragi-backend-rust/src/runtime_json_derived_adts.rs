fn native_json_adt_case(value: &V) -> (&str, &'static str, &V) {
    let V::Tuple(fields) = value else {
        panic!("Rust ABI: JSON ADT descriptor")
    };
    let [V::Text(tag), V::Adt(constructor), thunk] = fields.get().as_slice() else {
        panic!("Rust ABI: JSON ADT descriptor")
    };
    (tag.get(), constructor.get().0, thunk)
}
fn native_json_derived_adt_encode(captures: &[V], value: V) -> V {
    let _frame = NativeJsonCodecFrame::enter();
    let V::Array(cases) = &captures[0] else {
        unreachable!()
    };
    let case = cases
        .get()
        .iter()
        .find(|case| value.tagged(native_json_adt_case(case).1))
        .expect("Rust ABI: unknown JSON ADT constructor");
    let (tag, _, thunk) = native_json_adt_case(case);
    let mut entries = vec![("tag".into(), V::adt("std/json::JsonString", V::text(tag)))];
    if !matches!(thunk, V::Unit) {
        entries.push((
            "value".into(),
            thunk
                .apply(V::Unit)
                .field("encodeJson")
                .apply(value.payload()),
        ));
    }
    native_json_object(entries)
}
fn native_json_derived_adt_decode(captures: &[V], value: V) -> V {
    let _frame = NativeJsonCodecFrame::enter();
    let decoded_tag = native_json_field_decoder(
        &[
            V::text("tag"),
            native_json_codec_dictionary("String", false).field("decodeJson"),
        ],
        value.clone(),
        false,
    );
    if decoded_tag.tagged("std/prelude::Left") {
        return decoded_tag;
    }
    let V::Text(tag) = decoded_tag.payload() else {
        unreachable!()
    };
    let V::Array(cases) = &captures[0] else {
        unreachable!()
    };
    let Some(case) = cases
        .get()
        .iter()
        .find(|case| native_json_adt_case(case).0 == tag.get())
    else {
        return native_json_prepend_path(
            V::adt("std/json::JsonField", V::text("tag")),
            native_json_decode_error("std/json::UnknownJsonTag", tag.get()),
        );
    };
    let (_, constructor, thunk) = native_json_adt_case(case);
    let payload = !matches!(thunk, V::Unit);
    let object = value.payload();
    let map = native_map_value(&object);
    let mut id = map.first;
    while let Some(current) = id {
        let entry = native_map_entry(map, current);
        let V::Text(name) = &entry.key else {
            unreachable!()
        };
        if name.get() != "tag" && !(payload && name.get() == "value") {
            return native_json_decode_error("std/json::UnknownJsonField", name.get());
        }
        id = entry.next;
    }
    if !payload {
        return native_right(V::adt(constructor, V::Unit));
    }
    let Some(value) = native_json_field_value(&value, "value") else {
        return native_json_decode_error("std/json::MissingJsonField", "value");
    };
    let decoded = thunk.apply(V::Unit).field("decodeJson").apply(value);
    if decoded.tagged("std/prelude::Left") {
        native_json_prepend_path(V::adt("std/json::JsonField", V::text("value")), decoded)
    } else {
        native_right(V::adt(constructor, decoded.payload()))
    }
}
fn native_json_derived_adt_dictionary(
    encode: bool,
    cases: Vec<(&str, &'static str, Option<V>)>,
) -> V {
    let metadata = V::Array(Shared::new(
        cases
            .into_iter()
            .map(|(tag, constructor, thunk)| {
                V::Tuple(Shared::new(vec![
                    V::text(tag),
                    V::adt(constructor, V::Unit),
                    thunk.unwrap_or(V::Unit),
                ]))
            })
            .collect(),
    ));
    if encode {
        V::dict(vec![(
            "encodeJson",
            V::fun(vec![metadata], native_json_derived_adt_encode),
        )])
    } else {
        V::dict(vec![(
            "decodeJson",
            V::fun(vec![metadata], native_json_derived_adt_decode),
        )])
    }
}

#[cfg(test)]
mod json_derived_adt_tests {
    use super::*;
    thread_local! { static EVENTS: RefCell<Vec<&'static str>> = const { RefCell::new(vec![]) }; }
    fn events() -> Vec<&'static str> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }
    fn thunk(encode: bool) -> V {
        V::fun(vec![V::Bool(encode)], |captures, _| {
            let V::Bool(encode) = captures[0] else {
                unreachable!()
            };
            native_json_codec_dictionary("Int", encode)
        })
    }
    fn observed_thunk() -> V {
        V::fun(vec![], |_, _| {
            EVENTS.with(|e| e.borrow_mut().push("factory"));
            V::dict(vec![(
                "decodeJson",
                V::fun(vec![], |_, value| {
                    EVENTS.with(|e| e.borrow_mut().push("decode"));
                    native_json_codec_dictionary("Int", false)
                        .field("decodeJson")
                        .apply(value)
                }),
            )])
        })
    }
    fn codec(encode: bool) -> V {
        native_json_derived_adt_dictionary(
            encode,
            vec![
                ("Empty", "x::Empty", None),
                (
                    "Loaded",
                    "x::Loaded",
                    Some(if encode {
                        thunk(true)
                    } else {
                        observed_thunk()
                    }),
                ),
            ],
        )
    }
    fn parse(source: &str) -> V {
        let value = native_json_parse(source);
        assert!(value.tagged("std/prelude::Right"));
        value.payload()
    }
    fn decode(dictionary: V, source: &str) -> V {
        dictionary.field("decodeJson").apply(parse(source))
    }
    fn wire(dictionary: V, value: V) -> String {
        native_json_stringify(dictionary.field("encodeJson").apply(value))
    }
    fn error(value: &V, kind: &str, message: &str, path: &[&str]) {
        assert!(value.tagged("std/prelude::Left"));
        let detail = value.payload();
        let actual = detail.field("kind");
        assert!(actual.tagged(kind));
        assert_eq!(actual.payload().render(), message);
        let V::Array(parts) = detail.field("path") else {
            panic!()
        };
        assert_eq!(parts.get().len(), path.len());
        for (part, expected) in parts.get().iter().zip(path) {
            assert!(part.tagged("std/json::JsonField"));
            assert_eq!(part.payload().render(), *expected);
        }
    }
    #[test]
    fn dictionaries_and_unselected_branches_are_lazy_and_wire_tags_are_local() {
        events();
        let decoder = codec(false);
        assert!(events().is_empty());
        let empty = decode(decoder.clone(), r#"{"tag":"Empty"}"#);
        assert!(empty.payload().tagged("x::Empty"));
        assert!(events().is_empty());
        let loaded = decode(decoder, r#"{"value":42,"tag":"Loaded"}"#);
        assert_eq!(events(), ["factory", "decode"]);
        assert!(loaded.payload().tagged("x::Loaded"));
        assert_eq!(wire(codec(true), empty.payload()), r#"{"tag":"Empty"}"#);
        assert_eq!(
            wire(codec(true), loaded.payload()),
            r#"{"tag":"Loaded","value":42}"#
        );
    }
    #[test]
    fn tag_unknown_field_and_payload_failures_preserve_priority_and_paths() {
        for (source, kind, message, path) in [
            ("false", "std/json::ExpectedJsonType", "object", vec![]),
            (
                r#"{"extra":1}"#,
                "std/json::MissingJsonField",
                "tag",
                vec![],
            ),
            (
                r#"{"tag":false,"extra":1}"#,
                "std/json::ExpectedJsonType",
                "JsonString",
                vec!["tag"],
            ),
            (
                r#"{"tag":"Unknown","extra":1}"#,
                "std/json::UnknownJsonTag",
                "Unknown",
                vec!["tag"],
            ),
            (
                r#"{"tag":"Empty","value":null}"#,
                "std/json::UnknownJsonField",
                "value",
                vec![],
            ),
            (
                r#"{"tag":"Loaded","z":1,"a":2}"#,
                "std/json::UnknownJsonField",
                "z",
                vec![],
            ),
            (
                r#"{"tag":"Loaded"}"#,
                "std/json::MissingJsonField",
                "value",
                vec![],
            ),
            (
                r#"{"tag":"Loaded","value":false}"#,
                "std/json::ExpectedJsonType",
                "number",
                vec!["value"],
            ),
        ] {
            events();
            let result = decode(codec(false), source);
            error(&result, kind, message, &path);
            assert_eq!(
                events(),
                if source.ends_with("false}") {
                    vec!["factory", "decode"]
                } else {
                    vec![]
                }
            );
        }
    }
    #[test]
    fn payload_thunks_and_metadata_keep_and_then_reclaim_captured_reference_cycles() {
        clear_dictionaries();
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let child = V::fun(vec![cell.clone()], |_, _| {
            native_json_codec_dictionary("Int", true)
        });
        let parent =
            native_json_derived_adt_dictionary(true, vec![("Loaded", "x::Loaded", Some(child))]);
        cell.write_reference(parent.clone());
        drop(cell);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(
            wire(parent.clone(), V::adt("x::Loaded", V::Int(42))),
            r#"{"tag":"Loaded","value":42}"#
        );
        drop(parent);
        clear_dictionaries();
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn recursive_adt_dictionaries_bound_frames_and_restore_them_after_unwinding() {
        clear_dictionaries();
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let child = V::fun(vec![cell.clone()], |captures, _| {
            captures[0].read_reference()
        });
        let parent = native_json_derived_adt_dictionary(
            true,
            vec![("End", "x::End", None), ("Next", "x::Next", Some(child))],
        );
        cell.write_reference(parent.clone());
        let mut value = V::adt("x::End", V::Unit);
        for _ in 0..2048 {
            value = V::adt("x::Next", value);
        }
        let failed =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wire(parent.clone(), value)));
        assert!(failed.is_err());
        assert_eq!(JSON_CODEC_DEPTH.with(Cell::get), 0);
        assert_eq!(
            wire(parent.clone(), V::adt("x::End", V::Unit)),
            r#"{"tag":"End"}"#
        );
        drop(parent);
        drop(cell);
        clear_dictionaries();
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
