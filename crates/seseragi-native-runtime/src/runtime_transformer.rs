fn native_transformer_wrap(run: V) -> V {
    V::Record(Shared::new(BTreeMap::from([("run".into(), run)])))
}

pub fn native_writer_transformer_callable(name: &str, dictionaries: Vec<V>) -> V {
    match name {
        "std/transformer/writer::run" => V::fun(vec![], |_, value| value.field("run")),
        "std/transformer/writer::tell" => V::fun(dictionaries, |captures, output| {
            native_transformer_wrap(
                captures[0]
                    .field("pure")
                    .apply(V::Tuple(Shared::new(vec![V::Unit, output]))),
            )
        }),
        "std/transformer/writer::listen" => V::fun(dictionaries, |captures, value| {
            let listen = V::fun(vec![], |_, pair| {
                let output = pair.index(1);
                V::Tuple(Shared::new(vec![
                    V::Tuple(Shared::new(vec![pair.index(0), output.clone()])),
                    output,
                ]))
            });
            native_transformer_wrap(
                captures[0]
                    .field("map")
                    .apply(listen)
                    .apply(value.field("run")),
            )
        }),
        "std/transformer/writer::lift" => V::fun(dictionaries, |captures, value| {
            let pair = V::fun(vec![captures[1].clone()], |captures, item| {
                V::Tuple(Shared::new(vec![
                    item,
                    captures[0].field("empty").apply(V::Unit),
                ]))
            });
            native_transformer_wrap(captures[0].field("map").apply(pair).apply(value))
        }),
        _ => panic!("Rust ABI: unknown WriterT operation"),
    }
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
pub fn native_either_transformer_operation(name: &str, dictionaries: Vec<V>, value: V) -> V {
    if name == "std/transformer/either::run" {
        assert!(dictionaries.is_empty(), "Rust ABI: EitherT run dictionary");
        return value.field("run");
    }
    let [base]: [V; 1] = dictionaries
        .try_into()
        .unwrap_or_else(|_| panic!("Rust ABI: EitherT Monad dictionary"));
    let run = match name {
        "std/transformer/either::fromEither" => base.field("pure").apply(value),
        "std/transformer/either::lift" => base
            .field("map")
            .apply(V::fun(vec![], |_, value| {
                V::adt("std/prelude::Right", value)
            }))
            .apply(value),
        _ => panic!("Rust ABI: unsupported EitherT operation"),
    };
    native_transformer_wrap(run)
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
pub fn native_state_transformer_callable(name: &str, dictionaries: Vec<V>) -> V {
    match name {
        "std/transformer/state::run" => V::fun(vec![], |_, initial| {
            V::fun(vec![initial], |captures, value| {
                value.field("run").apply(captures[0].clone())
            })
        }),
        "std/transformer/state::get" => V::fun(dictionaries, |captures, _| {
            native_transformer_wrap(V::fun(captures.to_vec(), |captures, state| {
                captures[0]
                    .field("pure")
                    .apply(V::Tuple(Shared::new(vec![state.clone(), state])))
            }))
        }),
        "std/transformer/state::put" => V::fun(dictionaries, |captures, next| {
            native_transformer_wrap(V::fun(vec![captures[0].clone(), next], |captures, _| {
                captures[0]
                    .field("pure")
                    .apply(V::Tuple(Shared::new(vec![V::Unit, captures[1].clone()])))
            }))
        }),
        "std/transformer/state::modify" => V::fun(dictionaries, |captures, callback| {
            native_transformer_wrap(V::fun(
                vec![captures[0].clone(), callback],
                |captures, state| {
                    captures[0].field("pure").apply(V::Tuple(Shared::new(vec![
                        V::Unit,
                        captures[1].apply(state),
                    ])))
                },
            ))
        }),
        "std/transformer/state::lift" => V::fun(dictionaries, |captures, source| {
            native_transformer_wrap(V::fun(
                vec![captures[0].clone(), source],
                |captures, state| {
                    let pair = V::fun(vec![state], |captures, value| {
                        V::Tuple(Shared::new(vec![value, captures[0].clone()]))
                    });
                    captures[0]
                        .field("map")
                        .apply(pair)
                        .apply(captures[1].clone())
                },
            ))
        }),
        _ => panic!("Rust ABI: unknown StateT operation"),
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

#[cfg(test)]
mod either_transformer_tests {
    use super::*;

    fn run(value: V) -> V {
        native_either_transformer_operation("std/transformer/either::run", vec![], value)
    }

    fn cleanup() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn from_either_and_run_preserve_callable_payloads_and_release_cycles() {
        enable_cycle_collection();
        let root = V::reference(V::Unit);
        root.write_reference(root.clone());
        let callback = V::fun(vec![root.clone()], |_, value| value);
        let base = native_monad_dictionary(2, "Monad");
        for tag in ["std/prelude::Left", "std/prelude::Right"] {
            let either = V::adt(tag, callback.clone());
            let transformed = native_either_transformer_operation(
                "std/transformer/either::fromEither",
                vec![base.clone()],
                either.clone(),
            );
            let stored = transformed.field("run");
            let result = run(transformed);
            assert_eq!(result.node_id(), stored.node_id());
            assert!(result.tagged("std/prelude::Just"));
            assert_eq!(result.payload().node_id(), either.node_id());
            assert_eq!(result.payload().payload().node_id(), callback.node_id());
            assert_eq!(result.payload().payload().apply(V::Int(42)).render(), "42");
        }
        drop((root, callback, base));
        cleanup();
    }

    #[test]
    fn lift_uses_selected_map_and_from_either_uses_selected_pure_once() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let base = V::dict(vec![
            (
                "pure",
                V::fun(vec![calls.clone()], |captures, value| {
                    captures[0].write_reference(binary(
                        "+",
                        captures[0].read_reference(),
                        V::Int(1),
                    ));
                    V::adt("std/prelude::Just", value)
                }),
            ),
            (
                "map",
                V::fun(vec![calls.clone()], |captures, callback| {
                    V::fun(vec![captures[0].clone(), callback], |captures, value| {
                        captures[0].write_reference(binary(
                            "+",
                            captures[0].read_reference(),
                            V::Int(10),
                        ));
                        V::adt("std/prelude::Just", captures[1].apply(value))
                    })
                }),
            ),
        ]);
        let wrapped = native_either_transformer_operation(
            "std/transformer/either::fromEither",
            vec![base.clone()],
            V::adt("std/prelude::Left", V::Unit),
        );
        assert_eq!(calls.read_reference().render(), "1");
        assert!(run(wrapped).payload().tagged("std/prelude::Left"));
        let lifted = native_either_transformer_operation(
            "std/transformer/either::lift",
            vec![base.clone()],
            V::Int(42),
        );
        assert_eq!(calls.read_reference().render(), "11");
        let result = run(lifted).payload();
        assert!(result.tagged("std/prelude::Right"));
        assert_eq!(result.payload().render(), "42");
        assert_eq!(calls.read_reference().render(), "11");
        drop((calls, base, result));
        cleanup();
    }

    #[test]
    fn effect_lift_and_instances_are_cold_repeatable_and_preserve_base_exit_kinds() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let base = native_monad_dictionary(5, "Monad");
        let dictionary = native_transformer_dictionary(10, vec![base.clone()]);
        let source = V::leaf(vec![calls.clone()], |captures, _| {
            captures[0].write_reference(binary("+", captures[0].read_reference(), V::Int(1)));
            Ok(V::Int(41))
        });
        let lifted = native_either_transformer_operation(
            "std/transformer/either::lift",
            vec![base.clone()],
            source,
        );
        let mapped = dictionary
            .field("map")
            .apply(V::fun(vec![], |_, value| binary("+", value, V::Int(1))))
            .apply(lifted);
        let plan = run(mapped);
        assert_eq!(calls.read_reference().render(), "0");
        let mut context = NativeContext::default();
        for expected in [1, 2] {
            let result = context
                .root(plan.clone())
                .unwrap_or_else(|_| panic!("expected success"));
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(result.payload().render(), "42");
            assert_eq!(calls.read_reference().render(), expected.to_string());
        }
        for kind in [0, 1, 2] {
            let source = V::leaf(vec![V::Int(kind)], |captures, _| {
                Err(match captures[0] {
                    V::Int(0) => NativeExit::failure(V::text("typed")),
                    V::Int(1) => NativeExit::defects(vec!["defect".into()]),
                    _ => NativeExit::Cancelled,
                })
            });
            let lifted = native_either_transformer_operation(
                "std/transformer/either::lift",
                vec![base.clone()],
                source,
            );
            let mapped = dictionary
                .field("map")
                .apply(V::fun(vec![], |_, _| panic!("callback after base exit")))
                .apply(lifted);
            match (kind, context.root(run(mapped))) {
                (0, Err(NativeExit::Failure(error))) => assert_eq!(error.render(), "typed"),
                (1, Err(NativeExit::Defect(errors))) => assert_eq!(errors, ["defect"]),
                (2, Err(NativeExit::Cancelled)) => {}
                _ => panic!("base exit kind changed"),
            }
        }
        drop((context, calls, base, dictionary, plan));
        cleanup();
    }
}

#[cfg(test)]
mod state_transformer_tests {
    use super::*;

    fn run(initial: V, value: V) -> V {
        native_state_transformer_callable("std/transformer/state::run", vec![])
            .apply(initial)
            .apply(value)
    }

    #[test]
    fn operations_delay_selected_methods_until_run_and_preserve_state_and_value_identity() {
        enable_cycle_collection();
        let pure_calls = V::reference(V::Int(0));
        let map_calls = V::reference(V::Int(0));
        let modify_calls = V::reference(V::Int(0));
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
        let state = V::reference(V::Int(7));
        let value = V::reference(V::Int(42));
        let get = native_state_transformer_callable(
            "std/transformer/state::get",
            vec![dictionary.clone()],
        )
        .apply(V::Unit);
        let put = native_state_transformer_callable(
            "std/transformer/state::put",
            vec![dictionary.clone()],
        )
        .apply(state.clone());
        let modify = native_state_transformer_callable(
            "std/transformer/state::modify",
            vec![dictionary.clone()],
        )
        .apply(V::fun(vec![modify_calls.clone()], |c, state| {
            c[0].write_reference(binary("+", c[0].read_reference(), V::Int(1)));
            binary("+", state, V::Int(1))
        }));
        let lift = native_state_transformer_callable(
            "std/transformer/state::lift",
            vec![dictionary.clone()],
        )
        .apply(V::adt("test::Box", value.clone()));
        for counter in [&pure_calls, &map_calls, &modify_calls] {
            assert_eq!(counter.read_reference().render(), "0");
        }
        let observed = run(state.clone(), get.clone()).payload();
        assert_eq!(observed.index(0).node_id(), state.node_id());
        assert_eq!(observed.index(1).node_id(), state.node_id());
        let set = run(V::Int(-1), put.clone()).payload();
        assert_eq!(set.index(0).render(), "()");
        assert_eq!(set.index(1).node_id(), state.node_id());
        assert_eq!(run(V::Int(5), modify.clone()).payload().render(), "((), 6)");
        assert_eq!(
            run(V::Int(9), modify.clone()).payload().render(),
            "((), 10)"
        );
        let lifted = run(state.clone(), lift.clone()).payload();
        assert_eq!(lifted.index(0).node_id(), value.node_id());
        assert_eq!(lifted.index(1).node_id(), state.node_id());
        assert_eq!(pure_calls.read_reference().render(), "4");
        assert_eq!(map_calls.read_reference().render(), "1");
        assert_eq!(modify_calls.read_reference().render(), "2");
        drop((
            pure_calls,
            map_calls,
            modify_calls,
            dictionary,
            state,
            value,
            get,
            put,
            modify,
            lift,
            observed,
            set,
            lifted,
        ));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn base_failure_skips_state_updates_and_keeps_its_payload_identity() {
        enable_cycle_collection();
        let base = native_monad_dictionary(3, "Monad");
        let failure = V::adt("std/prelude::Left", V::reference(V::Int(7)));
        let lifted =
            native_state_transformer_callable("std/transformer/state::lift", vec![base.clone()])
                .apply(failure.clone());
        let dictionary = native_transformer_dictionary(12, vec![base.clone()]);
        let work = dictionary
            .field("flatMap")
            .apply(V::fun(vec![], |_, _| panic!("failed StateT callback")))
            .apply(lifted);
        let result = run(V::Int(0), work.clone());
        assert_eq!(result.node_id(), failure.node_id());
        drop((base, failure, dictionary, work, result));
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn saved_lift_traces_state_cycles_and_keeps_effect_failure_and_defect_distinct() {
        enable_cycle_collection();
        for defect in [false, true] {
            let cell = V::reference(V::Unit);
            let dictionary = native_monad_dictionary(5, "Monad");
            let lift = native_state_transformer_callable(
                "std/transformer/state::lift",
                vec![dictionary.clone()],
            );
            let source = V::leaf(vec![cell.clone(), V::Bool(defect)], |c, _| {
                if c[1].boolean() {
                    panic!("deliberate StateT base defect");
                }
                Err(NativeExit::failure(c[0].clone()))
            });
            let work = lift.apply(source);
            cell.write_reference(work.clone());
            let effect = run(cell.clone(), work.clone());
            let result = NativeContext::default().root(effect);
            match result {
                Err(NativeExit::Failure(error)) if !defect => {
                    assert_eq!(error.node_id(), cell.node_id());
                }
                Err(NativeExit::Defect(errors)) if defect => {
                    assert_eq!(errors.len(), 1);
                    assert_eq!(errors[0].value, "deliberate StateT base defect");
                }
                _ => panic!("StateT changed the base exit kind"),
            }
            drop((cell, dictionary, lift, work));
            clear_dictionaries();
            assert!(collect_cycles().unwrap().reclaimed > 0);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}

#[cfg(test)]
mod writer_transformer_tests {
    use super::*;

    fn run(value: V) -> V {
        native_writer_transformer_callable("std/transformer/writer::run", vec![]).apply(value)
    }

    fn cleanup() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn tell_and_listen_use_selected_base_methods_and_preserve_callable_output_identity() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let base = V::dict(vec![
            (
                "pure",
                V::fun(vec![calls.clone()], |captures, value| {
                    captures[0].write_reference(binary(
                        "+",
                        captures[0].read_reference(),
                        V::Int(1),
                    ));
                    V::adt("test::Box", value)
                }),
            ),
            (
                "map",
                V::fun(vec![calls.clone()], |captures, callback| {
                    V::fun(vec![captures[0].clone(), callback], |captures, value| {
                        captures[0].write_reference(binary(
                            "+",
                            captures[0].read_reference(),
                            V::Int(10),
                        ));
                        V::adt("test::Box", captures[1].apply(value.payload()))
                    })
                }),
            ),
        ]);
        let output = V::fun(vec![calls.clone()], |_, value| value);
        let unused_monoid = V::dict(vec![
            (
                "empty",
                V::fun(vec![], |_, _| panic!("unexpected WriterT empty")),
            ),
            (
                "append",
                V::fun(vec![], |_, _| panic!("unexpected WriterT append")),
            ),
        ]);
        let tell = native_writer_transformer_callable(
            "std/transformer/writer::tell",
            vec![base.clone(), unused_monoid.clone()],
        );
        let written = tell.apply(output.clone());
        let stored = written.field("run");
        assert_eq!(run(written.clone()).node_id(), stored.node_id());
        assert_eq!(calls.read_reference().render(), "1");
        let pair = run(written.clone()).payload();
        assert_eq!(pair.index(0).render(), "()");
        assert_eq!(pair.index(1).node_id(), output.node_id());
        let listened = native_writer_transformer_callable(
            "std/transformer/writer::listen",
            vec![base.clone(), unused_monoid.clone()],
        )
        .apply(written.clone());
        let observed = run(listened).payload();
        assert_eq!(observed.index(0).index(0).render(), "()");
        assert_eq!(observed.index(0).index(1).node_id(), output.node_id());
        assert_eq!(observed.index(1).node_id(), output.node_id());
        assert_eq!(observed.index(1).apply(V::Int(42)).render(), "42");
        assert_eq!(calls.read_reference().render(), "11");
        drop((
            calls,
            base,
            output,
            unused_monoid,
            tell,
            written,
            stored,
            pair,
            observed,
        ));
        cleanup();
    }

    #[test]
    fn lift_skips_empty_for_failed_bases_and_calls_it_once_for_each_array_item() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let output = V::dict(vec![(
            "empty",
            V::fun(vec![calls.clone()], |captures, _| {
                let value = binary("+", captures[0].read_reference(), V::Int(1));
                captures[0].write_reference(value.clone());
                value
            }),
        )]);
        let lift = native_writer_transformer_callable(
            "std/transformer/writer::lift",
            vec![native_monad_dictionary(0, "Monad"), output.clone()],
        );
        assert_eq!(
            run(lift.apply(V::Array(Shared::new(vec![])))).render(),
            "[]"
        );
        assert_eq!(calls.read_reference().render(), "0");
        let values = V::Array(Shared::new(vec![V::Int(7), V::Int(8)]));
        assert_eq!(run(lift.apply(values)).render(), "[(7, 1), (8, 2)]");
        for (kind, failure) in [
            (2, V::adt("std/prelude::Nothing", V::Unit)),
            (3, V::adt("std/prelude::Left", V::text("typed"))),
        ] {
            let lift = native_writer_transformer_callable(
                "std/transformer/writer::lift",
                vec![native_monad_dictionary(kind, "Monad"), output.clone()],
            );
            assert_eq!(
                run(lift.apply(failure.clone())).node_id(),
                failure.node_id()
            );
        }
        assert_eq!(calls.read_reference().render(), "2");
        drop((calls, output, lift));
        cleanup();
    }

    #[test]
    fn effect_lift_and_listen_delay_empty_until_each_run_and_preserve_exit_kinds() {
        enable_cycle_collection();
        let calls = V::reference(V::Int(0));
        let base = native_monad_dictionary(5, "Monad");
        let output = V::dict(vec![(
            "empty",
            V::fun(vec![calls.clone()], |captures, _| {
                captures[0].write_reference(binary("+", captures[0].read_reference(), V::Int(1)));
                V::text("")
            }),
        )]);
        let lift = native_writer_transformer_callable(
            "std/transformer/writer::lift",
            vec![base.clone(), output.clone()],
        );
        let listen = native_writer_transformer_callable(
            "std/transformer/writer::listen",
            vec![base.clone(), output.clone()],
        );
        let source = V::leaf(vec![], |_, _| Ok(V::Int(42)));
        let effect = run(listen.apply(lift.apply(source)));
        assert_eq!(calls.read_reference().render(), "0");
        let mut context = NativeContext::default();
        for count in [1, 2] {
            let result = context
                .root(effect.clone())
                .unwrap_or_else(|_| panic!("expected success"));
            assert_eq!(result.render(), "((42, ), )");
            assert_eq!(calls.read_reference().render(), count.to_string());
        }
        for kind in [0, 1, 2] {
            let source = V::leaf(vec![V::Int(kind)], |captures, _| {
                Err(match captures[0] {
                    V::Int(0) => NativeExit::failure(V::text("typed")),
                    V::Int(1) => NativeExit::defects(vec!["defect".into()]),
                    _ => NativeExit::Cancelled,
                })
            });
            let work = run(listen.apply(lift.apply(source)));
            match (kind, context.root(work)) {
                (0, Err(NativeExit::Failure(error))) => assert_eq!(error.render(), "typed"),
                (1, Err(NativeExit::Defect(errors))) => assert_eq!(errors, ["defect"]),
                (2, Err(NativeExit::Cancelled)) => {}
                _ => panic!("WriterT changed the base exit kind"),
            }
        }
        assert_eq!(calls.read_reference().render(), "2");
        drop((calls, base, output, lift, listen, effect, context));
        cleanup();
    }

    #[test]
    fn saved_lift_and_writer_instances_trace_output_and_callback_cycles() {
        enable_cycle_collection();
        let root = V::reference(V::Unit);
        let output = V::dict(vec![
            (
                "empty",
                V::fun(vec![root.clone()], |captures, _| captures[0].clone()),
            ),
            (
                "append",
                V::fun(vec![], |_, first| {
                    V::fun(vec![first], |captures, _| captures[0].clone())
                }),
            ),
        ]);
        let base = native_monad_dictionary(2, "Monad");
        let dictionary = native_transformer_dictionary(13, vec![base.clone(), output.clone()]);
        let lift = native_writer_transformer_callable(
            "std/transformer/writer::lift",
            vec![base.clone(), output.clone()],
        );
        let callback = V::fun(vec![root.clone()], |captures, value| {
            binary("+", value, captures[0].read_reference().index(0))
        });
        root.write_reference(V::Tuple(Shared::new(vec![
            V::Int(1),
            dictionary.clone(),
            lift.clone(),
            callback.clone(),
        ])));
        let mapped = dictionary
            .field("map")
            .apply(callback.clone())
            .apply(lift.apply(V::adt("std/prelude::Just", V::Int(41))));
        let result = run(mapped.clone()).payload();
        assert_eq!(result.index(0).render(), "42");
        assert_eq!(result.index(1).node_id(), root.node_id());
        drop((
            root, output, base, dictionary, lift, callback, mapped, result,
        ));
        clear_dictionaries();
        assert!(collect_cycles().unwrap().reclaimed > 0);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
