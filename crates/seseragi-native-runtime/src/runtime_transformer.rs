fn native_transformer_wrap(run: V) -> V {
    V::Record(Shared::new(BTreeMap::from([("run".into(), run)])))
}
pub fn native_maybe_transformer_callable(name: &str, dictionaries: Vec<V>) -> V {
    match name {
        "std/transformer/maybe::run" => V::fun(vec![], |_, value| value.field("run")),
        "std/transformer/maybe::fromMaybe" => V::fun(dictionaries, |captures, value| {
            native_transformer_wrap(captures[0].field("pure").apply(value))
        }),
        "std/transformer/maybe::lift" => V::fun(dictionaries, |captures, value| {
            let just = V::fun(vec![], |_, item| V::adt("std/prelude::Just", item));
            native_transformer_wrap(captures[0].field("map").apply(just).apply(value))
        }),
        _ => panic!("Rust ABI: unknown MaybeT operation"),
    }
}
pub fn native_reader_transformer_callable(name: &str, dictionaries: Vec<V>) -> V {
    match name {
        "std/transformer/reader::run" => V::fun(vec![], |_, environment| {
            V::fun(vec![environment], |captures, value| {
                value.field("run").apply(captures[0].clone())
            })
        }),
        "std/transformer/reader::ask" => V::fun(dictionaries, |captures, _| {
            native_transformer_wrap(V::fun(vec![captures[0].clone()], |captures, environment| {
                captures[0].field("pure").apply(environment)
            }))
        }),
        "std/transformer/reader::asks" => V::fun(dictionaries, |captures, callback| {
            native_transformer_wrap(V::fun(
                vec![captures[0].clone(), callback],
                |captures, environment| {
                    captures[0].field("pure").apply(captures[1].apply(environment))
                },
            ))
        }),
        "std/transformer/reader::local" => V::fun(vec![], |_, callback| {
            V::fun(vec![callback], |captures, value| {
                native_transformer_wrap(V::fun(
                    vec![captures[0].clone(), value],
                    |captures, environment| {
                        captures[1].field("run").apply(captures[0].apply(environment))
                    },
                ))
            })
        }),
        "std/transformer/reader::lift" => V::fun(vec![], |_, value| {
            native_transformer_wrap(V::fun(vec![value], |captures, _| captures[0].clone()))
        }),
        _ => panic!("Rust ABI: unknown ReaderT operation"),
    }
}
fn native_transformer_pure(kind: i64, children: &[V], value: V) -> V {
    let base = children[0].clone();
    let run = match kind {
        9 | 10 => base.field("pure").apply(V::adt(
            if kind == 9 {
                "std/prelude::Just"
            } else {
                "std/prelude::Right"
            },
            value,
        )),
        11 => V::fun(vec![base, value], |captures, _| {
            captures[0].field("pure").apply(captures[1].clone())
        }),
        12 => V::fun(vec![base, value], |captures, state| {
            captures[0]
                .field("pure")
                .apply(V::Tuple(Shared::new(vec![captures[1].clone(), state])))
        }),
        13 => base.field("pure").apply(V::Tuple(Shared::new(vec![
            value,
            children[1].field("empty").apply(V::Unit),
        ]))),
        _ => panic!("Rust ABI: transformer pure"),
    };
    native_transformer_wrap(run)
}
fn native_transformer_bind(kind: i64, children: &[V], callback: V, source: V) -> V {
    let base = children[0].clone();
    let run = match kind {
        9 | 10 => {
            let next = V::fun(
                vec![V::Int(kind), base.clone(), callback],
                |captures, item| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    if item.tagged(if kind == 9 {
                        "std/prelude::Nothing"
                    } else {
                        "std/prelude::Left"
                    }) {
                        captures[1].field("pure").apply(item)
                    } else {
                        captures[2].apply(item.payload()).field("run")
                    }
                },
            );
            base.field("flatMap").apply(next).apply(source.field("run"))
        }
        11 | 12 => V::fun(
            vec![V::Int(kind), base, callback, source],
            |captures, state| {
                let next = V::fun(
                    vec![captures[0].clone(), captures[2].clone(), state.clone()],
                    |captures, item| {
                        let V::Int(kind) = captures[0] else {
                            unreachable!()
                        };
                        if kind == 11 {
                            captures[1]
                                .apply(item)
                                .field("run")
                                .apply(captures[2].clone())
                        } else {
                            captures[1]
                                .apply(item.index(0))
                                .field("run")
                                .apply(item.index(1))
                        }
                    },
                );
                captures[1]
                    .field("flatMap")
                    .apply(next)
                    .apply(captures[3].field("run").apply(state))
            },
        ),
        13 => {
            let next = V::fun(
                vec![base.clone(), children[1].clone(), callback],
                |captures, item| {
                    let append = V::fun(
                        vec![captures[1].clone(), item.index(1)],
                        |captures, result| {
                            V::Tuple(Shared::new(vec![
                                result.index(0),
                                captures[0]
                                    .field("append")
                                    .apply(captures[1].clone())
                                    .apply(result.index(1)),
                            ]))
                        },
                    );
                    captures[0]
                        .field("map")
                        .apply(append)
                        .apply(captures[2].apply(item.index(0)).field("run"))
                },
            );
            base.field("flatMap").apply(next).apply(source.field("run"))
        }
        _ => panic!("Rust ABI: transformer flatMap"),
    };
    native_transformer_wrap(run)
}
fn native_transformer_map(kind: i64, children: &[V], callback: V, source: V) -> V {
    let mut captures = vec![V::Int(kind), callback];
    captures.extend_from_slice(children);
    let next = V::fun(captures, |captures, value| {
        let V::Int(kind) = captures[0] else {
            unreachable!()
        };
        native_transformer_pure(kind, &captures[2..], captures[1].apply(value))
    });
    native_transformer_bind(kind, children, next, source)
}
pub fn native_transformer_dictionary(kind: i64, children: Vec<V>) -> V {
    let mut captures = vec![V::Int(kind)];
    captures.extend(children);
    V::dict(vec![
        (
            "pure",
            V::fun(captures.clone(), |captures, value| {
                let V::Int(kind) = captures[0] else {
                    unreachable!()
                };
                native_transformer_pure(kind, &captures[1..], value)
            }),
        ),
        (
            "flatMap",
            V::fun(captures.clone(), |captures, callback| {
                let mut next = captures.to_vec();
                next.push(callback);
                V::fun(next, |captures, source| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    native_transformer_bind(
                        kind,
                        &captures[1..captures.len() - 1],
                        captures.last().unwrap().clone(),
                        source,
                    )
                })
            }),
        ),
        (
            "map",
            V::fun(captures.clone(), |captures, callback| {
                let mut next = captures.to_vec();
                next.push(callback);
                V::fun(next, |captures, source| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    native_transformer_map(
                        kind,
                        &captures[1..captures.len() - 1],
                        captures.last().unwrap().clone(),
                        source,
                    )
                })
            }),
        ),
        (
            "apply",
            V::fun(captures, |captures, functions| {
                let mut next = captures.to_vec();
                next.push(functions);
                V::fun(next, |captures, values| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    let mut next = captures[..captures.len() - 1].to_vec();
                    next.push(values);
                    let callback = V::fun(next, |captures, function| {
                        let V::Int(kind) = captures[0] else {
                            unreachable!()
                        };
                        native_transformer_map(
                            kind,
                            &captures[1..captures.len() - 1],
                            function,
                            captures.last().unwrap().clone(),
                        )
                    });
                    native_transformer_bind(
                        kind,
                        &captures[1..captures.len() - 1],
                        callback,
                        captures.last().unwrap().clone(),
                    )
                })
            }),
        ),
    ])
}

#[cfg(test)]
mod reader_transformer_tests {
    use super::*;

    #[test]
    fn ask_asks_and_local_wait_for_run_and_use_the_selected_base_pure() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let projections = V::reference(V::Int(0));
        let changes = V::reference(V::Int(0));
        let base = V::dict(vec![(
            "pure",
            V::fun(vec![calls.clone()], |c, value| {
                c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                V::adt("test::Box", value)
            }),
        )]);
        let ask = native_reader_transformer_callable("std/transformer/reader::ask", vec![base.clone()])
            .apply(V::Unit);
        let asks = native_reader_transformer_callable("std/transformer/reader::asks", vec![base.clone()])
            .apply(V::fun(vec![projections.clone()], |c, environment| {
                c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                binary("*", environment, V::Int(2))
            }));
        let local = native_reader_transformer_callable("std/transformer/reader::local", vec![base.clone()])
            .apply(V::fun(vec![changes.clone()], |c, environment| {
                c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                binary("+", environment, V::Int(10))
            }))
            .apply(asks.clone());
        assert_eq!(calls.read_reference().render(), "0");
        assert_eq!(projections.read_reference().render(), "0");
        assert_eq!(changes.read_reference().render(), "0");
        let run = native_reader_transformer_callable("std/transformer/reader::run", vec![]);
        let environment = V::reference(V::Int(7));
        let result = run.apply(environment.clone()).apply(ask.clone());
        assert!(result.tagged("test::Box"));
        assert_eq!(result.payload().node_id(), environment.node_id());
        for (value, expected) in [(2, "24"), (5, "30")] {
            assert_eq!(run.apply(V::Int(value)).apply(local.clone()).payload().render(), expected);
        }
        assert_eq!(run.apply(V::Int(3)).apply(asks.clone()).payload().render(), "6");
        assert_eq!(calls.read_reference().render(), "4");
        assert_eq!(projections.read_reference().render(), "3");
        assert_eq!(changes.read_reference().render(), "2");
        drop((calls, projections, changes, base, ask, asks, local, run, environment, result));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn lift_preserves_the_base_identity_and_local_never_invokes_base_methods() {
        enable_cycle_collection();
        let base = V::dict(vec![("pure", V::fun(vec![], |_, _| panic!("unused base pure")))]);
        let value = V::adt("std/prelude::Left", V::reference(V::Int(7)));
        let lifted = native_reader_transformer_callable("std/transformer/reader::lift", vec![base.clone()])
            .apply(value.clone());
        let localized = native_reader_transformer_callable("std/transformer/reader::local", vec![base.clone()])
            .apply(V::fun(vec![], |_, value| value))
            .apply(lifted.clone());
        let run = native_reader_transformer_callable("std/transformer/reader::run", vec![]);
        for environment in [V::Unit, V::Int(0), V::text("雪🙂"), V::Array(Shared::new(vec![]))] {
            assert_eq!(run.apply(environment).apply(localized.clone()).node_id(), value.node_id());
        }
        drop((base, value, lifted, localized, run));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn lifted_effects_keep_failure_and_defect_distinct_and_release_captured_cycles() {
        enable_cycle_collection();
        for defect in [false, true] {
            let cell = V::reference(V::Unit);
            let base = native_monad_dictionary(5, "Monad");
            let lift = native_reader_transformer_callable("std/transformer/reader::lift", vec![base.clone()]);
            let source = V::leaf(vec![cell.clone(), V::Bool(defect)], |c, _| {
                if c[1].boolean() {
                    panic!("deliberate ReaderT base defect");
                }
                Err(NativeExit::failure(c[0].clone()))
            });
            let work = lift.apply(source);
            cell.write_reference(work.clone());
            let effect = native_reader_transformer_callable("std/transformer/reader::run", vec![])
                .apply(V::Int(42))
                .apply(work.clone());
            let result = NativeContext::default().root(effect);
            match result {
                Err(NativeExit::Failure(error)) if !defect => assert_eq!(error.node_id(), cell.node_id()),
                Err(NativeExit::Defect(errors)) if defect => {
                    assert_eq!(errors.len(), 1);
                    assert_eq!(errors[0].value, "deliberate ReaderT base defect");
                }
                _ => panic!("ReaderT changed the base exit kind"),
            }
            drop((cell, base, lift, work));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}

#[cfg(test)]
mod maybe_transformer_tests {
    use super::*;

    #[test]
    fn run_preserves_the_base_value_and_from_maybe_and_lift_use_selected_methods() {
        enable_cycle_collection();
        let pure_calls = V::reference(V::Int(0));
        let map_calls = V::reference(V::Int(0));
        let dictionary = V::dict(vec![
            (
                "pure",
                V::fun(vec![pure_calls.clone()], |c, value| {
                    c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                    V::adt("test::Box", value)
                }),
            ),
            (
                "map",
                V::fun(vec![map_calls.clone()], |c, callback| {
                    V::fun(vec![c[0].clone(), callback], |c, value| {
                        c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
                        V::adt("test::Box", c[1].apply(value.payload()))
                    })
                }),
            ),
        ]);
        let value = V::adt("std/prelude::Nothing", V::Unit);
        let wrapped = native_maybe_transformer_callable(
            "std/transformer/maybe::fromMaybe",
            vec![dictionary.clone()],
        )
        .apply(value.clone());
        let run = native_maybe_transformer_callable("std/transformer/maybe::run", vec![])
            .apply(wrapped.clone());
        assert_eq!(run.node_id(), wrapped.field("run").node_id());
        assert_eq!(run.payload().node_id(), value.node_id());
        let lifted = native_maybe_transformer_callable(
            "std/transformer/maybe::lift",
            vec![dictionary.clone()],
        )
        .apply(V::adt("test::Box", V::Int(42)));
        assert_eq!(lifted.field("run").payload().payload().render(), "42");
        assert_eq!(pure_calls.read_reference().render(), "1");
        assert_eq!(map_calls.read_reference().render(), "1");
        drop((
            pure_calls, map_calls, dictionary, value, wrapped, run, lifted,
        ));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn nothing_skips_callbacks_and_base_failures_keep_their_payload_identity() {
        enable_cycle_collection();
        let base = native_monad_dictionary(3, "Monad");
        let dictionary = native_transformer_dictionary(9, vec![base.clone()]);
        let callback = V::fun(vec![], |_, _| panic!("Nothing callback must be skipped"));
        let absent = native_maybe_transformer_callable(
            "std/transformer/maybe::fromMaybe",
            vec![base.clone()],
        )
        .apply(V::adt("std/prelude::Nothing", V::Unit));
        let result = dictionary
            .field("flatMap")
            .apply(callback.clone())
            .apply(absent);
        assert!(result.field("run").tagged("std/prelude::Right"));
        assert!(result.field("run").payload().tagged("std/prelude::Nothing"));
        let failure = V::adt("std/prelude::Left", V::reference(V::Int(7)));
        let failed =
            native_maybe_transformer_callable("std/transformer/maybe::lift", vec![base.clone()])
                .apply(failure.clone());
        let bound = dictionary
            .field("flatMap")
            .apply(callback.clone())
            .apply(failed);
        assert_eq!(bound.field("run").node_id(), failure.node_id());
        drop((base, dictionary, callback, result, failure, bound));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn saved_lift_traces_dictionary_cycles_and_effect_failure_and_defect_remain_distinct() {
        enable_cycle_collection();
        for defect in [false, true] {
            let cell = V::reference(V::Unit);
            let dictionary = native_monad_dictionary(5, "Monad");
            let lift = native_maybe_transformer_callable(
                "std/transformer/maybe::lift",
                vec![dictionary.clone()],
            );
            let source = V::leaf(vec![cell.clone(), V::Bool(defect)], |c, _| {
                if c[1].boolean() {
                    panic!("deliberate MaybeT base defect");
                }
                Err(NativeExit::failure(c[0].clone()))
            });
            let work = lift.apply(source);
            cell.write_reference(work.clone());
            let result = NativeContext::default().root(work.field("run"));
            match result {
                Err(NativeExit::Failure(error)) if !defect => {
                    assert_eq!(error.node_id(), cell.node_id())
                }
                Err(NativeExit::Defect(errors)) if defect => {
                    assert_eq!(errors.len(), 1);
                    assert_eq!(errors[0].value, "deliberate MaybeT base defect");
                }
                _ => panic!("MaybeT changed the base exit kind"),
            }
            drop((cell, dictionary, lift, work));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
