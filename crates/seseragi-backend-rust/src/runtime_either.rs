fn native_either_applicative() -> V {
    cached("\0native:either-applicative", || {
        V::dict(vec![
            (
                "pure",
                V::fun(vec![], |_, value| V::adt("std/prelude::Right", value)),
            ),
            (
                "map",
                V::fun(vec![], |_, callback| {
                    V::fun(vec![callback], |captures, value| {
                        if value.tagged("std/prelude::Left") {
                            return value;
                        }
                        assert!(value.tagged("std/prelude::Right"), "Rust ABI: Either map");
                        V::adt("std/prelude::Right", captures[0].apply(value.payload()))
                    })
                }),
            ),
            (
                "apply",
                V::fun(vec![], |_, wrapped| {
                    V::fun(vec![wrapped], |captures, value| {
                        if captures[0].tagged("std/prelude::Left") {
                            return captures[0].clone();
                        }
                        if value.tagged("std/prelude::Left") {
                            return value;
                        }
                        assert!(
                            captures[0].tagged("std/prelude::Right")
                                && value.tagged("std/prelude::Right"),
                            "Rust ABI: Either apply"
                        );
                        V::adt(
                            "std/prelude::Right",
                            captures[0].payload().apply(value.payload()),
                        )
                    })
                }),
            ),
        ])
    })
}

fn native_either_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    match name {
        "std/either::mapLeft" | "std/either::mapRight" => {
            assert!(dictionaries.is_empty());
            let [callback, value]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: Either map arity"));
            let left = value.tagged("std/prelude::Left");
            assert!(
                left || value.tagged("std/prelude::Right"),
                "Rust ABI: Either map source"
            );
            if left == name.ends_with("::mapLeft") {
                if left {
                    V::adt("std/prelude::Left", callback.apply(value.payload()))
                } else {
                    V::adt("std/prelude::Right", callback.apply(value.payload()))
                }
            } else {
                value
            }
        }
        "std/either::bimap" | "std/either::fold" => {
            assert!(dictionaries.is_empty());
            let [left, right, value]: [V; 3] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: Either branch arity"));
            let is_left = value.tagged("std/prelude::Left");
            assert!(
                is_left || value.tagged("std/prelude::Right"),
                "Rust ABI: Either branch source"
            );
            let result = if is_left {
                left.apply(value.payload())
            } else {
                right.apply(value.payload())
            };
            if name.ends_with("::fold") {
                result
            } else if is_left {
                V::adt("std/prelude::Left", result)
            } else {
                V::adt("std/prelude::Right", result)
            }
        }
        "std/either::swap" => {
            assert!(dictionaries.is_empty());
            let [value]: [V; 1] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: Either swap arity"));
            if value.tagged("std/prelude::Left") {
                V::adt("std/prelude::Right", value.payload())
            } else {
                assert!(
                    value.tagged("std/prelude::Right"),
                    "Rust ABI: Either swap source"
                );
                V::adt("std/prelude::Left", value.payload())
            }
        }
        "std/either::sequence" | "std/either::traverse" => {
            let [dictionary] = dictionaries.as_slice() else {
                panic!("Rust ABI: Either Traversable");
            };
            let (callback, source) = if name.ends_with("::sequence") {
                let [source]: [V; 1] = arguments
                    .try_into()
                    .unwrap_or_else(|_| panic!("Rust ABI: Either sequence arity"));
                (V::fun(vec![], |_, value| value), source)
            } else {
                let [callback, source]: [V; 2] = arguments
                    .try_into()
                    .unwrap_or_else(|_| panic!("Rust ABI: Either traverse arity"));
                (callback, source)
            };
            dictionary
                .field("traverse")
                .apply(callback)
                .apply(source)
                .apply(native_either_applicative())
        }
        _ => panic!("Rust ABI: Either operation"),
    }
}

#[cfg(test)]
mod either_operation_tests {
    use super::*;
    fn sequence(list: bool, values: Vec<V>) -> V {
        native_sequence_collect(values, list)
    }
    fn text(value: &str) -> V {
        V::Text(Shared::new(value.into()))
    }
    fn number(value: &V) -> i64 {
        let V::Int(number) = value else {
            panic!("expected Int")
        };
        *number
    }
    fn cleanup() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    fn record(log: &V, value: V) {
        let V::Array(values) = log.read_reference() else {
            panic!()
        };
        let mut values = values.get().clone();
        values.push(value);
        log.write_reference(V::Array(Shared::new(values)));
    }
    #[test]
    fn either_traversal_evaluates_every_callback_in_source_order_and_keeps_the_first_left() {
        enable_cycle_collection();
        for list in [false, true] {
            for defect in [false, true] {
                let log = V::reference(V::Array(Shared::new(vec![])));
                let callback = V::fun(vec![log.clone(), V::Bool(defect)], |c, value| {
                    record(&c[0], value.clone());
                    let V::Int(n) = value else { panic!() };
                    assert!(!(c[1].boolean() && n == 3), "later callback defect");
                    if n == 1 {
                        V::adt("std/prelude::Left", text("first"))
                    } else if n == 2 {
                        V::adt("std/prelude::Left", text("second"))
                    } else {
                        V::adt("std/prelude::Right", V::Int(n))
                    }
                });
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    native_either_operation(
                        "std/either::traverse",
                        vec![native_sequence_traversable_dictionary(list)],
                        vec![
                            callback,
                            sequence(list, vec![V::Int(1), V::Int(2), V::Int(3)]),
                        ],
                    )
                }));
                assert_eq!(result.is_err(), defect);
                if let Ok(value) = &result {
                    assert!(value.tagged("std/prelude::Left"));
                    assert_eq!(value.payload().render(), "first");
                }
                let V::Array(values) = log.read_reference() else {
                    panic!()
                };
                assert_eq!(
                    values.get().iter().map(number).collect::<Vec<_>>(),
                    [1, 2, 3]
                );
                drop((values, result, log));
                cleanup();
            }
        }
    }
    #[test]
    fn either_mapping_preserves_unselected_nodes_and_only_invokes_the_selected_branch() {
        enable_cycle_collection();
        let retained = V::reference(V::Unit);
        let discarded = V::reference(V::Unit);
        retained.write_reference(retained.clone());
        discarded.write_reference(discarded.clone());
        let left = V::adt("std/prelude::Left", retained.clone());
        let unused = V::fun(vec![discarded.clone()], |_, _| {
            panic!("unselected callback")
        });
        let kept = native_either_operation(
            "std/either::mapRight",
            vec![],
            vec![unused.clone(), left.clone()],
        );
        assert_eq!(kept.node_id(), left.node_id());
        let selected = V::fun(vec![], |_, value| value);
        let folded = native_either_operation(
            "std/either::fold",
            vec![],
            vec![selected.clone(), unused.clone(), left.clone()],
        );
        assert_eq!(folded.node_id(), retained.node_id());
        let mapped = native_either_operation(
            "std/either::bimap",
            vec![],
            vec![selected, unused, left.clone()],
        );
        assert!(mapped.tagged("std/prelude::Left"));
        assert_eq!(mapped.payload().node_id(), retained.node_id());
        let swapped = native_either_operation("std/either::swap", vec![], vec![left.clone()]);
        assert!(swapped.tagged("std/prelude::Right"));
        assert_eq!(swapped.payload().node_id(), retained.node_id());
        drop((discarded, left, retained));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 4);
        drop((kept, folded, mapped, swapped));
        cleanup();
    }
    #[test]
    fn either_traversal_uses_the_selected_captured_dictionary_and_releases_defect_cycles() {
        enable_cycle_collection();
        for defect in [false, true] {
            let captured = V::reference(V::Unit);
            captured.write_reference(captured.clone());
            let dictionary = V::dict(vec![(
                "traverse",
                V::fun(vec![captured.clone(), V::Bool(defect)], |c, callback| {
                    V::fun(vec![c[0].clone(), c[1].clone(), callback], |c, source| {
                        V::fun(
                            vec![c[0].clone(), c[1].clone(), c[2].clone(), source],
                            |c, target| {
                                assert!(!c[1].boolean(), "captured traversal defect");
                                target
                                    .field("map")
                                    .apply(V::fun(vec![c[0].clone()], |_, value| value))
                                    .apply(c[2].apply(c[3].clone()))
                            },
                        )
                    })
                }),
            )]);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_either_operation(
                    "std/either::traverse",
                    vec![dictionary],
                    vec![
                        V::fun(vec![], |_, v| V::adt("std/prelude::Right", v)),
                        V::Int(42),
                    ],
                )
            }));
            assert_eq!(result.is_err(), defect);
            if let Ok(result) = &result {
                assert_eq!(number(&result.payload()), 42);
            }
            drop((result, captured));
            cleanup();
        }
    }
    #[test]
    fn either_hundred_thousand_list_traversal_keeps_only_the_result_edges() {
        enable_cycle_collection();
        for failure in [false, true] {
            let source = sequence(true, (0..100_000).map(V::Int).collect());
            let result = native_either_operation(
                "std/either::traverse",
                vec![native_sequence_traversable_dictionary(true)],
                vec![
                    V::fun(vec![V::Bool(failure)], |c, value| {
                        if c[0].boolean() && number(&value) == 1 {
                            V::adt("std/prelude::Left", text("first"))
                        } else {
                            V::adt("std/prelude::Right", value)
                        }
                    }),
                    source,
                ],
            );
            if failure {
                assert!(result.tagged("std/prelude::Left"));
                assert_eq!(result.payload().render(), "first");
            } else {
                assert!(result.tagged("std/prelude::Right"));
                assert_eq!(
                    native_sequence_iter(&result.payload(), true).count(),
                    100_000
                );
            }
            clear_dictionaries();
            collect_cycles().unwrap();
            assert!(LIVE.with(Cell::get) < if failure { 8 } else { 100_008 });
            drop(result);
            cleanup();
        }
    }
}
