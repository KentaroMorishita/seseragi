// The selected Reducible controls order and representation. Standard native
// providers share the same persistent cursors as their Iterable counterparts.
fn native_prelude_reducible_dictionary(identity: &'static str) -> V {
    cached(identity, || {
        let iterable = match identity {
            "std/array::Reducible" => "std/array::Iterable",
            "std/list::Reducible" => "std/list::Iterable",
            "std/map::Reducible" => "std/map::Iterable",
            "std/set::Reducible" => "std/set::Iterable",
            _ => panic!("Rust ABI: unproven Reducible provider"),
        };
        V::dict(vec![
            (
                "iterate",
                native_iterable_dictionary(iterable).field("iterate"),
            ),
            (
                "reduce",
                V::fun(vec![native_iterable_dictionary(iterable)], |c, initial| {
                    V::fun(vec![c[0].clone(), initial], |c, step| {
                        V::fun(vec![c[0].clone(), c[1].clone(), step], |c, source| {
                            let mut iterator = c[0].field("iterate").apply(source);
                            let mut accumulator = c[1].clone();
                            while let Some((value, rest)) = native_prelude_pull(iterator) {
                                accumulator = c[2].apply(accumulator).apply(value);
                                iterator = rest;
                            }
                            accumulator
                        })
                    })
                }),
            ),
        ])
    })
}

fn native_prelude_pull(iterator: V) -> Option<(V, V)> {
    let pulled = native_iterator_operation("std/iterator::next", vec![iterator]);
    if pulled.tagged("std/prelude::Nothing") {
        return None;
    }
    assert!(
        pulled.tagged("std/prelude::Just"),
        "Rust ABI: Prelude iterator Maybe"
    );
    let V::Tuple(pair) = pulled.payload() else {
        panic!("Rust ABI: Prelude iterator pair")
    };
    let [value, rest] = pair.get().as_slice() else {
        panic!("Rust ABI: Prelude iterator pair arity")
    };
    Some((value.clone(), rest.clone()))
}

fn native_prelude_for_each_step(action: V, iterator: V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![action, iterator], |c, _| {
            let Some((value, rest)) = native_prelude_pull(c[1].clone()) else {
                return V::leaf(vec![], |_, _| Ok(V::Unit));
            };
            V::bind(
                c[0].apply(value),
                V::fun(vec![c[0].clone(), rest], |c, _| {
                    native_prelude_for_each_step(c[0].clone(), c[1].clone())
                }),
            )
        })],
    )
}

fn native_prelude_collection(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    match name {
        "std/prelude::sum" => {
            let [reducible, zero, add] = dictionaries.as_slice() else {
                panic!("Rust ABI: sum dictionaries")
            };
            let [source]: [V; 1] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: sum arity"));
            reducible
                .field("reduce")
                .apply(zero.field("zero").apply(V::Unit))
                .apply(add.field("add"))
                .apply(source)
        }
        "std/prelude::join" => {
            let [reducible] = dictionaries.as_slice() else {
                panic!("Rust ABI: join dictionary")
            };
            let [separator, source]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: join arity"));
            // Persistent pieces keep each reduction step constant-time, even
            // when a provider retains or reuses an earlier accumulator.
            let step = V::fun(vec![], |_, state| {
                V::fun(vec![state], |c, value| {
                    native_text(&value);
                    V::list_cons(value, c[0].clone())
                })
            });
            let result = reducible
                .field("reduce")
                .apply(V::ListEmpty)
                .apply(step)
                .apply(source);
            // Only walk the accumulator here; the selected reduce remains
            // the sole traversal of the source collection.
            let mut pieces = Vec::new();
            let mut remaining = &result;
            while let Some(cell) = remaining.list_cell() {
                pieces.push(native_text(&cell.head));
                remaining = &cell.tail;
            }
            pieces.reverse();
            V::Text(Shared::new(pieces.join(native_text(&separator))))
        }
        "std/prelude::forEach" => {
            let [iterable] = dictionaries.as_slice() else {
                panic!("Rust ABI: forEach dictionary")
            };
            let [action, source]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: forEach arity"));
            V::effect(
                Instruction::Defer,
                vec![V::fun(vec![iterable.clone(), action, source], |c, _| {
                    native_prelude_for_each_step(
                        c[1].clone(),
                        c[0].field("iterate").apply(c[2].clone()),
                    )
                })],
            )
        }
        _ => panic!("Rust ABI: unproven Prelude collection operation"),
    }
}

#[cfg(test)]
mod prelude_collection_tests {
    use super::*;

    #[test]
    fn join_large_collection_uses_selected_reduce_order_and_releases_pieces() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        // This provider exposes only reduce and chooses reverse source order.
        let reducible = V::dict(vec![(
            "reduce",
            V::fun(vec![calls.clone()], |c, initial| {
                c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                V::fun(vec![initial], |c, step| {
                    V::fun(vec![c[0].clone(), step], |c, source| {
                        let V::Array(values) = source else { panic!() };
                        values
                            .get()
                            .iter()
                            .rev()
                            .fold(c[0].clone(), |state, value| {
                                c[1].apply(state).apply(value.clone())
                            })
                    })
                })
            }),
        )]);
        let mut values = vec![V::text("x"); 100_000];
        values[0] = V::text("a");
        values[99_999] = V::text("z");
        let output = native_prelude_collection(
            "std/prelude::join",
            vec![reducible.clone()],
            vec![V::text(","), V::Array(Shared::new(values))],
        );
        assert_eq!(native_text(&output).len(), 199_999);
        assert!(native_text(&output).starts_with("z,x,"));
        assert!(native_text(&output).ends_with(",x,a"));
        assert_eq!(calls.read_reference().render(), "1");
        let empty = native_prelude_collection(
            "std/prelude::join",
            vec![reducible],
            vec![V::text(","), V::Array(Shared::new(vec![]))],
        );
        assert_eq!(native_text(&empty), "");
        assert_eq!(calls.read_reference().render(), "2");
        drop((output, empty, calls));
        clear_dictionaries();
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn for_each_is_cold_replays_and_never_pulls_after_failure_or_defect() {
        enable_cycle_collection();
        for defect in [false, true] {
            let log = V::reference(V::Int(0));
            let iterable = V::dict(vec![(
                "iterate",
                V::fun(vec![log.clone()], |c, _| {
                    c[0].write_reference(binary("+", c[0].read_reference(), V::Int(10)));
                    native_iterator_unfold(
                        V::fun(vec![c[0].clone()], |c, state| {
                            assert_eq!(state.render(), "0", "must not pull after failed action");
                            c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                            V::adt(
                                "std/prelude::Just",
                                V::Tuple(Shared::new(vec![state, V::Int(1)])),
                            )
                        }),
                        V::Int(0),
                    )
                }),
            )]);
            let action = V::fun(vec![V::Bool(defect)], |c, _| {
                V::leaf(vec![c[0].clone()], |c, _| {
                    if c[0].boolean() {
                        panic!("callback defect")
                    }
                    Err(NativeExit::Failure(V::text("typed")))
                })
            });
            let plan = native_prelude_collection(
                "std/prelude::forEach",
                vec![iterable],
                vec![action, V::Unit],
            );
            assert_eq!(log.read_reference().render(), "0");
            for run in 1..=2 {
                let outcome = NativeContext {
                    gc_every: 1,
                    ..NativeContext::default()
                }
                .root(plan.clone());
                if defect {
                    assert!(matches!(outcome, Err(NativeExit::Defect(_))));
                } else {
                    assert!(
                        matches!(outcome, Err(NativeExit::Failure(value)) if value.render() == "typed")
                    );
                }
                assert_eq!(log.read_reference().render(), (run * 11).to_string());
            }
            drop((plan, log));
            clear_dictionaries();
            assert_eq!(collect_cycles().unwrap().reclaimed, 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn for_each_large_iterator_bounds_frames_and_releases_cursors() {
        enable_cycle_collection();
        let iterator = native_iterator_unfold(
            V::fun(vec![], |_, state| {
                let V::Int(n) = state else { panic!() };
                if n == 100_000 {
                    V::adt("std/prelude::Nothing", V::Unit)
                } else {
                    V::adt(
                        "std/prelude::Just",
                        V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
                    )
                }
            }),
            V::Int(0),
        );
        let plan = native_prelude_collection(
            "std/prelude::forEach",
            vec![native_iterable_dictionary("std/iterator::Iterable")],
            vec![
                V::fun(vec![], |_, _| V::leaf(vec![], |_, _| Ok(V::Unit))),
                iterator,
            ],
        );
        let mut context = NativeContext::default();
        assert!(matches!(context.root(plan), Ok(V::Unit)));
        assert!(context.peak_frames <= 2, "{}", context.peak_frames);
        clear_dictionaries();
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
