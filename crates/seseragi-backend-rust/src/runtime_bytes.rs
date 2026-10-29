fn native_bytes(value: &V) -> &[u8] {
    match value {
        V::Bytes(bytes) => bytes.get(),
        _ => panic!("Rust ABI: expected Bytes"),
    }
}

fn native_bytes_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
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
        ("std/bytes::toInts", [bytes]) => V::Array(Shared::new(
            native_bytes(bytes)
                .iter()
                .map(|byte| V::Int(i64::from(*byte)))
                .collect(),
        )),
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
        _ => panic!("Rust ABI: unsupported Bytes operation or arity"),
    }
}

#[cfg(test)]
mod bytes_tests {
    use super::*;

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
