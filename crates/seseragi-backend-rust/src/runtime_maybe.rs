fn native_maybe_applicative() -> V {
    cached("\0native:maybe-applicative", || {
        V::dict(vec![
            (
                "pure",
                V::fun(vec![], |_, value| V::adt("std/prelude::Just", value)),
            ),
            (
                "map",
                V::fun(vec![], |_, callback| {
                    V::fun(vec![callback], |captures, value| {
                        if value.tagged("std/prelude::Nothing") {
                            return value;
                        }
                        assert!(value.tagged("std/prelude::Just"), "Rust ABI: Maybe map");
                        V::adt("std/prelude::Just", captures[0].apply(value.payload()))
                    })
                }),
            ),
            (
                "apply",
                V::fun(vec![], |_, wrapped| {
                    V::fun(vec![wrapped], |captures, value| {
                        if captures[0].tagged("std/prelude::Nothing") {
                            return captures[0].clone();
                        }
                        if value.tagged("std/prelude::Nothing") {
                            return value;
                        }
                        assert!(
                            captures[0].tagged("std/prelude::Just")
                                && value.tagged("std/prelude::Just"),
                            "Rust ABI: Maybe apply"
                        );
                        V::adt(
                            "std/prelude::Just",
                            captures[0].payload().apply(value.payload()),
                        )
                    })
                }),
            ),
        ])
    })
}

fn native_maybe_operation(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    match name {
        "std/maybe::withDefault" | "std/maybe::orElse" => {
            assert!(dictionaries.is_empty());
            let [fallback, value]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: Maybe fallback arity"));
            if value.tagged("std/prelude::Nothing") {
                fallback
            } else {
                assert!(
                    value.tagged("std/prelude::Just"),
                    "Rust ABI: Maybe fallback source"
                );
                if name == "std/maybe::withDefault" {
                    value.payload()
                } else {
                    value
                }
            }
        }
        "std/maybe::sequence" | "std/maybe::traverse" => {
            let [dictionary] = dictionaries.as_slice() else {
                panic!("Rust ABI: Maybe Traversable")
            };
            let (callback, source) = if name == "std/maybe::sequence" {
                let [source]: [V; 1] = arguments
                    .try_into()
                    .unwrap_or_else(|_| panic!("Rust ABI: Maybe sequence arity"));
                (V::fun(vec![], |_, value| value), source)
            } else {
                let [callback, source]: [V; 2] = arguments
                    .try_into()
                    .unwrap_or_else(|_| panic!("Rust ABI: Maybe traverse arity"));
                (callback, source)
            };
            dictionary
                .field("traverse")
                .apply(callback)
                .apply(source)
                .apply(native_maybe_applicative())
        }
        _ => panic!("Rust ABI: Maybe standard operation"),
    }
}

#[cfg(test)]
mod maybe_operation_tests {
    use super::*;
    fn seq(list: bool, values: &[i64]) -> V {
        native_sequence_collect(values.iter().copied().map(V::Int).collect(), list)
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
    fn maybe_traversal_keeps_source_order_and_later_callbacks_after_nothing_including_their_defects(
    ) {
        enable_cycle_collection();
        for list in [false, true] {
            for defect in [false, true] {
                let log = V::reference(V::Array(Shared::new(vec![])));
                let callback = V::fun(vec![log.clone(), V::Bool(defect)], |c, value| {
                    record(&c[0], value.clone());
                    let V::Int(n) = value else { panic!() };
                    assert!(
                        !(c[1].boolean() && n == 3),
                        "later callback defect must remain observable"
                    );
                    if n == 2 {
                        V::adt("std/prelude::Nothing", V::Unit)
                    } else {
                        V::adt("std/prelude::Just", V::Int(n * 10))
                    }
                });
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    native_maybe_operation(
                        "std/maybe::traverse",
                        vec![native_sequence_traversable_dictionary(list)],
                        vec![callback.clone(), seq(list, &[1, 2, 3])],
                    )
                }));
                if defect {
                    assert!(result.is_err());
                } else {
                    assert!(result.unwrap().tagged("std/prelude::Nothing"));
                }
                let V::Array(events) = log.read_reference() else {
                    panic!()
                };
                assert_eq!(
                    events.get().iter().map(V::render).collect::<Vec<_>>(),
                    ["1", "2", "3"]
                );
                drop((log, callback, events));
                clear_dictionaries();
                collect_cycles().unwrap();
                assert_eq!(LIVE.with(Cell::get), 0);
            }
        }
    }

    #[test]
    fn fallbacks_keep_only_the_selected_payload_and_or_else_preserves_the_selected_maybe_node() {
        enable_cycle_collection();
        for name in ["std/maybe::withDefault", "std/maybe::orElse"] {
            let fallback = V::reference(V::Int(1));
            let value = V::reference(V::Int(42));
            let fallback_id = fallback.node_id().unwrap();
            let value_id = value.node_id().unwrap();
            let input = V::adt("std/prelude::Just", value);
            let input_id = input.node_id();
            let fallback = if name.ends_with("::orElse") {
                V::adt("std/prelude::Just", fallback)
            } else {
                fallback
            };
            let result = native_maybe_operation(name, vec![], vec![fallback, input]);
            collect_cycles().unwrap();
            assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&fallback_id)));
            assert!(NODES.with(|nodes| nodes.borrow().contains_key(&value_id)));
            if name.ends_with("::orElse") {
                assert_eq!(result.node_id(), input_id);
                assert_eq!(result.payload().read_reference().render(), "42");
            } else {
                assert_eq!(result.read_reference().render(), "42");
            }
            drop(result);
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn custom_traversable_receives_the_target_dictionary_and_traces_state_cycles_through_failures()
    {
        enable_cycle_collection();
        for defect in [false, true] {
            let cell = V::reference(V::Unit);
            let dictionary = V::dict(vec![(
                "traverse",
                V::fun(vec![cell.clone()], |c, callback| {
                    V::fun(vec![c[0].clone(), callback], |c, source| {
                        V::fun(vec![c[0].clone(), c[1].clone(), source], |c, target| {
                            target
                                .field("map")
                                .apply(V::fun(vec![], |_, value| V::adt("test::Box", value)))
                                .apply(c[1].apply(c[2].payload()))
                        })
                    })
                }),
            )]);
            cell.write_reference(dictionary.clone());
            let callback = V::fun(vec![cell.clone(), V::Bool(defect)], |c, value| {
                assert!(!c[1].boolean(), "custom callback defect");
                V::adt("std/prelude::Just", value)
            });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_maybe_operation(
                    "std/maybe::traverse",
                    vec![dictionary.clone()],
                    vec![callback.clone(), V::adt("test::Box", V::Int(42))],
                )
            }));
            if defect {
                assert!(result.is_err());
            } else {
                assert_eq!(result.unwrap().payload().payload().render(), "42");
            }
            drop((cell, dictionary, callback));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn balanced_standard_traversal_materializes_large_lists_without_retaining_source_or_builder_nodes(
    ) {
        enable_cycle_collection();
        let source = native_sequence_collect((0..100_000).map(V::Int).collect(), true);
        let callback = V::fun(vec![], |_, value| V::adt("std/prelude::Just", value));
        let result = native_maybe_operation(
            "std/maybe::traverse",
            vec![native_sequence_traversable_dictionary(true)],
            vec![callback, source],
        );
        let output = result.payload();
        assert_eq!(native_sequence_iter(&output, true).count(), 100_000);
        assert_eq!(
            native_sequence_iter(&output, true).last().unwrap().render(),
            "99999"
        );
        assert!(LIVE.with(Cell::get) < 100_032);
        drop((result, output));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
