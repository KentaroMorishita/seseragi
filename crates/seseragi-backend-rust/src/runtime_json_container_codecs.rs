thread_local! { static JSON_CODEC_DEPTH: Cell<usize> = const { Cell::new(0) }; }
struct NativeJsonCodecFrame(usize);
impl NativeJsonCodecFrame {
    fn enter() -> Self {
        JSON_CODEC_DEPTH.with(|depth| {
            let previous = depth.get();
            if previous >= 128 {
                panic!("JSON codec nesting exceeds host limits");
            }
            depth.set(previous + 1);
            Self(previous)
        })
    }
}
impl Drop for NativeJsonCodecFrame {
    fn drop(&mut self) {
        JSON_CODEC_DEPTH.with(|depth| depth.set(self.0));
    }
}
fn native_json_codec_field(value: &V) -> (&str, bool) {
    let V::Tuple(pair) = value else {
        panic!("Rust ABI: JSON record field descriptor")
    };
    let [V::Text(name), V::Bool(optional)] = pair.get().as_slice() else {
        panic!("Rust ABI: JSON record field descriptor")
    };
    (name.get(), *optional)
}
fn native_json_container_encode(kind: &str, captures: &[V], value: V) -> V {
    let _frame = NativeJsonCodecFrame::enter();
    let children = &captures[1..];
    let encode = |index: usize, value| children[index].field("encodeJson").apply(value);
    let values = match kind {
        "Array" => {
            let V::Array(values) = value else {
                panic!("Rust ABI: JSON Array")
            };
            values.get().iter().cloned().map(|v| encode(0, v)).collect()
        }
        "List" => value
            .list_values()
            .into_iter()
            .map(|v| encode(0, v))
            .collect(),
        "Tuple" => {
            let V::Tuple(values) = value else {
                panic!("Rust ABI: JSON Tuple")
            };
            assert_eq!(
                values.get().len(),
                children.len(),
                "Rust ABI: JSON Tuple arity"
            );
            values
                .get()
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, v)| encode(i, v))
                .collect()
        }
        "Maybe" => {
            return if value.tagged("std/prelude::Nothing") {
                V::adt("std/json::JsonNull", V::Unit)
            } else {
                assert!(value.tagged("std/prelude::Just"));
                encode(0, value.payload())
            };
        }
        "Either" => {
            let left = value.tagged("std/prelude::Left");
            assert!(left || value.tagged("std/prelude::Right"));
            return native_json_object(vec![
                (
                    "tag".into(),
                    V::adt(
                        "std/json::JsonString",
                        V::text(if left { "Left" } else { "Right" }),
                    ),
                ),
                (
                    "value".into(),
                    encode(if left { 0 } else { 1 }, value.payload()),
                ),
            ]);
        }
        "Map" => {
            let V::Array(entries) = native_map_operation("std/map::entries", vec![], vec![value])
            else {
                unreachable!()
            };
            entries
                .get()
                .iter()
                .map(|pair| {
                    V::adt(
                        "std/json::JsonArray",
                        V::Array(Shared::new(vec![
                            encode(0, pair.index(0)),
                            encode(1, pair.index(1)),
                        ])),
                    )
                })
                .collect()
        }
        "Set" => {
            let V::Array(values) = native_set_operation("std/set::toArray", vec![], vec![value])
            else {
                unreachable!()
            };
            values.get().iter().cloned().map(|v| encode(0, v)).collect()
        }
        "Record" => {
            let V::Array(fields) = &captures[0] else {
                panic!("Rust ABI: JSON Record")
            };
            let values = value.record_fields();
            let mut entries = vec![];
            for (index, field) in fields.get().iter().enumerate() {
                let (name, optional) = native_json_codec_field(field);
                if optional && !values.contains_key(name) {
                    continue;
                }
                entries.push((
                    name.to_owned(),
                    encode(
                        index,
                        values
                            .get(name)
                            .expect("Rust ABI: required JSON Record field")
                            .clone(),
                    ),
                ));
            }
            return native_json_object(entries);
        }
        _ => panic!("Rust ABI: JSON container encoder"),
    };
    V::adt("std/json::JsonArray", V::Array(Shared::new(values)))
}
fn native_json_container_decode(kind: &str, captures: &[V], value: V) -> V {
    let _frame = NativeJsonCodecFrame::enter();
    let children = &captures[1..];
    let method = |index: usize| children[index].field("decodeJson");
    match kind {
        "Array" | "List" => {
            let decoded =
                native_json_decoder_operation("std/json::array", vec![method(0)]).apply(value);
            if kind == "Array" || decoded.tagged("std/prelude::Left") {
                return decoded;
            }
            let V::Array(values) = decoded.payload() else {
                unreachable!()
            };
            native_right(V::list_from_values(values.get().clone()))
        }
        "Maybe" => {
            if value.tagged("std/json::JsonNull") {
                return native_right(V::adt("std/prelude::Nothing", V::Unit));
            }
            let decoded = method(0).apply(value);
            if decoded.tagged("std/prelude::Left") {
                decoded
            } else {
                native_right(V::adt("std/prelude::Just", decoded.payload()))
            }
        }
        "Either" => {
            let tag = native_json_field_decoder(
                &[
                    V::text("tag"),
                    native_json_codec_dictionary("String", false).field("decodeJson"),
                ],
                value.clone(),
                false,
            );
            if tag.tagged("std/prelude::Left") {
                return tag;
            }
            let V::Text(tag) = tag.payload() else {
                unreachable!()
            };
            let (index, variant) = match tag.get().as_str() {
                "Left" => (0, "std/prelude::Left"),
                "Right" => (1, "std/prelude::Right"),
                _ => {
                    return native_json_prepend_path(
                        V::adt("std/json::JsonField", V::text("tag")),
                        native_json_decode_error("std/json::UnknownJsonTag", tag.get()),
                    )
                }
            };
            let decoded =
                native_json_field_decoder(&[V::text("value"), method(index)], value, false);
            if decoded.tagged("std/prelude::Left") {
                decoded
            } else {
                native_right(V::adt(variant, decoded.payload()))
            }
        }
        "Tuple" => {
            if !value.tagged("std/json::JsonArray") {
                return native_json_decode_error("std/json::ExpectedJsonType", "array");
            }
            let V::Array(values) = value.payload() else {
                unreachable!()
            };
            if values.get().len() != children.len() {
                return native_json_decode_error(
                    "std/json::InvalidJsonValue",
                    &format!("expected tuple length {}", children.len()),
                );
            }
            let mut output = vec![];
            for (index, value) in values.get().iter().cloned().enumerate() {
                let decoded = method(index).apply(value);
                if decoded.tagged("std/prelude::Left") {
                    return native_json_prepend_path(
                        V::adt(
                            "std/json::JsonIndex",
                            checked_int(i64::try_from(index).unwrap()),
                        ),
                        decoded,
                    );
                }
                output.push(decoded.payload());
            }
            native_right(V::Tuple(Shared::new(output)))
        }
        "Map" | "Set" => {
            let element = if kind == "Map" {
                native_json_container_dictionary(
                    "Tuple",
                    false,
                    vec![children[2].clone(), children[3].clone()],
                    vec![],
                )
                .field("decodeJson")
            } else {
                method(2)
            };
            // Decode the complete array before invoking any Eq/Hash method,
            // matching the strict TS factory even when a later element fails.
            let decoded =
                native_json_decoder_operation("std/json::array", vec![element]).apply(value);
            if decoded.tagged("std/prelude::Left") {
                return decoded;
            }
            let V::Array(values) = decoded.payload() else {
                unreachable!()
            };
            let dictionaries = vec![children[0].clone(), children[1].clone()];
            let mut output = native_map_operation("std/map::empty", vec![], vec![V::Unit]);
            for value in values.get() {
                let (key, value) = if kind == "Map" {
                    (value.index(0), value.index(1))
                } else {
                    (value.clone(), V::Unit)
                };
                output = native_map_operation(
                    "std/map::insert",
                    dictionaries.clone(),
                    vec![key, value, output],
                );
            }
            native_right(if kind == "Map" {
                output
            } else {
                native_set_from_map(output)
            })
        }
        "Record" => {
            if !value.tagged("std/json::JsonObject") {
                return native_json_decode_error("std/json::ExpectedJsonType", "object");
            }
            let V::Array(fields) = &captures[0] else {
                unreachable!()
            };
            let expected = fields
                .get()
                .iter()
                .map(|f| native_json_codec_field(f).0)
                .collect::<std::collections::BTreeSet<_>>();
            let payload = value.payload();
            let map = native_map_value(&payload);
            let mut id = map.first;
            while let Some(current) = id {
                let entry = native_map_entry(map, current);
                let V::Text(name) = &entry.key else {
                    unreachable!()
                };
                if !expected.contains(name.get().as_str()) {
                    return native_json_decode_error("std/json::UnknownJsonField", name.get());
                }
                id = entry.next;
            }
            let mut output = BTreeMap::new();
            for (index, field) in fields.get().iter().enumerate() {
                let (name, optional) = native_json_codec_field(field);
                let Some(selected) = native_json_field_value(&value, name) else {
                    if optional {
                        continue;
                    }
                    return native_json_decode_error("std/json::MissingJsonField", name);
                };
                let decoded = method(index).apply(selected);
                if decoded.tagged("std/prelude::Left") {
                    return native_json_prepend_path(
                        V::adt("std/json::JsonField", V::text(name)),
                        decoded,
                    );
                }
                output.insert(name.to_owned(), decoded.payload());
            }
            native_right(V::Record(Shared::new(output)))
        }
        _ => panic!("Rust ABI: JSON container decoder"),
    }
}
fn native_json_container_dictionary(
    kind: &'static str,
    encode: bool,
    children: Vec<V>,
    fields: Vec<(&str, bool)>,
) -> V {
    let metadata = V::Array(Shared::new(
        fields
            .into_iter()
            .map(|(name, optional)| V::Tuple(Shared::new(vec![V::text(name), V::Bool(optional)])))
            .collect(),
    ));
    let mut captures = vec![metadata];
    captures.extend(children);
    macro_rules! dictionary {
        ($kind:literal) => {
            if encode {
                V::dict(vec![(
                    "encodeJson",
                    V::fun(captures, |c, v| native_json_container_encode($kind, c, v)),
                )])
            } else {
                V::dict(vec![(
                    "decodeJson",
                    V::fun(captures, |c, v| native_json_container_decode($kind, c, v)),
                )])
            }
        };
    }
    match kind {
        "Array" => dictionary!("Array"),
        "List" => dictionary!("List"),
        "Maybe" => dictionary!("Maybe"),
        "Either" => dictionary!("Either"),
        "Tuple" => dictionary!("Tuple"),
        "Map" => dictionary!("Map"),
        "Set" => dictionary!("Set"),
        "Record" => dictionary!("Record"),
        _ => panic!("Rust ABI: JSON container factory"),
    }
}

#[cfg(test)]
mod json_container_codec_tests {
    use super::*;
    thread_local! { static EVENTS: RefCell<Vec<String>> = const { RefCell::new(vec![]) }; }
    fn events() -> Vec<String> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }
    fn parse(source: &str) -> V {
        let p = native_json_parse(source);
        assert!(p.tagged("std/prelude::Right"));
        p.payload()
    }
    fn codec(kind: &'static str, encode: bool, children: Vec<V>) -> V {
        native_json_container_dictionary(kind, encode, children, vec![])
    }
    fn primitive(encode: bool) -> V {
        native_json_codec_dictionary("Int", encode)
    }
    fn decode(dict: V, source: &str) -> V {
        dict.field("decodeJson").apply(parse(source))
    }
    fn wire(dict: V, value: V) -> String {
        native_json_stringify(dict.field("encodeJson").apply(value))
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
        for (part, (tag, payload)) in parts.get().iter().zip(path) {
            assert!(part.tagged(tag));
            assert_eq!(part.payload().render(), *payload);
        }
    }
    fn observed_decode() -> V {
        V::dict(vec![(
            "decodeJson",
            V::fun(vec![], |_, v| {
                EVENTS.with(|e| {
                    e.borrow_mut()
                        .push(format!("d{}", native_json_stringify(v.clone())))
                });
                native_json_codec_dictionary("Int", false)
                    .field("decodeJson")
                    .apply(v)
            }),
        )])
    }
    fn key_evidence() -> Vec<V> {
        vec![
            V::dict(vec![(
                "eq",
                V::fun(vec![], |_, left| {
                    V::fun(vec![left], |c, right| {
                        EVENTS.with(|e| {
                            e.borrow_mut()
                                .push(format!("e{}:{}", c[0].render(), right.render()))
                        });
                        V::Bool(c[0].render() == right.render())
                    })
                }),
            )]),
            V::dict(vec![(
                "hash",
                V::fun(vec![], |_, key| {
                    EVENTS.with(|e| e.borrow_mut().push(format!("h{}", key.render())));
                    V::Int(0)
                }),
            )]),
        ]
    }
    #[test]
    fn standard_containers_keep_nested_values_order_and_wire_conventions() {
        for (kind, source) in [
            ("Array", "[1,2,3]"),
            ("List", "[1,2,3]"),
            ("Maybe", "42"),
            ("Maybe", "null"),
        ] {
            let result = decode(codec(kind, false, vec![primitive(false)]), source);
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(
                wire(codec(kind, true, vec![primitive(true)]), result.payload()),
                source
            );
        }
        for source in [
            r#"{"tag":"Left","value":7}"#,
            r#"{"tag":"Right","value":42}"#,
        ] {
            let result = decode(
                codec("Either", false, vec![primitive(false), primitive(false)]),
                source,
            );
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(
                wire(
                    codec("Either", true, vec![primitive(true), primitive(true)]),
                    result.payload()
                ),
                source
            );
        }
        let result = decode(
            codec(
                "Tuple",
                false,
                vec![
                    primitive(false),
                    native_json_codec_dictionary("String", false),
                ],
            ),
            "[42,\"雪🙂\"]",
        );
        assert!(result.tagged("std/prelude::Right"));
        assert_eq!(
            wire(
                codec(
                    "Tuple",
                    true,
                    vec![
                        primitive(true),
                        native_json_codec_dictionary("String", true)
                    ]
                ),
                result.payload()
            ),
            "[42,\"雪🙂\"]"
        );
        let inner = codec("Array", false, vec![primitive(false)]);
        let decoded = decode(codec("Array", false, vec![inner]), "[[1],[2,3],[]]");
        assert!(decoded.tagged("std/prelude::Right"));
        let inner = codec("Array", true, vec![primitive(true)]);
        assert_eq!(
            wire(codec("Array", true, vec![inner]), decoded.payload()),
            "[[1],[2,3],[]]"
        );
    }
    #[test]
    fn paths_and_callback_short_circuit_match_standard_decoder_order() {
        events();
        let value = decode(
            codec("Tuple", false, vec![observed_decode(), observed_decode()]),
            "[1]",
        );
        error(
            &value,
            "std/json::InvalidJsonValue",
            "expected tuple length 2",
            &[],
        );
        assert!(events().is_empty());
        let value = decode(codec("Maybe", false, vec![observed_decode()]), "null");
        assert!(value.tagged("std/prelude::Right"));
        assert!(events().is_empty());
        let value = decode(
            codec(
                "Array",
                false,
                vec![codec(
                    "Tuple",
                    false,
                    vec![primitive(false), primitive(false)],
                )],
            ),
            "[[1,2],[3,\"bad\"],[4,5]]",
        );
        error(
            &value,
            "std/json::ExpectedJsonType",
            "number",
            &[("std/json::JsonIndex", "1"), ("std/json::JsonIndex", "1")],
        );
        let value = decode(
            codec("Either", false, vec![observed_decode(), observed_decode()]),
            r#"{"tag":"Other","value":1}"#,
        );
        error(
            &value,
            "std/json::UnknownJsonTag",
            "Other",
            &[("std/json::JsonField", "tag")],
        );
        assert!(events().is_empty());
        let value = decode(
            codec("Either", false, vec![observed_decode(), observed_decode()]),
            r#"{"tag":"Left","value":7,"ignored":0}"#,
        );
        assert!(value.tagged("std/prelude::Right"));
        assert_eq!(events(), ["d7"]);
        let value = decode(
            codec("Either", false, vec![primitive(false), primitive(false)]),
            r#"{"tag":"Right"}"#,
        );
        error(&value, "std/json::MissingJsonField", "value", &[]);
    }
    #[test]
    fn map_set_decode_all_elements_before_hash_and_keep_duplicate_representatives() {
        for kind in ["Map", "Set"] {
            events();
            let mut children = key_evidence();
            children.push(observed_decode());
            if kind == "Map" {
                children.push(observed_decode());
            }
            let source = if kind == "Map" {
                "[[1,10],[2,\"bad\"]]"
            } else {
                "[1,\"bad\"]"
            };
            let result = decode(codec(kind, false, children), source);
            assert!(result.tagged("std/prelude::Left"));
            assert!(events().iter().all(|e| e.starts_with('d')));
        }
        events();
        let mut children = key_evidence();
        children.extend([observed_decode(), observed_decode()]);
        let result = decode(codec("Map", false, children), "[[2,20],[1,10],[2,99]]");
        assert!(result.tagged("std/prelude::Right"));
        let order = events();
        assert!(order[..6].iter().all(|e| e.starts_with('d')));
        assert_eq!(order.iter().filter(|e| e.starts_with('h')).count(), 3);
        assert_eq!(
            wire(
                codec("Map", true, vec![primitive(true), primitive(true)]),
                result.payload()
            ),
            "[[2,99],[1,10]]"
        );
        let mut children = key_evidence();
        children.push(primitive(false));
        let result = decode(codec("Set", false, children), "[2,1,2]");
        assert!(result.tagged("std/prelude::Right"));
        assert_eq!(
            wire(codec("Set", true, vec![primitive(true)]), result.payload()),
            "[2,1]"
        );
    }
    #[test]
    fn records_preserve_sorted_factory_fields_optional_absence_and_unknown_first() {
        let record = |encode| {
            native_json_container_dictionary(
                "Record",
                encode,
                vec![primitive(encode), primitive(encode)],
                vec![("a", false), ("b", true)],
            )
        };
        for source in ["{\"a\":1}", "{\"a\":1,\"b\":2}"] {
            let result = decode(record(false), source);
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(wire(record(true), result.payload()), source);
        }
        let result = decode(record(false), "{\"b\":2}");
        error(&result, "std/json::MissingJsonField", "a", &[]);
        let result = decode(record(false), "{\"a\":1,\"b\":null}");
        error(
            &result,
            "std/json::ExpectedJsonType",
            "number",
            &[("std/json::JsonField", "b")],
        );
        events();
        let record = native_json_container_dictionary(
            "Record",
            false,
            vec![observed_decode()],
            vec![("a", false)],
        );
        let result = decode(record, "{\"z\":0,\"extra\":1}");
        error(&result, "std/json::UnknownJsonField", "z", &[]);
        assert!(events().is_empty());
        let source = V::Record(Shared::new(BTreeMap::from([
            ("a".into(), V::Int(42)),
            ("unselected".into(), V::text("drop")),
        ])));
        assert_eq!(wire(record_for_a(), source), "{\"a\":42}");
        fn record_for_a() -> V {
            native_json_container_dictionary(
                "Record",
                true,
                vec![primitive(true)],
                vec![("a", false)],
            )
        }
    }
    #[test]
    fn container_children_and_all_metadata_captures_participate_in_cycle_collection() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let child = V::dict(vec![(
            "encodeJson",
            V::fun(vec![cell.clone()], |_, v| {
                native_json_encode_primitive("Int", v)
            }),
        )]);
        let parent = codec("Array", true, vec![child]);
        cell.write_reference(parent.clone());
        drop(cell);
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(
            wire(parent.clone(), V::Array(Shared::new(vec![V::Int(42)]))),
            "[42]"
        );
        drop(parent);
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn dynamic_container_nesting_has_a_recoverable_limit_and_restores_the_frame() {
        let mut nested = primitive(true);
        let mut value = V::Int(42);
        for _ in 0..2048 {
            nested = codec("Array", true, vec![nested]);
            value = V::Array(Shared::new(vec![value]));
        }
        let failure =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wire(nested, value)));
        assert!(failure.is_err());
        assert_eq!(JSON_CODEC_DEPTH.with(Cell::get), 0);
        assert_eq!(
            wire(
                codec("Array", true, vec![primitive(true)]),
                V::Array(Shared::new(vec![V::Int(7)]))
            ),
            "[7]"
        );
    }
}
