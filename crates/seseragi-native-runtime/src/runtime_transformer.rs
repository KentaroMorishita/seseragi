fn native_transformer_wrap(run: V) -> V {
    V::Record(Shared::new(BTreeMap::from([("run".into(), run)])))
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
