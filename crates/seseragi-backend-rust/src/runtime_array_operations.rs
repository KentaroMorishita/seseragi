fn native_array_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/array::get", [offset, values]) => native_array_index(values.clone(), offset.clone()),
        ("std/array::length", [V::Array(values)]) => {
            V::Int(i64::try_from(values.get().len()).expect("Array length fits Int"))
        }
        ("std/array::take" | "std/array::drop", [V::Int(count), V::Array(values)]) => {
            let count = if *count <= 0 {
                0
            } else {
                usize::try_from(*count)
                    .unwrap_or(usize::MAX)
                    .min(values.get().len())
            };
            let slice = if name == "std/array::take" {
                &values.get()[..count]
            } else {
                &values.get()[count..]
            };
            V::Array(Shared::new(slice.to_vec()))
        }
        ("std/array::filter" | "std/array::find", [predicate, V::Array(values)]) => {
            let mut selected = Vec::new();
            // Immutable Array edges may be read across user code. No Ref or
            // collector borrow is held while calling the selected predicate.
            for value in values.get() {
                let V::Bool(keep) = predicate.apply(value.clone()) else {
                    panic!("Rust ABI: Array predicate must return Bool")
                };
                if keep {
                    if name == "std/array::find" {
                        return V::adt("std/prelude::Just", value.clone());
                    }
                    selected.push(value.clone());
                }
            }
            if name == "std/array::find" {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::Array(Shared::new(selected))
            }
        }
        _ => panic!("Rust ABI: unsupported Array operation or arity"),
    }
}

#[cfg(test)]
mod array_operation_tests {
    use super::*;

    #[test]
    fn predicates_run_in_order_find_short_circuits_and_empty_inputs_skip_user_code() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let predicate = V::fun(vec![calls.clone()], |capture, value| {
            let V::Int(previous) = capture[0].read_reference() else {
                panic!()
            };
            let V::Int(value) = value else { panic!() };
            capture[0].write_reference(V::Int(previous * 10 + value));
            assert_ne!(value, 3, "find must not reach this defect");
            V::Bool(value == 2)
        });
        let values = V::Array(Shared::new(vec![V::Int(1), V::Int(2), V::Int(3)]));
        let found =
            native_array_operation("std/array::find", vec![predicate.clone(), values.clone()]);
        assert_eq!(found.payload().render(), "2");
        assert_eq!(calls.read_reference().render(), "12");
        calls.write_reference(V::Int(0));
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            native_array_operation("std/array::filter", vec![predicate.clone(), values.clone()])
        }));
        assert!(failed.is_err());
        assert_eq!(calls.read_reference().render(), "123");
        for name in ["std/array::filter", "std/array::find"] {
            let empty = native_array_operation(
                name,
                vec![predicate.clone(), V::Array(Shared::new(vec![]))],
            );
            if name.ends_with("filter") {
                let V::Array(values) = empty else { panic!() };
                assert!(values.get().is_empty());
            } else {
                assert!(empty.tagged("std/prelude::Nothing"));
            }
        }
        assert_eq!(calls.read_reference().render(), "123");
        assert_eq!(values.render(), "[1, 2, 3]");
        drop((calls, predicate, values, found));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn counts_and_indices_clamp_without_overflow_and_copies_preserve_inputs() {
        let input = V::Array(Shared::new(vec![V::Int(1), V::Int(2), V::Int(3)]));
        for (count, take, drop) in [
            (i64::MIN, "[]", "[1, 2, 3]"),
            (-1, "[]", "[1, 2, 3]"),
            (0, "[]", "[1, 2, 3]"),
            (2, "[1, 2]", "[3]"),
            (3, "[1, 2, 3]", "[]"),
            (i64::MAX, "[1, 2, 3]", "[]"),
        ] {
            for (name, expected) in [("std/array::take", take), ("std/array::drop", drop)] {
                let output = native_array_operation(name, vec![V::Int(count), input.clone()]);
                assert_eq!(output.render(), expected);
                assert_ne!(input.node_id(), output.node_id());
            }
        }
        for offset in [i64::MIN, -1, 3, i64::MAX] {
            assert!(
                native_array_operation("std/array::get", vec![V::Int(offset), input.clone()])
                    .tagged("std/prelude::Nothing")
            );
        }
        assert_eq!(
            native_array_operation("std/array::length", vec![input.clone()]).render(),
            "3"
        );
        assert_eq!(input.render(), "[1, 2, 3]");
        drop(input);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn selected_callbacks_keep_cyclic_captures_until_the_last_array_edge_is_dropped() {
        enable_cycle_collection();
        let cell = V::reference(V::Int(41));
        let callback = V::fun(vec![cell.clone()], |captures, _| {
            captures[0].read_reference()
        });
        let input = V::Array(Shared::new(vec![callback.clone(), callback.clone()]));
        let predicate = V::fun(vec![], |_, _| V::Bool(true));
        let selected = native_array_operation("std/array::filter", vec![predicate, input.clone()]);
        let first =
            native_array_operation("std/array::get", vec![V::Int(0), selected.clone()]).payload();
        drop((input, callback));
        assert_eq!(first.apply(V::Unit).render(), "41");
        cell.write_reference(selected.clone());
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((cell, first, selected));
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
