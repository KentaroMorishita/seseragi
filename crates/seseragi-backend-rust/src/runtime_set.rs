// Set has its own native tag while sharing the persistent Map storage and
// tracing implementation. The internal Unit values are never exposed.
fn native_set_into_map(value: V) -> V {
    let V::Set(storage) = value else {
        panic!("Rust ABI: expected Set")
    };
    V::Map(storage)
}
fn native_set_from_map(value: V) -> V {
    let V::Map(storage) = value else {
        panic!("Rust ABI: expected internal Set Map")
    };
    V::Set(storage)
}

fn native_set_select(values: &NativeMap, mut keep: impl FnMut(&V) -> bool) -> V {
    native_set_from_map(native_map_select(values, |key, _| keep(key)))
}

fn native_set_operation(name: &str, dictionaries: Vec<V>, mut arguments: Vec<V>) -> V {
    if name == "std/set::empty" {
        return native_set_from_map(native_map_operation(
            "std/map::empty",
            dictionaries,
            arguments,
        ));
    }
    if name == "std/set::singleton" {
        assert_eq!(arguments.len(), 1, "Rust ABI: Set singleton arity");
        arguments.push(V::Unit);
        return native_set_from_map(native_map_operation(
            "std/map::singleton",
            dictionaries,
            arguments,
        ));
    }
    let values = native_set_into_map(arguments.pop().expect("Rust ABI: Set source arity"));
    if name == "std/set::filter" {
        assert!(dictionaries.is_empty());
        assert_eq!(arguments.len(), 1, "Rust ABI: Set filter arity");
        return native_set_select(native_map_value(&values), |key| {
            arguments[0].apply(key.clone()).boolean()
        });
    }
    if matches!(
        name,
        "std/set::union" | "std/set::intersection" | "std/set::difference" | "std/set::isSubsetOf"
    ) {
        assert_eq!(arguments.len(), 1, "Rust ABI: Set binary operation arity");
        let right = native_set_into_map(arguments.pop().unwrap());
        if name == "std/set::union" {
            let mut result = values;
            let source = native_map_value(&right);
            let mut id = source.first;
            let mut visited = 0;
            while let Some(current) = id {
                assert!(
                    visited < native_map_index_size(&source.order),
                    "Rust ABI: cyclic Set order"
                );
                visited += 1;
                let entry = native_map_entry(source, current);
                result = native_map_operation(
                    "std/map::insert",
                    dictionaries.clone(),
                    vec![entry.key.clone(), V::Unit, result],
                );
                id = entry.next;
            }
            return native_set_from_map(result);
        }
        let contains = |key: &V| {
            native_map_operation(
                "std/map::containsKey",
                dictionaries.clone(),
                vec![key.clone(), right.clone()],
            )
            .boolean()
        };
        if name == "std/set::isSubsetOf" {
            let source = native_map_value(&values);
            let mut id = source.first;
            let mut visited = 0;
            while let Some(current) = id {
                assert!(
                    visited < native_map_index_size(&source.order),
                    "Rust ABI: cyclic Set order"
                );
                visited += 1;
                let entry = native_map_entry(source, current);
                if !contains(&entry.key) {
                    return V::Bool(false);
                }
                id = entry.next;
            }
            return V::Bool(true);
        }
        return native_set_select(native_map_value(&values), |key| {
            contains(key) == (name == "std/set::intersection")
        });
    }
    let (operation, returns_set) = match name {
        "std/set::contains" => ("std/map::containsKey", false),
        "std/set::insert" => {
            arguments.push(V::Unit);
            ("std/map::insert", true)
        }
        "std/set::remove" => ("std/map::remove", true),
        "std/set::toArray" | "std/set::toList" => ("std/map::keys", false),
        "std/set::size" => ("std/map::size", false),
        "std/set::isEmpty" => ("std/map::isEmpty", false),
        _ => panic!("Rust ABI: unsupported Set operation"),
    };
    arguments.push(values);
    let result = native_map_operation(operation, dictionaries, arguments);
    if name == "std/set::toList" {
        let V::Array(values) = result else {
            panic!("Rust ABI: Set keys array")
        };
        return V::list_from_values(values.get().clone());
    }
    if returns_set {
        native_set_from_map(result)
    } else {
        result
    }
}

#[cfg(test)]
mod set_tests {
    use super::*;

    thread_local! { static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; }
    fn events() -> Vec<String> {
        EVENTS.with(|e| std::mem::take(&mut *e.borrow_mut()))
    }
    fn dictionaries() -> Vec<V> {
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
                    let V::Int(key) = key else { panic!() };
                    EVENTS.with(|e| e.borrow_mut().push(format!("h{key}")));
                    V::Int(key % 2)
                }),
            )]),
        ]
    }

    fn ints(values: &[i64], dictionaries: &[V]) -> V {
        let mut result = native_set_operation("std/set::empty", vec![], vec![V::Unit]);
        for value in values {
            result = native_set_operation(
                "std/set::insert",
                dictionaries.to_vec(),
                vec![V::Int(*value), result],
            );
        }
        result
    }

    #[test]
    fn set_operations_keep_cached_hashes_order_snapshots_and_subset_short_circuit() {
        clear_dictionaries();
        enable_cycle_collection();
        let d = dictionaries();
        let left = ints(&[1, 3, 2], &d);
        let right = ints(&[3, 5], &d);
        events();
        let union = native_set_operation(
            "std/set::union",
            d.clone(),
            vec![right.clone(), left.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3", "h5", "e1:5", "e3:5"]);
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![union.clone()]).render(),
            "`[1, 3, 2, 5]"
        );
        let intersection = native_set_operation(
            "std/set::intersection",
            d.clone(),
            vec![right.clone(), left.clone()],
        );
        assert_eq!(events(), ["h1", "e3:1", "e5:1", "h3", "e3:3", "h2"]);
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![intersection.clone()]).render(),
            "`[3]"
        );
        let difference = native_set_operation(
            "std/set::difference",
            d.clone(),
            vec![right.clone(), left.clone()],
        );
        assert_eq!(events(), ["h1", "e3:1", "e5:1", "h3", "e3:3", "h2"]);
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![difference.clone()]).render(),
            "`[1, 2]"
        );
        assert!(!native_set_operation(
            "std/set::isSubsetOf",
            d.clone(),
            vec![right.clone(), left.clone()]
        )
        .boolean());
        assert_eq!(events(), ["h1", "e3:1", "e5:1"]);
        let log = V::reference(V::Int(0));
        let predicate = V::fun(vec![log.clone()], |captures, key| {
            let (V::Int(old), V::Int(key)) = (captures[0].read_reference(), key) else {
                panic!()
            };
            captures[0].write_reference(V::Int(old * 10 + key));
            V::Bool(key % 2 != 0)
        });
        let filtered = native_set_operation(
            "std/set::filter",
            vec![],
            vec![predicate.clone(), left.clone()],
        );
        assert!(events().is_empty());
        assert_eq!(log.read_reference().render(), "132");
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![filtered.clone()]).render(),
            "`[1, 3]"
        );
        assert!(native_set_operation(
            "std/set::contains",
            d.clone(),
            vec![V::Int(3), filtered.clone()]
        )
        .boolean());
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![left.clone()]).render(),
            "`[1, 3, 2]"
        );
        let empty = ints(&[], &d);
        assert!(native_set_operation(
            "std/set::isSubsetOf",
            d.clone(),
            vec![right.clone(), empty.clone()]
        )
        .boolean());
        assert!(events().is_empty());
        assert_eq!(
            native_set_operation("std/set::toList", vec![], vec![empty.clone()]).render(),
            "`[]"
        );
        drop((
            left,
            right,
            union,
            intersection,
            difference,
            log,
            predicate,
            filtered,
            empty,
            d,
        ));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn filtered_result_keeps_no_excluded_key_and_list_conversion_keeps_no_container() {
        clear_dictionaries();
        enable_cycle_collection();
        let d = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        let keep = V::text("keep-雪🙂");
        let exclude = V::text("excluded");
        let V::Text(node) = &exclude else { panic!() };
        let excluded = Rc::downgrade(node.0.as_ref().unwrap());
        let source = native_set_operation("std/set::singleton", d.clone(), vec![exclude.clone()]);
        let source = native_set_operation("std/set::insert", d.clone(), vec![keep.clone(), source]);
        let source_id = source.node_id().unwrap();
        let predicate = V::fun(vec![], |_, key| V::Bool(native_text(&key) != "excluded"));
        let filtered =
            native_set_operation("std/set::filter", vec![], vec![predicate, source.clone()]);
        let filtered_id = filtered.node_id().unwrap();
        let list = native_set_operation("std/set::toList", vec![], vec![filtered.clone()]);
        drop((keep, exclude, source, filtered));
        collect_cycles().unwrap();
        assert!(excluded.upgrade().is_none());
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&source_id)));
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&filtered_id)));
        assert_eq!(list.render(), "`[keep-雪🙂]");
        drop((list, d));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn predicate_defect_unwinds_selected_keys_and_captured_reference_cycles() {
        enable_cycle_collection();
        let cell = V::reference(V::Unit);
        let predicate = V::fun(vec![cell.clone()], |_, key| {
            let V::Int(key) = key else { panic!() };
            assert_ne!(key, 2, "deliberate predicate defect");
            V::Bool(true)
        });
        cell.write_reference(predicate.clone());
        let d = dictionaries();
        let source = ints(&[1, 2, 3], &d);
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            native_set_operation(
                "std/set::filter",
                vec![],
                vec![predicate.clone(), source.clone()],
            )
        }));
        assert!(failed.is_err());
        drop((predicate, cell, source, d));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn binary_operations_unwind_hash_and_eq_defects_with_cyclic_dictionaries() {
        enable_cycle_collection();
        for in_hash in [false, true] {
            for operation in ["union", "intersection", "difference", "isSubsetOf"] {
                let normal = dictionaries();
                let (left, right) = match operation {
                    "union" => (ints(&[1], &normal), ints(&[2, 3], &normal)),
                    "isSubsetOf" => (ints(&[2, 3], &normal), ints(&[2, 3], &normal)),
                    "intersection" => (ints(&[2, 3], &normal), ints(&[2, 1], &normal)),
                    _ => (ints(&[4, 3], &normal), ints(&[2, 1], &normal)),
                };
                let cell = V::reference(V::Unit);
                let eq = V::dict(vec![(
                    "eq",
                    V::fun(vec![cell.clone(), V::Bool(in_hash)], |captures, left| {
                        V::fun(
                            vec![captures[0].clone(), captures[1].clone(), left],
                            |captures, right| {
                                let (V::Int(left), V::Int(right)) = (&captures[2], right) else {
                                    panic!()
                                };
                                assert!(
                                    captures[1].boolean() || right != 3,
                                    "deliberate Eq defect"
                                );
                                V::Bool(*left == right)
                            },
                        )
                    }),
                )]);
                let hash = V::dict(vec![(
                    "hash",
                    V::fun(vec![cell.clone(), V::Bool(in_hash)], |captures, key| {
                        let V::Int(key) = key else { panic!() };
                        assert!(!captures[1].boolean() || key != 3, "deliberate Hash defect");
                        V::Int(key % 2)
                    }),
                )]);
                let bad = vec![eq, hash];
                cell.write_reference(V::Tuple(Shared::new(bad.clone())));
                let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    native_set_operation(
                        &format!("std/set::{operation}"),
                        bad.clone(),
                        vec![right.clone(), left.clone()],
                    )
                }));
                assert!(failed.is_err(), "{operation}/{in_hash}");
                drop((normal, left, right, cell, bad));
                collect_cycles().unwrap();
                assert_eq!(LIVE.with(Cell::get), 0, "{operation}/{in_hash}");
            }
        }
    }

    #[test]
    fn set_preserves_snapshots_order_and_single_selected_hash_then_collision_eq() {
        assert_eq!(std::mem::size_of::<V>(), 16);
        let d = dictionaries();
        events();
        let empty = native_set_operation("std/set::empty", vec![], vec![V::Unit]);
        let one = native_set_operation("std/set::singleton", d.clone(), vec![V::Int(1)]);
        assert_eq!(events(), ["h1"]);
        let two = native_set_operation("std/set::insert", d.clone(), vec![V::Int(3), one.clone()]);
        assert_eq!(events(), ["h3", "e1:3"]);
        assert_eq!(
            native_set_operation("std/set::contains", d.clone(), vec![V::Int(3), two.clone()])
                .render(),
            "True"
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        let duplicate =
            native_set_operation("std/set::insert", d.clone(), vec![V::Int(1), two.clone()]);
        assert_eq!(events(), ["h1", "e1:1"]);
        let removed = native_set_operation(
            "std/set::remove",
            d.clone(),
            vec![V::Int(3), duplicate.clone()],
        );
        assert_eq!(events(), ["h3", "e1:3", "e3:3"]);
        assert_eq!(
            native_set_operation("std/set::toArray", vec![], vec![two.clone()]).render(),
            "[1, 3]"
        );
        assert_eq!(
            native_set_operation("std/set::toArray", vec![], vec![removed.clone()]).render(),
            "[1]"
        );
        assert_eq!(
            native_set_operation("std/set::size", vec![], vec![two.clone()]).render(),
            "2"
        );
        assert_eq!(
            native_set_operation("std/set::isEmpty", vec![], vec![empty.clone()]).render(),
            "True"
        );
        assert!(events().is_empty());
        drop((d, empty, one, two, duplicate, removed));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn set_keys_trace_saved_callback_reference_cycles() {
        enable_cycle_collection();
        let reference = V::reference(V::Unit);
        let callback = V::fun(vec![reference.clone()], |_, value| value);
        let key = V::adt("proof::Key", callback.clone());
        let d = vec![
            V::dict(vec![(
                "eq",
                V::fun(vec![], |_, _| V::fun(vec![], |_, _| V::Bool(true))),
            )]),
            V::dict(vec![("hash", V::fun(vec![], |_, _| V::Int(0)))]),
        ];
        let values = native_set_operation("std/set::singleton", d.clone(), vec![key.clone()]);
        reference.write_reference(values.clone());
        let selected = native_set_operation("std/set::toArray", vec![], vec![values.clone()])
            .index(0)
            .payload();
        drop((values, key, callback));
        assert_eq!(selected.apply(V::Int(42)).render(), "42");
        drop((reference, selected, d));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn removed_set_keys_are_not_retained_by_the_result() {
        clear_dictionaries();
        let d = vec![
            native_operator_dictionary("std/string::Eq"),
            native_hash_dictionary("std/string::Hash"),
        ];
        let key = V::text("discarded-雪🙂");
        let V::Text(node) = &key else { panic!() };
        let weak = Rc::downgrade(node.0.as_ref().unwrap());
        let values = native_set_operation("std/set::singleton", d.clone(), vec![key.clone()]);
        let removed = native_set_operation(
            "std/set::remove",
            d.clone(),
            vec![V::text("discarded-雪🙂"), values.clone()],
        );
        drop((key, values));
        assert!(weak.upgrade().is_none());
        assert_eq!(
            native_set_operation("std/set::size", vec![], vec![removed.clone()]).render(),
            "0"
        );
        drop((d, removed));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
