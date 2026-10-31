fn native_monad_kind(dictionary: &V) -> Option<i64> {
    let V::Dict(fields) = dictionary else {
        return None;
    };
    match fields.get().get("\0nativeMonad") {
        Some(V::Int(kind)) => Some(*kind),
        _ => None,
    }
}
fn native_non_empty_values(value: &V) -> Vec<V> {
    let mut values = vec![value.field("head")];
    values.extend(value.field("tail").list_values());
    values
}
fn native_non_empty(values: Vec<V>) -> V {
    let mut values = values.into_iter();
    let head = values
        .next()
        .expect("Rust ABI: NonEmptyList cannot be empty");
    V::Record(Shared::new(BTreeMap::from([
        ("head".into(), head),
        ("tail".into(), V::list_from_values(values.collect())),
    ])))
}
fn native_monad_values(kind: i64, value: &V) -> Vec<V> {
    match kind {
        0 => {
            let V::Array(values) = value else {
                panic!("Rust ABI: expected Array")
            };
            values.get().clone()
        }
        1 => value.list_values(),
        4 => native_non_empty_values(value),
        _ => panic!("Rust ABI: expected sequence Monad"),
    }
}
fn native_monad_collect(kind: i64, values: Vec<V>) -> V {
    match kind {
        0 => V::Array(Shared::new(values)),
        1 => V::list_from_values(values),
        4 => native_non_empty(values),
        _ => panic!("Rust ABI: sequence Monad"),
    }
}
fn native_monad_pure(kind: i64, value: V) -> V {
    match kind {
        0 | 1 | 4 => native_monad_collect(kind, vec![value]),
        2 => V::adt("std/prelude::Just", value),
        3 => V::adt("std/prelude::Right", value),
        5 => V::leaf(vec![value], |captures, _| Ok(captures[0].clone())),
        6 => V::adt("std/validation::Valid", value),
        8 => native_stream_singleton(value),
        _ => panic!("Rust ABI: monad pure"),
    }
}
fn native_monad_map(kind: i64, callback: &V, source: V) -> V {
    match kind {
        0 | 1 | 4 => native_monad_collect(
            kind,
            native_monad_values(kind, &source)
                .into_iter()
                .map(|value| callback.apply(value))
                .collect(),
        ),
        2 | 3 | 6 => {
            let success = match kind {
                2 => "std/prelude::Just",
                3 => "std/prelude::Right",
                _ => "std/validation::Valid",
            };
            if source.tagged(success) {
                V::adt(success, callback.apply(source.payload()))
            } else {
                source
            }
        }
        5 => V::bind(
            source,
            V::fun(vec![callback.clone()], |captures, value| {
                native_monad_pure(5, captures[0].apply(value))
            }),
        ),
        8 => native_stream_bind(
            V::fun(vec![callback.clone()], |captures, value| {
                native_monad_pure(8, captures[0].apply(value))
            }),
            source,
        ),
        7 => native_map_operation("std/map::mapValues", vec![], vec![callback.clone(), source]),
        _ => panic!("Rust ABI: monad map"),
    }
}
fn native_monad_bind(kind: i64, callback: &V, source: V) -> V {
    match kind {
        0 | 1 | 4 => {
            let mut result = Vec::new();
            for value in native_monad_values(kind, &source) {
                result.extend(native_monad_values(kind, &callback.apply(value)));
            }
            native_monad_collect(kind, result)
        }
        2 | 3 => {
            if source.tagged(if kind == 2 {
                "std/prelude::Just"
            } else {
                "std/prelude::Right"
            }) {
                callback.apply(source.payload())
            } else {
                source
            }
        }
        5 => V::bind(source, callback.clone()),
        8 => native_stream_bind(callback.clone(), source),
        _ => panic!("Rust ABI: monad flatMap"),
    }
}
fn native_monad_apply(kind: i64, functions: V, values: V) -> V {
    match kind {
        0 | 1 | 4 => {
            let values = native_monad_values(kind, &values);
            let mut result = Vec::new();
            for function in native_monad_values(kind, &functions) {
                for value in &values {
                    result.push(function.apply(value.clone()));
                }
            }
            native_monad_collect(kind, result)
        }
        2 | 3 | 6 => {
            let success = match kind {
                2 => "std/prelude::Just",
                3 => "std/prelude::Right",
                _ => "std/validation::Valid",
            };
            if kind == 6 && !functions.tagged(success) && !values.tagged(success) {
                let mut errors = native_non_empty_values(&functions.payload());
                errors.extend(native_non_empty_values(&values.payload()));
                return V::adt("std/validation::Invalid", native_non_empty(errors));
            }
            if !functions.tagged(success) {
                functions
            } else if !values.tagged(success) {
                values
            } else {
                V::adt(success, functions.payload().apply(values.payload()))
            }
        }
        5 => V::bind(
            functions,
            V::fun(vec![values], |captures, function| {
                native_monad_map(5, &function, captures[0].clone())
            }),
        ),
        8 => native_stream_bind(
            V::fun(vec![values], |captures, function| {
                native_monad_map(8, &function, captures[0].clone())
            }),
            functions,
        ),
        _ => panic!("Rust ABI: monad apply"),
    }
}
fn native_monad_dictionary(kind: i64, level: &'static str) -> V {
    const KEYS: [[&str; 3]; 9] = [
        ["\0array-functor", "\0array-applicative", "\0array-monad"],
        ["\0list-functor", "\0list-applicative", "\0list-monad"],
        ["\0maybe-functor", "\0maybe-applicative", "\0maybe-monad"],
        ["\0either-functor", "\0either-applicative", "\0either-monad"],
        [
            "\0non-empty-functor",
            "\0non-empty-applicative",
            "\0non-empty-monad",
        ],
        ["\0effect-functor", "\0effect-applicative", "\0effect-monad"],
        [
            "\0validation-functor",
            "\0validation-applicative",
            "\0validation-monad",
        ],
        ["\0map-functor", "\0map-applicative", "\0map-monad"],
        ["\0stream-functor", "\0stream-applicative", "\0stream-monad"],
    ];
    let grade = match level {
        "Functor" => 0,
        "Applicative" => 1,
        "Monad" => 2,
        _ => panic!("Rust ABI: dictionary grade"),
    };
    cached(KEYS[kind as usize][grade], || {
        native_monad_methods(kind, level)
    })
}
fn native_monad_methods(kind: i64, level: &'static str) -> V {
    let mut methods = vec![
        ("\0nativeMonad", V::Int(kind)),
        (
            "map",
            V::fun(vec![V::Int(kind)], |captures, callback| {
                V::fun(vec![captures[0].clone(), callback], |captures, source| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    native_monad_map(kind, &captures[1], source)
                })
            }),
        ),
    ];
    if level != "Functor" {
        methods.push((
            "pure",
            V::fun(vec![V::Int(kind)], |captures, value| {
                let V::Int(kind) = captures[0] else {
                    unreachable!()
                };
                native_monad_pure(kind, value)
            }),
        ));
        methods.push((
            "apply",
            V::fun(vec![V::Int(kind)], |captures, functions| {
                V::fun(vec![captures[0].clone(), functions], |captures, values| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    native_monad_apply(kind, captures[1].clone(), values)
                })
            }),
        ));
    }
    if level == "Monad" {
        methods.push((
            "flatMap",
            V::fun(vec![V::Int(kind)], |captures, callback| {
                V::fun(vec![captures[0].clone(), callback], |captures, source| {
                    let V::Int(kind) = captures[0] else {
                        unreachable!()
                    };
                    native_monad_bind(kind, &captures[1], source)
                })
            }),
        ));
    }
    V::dict(methods)
}

// Standard strict monads run a depth-first work queue. Each step is a separate
// function, so neither generated Rust syntax nor the host stack grows with do.
fn native_monad_do(dictionary: V, program: V, mut index: usize, mut environment: V) -> V {
    let kind = native_monad_kind(&dictionary);
    let tail_bind = matches!(&dictionary, V::Dict(fields) if matches!(fields.get().get("\0nativeTailBind"), Some(V::Bool(true))));
    let sequential = matches!(kind, Some(0 | 1 | 4));
    let mut pending = Vec::new();
    let mut output = Vec::new();
    loop {
        let step = program.index(index);
        let V::Int(instruction) = step.index(0) else {
            panic!("Rust ABI: do instruction")
        };
        let staged =
            if instruction != 0 && instruction != 3 && !matches!(kind, Some(0 | 1 | 2 | 3 | 4)) {
                let callback = V::fun(
                    vec![
                        dictionary.clone(),
                        program.clone(),
                        V::Int((index + 1) as i64),
                        environment.clone(),
                        V::Bool(instruction == 2),
                        V::Bool(tail_bind),
                    ],
                    |captures, value| {
                        if captures[5].boolean() {
                            return V::adt("\0doResume", value);
                        }
                        let V::Int(index) = captures[2] else {
                            unreachable!()
                        };
                        let environment = if captures[4].boolean() {
                            V::list_cons(value, captures[3].clone())
                        } else {
                            captures[3].clone()
                        };
                        native_monad_do(
                            captures[0].clone(),
                            captures[1].clone(),
                            index as usize,
                            environment,
                        )
                    },
                );
                Some(dictionary.field("flatMap").apply(callback))
            } else {
                None
            };
        let value = step.index(1).apply(environment.clone());
        if instruction == 0 {
            environment = V::list_cons(value, environment);
            index += 1;
            continue;
        }
        if instruction == 3 {
            if !sequential {
                return value;
            }
            output.extend(native_monad_values(kind.unwrap(), &value));
        } else if sequential {
            let values = native_monad_values(kind.unwrap(), &value);
            for value in values.into_iter().rev() {
                pending.push((
                    index + 1,
                    if instruction == 2 {
                        V::list_cons(value, environment.clone())
                    } else {
                        environment.clone()
                    },
                ));
            }
        } else if matches!(kind, Some(2 | 3)) {
            if !value.tagged(if kind == Some(2) {
                "std/prelude::Just"
            } else {
                "std/prelude::Right"
            }) {
                return value;
            }
            if instruction == 2 {
                environment = V::list_cons(value.payload(), environment);
            }
            index += 1;
            continue;
        } else {
            let value = staged.expect("Rust ABI: staged monad bind").apply(value);
            if tail_bind && value.tagged("\0doResume") {
                if instruction == 2 {
                    environment = V::list_cons(value.payload(), environment);
                }
                index += 1;
                continue;
            }
            return value;
        }
        match pending.pop() {
            Some((next, state)) => {
                index = next;
                environment = state;
            }
            None => return native_monad_collect(kind.unwrap(), output),
        }
    }
}
