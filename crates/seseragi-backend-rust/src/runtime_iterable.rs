fn native_iterable_map_cursor(source: V, keys_only: bool) -> V {
    let initial = native_map_value(&source)
        .first
        .map_or(V::Unit, |id| V::Int(id as i64));
    native_iterator_unfold(
        V::fun(vec![source, V::Bool(keys_only)], |captures, state| {
            if matches!(state, V::Unit) {
                return V::adt("std/prelude::Nothing", V::Unit);
            }
            let entry =
                native_map_entry(native_map_value(&captures[0]), native_map_address(&state));
            let value = if captures[1].boolean() {
                entry.key.clone()
            } else {
                V::Tuple(Shared::new(vec![entry.key.clone(), entry.value.clone()]))
            };
            let next = entry.next.map_or(V::Unit, |id| V::Int(id as i64));
            V::adt(
                "std/prelude::Just",
                V::Tuple(Shared::new(vec![value, next])),
            )
        }),
        initial,
    )
}

fn native_iterable_dictionary(identity: &'static str) -> V {
    cached(identity, || {
        let iterate: fn(&[V], V) -> V = match identity {
            "std/array::Iterable" => |_, source| {
                native_iterator_unfold(
                    V::fun(vec![source], |captures, position| {
                        let V::Array(values) = &captures[0] else {
                            panic!("Rust ABI: Iterable Array")
                        };
                        let index = native_map_address(&position) as usize;
                        match values.get().get(index) {
                            None => V::adt("std/prelude::Nothing", V::Unit),
                            Some(value) => V::adt(
                                "std/prelude::Just",
                                V::Tuple(Shared::new(vec![
                                    value.clone(),
                                    V::Int((index + 1) as i64),
                                ])),
                            ),
                        }
                    }),
                    V::Int(0),
                )
            },
            "std/list::Iterable" => |_, source| {
                native_iterator_unfold(
                    V::fun(vec![], |_, tail| match tail.list_cell() {
                        None => V::adt("std/prelude::Nothing", V::Unit),
                        Some(cell) => V::adt(
                            "std/prelude::Just",
                            V::Tuple(Shared::new(vec![cell.head.clone(), cell.tail.clone()])),
                        ),
                    }),
                    source,
                )
            },
            "std/iterator::Iterable" => |_, source| source,
            "std/map::Iterable" => |_, source| native_iterable_map_cursor(source, false),
            "std/set::Iterable" => {
                |_, source| native_iterable_map_cursor(native_set_into_map(source), true)
            }
            _ => panic!("Rust ABI: unproven Iterable provider"),
        };
        V::dict(vec![("iterate", V::fun(vec![], iterate))])
    })
}

fn native_iterable_build(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    let [source]: [V; 1] = arguments
        .try_into()
        .unwrap_or_else(|_| panic!("Rust ABI: Iterable constructor source arity"));
    let keyed = matches!(name, "std/map::fromEntries" | "std/set::fromIterable");
    assert_eq!(
        dictionaries.len(),
        if keyed { 3 } else { 1 },
        "Rust ABI: Iterable constructor dictionaries"
    );
    // Keyed TS constructors create the empty Map/Set (including its seed)
    // before invoking iterate. Array/List never acquire a process hash seed.
    let mut keyed_result = match name {
        "std/map::fromEntries" => Some(native_map_operation(
            "std/map::empty",
            vec![],
            vec![V::Unit],
        )),
        "std/set::fromIterable" => Some(native_set_operation(
            "std/set::empty",
            vec![],
            vec![V::Unit],
        )),
        "std/array::fromIterable" | "std/list::fromIterable" => None,
        _ => panic!("Rust ABI: Iterable constructor"),
    };
    let mut iterator = dictionaries[0].field("iterate").apply(source);
    let mut output = Vec::new();
    loop {
        let result = native_iterator_operation("std/iterator::next", vec![iterator]);
        if result.tagged("std/prelude::Nothing") {
            break;
        }
        assert!(
            result.tagged("std/prelude::Just"),
            "Rust ABI: Iterable step Maybe"
        );
        let V::Tuple(pair) = result.payload() else {
            panic!("Rust ABI: Iterable step pair")
        };
        let [value, rest] = pair.get().as_slice() else {
            panic!("Rust ABI: Iterable step pair arity")
        };
        if name == "std/map::fromEntries" {
            let V::Tuple(entry) = value else {
                panic!("Rust ABI: Map entry pair")
            };
            let [key, value] = entry.get().as_slice() else {
                panic!("Rust ABI: Map entry pair arity")
            };
            keyed_result = Some(native_map_operation(
                "std/map::insert",
                dictionaries[1..].to_vec(),
                vec![key.clone(), value.clone(), keyed_result.take().unwrap()],
            ));
        } else if name == "std/set::fromIterable" {
            keyed_result = Some(native_set_operation(
                "std/set::insert",
                dictionaries[1..].to_vec(),
                vec![value.clone(), keyed_result.take().unwrap()],
            ));
        } else {
            output.push(value.clone());
        }
        iterator = rest.clone();
    }
    keyed_result
        .unwrap_or_else(|| native_sequence_collect(output, name == "std/list::fromIterable"))
}

#[cfg(test)]
mod iterable_tests {
    use super::*;
    fn array(values: Vec<V>) -> V {
        V::Array(Shared::new(values))
    }
    fn ints(values: &[i64]) -> V {
        array(values.iter().copied().map(V::Int).collect())
    }
    fn values(value: &V, list: bool) -> Vec<String> {
        native_sequence_iter(value, list).map(V::render).collect()
    }
    fn increment(counter: &V) {
        let V::Int(count) = counter.read_reference() else {
            panic!()
        };
        counter.write_reference(V::Int(count + 1));
    }
    fn selected_ints(iterable: V) -> Vec<V> {
        vec![
            iterable,
            native_operator_dictionary("std/int::Eq"),
            native_hash_dictionary("std/int::Hash"),
        ]
    }

    #[test]
    fn arbitrary_iterable_is_selected_once_steps_include_one_terminal_and_keyed_results_keep_first_order(
    ) {
        enable_cycle_collection();
        for operation in [
            "std/array::fromIterable",
            "std/list::fromIterable",
            "std/set::fromIterable",
            "std/map::fromEntries",
        ] {
            let iterates = V::reference(V::Int(0));
            let steps = V::reference(V::Int(0));
            let dictionary = V::dict(vec![(
                "iterate",
                V::fun(vec![iterates.clone(), steps.clone()], |c, source| {
                    increment(&c[0]);
                    native_iterator_unfold(
                        V::fun(vec![source.field("items"), c[1].clone()], |c, position| {
                            increment(&c[1]);
                            let V::Array(values) = &c[0] else { panic!() };
                            let index = native_map_address(&position) as usize;
                            values.get().get(index).map_or_else(
                                || V::adt("std/prelude::Nothing", V::Unit),
                                |value| {
                                    V::adt(
                                        "std/prelude::Just",
                                        V::Tuple(Shared::new(vec![
                                            value.clone(),
                                            V::Int(index as i64 + 1),
                                        ])),
                                    )
                                },
                            )
                        }),
                        V::Int(0),
                    )
                }),
            )]);
            let items = if operation.ends_with("fromEntries") {
                array(
                    [(3, "first"), (1, "second"), (3, "replacement")]
                        .map(|(key, value)| {
                            V::Tuple(Shared::new(vec![V::Int(key), V::text(value)]))
                        })
                        .to_vec(),
                )
            } else {
                ints(&[3, 1, 3])
            };
            let source = V::dict(vec![("items", items)]);
            let dictionaries =
                if operation.starts_with("std/map") || operation.starts_with("std/set") {
                    selected_ints(dictionary.clone())
                } else {
                    vec![dictionary.clone()]
                };
            let result = native_iterable_build(operation, dictionaries, vec![source.clone()]);
            assert_eq!(iterates.read_reference().render(), "1");
            assert_eq!(steps.read_reference().render(), "4");
            if operation.starts_with("std/map") {
                let map = native_map_value(&result);
                assert_eq!(native_map_index_size(&map.order), 2);
                let first = native_map_entry(map, map.first.unwrap());
                assert_eq!(first.key.render(), "3");
                assert_eq!(first.value.render(), "replacement");
                assert_eq!(native_map_entry(map, first.next.unwrap()).key.render(), "1");
            } else if operation.starts_with("std/set") {
                let collected =
                    native_set_operation("std/set::toArray", vec![], vec![result.clone()]);
                assert_eq!(values(&collected, false), ["3", "1"]);
            } else {
                assert_eq!(
                    values(&result, operation.starts_with("std/list")),
                    ["3", "1", "3"]
                );
            }
            drop((iterates, steps, dictionary, source, result));
            clear_dictionaries();
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn admitted_standard_iterables_keep_their_order_and_release_source_container_nodes() {
        enable_cycle_collection();
        let input = ints(&[3, 1, 3]);
        let list = V::list_from_values(vec![V::Int(3), V::Int(1), V::Int(3)]);
        let set = native_iterable_build(
            "std/set::fromIterable",
            selected_ints(native_iterable_dictionary("std/array::Iterable")),
            vec![input.clone()],
        );
        let map = native_map_operation(
            "std/map::singleton",
            vec![
                native_operator_dictionary("std/int::Eq"),
                native_hash_dictionary("std/int::Hash"),
            ],
            vec![V::Int(7), V::text("雪")],
        );
        let map_id = map.node_id().unwrap();
        let iterator = native_iterable_dictionary("std/list::Iterable")
            .field("iterate")
            .apply(list.clone());
        for (identity, source, expected) in [
            ("std/array::Iterable", input.clone(), vec!["3", "1", "3"]),
            ("std/list::Iterable", list.clone(), vec!["3", "1", "3"]),
            (
                "std/iterator::Iterable",
                iterator.clone(),
                vec!["3", "1", "3"],
            ),
            ("std/set::Iterable", set.clone(), vec!["3", "1"]),
        ] {
            let result = native_iterable_build(
                "std/array::fromIterable",
                vec![native_iterable_dictionary(identity)],
                vec![source],
            );
            assert_eq!(values(&result, false), expected);
        }
        let entries = native_iterable_build(
            "std/array::fromIterable",
            vec![native_iterable_dictionary("std/map::Iterable")],
            vec![map.clone()],
        );
        assert_eq!(values(&entries, false), ["(7, 雪)"]);
        drop((input, list, set, map, iterator));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&map_id)));
        assert_eq!(values(&entries, false), ["(7, 雪)"]);
        drop(entries);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn a_step_defect_drops_partial_collections_and_traced_iterable_state_cycles() {
        enable_cycle_collection();
        for operation in [
            "std/array::fromIterable",
            "std/list::fromIterable",
            "std/set::fromIterable",
            "std/map::fromEntries",
        ] {
            let cell = V::reference(V::Unit);
            let calls = V::reference(V::Int(0));
            let dictionary = V::dict(vec![(
                "iterate",
                V::fun(
                    vec![
                        cell.clone(),
                        calls.clone(),
                        V::Bool(operation.ends_with("fromEntries")),
                    ],
                    |c, _| {
                        native_iterator_unfold(
                            V::fun(c.to_vec(), |c, state| {
                                increment(&c[1]);
                                let V::Int(n) = state else { panic!() };
                                assert_ne!(n, 2, "deliberate step defect");
                                let value = if c[2].boolean() {
                                    V::Tuple(Shared::new(vec![V::Int(n), c[0].clone()]))
                                } else {
                                    V::Int(n)
                                };
                                V::adt(
                                    "std/prelude::Just",
                                    V::Tuple(Shared::new(vec![value, V::Int(n + 1)])),
                                )
                            }),
                            V::Int(0),
                        )
                    },
                ),
            )]);
            cell.write_reference(dictionary.clone());
            let dictionaries =
                if operation.starts_with("std/map") || operation.starts_with("std/set") {
                    selected_ints(dictionary.clone())
                } else {
                    vec![dictionary.clone()]
                };
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_iterable_build(operation, dictionaries, vec![V::Unit])
            }))
            .is_err());
            assert_eq!(calls.read_reference().render(), "3");
            drop((cell, calls, dictionary));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn large_list_constructor_consumes_persistent_iterator_without_accumulating_old_cursors() {
        enable_cycle_collection();
        let callback = V::fun(vec![], |_, state| {
            let V::Int(n) = state else { panic!() };
            if n == 100_000 {
                V::adt("std/prelude::Nothing", V::Unit)
            } else {
                V::adt(
                    "std/prelude::Just",
                    V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
                )
            }
        });
        let iterator = native_iterator_unfold(callback, V::Int(0));
        let result = native_iterable_build(
            "std/list::fromIterable",
            vec![native_iterable_dictionary("std/iterator::Iterable")],
            vec![iterator],
        );
        assert_eq!(native_sequence_iter(&result, true).count(), 100_000);
        assert_eq!(
            native_sequence_iter(&result, true).last().unwrap().render(),
            "99999"
        );
        assert!(LIVE.with(Cell::get) < 100_016);
        drop(result);
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
