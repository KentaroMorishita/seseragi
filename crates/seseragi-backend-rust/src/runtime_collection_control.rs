fn native_collection_control(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    if matches!(name, "std/collection::Next" | "std/collection::Done") {
        assert!(dictionaries.is_empty());
        let [value]: [V; 1] = arguments
            .try_into()
            .unwrap_or_else(|_| panic!("Rust ABI: ReduceStep constructor arity"));
        return V::adt(
            if name == "std/collection::Next" {
                "std/collection::Next"
            } else {
                "std/collection::Done"
            },
            value,
        );
    }
    assert_eq!(name, "std/collection::reduceUntil");
    let [dictionary] = dictionaries.as_slice() else {
        panic!("Rust ABI: reduceUntil Iterable")
    };
    let [initial, step, source]: [V; 3] = arguments
        .try_into()
        .unwrap_or_else(|_| panic!("Rust ABI: reduceUntil source arity"));
    let mut iterator = dictionary.field("iterate").apply(source);
    let mut accumulator = initial;
    loop {
        let pulled = native_iterator_operation("std/iterator::next", vec![iterator]);
        if pulled.tagged("std/prelude::Nothing") {
            return accumulator;
        }
        assert!(
            pulled.tagged("std/prelude::Just"),
            "Rust ABI: reduceUntil pulled Maybe"
        );
        let V::Tuple(pair) = pulled.payload() else {
            panic!("Rust ABI: reduceUntil pulled pair")
        };
        let [value, rest] = pair.get().as_slice() else {
            panic!("Rust ABI: reduceUntil pulled pair arity")
        };
        let result = step.apply(accumulator).apply(value.clone());
        if result.tagged("std/collection::Done") {
            return result.payload();
        }
        assert!(
            result.tagged("std/collection::Next"),
            "Rust ABI: reduceUntil callback control"
        );
        accumulator = result.payload();
        iterator = rest.clone();
    }
}

#[cfg(test)]
mod collection_control_tests {
    use super::*;
    fn add(accumulator: V, value: V) -> V {
        let (V::Int(a), V::Int(b)) = (accumulator, value) else {
            panic!()
        };
        V::Int(a + b)
    }
    fn successor(value: V) -> V {
        let V::Int(n) = value else { panic!() };
        V::Int(n + 1)
    }
    fn increment(counter: &V) {
        counter.write_reference(successor(counter.read_reference()));
    }

    #[test]
    fn reduce_until_evaluates_both_callback_stages_once_and_never_pulls_after_done() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let stages = V::reference(V::Int(0));
        let iterator = native_iterator_unfold(
            V::fun(vec![calls.clone()], |c, state| {
                increment(&c[0]);
                let V::Int(n) = state else { panic!() };
                assert!(n <= 2, "must not pull after Done");
                V::adt(
                    "std/prelude::Just",
                    V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
                )
            }),
            V::Int(1),
        );
        let callback = V::fun(vec![stages.clone()], |c, accumulator| {
            increment(&c[0]);
            V::fun(vec![accumulator, c[0].clone()], |c, value| {
                increment(&c[1]);
                let result = add(c[0].clone(), value.clone());
                V::adt(
                    if value.render() == "2" {
                        "std/collection::Done"
                    } else {
                        "std/collection::Next"
                    },
                    result,
                )
            })
        });
        let result = native_collection_control(
            "std/collection::reduceUntil",
            vec![native_iterable_dictionary("std/iterator::Iterable")],
            vec![V::Int(0), callback.clone(), iterator.clone()],
        );
        assert_eq!(result.render(), "3");
        assert_eq!(calls.read_reference().render(), "2");
        assert_eq!(stages.read_reference().render(), "4");
        drop((calls, stages, callback, iterator, result));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn empty_input_returns_the_identical_initial_value_and_completed_reduction_observes_the_terminal_once(
    ) {
        enable_cycle_collection();
        let initial = V::reference(V::Int(42));
        let result = native_collection_control(
            "std/collection::reduceUntil",
            vec![native_iterable_dictionary("std/array::Iterable")],
            vec![
                initial.clone(),
                V::fun(vec![], |_, _| panic!("empty must not call callback")),
                V::Array(Shared::new(vec![])),
            ],
        );
        assert_eq!(result.node_id(), initial.node_id());
        drop((initial, result));
        let calls = V::reference(V::Int(0));
        let iterator = native_iterator_unfold(
            V::fun(vec![calls.clone()], |c, state| {
                increment(&c[0]);
                let V::Int(n) = state else { panic!() };
                if n == 4 {
                    V::adt("std/prelude::Nothing", V::Unit)
                } else {
                    V::adt(
                        "std/prelude::Just",
                        V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
                    )
                }
            }),
            V::Int(1),
        );
        let callback = V::fun(vec![], |_, accumulator| {
            V::fun(vec![accumulator], |c, value| {
                V::adt("std/collection::Next", add(c[0].clone(), value))
            })
        });
        let result = native_collection_control(
            "std/collection::reduceUntil",
            vec![native_iterable_dictionary("std/iterator::Iterable")],
            vec![V::Int(0), callback, iterator],
        );
        assert_eq!(result.render(), "6");
        assert_eq!(calls.read_reference().render(), "4");
        drop((calls, result));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn done_retains_only_its_payload_and_releases_unconsumed_list_tails_and_iterator_frames() {
        enable_cycle_collection();
        let refs: Vec<_> = (0..4).map(|n| V::reference(V::Int(n))).collect();
        let ids: Vec<_> = refs.iter().map(|v| v.node_id().unwrap()).collect();
        let source = V::list_from_values(refs.clone());
        let callback = V::fun(vec![], |_, _| {
            V::fun(vec![], |_, value| V::adt("std/collection::Done", value))
        });
        let result = native_collection_control(
            "std/collection::reduceUntil",
            vec![native_iterable_dictionary("std/list::Iterable")],
            vec![V::reference(V::Int(-1)), callback, source.clone()],
        );
        drop((source, refs));
        clear_dictionaries();
        collect_cycles().unwrap();
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(NODES.with(|nodes| nodes.borrow().contains_key(id)), i == 0);
        }
        assert_eq!(result.read_reference().render(), "0");
        drop(result);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn iterate_next_and_both_callback_defects_release_captured_ref_cycles() {
        enable_cycle_collection();
        for phase in ["iterate", "next", "first-callback", "second-callback"] {
            let cell = V::reference(V::Unit);
            let stage = V::text(phase);
            let dictionary = V::dict(vec![(
                "iterate",
                V::fun(vec![cell.clone(), stage.clone()], |c, _| {
                    assert_ne!(c[1].render(), "iterate", "deliberate iterate defect");
                    native_iterator_unfold(
                        V::fun(c.to_vec(), |c, state| {
                            assert_ne!(c[1].render(), "next", "deliberate next defect");
                            V::adt(
                                "std/prelude::Just",
                                V::Tuple(Shared::new(vec![state.clone(), successor(state)])),
                            )
                        }),
                        V::Int(1),
                    )
                }),
            )]);
            let callback = V::fun(vec![cell.clone(), stage.clone()], |c, accumulator| {
                assert_ne!(
                    c[1].render(),
                    "first-callback",
                    "deliberate first callback defect"
                );
                V::fun(vec![c[0].clone(), c[1].clone(), accumulator], |_, _| {
                    panic!("deliberate second callback defect")
                })
            });
            cell.write_reference(V::Tuple(Shared::new(vec![
                dictionary.clone(),
                callback.clone(),
            ])));
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                native_collection_control(
                    "std/collection::reduceUntil",
                    vec![dictionary.clone()],
                    vec![V::Int(0), callback.clone(), V::Unit],
                )
            }))
            .is_err());
            drop((cell, stage, dictionary, callback));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
