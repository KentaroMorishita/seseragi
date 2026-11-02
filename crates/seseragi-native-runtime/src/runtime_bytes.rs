fn native_bytes(value: &V) -> &[u8] {
    match value {
        V::Bytes(bytes) => bytes.get(),
        _ => panic!("Rust ABI: expected Bytes"),
    }
}

pub fn native_bytes_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/bytes::byte", [V::Int(value)]) => match u8::try_from(*value) {
            Ok(byte) => native_right(V::Byte(byte)),
            Err(_) => native_left(V::adt("std/bytes::ByteOutOfRange", V::Int(*value))),
        },
        ("std/bytes::empty", [V::Unit]) => V::Bytes(Shared::new(vec![])),
        ("std/bytes::singleton", [V::Byte(byte)]) => V::Bytes(Shared::new(vec![*byte])),
        ("std/bytes::fromArray", [V::Array(values)]) => V::Bytes(Shared::new(
            values
                .get()
                .iter()
                .map(|value| match value {
                    V::Byte(byte) => *byte,
                    _ => panic!("Rust ABI: expected Byte in Bytes.fromArray"),
                })
                .collect(),
        )),
        ("std/bytes::fromInts", [V::Array(values)]) => {
            let mut bytes = Vec::with_capacity(values.get().len());
            for value in values.get() {
                let V::Int(value) = value else {
                    panic!("Rust ABI: expected Int in Bytes.fromInts")
                };
                let Ok(byte) = u8::try_from(*value) else {
                    return native_left(V::adt("std/bytes::ByteOutOfRange", V::Int(*value)));
                };
                bytes.push(byte);
            }
            native_right(V::Bytes(Shared::new(bytes)))
        }
        ("std/bytes::toArray", [bytes]) => V::Array(Shared::new(
            native_bytes(bytes).iter().copied().map(V::Byte).collect(),
        )),
        ("std/bytes::toInts", [bytes]) => V::Array(Shared::new(
            native_bytes(bytes)
                .iter()
                .map(|byte| V::Int(i64::from(*byte)))
                .collect(),
        )),
        ("std/bytes::length", [bytes]) => {
            V::Int(i64::try_from(native_bytes(bytes).len()).expect("Bytes length fits Int"))
        }
        ("std/bytes::isEmpty", [bytes]) => V::Bool(native_bytes(bytes).is_empty()),
        ("std/bytes::get", [V::Int(index), bytes]) => {
            match usize::try_from(*index)
                .ok()
                .and_then(|i| native_bytes(bytes).get(i))
            {
                Some(byte) => V::adt("std/prelude::Just", V::Byte(*byte)),
                None => V::adt("std/prelude::Nothing", V::Unit),
            }
        }
        ("std/bytes::toInt", [V::Byte(byte)]) => V::Int(i64::from(*byte)),
        ("std/bytes::slice", [V::Int(start), V::Int(end), values]) => {
            let bytes = native_bytes(values);
            let length = i64::try_from(bytes.len()).expect("Bytes length fits Int");
            if *start < 0 || start > end || *end > length {
                return native_left(V::adt(
                    "std/bytes::InvalidByteRange",
                    V::Record(Shared::new(BTreeMap::from([
                        ("start".into(), V::Int(*start)),
                        ("end".into(), V::Int(*end)),
                        ("length".into(), V::Int(length)),
                    ]))),
                ));
            }
            native_right(V::Bytes(Shared::new(
                bytes[*start as usize..*end as usize].to_vec(),
            )))
        }
        ("std/bytes::copy", [bytes]) => V::Bytes(Shared::new(native_bytes(bytes).to_vec())),
        ("std/bytes::append", [suffix, values]) => {
            let (values, suffix) = (native_bytes(values), native_bytes(suffix));
            let mut result = Vec::with_capacity(values.len() + suffix.len());
            result.extend_from_slice(values);
            result.extend_from_slice(suffix);
            V::Bytes(Shared::new(result))
        }
        ("std/bytes::concat", [V::Array(values)]) => {
            let length = values
                .get()
                .iter()
                .map(|value| native_bytes(value).len())
                .sum();
            let mut result = Vec::with_capacity(length);
            for value in values.get() {
                result.extend_from_slice(native_bytes(value));
            }
            V::Bytes(Shared::new(result))
        }
        _ => panic!("Rust ABI: unsupported Bytes operation or arity"),
    }
}

#[cfg(test)]
mod bytes_tests {
    use super::*;

    #[test]
    fn byte_accepts_every_unsigned_value_and_preserves_out_of_range_ints() {
        clear_dictionaries();
        for value in 0..=255 {
            let result = native_bytes_operation("std/bytes::byte", vec![V::Int(value)]);
            assert!(result.tagged("std/prelude::Right"));
            assert!(matches!(result.payload(), V::Byte(byte) if i64::from(byte) == value));
        }
        for value in [-9_007_199_254_740_991, -1, 256, 9_007_199_254_740_991] {
            let result = native_bytes_operation("std/bytes::byte", vec![V::Int(value)]);
            assert!(result.tagged("std/prelude::Left"));
            let error = result.payload();
            assert!(error.tagged("std/bytes::ByteOutOfRange"));
            assert!(matches!(error.payload(), V::Int(actual) if actual == value));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn constructors_and_array_conversions_copy_storage_and_keep_opaque_bytes() {
        clear_dictionaries();
        let empty = native_bytes_operation("std/bytes::empty", vec![V::Unit]);
        let singleton = native_bytes_operation("std/bytes::singleton", vec![V::Byte(255)]);
        assert!(native_bytes(&empty).is_empty());
        assert_eq!(native_bytes(&singleton), [255]);
        assert!(matches!(
            native_bytes_operation("std/bytes::length", vec![empty.clone()]),
            V::Int(0)
        ));
        assert!(matches!(
            native_bytes_operation("std/bytes::isEmpty", vec![empty.clone()]),
            V::Bool(true)
        ));
        assert!(matches!(
            native_bytes_operation("std/bytes::isEmpty", vec![singleton.clone()]),
            V::Bool(false)
        ));
        for input in [vec![], (0..=255).collect::<Vec<u8>>()] {
            let array = V::Array(Shared::new(input.iter().copied().map(V::Byte).collect()));
            let bytes = native_bytes_operation("std/bytes::fromArray", vec![array.clone()]);
            let restored = native_bytes_operation("std/bytes::toArray", vec![bytes.clone()]);
            let V::Array(values) = &restored else {
                panic!()
            };
            assert_eq!(values.get().len(), input.len());
            for (value, expected) in values.get().iter().zip(&input) {
                assert!(matches!(value, V::Byte(actual) if actual == expected));
            }
            assert_ne!(restored.node_id(), array.node_id());
            let copy = native_bytes_operation("std/bytes::copy", vec![bytes.clone()]);
            assert_ne!(copy.node_id(), bytes.node_id());
            if !input.is_empty() {
                assert_ne!(native_bytes(&copy).as_ptr(), native_bytes(&bytes).as_ptr());
            }
            assert!(matches!(
                native_bytes_operation("std/bytes::length", vec![bytes.clone()]),
                V::Int(length) if length == input.len() as i64
            ));
            drop((array, restored, bytes));
            assert_eq!(native_bytes(&copy), input);
        }
        drop((empty, singleton));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn slice_checks_half_open_bounds_before_index_conversion_and_preserves_error_fields() {
        clear_dictionaries();
        let indices = [
            -9_007_199_254_740_991,
            -1,
            0,
            1,
            2,
            3,
            4,
            9_007_199_254_740_991,
        ];
        for input in [vec![], vec![0, 128, 255]] {
            let length = input.len() as i64;
            let bytes = V::Bytes(Shared::new(input.clone()));
            for start in indices {
                for end in indices {
                    let result = native_bytes_operation(
                        "std/bytes::slice",
                        vec![V::Int(start), V::Int(end), bytes.clone()],
                    );
                    if start >= 0 && start <= end && end <= length {
                        assert!(
                            result.tagged("std/prelude::Right"),
                            "{start}/{end}/{length}"
                        );
                        assert_eq!(
                            native_bytes(&result.payload()),
                            &input[start as usize..end as usize]
                        );
                    } else {
                        assert!(result.tagged("std/prelude::Left"), "{start}/{end}/{length}");
                        let error = result.payload();
                        assert!(error.tagged("std/bytes::InvalidByteRange"));
                        let range = error.payload();
                        for (field, expected) in
                            [("start", start), ("end", end), ("length", length)]
                        {
                            assert!(
                                matches!(range.field(field), V::Int(actual) if actual == expected)
                            );
                        }
                    }
                    assert_eq!(native_bytes(&bytes), input);
                }
            }
            let retained = native_bytes_operation(
                "std/bytes::slice",
                vec![V::Int(0), V::Int(length), bytes.clone()],
            );
            drop(bytes);
            assert_eq!(native_bytes(&retained.payload()), input);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn append_and_concat_preserve_suffix_order_empty_inputs_and_repeated_buffers() {
        clear_dictionaries();
        let empty = V::Bytes(Shared::new(vec![]));
        let first = V::Bytes(Shared::new(vec![0, 128]));
        let last = V::Bytes(Shared::new(vec![255, 1]));
        for (suffix, values, expected) in [
            (&last, &first, vec![0, 128, 255, 1]),
            (&empty, &first, vec![0, 128]),
            (&first, &empty, vec![0, 128]),
            (&empty, &empty, vec![]),
        ] {
            let result =
                native_bytes_operation("std/bytes::append", vec![suffix.clone(), values.clone()]);
            assert_eq!(native_bytes(&result), expected);
        }
        for (values, expected) in [
            (vec![], vec![]),
            (vec![empty.clone(), empty.clone()], vec![]),
            (vec![first.clone()], vec![0, 128]),
            (
                vec![
                    empty.clone(),
                    first.clone(),
                    empty.clone(),
                    last.clone(),
                    first.clone(),
                    empty.clone(),
                ],
                vec![0, 128, 255, 1, 0, 128],
            ),
        ] {
            let result =
                native_bytes_operation("std/bytes::concat", vec![V::Array(Shared::new(values))]);
            assert_eq!(native_bytes(&result), expected);
        }
        assert!(native_bytes(&empty).is_empty());
        assert_eq!(native_bytes(&first), [0, 128]);
        assert_eq!(native_bytes(&last), [255, 1]);
        let retained =
            native_bytes_operation("std/bytes::append", vec![last.clone(), first.clone()]);
        drop((empty, first, last));
        assert_eq!(native_bytes(&retained), [0, 128, 255, 1]);
        drop(retained);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn from_ints_copies_every_unsigned_byte_and_preserves_the_input_and_first_error() {
        clear_dictionaries();
        let input = V::Array(Shared::new((0..=255).map(V::Int).collect()));
        let result = native_bytes_operation("std/bytes::fromInts", vec![input.clone()]);
        assert!(result.tagged("std/prelude::Right"));
        assert_eq!(
            native_bytes(&result.payload()),
            (0..=255).collect::<Vec<u8>>()
        );
        let restored = native_bytes_operation("std/bytes::toInts", vec![result.payload()]);
        assert_eq!(restored.render(), input.render());
        drop((input, result, restored));
        for values in [vec![-1, 256], vec![0, 256, -1], vec![9_007_199_254_740_991]] {
            let first = *values
                .iter()
                .find(|value| u8::try_from(**value).is_err())
                .unwrap();
            let input = V::Array(Shared::new(values.into_iter().map(V::Int).collect()));
            let before = input.render();
            let result = native_bytes_operation("std/bytes::fromInts", vec![input.clone()]);
            assert!(result.tagged("std/prelude::Left"));
            let error = result.payload();
            assert!(error.tagged("std/bytes::ByteOutOfRange"));
            assert_eq!(error.payload().render(), first.to_string());
            assert_eq!(input.render(), before);
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn get_returns_owned_byte_values_and_rejects_negative_or_out_of_bounds_indices() {
        clear_dictionaries();
        let bytes = V::Bytes(Shared::new(vec![0, 128, 255]));
        for index in [-9_007_199_254_740_991, -1, 3, 9_007_199_254_740_991] {
            assert!(
                native_bytes_operation("std/bytes::get", vec![V::Int(index), bytes.clone()])
                    .tagged("std/prelude::Nothing")
            );
        }
        let selected = native_bytes_operation("std/bytes::get", vec![V::Int(2), bytes.clone()]);
        drop(bytes);
        assert_eq!(
            native_bytes_operation("std/bytes::toInt", vec![selected.payload()]).render(),
            "255"
        );
        drop(selected);
        let empty =
            native_bytes_operation("std/bytes::fromInts", vec![V::Array(Shared::new(vec![]))]);
        assert!(empty.tagged("std/prelude::Right"));
        assert!(native_bytes(&empty.payload()).is_empty());
        assert!(
            native_bytes_operation("std/bytes::get", vec![V::Int(0), empty.payload()])
                .tagged("std/prelude::Nothing")
        );
        drop(empty);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn tracked_bytes_are_leaf_nodes_and_leave_no_storage_in_a_ref_cycle() {
        enable_cycle_collection();
        let bytes = V::Bytes(Shared::new(vec![0, 128, 255]));
        let reference = V::reference(V::Unit);
        reference.write_reference(V::Tuple(Shared::new(vec![
            reference.clone(),
            bytes.clone(),
        ])));
        assert_eq!(LIVE.with(Cell::get), 3);
        drop((bytes, reference));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
