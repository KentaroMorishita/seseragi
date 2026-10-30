// Use the same bottom-up stable merge as runtime/ts/src/sequence.ts. In
// particular, key extraction finishes once in source order before comparison,
// and Equal takes the left element without asking the dictionary again.
fn native_sequence_stable_sort(mut source: Vec<(V, V)>, dictionary: &V) -> Vec<V> {
    let compare = dictionary.field("compare");
    let mut target = source.clone();
    let mut width = 1usize;
    while width < source.len() {
        for start in (0..source.len()).step_by(width.saturating_mul(2)) {
            let middle = start.saturating_add(width).min(source.len());
            let end = middle.saturating_add(width).min(source.len());
            let (mut left, mut right) = (start, middle);
            for slot in &mut target[start..end] {
                let take_left = left < middle
                    && (right >= end || {
                        let order = compare
                            .apply(source[left].1.clone())
                            .apply(source[right].1.clone());
                        !order.tagged("std/prelude::Greater")
                    });
                let index = if take_left {
                    let index = left;
                    left += 1;
                    index
                } else {
                    let index = right;
                    right += 1;
                    index
                };
                *slot = source[index].clone();
            }
        }
        std::mem::swap(&mut source, &mut target);
        width = width.saturating_mul(2);
    }
    source.into_iter().map(|(value, _)| value).collect()
}

fn native_sequence_group_by(dictionaries: &[V], key: &V, input: &V, list: bool) -> V {
    let mut index = V::Map(Shared::new(NativeMap::empty(native_process_hash_seed())));
    let mut groups: Vec<Vec<V>> = Vec::new();
    for value in native_sequence_iter(input, list) {
        let k = key.apply(value.clone());
        if list {
            index = native_map_upsert(native_map_value(&index), dictionaries, &k, |current| {
                let address = current.map_or_else(
                    || {
                        groups.push(Vec::new());
                        groups.len() - 1
                    },
                    |address| native_map_address(address) as usize,
                );
                groups[address].push(value.clone());
                V::Int(address as i64)
            });
        } else {
            // Array's TS builder performs get, then insert only on a miss.
            // List uses one upsert instead. Keep those dictionary call stages.
            let current = native_map_operation(
                "std/map::get",
                dictionaries.to_vec(),
                vec![k.clone(), index.clone()],
            );
            if current.tagged("std/prelude::Just") {
                groups[native_map_address(&current.payload()) as usize].push(value.clone());
            } else {
                let address = groups.len();
                groups.push(vec![value.clone()]);
                index = native_map_operation(
                    "std/map::insert",
                    dictionaries.to_vec(),
                    vec![k, V::Int(address as i64), index],
                );
            }
        }
    }
    // Materialize each group once, in source order. The private index values
    // are replaced without calling Hash/Eq or retaining builder containers.
    let groups = V::Array(Shared::new(
        groups
            .into_iter()
            .map(|values| native_sequence_collect(values, list))
            .collect(),
    ));
    native_map_operation(
        "std/map::mapValues",
        vec![],
        vec![
            V::fun(vec![groups], |captures, address| {
                let V::Array(groups) = &captures[0] else {
                    panic!("Rust ABI: group builder")
                };
                groups.get()[native_map_address(&address) as usize].clone()
            }),
            index,
        ],
    )
}

fn native_sequence_order_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    let (owner, operation) = name
        .split_once("::")
        .expect("canonical sequence ordering name");
    let list = match owner {
        "std/list" => true,
        "std/array" => false,
        _ => panic!("Rust ABI: sequence ordering owner"),
    };
    assert_eq!(arguments.len(), if operation == "sort" { 1 } else { 2 });
    let input = arguments.last().expect("sequence ordering input");
    if operation == "groupBy" {
        return native_sequence_group_by(&dictionaries, &arguments[0], input, list);
    }
    assert!(matches!(operation, "sort" | "sortBy"));
    let [dictionary] = dictionaries.as_slice() else {
        panic!("Rust ABI: selected Ord")
    };
    let keyed = native_sequence_iter(input, list)
        .map(|value| {
            let key = if operation == "sortBy" {
                arguments[0].apply(value.clone())
            } else {
                value.clone()
            };
            (value.clone(), key)
        })
        .collect();
    native_sequence_collect(native_sequence_stable_sort(keyed, dictionary), list)
}

#[cfg(test)]
mod sequence_order_tests {
    use super::*;

    fn integers(value: &V, list: bool) -> Vec<i64> {
        native_sequence_iter(value, list)
            .map(|value| match value {
                V::Int(n) => *n,
                _ => panic!(),
            })
            .collect()
    }
    fn sequence(values: &[i64], list: bool) -> V {
        native_sequence_collect(values.iter().copied().map(V::Int).collect(), list)
    }
    fn name(list: bool, operation: &str) -> String {
        format!("std/{}::{operation}", if list { "list" } else { "array" })
    }
    fn record(log: &V, value: String) {
        let V::Array(events) = log.read_reference() else {
            panic!()
        };
        let mut events = events.get().clone();
        events.push(V::text(&value));
        log.write_reference(V::Array(Shared::new(events)));
    }
    fn events(log: &V) -> Vec<String> {
        let V::Array(events) = log.read_reference() else {
            panic!()
        };
        events.get().iter().map(V::render).collect()
    }

    #[test]
    fn stable_sort_extracts_keys_once_before_the_ts_comparison_order_and_keeps_equal_elements() {
        enable_cycle_collection();
        for list in [false, true] {
            let log = V::reference(V::Array(Shared::new(vec![])));
            let key = V::fun(vec![log.clone()], |c, value| {
                let V::Int(n) = value else { panic!() };
                record(&c[0], format!("key:{n}"));
                V::Int(n % 10)
            });
            let dictionary = V::dict(vec![(
                "compare",
                V::fun(vec![log.clone()], |c, left| {
                    V::fun(vec![c[0].clone(), left], |c, right| {
                        record(
                            &c[0],
                            format!("compare:{}:{}", c[1].render(), right.render()),
                        );
                        native_int_ord_dictionary()
                            .field("compare")
                            .apply(c[1].clone())
                            .apply(right)
                    })
                }),
            )]);
            let input = sequence(&[22, 11, 42, 31], list);
            let result = native_sequence_order_operation(
                &name(list, "sortBy"),
                vec![dictionary.clone()],
                vec![key.clone(), input.clone()],
            );
            assert_eq!(integers(&result, list), vec![11, 31, 22, 42]);
            assert_eq!(integers(&input, list), vec![22, 11, 42, 31]);
            assert_eq!(
                events(&log),
                [
                    "key:22",
                    "key:11",
                    "key:42",
                    "key:31",
                    "compare:2:1",
                    "compare:2:1",
                    "compare:1:1",
                    "compare:2:1",
                    "compare:2:2"
                ]
            );
            let empty = native_sequence_order_operation(
                &name(list, "sortBy"),
                vec![dictionary.clone()],
                vec![key.clone(), sequence(&[], list)],
            );
            assert!(integers(&empty, list).is_empty());
            assert_eq!(events(&log).len(), 9);
            let sorted = native_sequence_order_operation(
                &name(list, "sort"),
                vec![native_int_ord_dictionary()],
                vec![input.clone()],
            );
            assert_eq!(integers(&sorted, list), vec![11, 22, 31, 42]);
            drop((log, key, dictionary, input, result, empty, sorted));
            clear_dictionaries();
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn grouping_keeps_first_key_order_collision_buckets_custom_equality_and_dictionary_call_stages()
    {
        enable_cycle_collection();
        for list in [false, true] {
            let log = V::reference(V::Array(Shared::new(vec![])));
            let eq = V::dict(vec![(
                "eq",
                V::fun(vec![log.clone()], |c, left| {
                    V::fun(vec![c[0].clone(), left], |c, right| {
                        record(&c[0], "eq".into());
                        let (V::Int(left), V::Int(right)) = (&c[1], right) else {
                            panic!()
                        };
                        V::Bool(left.abs() == right.abs())
                    })
                }),
            )]);
            let hash = V::dict(vec![(
                "hash",
                V::fun(vec![log.clone()], |c, _| {
                    record(&c[0], "hash".into());
                    V::Int(7)
                }),
            )]);
            let input = sequence(&[1, -1, 2, -2], list);
            let grouped = native_sequence_order_operation(
                &name(list, "groupBy"),
                vec![eq, hash],
                vec![V::fun(vec![], |_, value| value), input.clone()],
            );
            let V::Array(entries) =
                native_map_operation("std/map::entries", vec![], vec![grouped.clone()])
            else {
                panic!()
            };
            let mut rows = Vec::new();
            for entry in entries.get() {
                let V::Tuple(pair) = entry else { panic!() };
                rows.push((pair.get()[0].render(), integers(&pair.get()[1], list)));
            }
            assert_eq!(
                rows,
                vec![("1".into(), vec![1, -1]), ("2".into(), vec![2, -2])]
            );
            let trace = events(&log);
            assert_eq!(
                trace.iter().filter(|s| s.as_str() == "hash").count(),
                if list { 4 } else { 6 }
            );
            assert_eq!(
                trace.iter().filter(|s| s.as_str() == "eq").count(),
                if list { 4 } else { 5 }
            );
            assert_eq!(integers(&input, list), vec![1, -1, 2, -2]);
            drop((log, input, grouped, entries));
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn key_and_comparison_defects_stop_later_calls_and_release_dictionary_callback_cycles() {
        enable_cycle_collection();
        for list in [false, true] {
            for operation in ["sort", "sortBy", "groupBy"] {
                let cell = V::reference(V::Unit);
                let log = V::reference(V::Array(Shared::new(vec![])));
                let callback = V::fun(vec![cell.clone(), log.clone()], |c, value| {
                    record(&c[1], value.render());
                    let V::Int(n) = value else { panic!() };
                    assert_ne!(n, 2, "deliberate key defect");
                    V::Int(n)
                });
                let dictionary = V::dict(vec![(
                    "compare",
                    V::fun(vec![cell.clone(), log.clone()], |c, left| {
                        V::fun(vec![c[0].clone(), c[1].clone(), left], |c, _| {
                            record(&c[1], "comparison".into());
                            panic!("deliberate comparison defect")
                        })
                    }),
                )]);
                cell.write_reference(V::Tuple(Shared::new(vec![
                    callback.clone(),
                    dictionary.clone(),
                ])));
                let input = sequence(&[1, 2, 3], list);
                let dictionaries = if operation == "groupBy" {
                    vec![
                        native_operator_dictionary("std/int::Eq"),
                        native_hash_dictionary("std/int::Hash"),
                    ]
                } else {
                    vec![dictionary.clone()]
                };
                let arguments = if operation == "sort" {
                    vec![input.clone()]
                } else {
                    vec![callback.clone(), input.clone()]
                };
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    native_sequence_order_operation(&name(list, operation), dictionaries, arguments)
                }))
                .is_err());
                assert_eq!(
                    events(&log),
                    if operation == "sort" {
                        vec!["comparison"]
                    } else {
                        vec!["1", "2"]
                    }
                );
                assert_eq!(integers(&input, list), vec![1, 2, 3]);
                drop((cell, log, callback, dictionary, input));
                clear_dictionaries();
                assert!(collect_cycles().unwrap().reclaimed > 0);
                assert_eq!(LIVE.with(Cell::get), 0);
            }
        }
    }

    #[test]
    fn one_hundred_thousand_element_group_builds_once_and_drops_list_tails_iteratively() {
        enable_cycle_collection();
        let input = native_sequence_collect((0..100_000).map(V::Int).collect(), true);
        let grouped = native_sequence_order_operation(
            "std/list::groupBy",
            vec![
                native_operator_dictionary("std/unit::Eq"),
                native_hash_dictionary("std/unit::Hash"),
            ],
            vec![V::fun(vec![], |_, _| V::Unit), input.clone()],
        );
        let entry = native_map_entry(native_map_value(&grouped), 0);
        assert_eq!(native_sequence_iter(&entry.value, true).count(), 100_000);
        assert_eq!(
            native_sequence_iter(&entry.value, true)
                .last()
                .unwrap()
                .render(),
            "99999"
        );
        drop((input, grouped));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
