fn native_transformer_wrap(run: V) -> V {
    V::Record(Shared::new(BTreeMap::from([("run".into(), run)])))
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
fn native_transformer_dictionary(kind: i64, children: Vec<V>) -> V {
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
