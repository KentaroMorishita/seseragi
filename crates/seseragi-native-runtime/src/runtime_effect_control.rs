pub fn native_effect_control(name: &str, dictionaries: Vec<V>, arguments: Vec<V>) -> V {
    match name {
        "std/effect::Continue" | "std/effect::Break" => {
            assert!(dictionaries.is_empty() && arguments.is_empty());
            V::adt(
                if name == "std/effect::Continue" {
                    "std/effect::Continue"
                } else {
                    "std/effect::Break"
                },
                V::Unit,
            )
        }
        "std/effect::attempt" => {
            assert!(dictionaries.is_empty());
            let [_] = arguments.as_slice() else {
                panic!("Rust ABI: attempt source arity")
            };
            V::effect(Instruction::Attempt, arguments)
        }
        "std/effect::fromMaybe" => {
            assert!(dictionaries.is_empty());
            let [error, value]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: fromMaybe source arity"));
            if value.tagged("std/prelude::Nothing") {
                native_operation("effect.fail", vec![error])
            } else {
                assert!(
                    value.tagged("std/prelude::Just"),
                    "Rust ABI: fromMaybe owner"
                );
                native_operation("effect.succeed", vec![value.payload()])
            }
        }
        "std/effect::forEachUntil" => {
            let [dictionary]: [V; 1] = dictionaries
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: forEachUntil Iterable"));
            let [action, values]: [V; 2] = arguments
                .try_into()
                .unwrap_or_else(|_| panic!("Rust ABI: forEachUntil source arity"));
            V::effect(
                Instruction::Defer,
                vec![V::fun(vec![dictionary, action, values], |c, _| {
                    let iterator = c[0].field("iterate").apply(c[2].clone());
                    native_effect_for_each_until_step(c[1].clone(), iterator)
                })],
            )
        }
        _ => panic!("Rust ABI: unsupported Effect control {name}"),
    }
}

fn native_effect_for_each_until_step(action: V, iterator: V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![action, iterator], |c, _| {
            let pulled = native_iterator_operation("std/iterator::next", vec![c[1].clone()]);
            if pulled.tagged("std/prelude::Nothing") {
                return native_operation("effect.succeed", vec![V::Unit]);
            }
            assert!(
                pulled.tagged("std/prelude::Just"),
                "Rust ABI: forEachUntil Maybe"
            );
            let V::Tuple(pair) = pulled.payload() else {
                panic!("Rust ABI: forEachUntil pair")
            };
            let [value, rest] = pair.get().as_slice() else {
                panic!("Rust ABI: forEachUntil pair arity")
            };
            V::bind(
                c[0].apply(value.clone()),
                V::fun(vec![c[0].clone(), rest.clone()], |c, control| {
                    if control.tagged("std/effect::Break") {
                        native_operation("effect.succeed", vec![V::Unit])
                    } else {
                        assert!(
                            control.tagged("std/effect::Continue"),
                            "Rust ABI: forEachUntil LoopControl"
                        );
                        native_effect_for_each_until_step(c[0].clone(), c[1].clone())
                    }
                }),
            )
        })],
    )
}

#[cfg(test)]
mod effect_control_tests {
    use super::*;

    fn operation(name: &str, arguments: Vec<V>) -> V {
        native_effect_control(name, vec![], arguments)
    }
    fn attempt(plan: V) -> V {
        operation("std/effect::attempt", vec![plan])
    }
    fn succeed(value: V) -> V {
        native_operation("effect.succeed", vec![value])
    }
    fn increment(counter: &V) {
        counter.write_reference(binary("+", counter.read_reference(), V::Int(1)));
    }
    fn finish() {
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn from_maybe_and_attempt_retain_identical_payloads_and_rerun_cold_effects() {
        enable_cycle_collection();
        for missing in [false, true] {
            let error = V::reference(V::Int(9));
            let payload = V::reference(V::Int(42));
            let plan = operation(
                "std/effect::fromMaybe",
                vec![
                    error.clone(),
                    V::adt(
                        if missing {
                            "std/prelude::Nothing"
                        } else {
                            "std/prelude::Just"
                        },
                        if missing { V::Unit } else { payload.clone() },
                    ),
                ],
            );
            let captured = attempt(plan);
            for _ in 0..2 {
                let result = NativeContext::default()
                    .root(captured.clone())
                    .ok()
                    .unwrap();
                assert!(result.tagged(if missing {
                    "std/prelude::Left"
                } else {
                    "std/prelude::Right"
                }));
                assert_eq!(
                    result.payload().node_id(),
                    if missing {
                        error.node_id()
                    } else {
                        payload.node_id()
                    }
                );
                assert_eq!(
                    result.payload().read_reference().render(),
                    if missing { "9" } else { "42" }
                );
            }
            drop((error, payload, captured));
            finish();
        }
        let counter = V::reference(V::Int(0));
        let plan = attempt(V::leaf(vec![counter.clone()], |c, _| {
            increment(&c[0]);
            Ok(c[0].read_reference())
        }));
        assert_eq!(counter.read_reference().render(), "0");
        for expected in ["1", "2"] {
            assert_eq!(
                NativeContext::default()
                    .root(plan.clone())
                    .ok()
                    .unwrap()
                    .payload()
                    .render(),
                expected
            );
        }
        drop((counter, plan));
        finish();
    }

    #[test]
    fn from_maybe_success_does_not_keep_the_unused_failure_alive() {
        enable_cycle_collection();
        let unused = V::reference(V::Unit);
        let id = unused.node_id().unwrap();
        let plan = operation(
            "std/effect::fromMaybe",
            vec![unused, V::adt("std/prelude::Just", V::Unit)],
        );
        assert!(!NODES.with(|nodes| nodes.borrow().contains_key(&id)));
        assert!(matches!(NativeContext::default().root(plan), Ok(V::Unit)));
        finish();
    }

    #[test]
    fn attempt_propagates_defects_cancellation_and_finalizer_defects() {
        enable_cycle_collection();
        for (plan, defect) in [
            (native_effect_die(V::text("body defect")), true),
            (V::leaf(vec![], |_, _| Err(NativeExit::Cancelled)), false),
            (
                V::leaf(vec![], |_, context| {
                    context.cancelled = true;
                    Ok(V::Unit)
                }),
                false,
            ),
            (
                V::leaf(vec![], |_, context| {
                    context.cancelled = true;
                    Err(NativeExit::failure(V::text("typed")))
                }),
                false,
            ),
        ] {
            match NativeContext::default().root(attempt(plan)) {
                Err(NativeExit::Defect(errors)) if defect => {
                    assert_eq!(errors[0].value, "body defect")
                }
                Err(NativeExit::Cancelled) if !defect => {}
                _ => panic!("attempt changed the exit kind"),
            }
        }
        let acquire = native_operation(
            "std/effect::acquireRelease",
            vec![
                succeed(V::Unit),
                V::fun(vec![], |_, _| native_effect_die(V::text("release defect"))),
            ],
        );
        let body = V::bind(
            acquire,
            V::fun(vec![], |_, _| {
                native_operation("effect.fail", vec![V::text("typed")])
            }),
        );
        let scoped = native_operation("std/effect::scoped", vec![body]);
        let Err(NativeExit::Defect(errors)) = NativeContext::default().root(attempt(scoped)) else {
            panic!("attempt must propagate cleanup defects")
        };
        assert_eq!(
            errors
                .iter()
                .map(|error| error.value.as_str())
                .collect::<Vec<_>>(),
            ["release defect"]
        );
        drop(errors);
        finish();
    }

    #[test]
    fn for_each_until_is_cold_replayable_and_never_pulls_after_break() {
        enable_cycle_collection();
        let iterates = V::reference(V::Int(0));
        let pulls = V::reference(V::Int(0));
        let calls = V::reference(V::Int(0));
        let dictionary = V::dict(vec![(
            "iterate",
            V::fun(vec![iterates.clone(), pulls.clone()], |c, _| {
                increment(&c[0]);
                native_iterator_unfold(
                    V::fun(vec![c[1].clone()], |c, state| {
                        increment(&c[0]);
                        let V::Int(n) = state else { panic!() };
                        assert!(n <= 3, "pulled after Break");
                        V::adt(
                            "std/prelude::Just",
                            V::Tuple(Shared::new(vec![V::Int(n), V::Int(n + 1)])),
                        )
                    }),
                    V::Int(1),
                )
            }),
        )]);
        let action = V::fun(vec![calls.clone()], |c, value| {
            V::leaf(vec![c[0].clone(), value], |c, _| {
                increment(&c[0]);
                Ok(operation(
                    if c[1].render() == "3" {
                        "std/effect::Break"
                    } else {
                        "std/effect::Continue"
                    },
                    vec![],
                ))
            })
        });
        let plan = native_effect_control(
            "std/effect::forEachUntil",
            vec![dictionary],
            vec![action, V::Unit],
        );
        for count in ["0", "3", "6"] {
            assert_eq!(pulls.read_reference().render(), count);
            assert_eq!(calls.read_reference().render(), count);
            if count != "6" {
                assert!(matches!(
                    NativeContext::default().root(plan.clone()),
                    Ok(V::Unit)
                ));
            }
        }
        assert_eq!(iterates.read_reference().render(), "2");
        drop((plan, iterates, pulls, calls));
        finish();
    }

    #[test]
    fn for_each_until_stops_on_failure_defect_or_cancellation_and_releases_resources() {
        enable_cycle_collection();
        for outcome in ["failure", "defect", "cancelled"] {
            let calls = V::reference(V::Int(0));
            let releases = V::reference(V::Int(0));
            let action = V::fun(
                vec![calls.clone(), releases.clone(), V::text(outcome)],
                |c, value| {
                    let acquire = native_operation(
                        "std/effect::acquireRelease",
                        vec![
                            succeed(V::Unit),
                            V::fun(vec![c[1].clone()], |c, _| {
                                V::leaf(c.to_vec(), |c, context| {
                                    assert!(context.masked);
                                    increment(&c[0]);
                                    Ok(V::Unit)
                                })
                            }),
                        ],
                    );
                    V::bind(
                        acquire,
                        V::fun(vec![c[0].clone(), c[2].clone(), value], |c, _| {
                            V::leaf(c.to_vec(), |c, context| {
                                increment(&c[0]);
                                if c[2].render() != "2" {
                                    return Ok(operation("std/effect::Continue", vec![]));
                                }
                                match c[1].render().as_str() {
                                    "failure" => Err(NativeExit::failure(c[0].clone())),
                                    "defect" => {
                                        Err(NativeExit::defects(vec!["action defect".into()]))
                                    }
                                    _ => {
                                        context.cancelled = true;
                                        Ok(operation("std/effect::Continue", vec![]))
                                    }
                                }
                            })
                        }),
                    )
                },
            );
            let plan = native_effect_control(
                "std/effect::forEachUntil",
                vec![native_iterable_dictionary("std/array::Iterable")],
                vec![
                    action,
                    V::Array(Shared::new(vec![V::Int(1), V::Int(2), V::Int(3)])),
                ],
            );
            let result = NativeContext::default().root(attempt(plan));
            match result {
                Ok(value) if outcome == "failure" => {
                    assert!(value.tagged("std/prelude::Left"));
                    assert_eq!(value.payload().node_id(), calls.node_id());
                }
                Err(NativeExit::Defect(errors)) if outcome == "defect" => {
                    assert_eq!(errors[0].value, "action defect")
                }
                Err(NativeExit::Cancelled) if outcome == "cancelled" => {}
                _ => panic!("wrong loop outcome: {outcome}"),
            }
            assert_eq!(calls.read_reference().render(), "2");
            assert_eq!(releases.read_reference().render(), "2");
            drop((calls, releases));
            finish();
        }
    }

    #[test]
    fn for_each_until_does_not_access_an_empty_or_precancelled_input() {
        enable_cycle_collection();
        let action = V::fun(vec![], |_, _| panic!("must not call empty action"));
        let plan = native_effect_control(
            "std/effect::forEachUntil",
            vec![native_iterable_dictionary("std/array::Iterable")],
            vec![action.clone(), V::Array(Shared::new(vec![]))],
        );
        assert!(matches!(NativeContext::default().root(plan), Ok(V::Unit)));
        let dictionary = V::dict(vec![(
            "iterate",
            V::fun(vec![], |_, _| panic!("must not iterate after cancellation")),
        )]);
        let plan = native_effect_control(
            "std/effect::forEachUntil",
            vec![dictionary],
            vec![action, V::Unit],
        );
        assert!(matches!(
            NativeContext {
                cancelled: true,
                ..NativeContext::default()
            }
            .root(attempt(plan)),
            Err(NativeExit::Cancelled)
        ));
        finish();
    }

    #[test]
    fn for_each_until_drops_captured_ref_cycles_after_each_defect_stage() {
        enable_cycle_collection();
        for stage in ["iterate", "next", "construct", "execute"] {
            let cell = V::reference(V::Unit);
            let captures = vec![cell.clone(), V::text(stage)];
            let dictionary = V::dict(vec![(
                "iterate",
                V::fun(captures.clone(), |c, _| {
                    assert_ne!(c[1].render(), "iterate", "iterate defect");
                    native_iterator_unfold(
                        V::fun(c.to_vec(), |c, state| {
                            assert_ne!(c[1].render(), "next", "next defect");
                            V::adt(
                                "std/prelude::Just",
                                V::Tuple(Shared::new(vec![state, V::Unit])),
                            )
                        }),
                        V::Int(1),
                    )
                }),
            )]);
            let action = V::fun(captures, |c, _| {
                assert_ne!(c[1].render(), "construct", "construct defect");
                V::leaf(c.to_vec(), |_, _| {
                    Err(NativeExit::defects(vec!["execute defect".into()]))
                })
            });
            cell.write_reference(V::Tuple(Shared::new(vec![
                dictionary.clone(),
                action.clone(),
            ])));
            let plan = native_effect_control(
                "std/effect::forEachUntil",
                vec![dictionary],
                vec![action, V::Unit],
            );
            let Err(NativeExit::Defect(errors)) = NativeContext::default().root(attempt(plan))
            else {
                panic!("loop defect became a value")
            };
            assert_eq!(errors.len(), 1);
            assert!(errors[0].value.contains(&format!("{stage} defect")));
            drop((cell, errors));
            assert!(collect_cycles().unwrap().reclaimed > 0);
            finish();
        }
    }

    #[test]
    fn for_each_until_runs_a_long_input_with_bounded_frames_and_live_cursor_state() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                enable_cycle_collection();
                let iterator = native_iterator_unfold(
                    V::fun(vec![], |_, state| {
                        let V::Int(n) = state else { panic!() };
                        assert!(LIVE.with(Cell::get) < 64);
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
                let plan = native_effect_control(
                    "std/effect::forEachUntil",
                    vec![native_iterable_dictionary("std/iterator::Iterable")],
                    vec![
                        V::fun(vec![], |_, _| {
                            succeed(operation("std/effect::Continue", vec![]))
                        }),
                        iterator,
                    ],
                );
                let mut context = NativeContext::default();
                assert!(matches!(context.root(plan), Ok(V::Unit)));
                assert!(context.peak_frames < 8, "{}", context.peak_frames);
                finish();
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
