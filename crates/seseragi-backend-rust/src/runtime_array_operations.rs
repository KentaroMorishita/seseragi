fn native_array_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/array::empty", [V::Unit]) => V::Array(Shared::new(vec![])),
        ("std/array::singleton", [value]) => V::Array(Shared::new(vec![value.clone()])),
        ("std/array::append", [V::Array(suffix), V::Array(values)]) => V::Array(Shared::new(
            values.get().iter().chain(suffix.get()).cloned().collect(),
        )),
        ("std/array::reverse", [V::Array(values)]) => {
            V::Array(Shared::new(values.get().iter().rev().cloned().collect()))
        }
        ("std/array::isEmpty", [V::Array(values)]) => V::Bool(values.get().is_empty()),
        ("std/array::head", [V::Array(values)]) => match values.get().first() {
            Some(value) => V::adt("std/prelude::Just", value.clone()),
            None => V::adt("std/prelude::Nothing", V::Unit),
        },
        ("std/array::tail", [V::Array(values)]) => {
            if values.get().is_empty() {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::adt(
                    "std/prelude::Just",
                    V::Array(Shared::new(values.get()[1..].to_vec())),
                )
            }
        }
        ("std/array::filterMap" | "std/array::flatMap", [callback, V::Array(values)]) => {
            let mut output = Vec::new();
            // Array edges are immutable. Invoke user code without holding a
            // mutable reference or collector borrow, and retain each result.
            for value in values.get() {
                let mapped = callback.apply(value.clone());
                if name == "std/array::flatMap" {
                    let V::Array(mapped) = mapped else {
                        panic!("Rust ABI: Array flatMap callback must return Array")
                    };
                    output.extend(mapped.get().iter().cloned());
                } else if mapped.tagged("std/prelude::Just") {
                    output.push(mapped.payload());
                } else {
                    assert!(
                        mapped.tagged("std/prelude::Nothing"),
                        "Rust ABI: Array filterMap callback must return Maybe"
                    );
                }
            }
            V::Array(Shared::new(output))
        }
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

#[cfg(test)]
mod array_rest_tests {
    use super::*;
    fn array(numbers: &[i64]) -> V {
        V::Array(Shared::new(numbers.iter().map(|n| V::Int(*n)).collect()))
    }
    fn elements(value: &V) -> Vec<String> {
        let V::Array(values) = value else { panic!() };
        values.get().iter().map(V::render).collect()
    }
    #[test]
    fn constructors_append_reverse_head_tail_preserve_inputs_and_empty_boundaries() {
        let values = array(&[1, 2, 3]);
        let suffix = array(&[4, 5]);
        let empty = native_array_operation("std/array::empty", vec![V::Unit]);
        assert_eq!(
            native_array_operation("std/array::isEmpty", vec![empty.clone()]).render(),
            "True"
        );
        assert_eq!(
            native_array_operation("std/array::isEmpty", vec![values.clone()]).render(),
            "False"
        );
        let appended =
            native_array_operation("std/array::append", vec![suffix.clone(), values.clone()]);
        assert_eq!(elements(&appended), ["1", "2", "3", "4", "5"]);
        assert_ne!(appended.node_id(), values.node_id());
        assert_eq!(
            elements(&native_array_operation(
                "std/array::reverse",
                vec![values.clone()]
            )),
            ["3", "2", "1"]
        );
        assert_eq!(elements(&values), ["1", "2", "3"]);
        assert_eq!(elements(&suffix), ["4", "5"]);
        assert_eq!(
            native_array_operation("std/array::head", vec![values.clone()])
                .payload()
                .render(),
            "1"
        );
        assert_eq!(
            elements(&native_array_operation("std/array::tail", vec![values]).payload()),
            ["2", "3"]
        );
        for name in ["std/array::head", "std/array::tail"] {
            assert!(
                native_array_operation(name, vec![empty.clone()]).tagged("std/prelude::Nothing")
            );
        }
        let singleton = native_array_operation(
            "std/array::singleton",
            vec![V::Text(Shared::new("雪🙂".into()))],
        );
        assert_eq!(
            native_array_operation("std/array::head", vec![singleton.clone()])
                .payload()
                .render(),
            "雪🙂"
        );
        let tail = native_array_operation("std/array::tail", vec![singleton]);
        assert!(tail.tagged("std/prelude::Just"));
        assert!(elements(&tail.payload()).is_empty());
    }
    #[test]
    fn transformations_keep_callback_order_output_types_empty_skips_and_reentry() {
        enable_cycle_collection();
        let input = array(&[1, 2, 3]);
        let calls = V::reference(V::Int(0));
        let keep = V::fun(vec![calls.clone()], |captures, value| {
            let V::Int(number) = value else { panic!() };
            let V::Int(previous) = captures[0].read_reference() else {
                panic!()
            };
            captures[0].write_reference(V::Int(previous * 10 + number));
            if number % 2 == 0 {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::adt(
                    "std/prelude::Just",
                    V::Text(Shared::new(number.to_string())),
                )
            }
        });
        assert_eq!(
            elements(&native_array_operation(
                "std/array::filterMap",
                vec![keep.clone(), input.clone()]
            )),
            ["1", "3"]
        );
        assert_eq!(calls.read_reference().render(), "123");
        calls.write_reference(V::Int(0));
        let expand = V::fun(vec![calls.clone()], |captures, value| {
            let V::Int(number) = value else { panic!() };
            let V::Int(previous) = captures[0].read_reference() else {
                panic!()
            };
            captures[0].write_reference(V::Int(previous * 10 + number));
            V::Array(Shared::new(vec![
                V::Text(Shared::new(number.to_string())),
                V::Text(Shared::new((number * 10).to_string())),
            ]))
        });
        assert_eq!(
            elements(&native_array_operation(
                "std/array::flatMap",
                vec![expand.clone(), input.clone()]
            )),
            ["1", "10", "2", "20", "3", "30"]
        );
        assert_eq!(calls.read_reference().render(), "123");
        calls.write_reference(V::Int(0));
        for (name, callback) in [
            ("std/array::filterMap", keep),
            ("std/array::flatMap", expand),
        ] {
            assert!(elements(&native_array_operation(name, vec![callback, array(&[])])).is_empty());
        }
        assert_eq!(calls.read_reference().render(), "0");
        let reenter = V::fun(vec![input.clone()], |captures, _| {
            native_array_operation("std/array::reverse", vec![captures[0].clone()])
        });
        assert_eq!(
            elements(&native_array_operation(
                "std/array::flatMap",
                vec![reenter, input.clone()]
            )),
            ["3", "2", "1", "3", "2", "1", "3", "2", "1"]
        );
        assert_eq!(elements(&input), ["1", "2", "3"]);
    }
    #[test]
    fn callback_defects_stop_at_the_first_failure_and_release_saved_frames() {
        enable_cycle_collection();
        let input = array(&[1, 2, 3, 4]);
        let calls = V::reference(V::Int(0));
        for name in ["std/array::filterMap", "std/array::flatMap"] {
            calls.write_reference(V::Int(0));
            let callback = V::fun(
                vec![calls.clone(), V::Bool(name == "std/array::flatMap")],
                |captures, value| {
                    let V::Int(number) = value else { panic!() };
                    let V::Int(previous) = captures[0].read_reference() else {
                        panic!()
                    };
                    captures[0].write_reference(V::Int(previous * 10 + number));
                    assert_ne!(number, 3, "callback stop");
                    if matches!(captures[1], V::Bool(true)) {
                        V::Array(Shared::new(vec![V::Int(number)]))
                    } else {
                        V::adt("std/prelude::Just", V::Int(number))
                    }
                },
            );
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_array_operation(name, vec![callback, input.clone()])
            }));
            assert!(result.is_err());
            assert_eq!(calls.read_reference().render(), "123");
        }
        assert_eq!(elements(&input), ["1", "2", "3", "4"]);
        drop((input, calls));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn constructed_arrays_retain_callbacks_and_collect_reference_cycles() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let callback = V::fun(vec![cell.clone()], |captures, _| {
            native_array_operation("std/array::isEmpty", vec![captures[0].read_reference()])
        });
        let values = native_array_operation("std/array::singleton", vec![callback.clone()]);
        cell.write_reference(values.clone());
        let head = native_array_operation("std/array::head", vec![values.clone()]).payload();
        assert_eq!(head.apply(V::Unit).render(), "False");
        let reversed = native_array_operation("std/array::reverse", vec![values.clone()]);
        drop((callback, cell, values, head, reversed));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
