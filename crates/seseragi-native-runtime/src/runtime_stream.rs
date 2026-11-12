// A Stream opens a fresh pull cursor for each terminal execution. A cursor
// returns one Effect per demand; the Effect machine owns callback continuations.
fn native_stream_defer(captures: Vec<V>, code: fn(&[V], V) -> V) -> V {
    V::effect(Instruction::Defer, vec![V::fun(captures, code)])
}
fn stream_pure(value: V) -> V {
    V::leaf(vec![value], |captures, _| Ok(captures[0].clone()))
}
fn stream_done() -> V {
    V::adt("std/prelude::Nothing", V::Unit)
}
fn stream_item(value: V) -> V {
    V::adt("std/prelude::Just", value)
}
fn stream_cell(value: V) -> V {
    V::Reference(Shared::new(RefCell::new(value)))
}
fn stream_open(source: &V) -> V {
    let cursor = source.apply(V::Unit);
    if matches!(cursor, V::Effect(_)) {
        cursor
    } else {
        stream_pure(cursor)
    }
}
fn stream_pull(cursor: &V) -> V {
    if matches!(cursor, V::Record(_)) {
        cursor.field("pull").apply(V::Unit)
    } else {
        cursor.apply(V::Unit)
    }
}
fn stream_close(cursor: &V) -> V {
    if matches!(cursor, V::Record(_)) {
        cursor.field("close").apply(V::Unit)
    } else {
        stream_pure(V::Unit)
    }
}
fn stream_cursor(pull: V, close: V) -> V {
    let closed = stream_cell(V::Bool(false));
    V::Record(Shared::new(BTreeMap::from([
        (
            "pull".into(),
            V::fun(vec![pull, closed.clone()], |captures, _| {
                if captures[1].read_reference().boolean() {
                    stream_pure(stream_done())
                } else {
                    captures[0].apply(V::Unit)
                }
            }),
        ),
        (
            "close".into(),
            V::fun(vec![close, closed], |captures, _| {
                native_stream_defer(captures.to_vec(), |captures, _| {
                    if captures[1].read_reference().boolean() {
                        return stream_pure(V::Unit);
                    }
                    captures[1].write_reference(V::Bool(true));
                    captures[0].apply(V::Unit)
                })
            }),
        ),
    ])))
}
fn stream_forward_close(pull: V, source: V) -> V {
    stream_cursor(
        pull,
        V::fun(vec![source], |captures, _| stream_close(&captures[0])),
    )
}
fn native_stream_singleton(value: V) -> V {
    V::fun(vec![value], |captures, _| {
        V::fun(
            vec![stream_cell(stream_item(captures[0].clone()))],
            |captures, _| {
                V::leaf(captures.to_vec(), |captures, _| {
                    let value = captures[0].read_reference();
                    captures[0].write_reference(stream_done());
                    Ok(value)
                })
            },
        )
    })
}
fn native_stream_bind(callback: V, source: V) -> V {
    V::fun(vec![callback, source], |captures, _| {
        V::bind(
            stream_open(&captures[1]),
            V::fun(vec![captures[0].clone()], |captures, outer| {
                stream_pure(stream_flat_cursor(
                    captures[0].clone(),
                    outer,
                    stream_cell(V::Unit),
                ))
            }),
        )
    })
}
fn stream_flat_cursor(callback: V, outer: V, inner: V) -> V {
    let close = V::fun(vec![outer.clone(), inner.clone()], |captures, _| {
        V::bind(
            stream_close(&captures[1].read_reference()),
            V::fun(vec![captures[0].clone()], |captures, _| {
                stream_close(&captures[0])
            }),
        )
    });
    let pull = V::fun(vec![callback, outer, inner], |captures, _| {
        native_stream_defer(captures.to_vec(), |captures, _| {
            let inner = captures[2].read_reference();
            if matches!(inner, V::Unit) {
                V::bind(
                    stream_pull(&captures[1]),
                    V::fun(captures.to_vec(), |captures, value| {
                        if value.tagged("std/prelude::Nothing") {
                            return stream_pure(value);
                        }
                        V::bind(
                            stream_open(&captures[0].apply(value.payload())),
                            V::fun(captures.to_vec(), |captures, inner| {
                                captures[2].write_reference(inner);
                                stream_pull(&stream_flat_cursor(
                                    captures[0].clone(),
                                    captures[1].clone(),
                                    captures[2].clone(),
                                ))
                            }),
                        )
                    }),
                )
            } else {
                V::bind(
                    stream_pull(&inner),
                    V::fun(captures.to_vec(), |captures, value| {
                        if value.tagged("std/prelude::Just") {
                            return stream_pure(value);
                        }
                        V::bind(
                            stream_close(&captures[2].read_reference()),
                            V::fun(captures.to_vec(), |captures, _| {
                                captures[2].write_reference(V::Unit);
                                stream_pull(&stream_flat_cursor(
                                    captures[0].clone(),
                                    captures[1].clone(),
                                    captures[2].clone(),
                                ))
                            }),
                        )
                    }),
                )
            }
        })
    });
    stream_cursor(pull, close)
}
fn stream_transform(name: &'static str, callback: V, source: V) -> V {
    V::fun(vec![V::text(name), callback, source], |captures, _| {
        let opening = stream_open(&captures[2]);
        V::bind(
            opening,
            V::fun(captures[..2].to_vec(), |captures, source| {
                stream_pure(stream_transform_cursor(
                    captures[0].clone(),
                    captures[1].clone(),
                    source,
                ))
            }),
        )
    })
}
fn stream_transform_cursor(name: V, callback: V, source: V) -> V {
    let upstream = source.clone();
    let pull = V::fun(vec![name, callback, source], |captures, _| {
        native_stream_defer(captures.to_vec(), |captures, _| {
            let source = stream_pull(&captures[2]);
            if captures[0].render() == "std/stream::mapError" {
                return V::effect(Instruction::MapError, vec![captures[1].clone(), source]);
            }
            V::bind(
                source,
                V::fun(captures.to_vec(), |captures, value| {
                    if value.tagged("std/prelude::Nothing") {
                        return stream_pure(value);
                    }
                    let mapped = captures[1].apply(value.payload());
                    let result = match captures[0].render().as_str() {
                        "std/stream::map" => Some(stream_item(mapped)),
                        "std/stream::filter" => mapped.boolean().then_some(value),
                        "std/stream::filterMap" => {
                            mapped.tagged("std/prelude::Just").then_some(mapped)
                        }
                        _ => unreachable!(),
                    };
                    match result {
                        Some(result) => stream_pure(result),
                        None => stream_pull(&stream_transform_cursor(
                            captures[0].clone(),
                            captures[1].clone(),
                            captures[2].clone(),
                        )),
                    }
                }),
            )
        })
    });
    stream_forward_close(pull, upstream)
}
fn stream_count_cursor(take: bool, count: V, source: V) -> V {
    let upstream = source.clone();
    let pull = V::fun(vec![V::Bool(take), count, source], |captures, _| {
        native_stream_defer(captures.to_vec(), |captures, _| {
            let V::Int(count) = captures[1].read_reference() else {
                unreachable!()
            };
            if captures[0].boolean() && count <= 0 {
                return V::bind(
                    stream_close(&captures[2]),
                    V::fun(vec![], |_, _| stream_pure(stream_done())),
                );
            }
            V::bind(
                stream_pull(&captures[2]),
                V::fun(captures.to_vec(), |captures, value| {
                    let V::Int(count) = captures[1].read_reference() else {
                        unreachable!()
                    };
                    if value.tagged("std/prelude::Nothing") {
                        return stream_pure(value);
                    }
                    if count > 0 {
                        captures[1].write_reference(V::Int(count - 1));
                    }
                    if captures[0].boolean() || count <= 0 {
                        return stream_pure(value);
                    }
                    stream_pull(&stream_count_cursor(
                        false,
                        captures[1].clone(),
                        captures[2].clone(),
                    ))
                }),
            )
        })
    });
    stream_forward_close(pull, upstream)
}
// `chunk` pulls until it holds `size` values or the source ends; a failure
// while filling propagates and the partial chunk is never emitted.
fn stream_chunk_cursor(size: V, finished: V, source: V) -> V {
    let upstream = source.clone();
    let pull = V::fun(vec![size, finished, source], |captures, _| {
        stream_chunk_fill(
            captures[0].clone(),
            captures[1].clone(),
            captures[2].clone(),
            stream_cell(V::ListEmpty),
            stream_cell(V::Int(0)),
        )
    });
    stream_forward_close(pull, upstream)
}
fn stream_chunk_fill(size: V, finished: V, source: V, values: V, count: V) -> V {
    native_stream_defer(
        vec![size, finished, source, values, count],
        |captures, _| {
            let (V::Int(size), V::Int(count)) = (&captures[0], captures[4].read_reference()) else {
                unreachable!()
            };
            if captures[1].read_reference().boolean() || count >= *size {
                let mut values = captures[3].read_reference().list_values();
                if values.is_empty() {
                    return stream_pure(stream_done());
                }
                values.reverse();
                return stream_pure(stream_item(V::Array(Shared::new(values))));
            }
            V::bind(
                stream_pull(&captures[2]),
                V::fun(captures.to_vec(), |captures, value| {
                    if value.tagged("std/prelude::Nothing") {
                        captures[1].write_reference(V::Bool(true));
                    } else {
                        captures[3].write_reference(V::list_cons(
                            value.payload(),
                            captures[3].read_reference(),
                        ));
                        let V::Int(count) = captures[4].read_reference() else {
                            unreachable!()
                        };
                        captures[4].write_reference(V::Int(count + 1));
                    }
                    stream_chunk_fill(
                        captures[0].clone(),
                        captures[1].clone(),
                        captures[2].clone(),
                        captures[3].clone(),
                        captures[4].clone(),
                    )
                }),
            )
        },
    )
}
fn stream_concat_cursor(suffix: V, current: V) -> V {
    let current_close = current.clone();
    let pull = V::fun(vec![suffix, current], |captures, _| {
        native_stream_defer(captures.to_vec(), |captures, _| {
            V::bind(
                stream_pull(&captures[1].read_reference()),
                V::fun(captures.to_vec(), |captures, value| {
                    if value.tagged("std/prelude::Just")
                        || matches!(captures[0].read_reference(), V::Unit)
                    {
                        return stream_pure(value);
                    }
                    V::bind(
                        stream_close(&captures[1].read_reference()),
                        V::fun(captures.to_vec(), |captures, _| {
                            V::bind(
                                stream_open(&captures[0].read_reference()),
                                V::fun(captures.to_vec(), |captures, suffix| {
                                    captures[0].write_reference(V::Unit);
                                    captures[1].write_reference(suffix.clone());
                                    stream_pull(&suffix)
                                }),
                            )
                        }),
                    )
                }),
            )
        })
    });
    stream_cursor(
        pull,
        V::fun(vec![current_close], |captures, _| {
            stream_close(&captures[0].read_reference())
        }),
    )
}
fn stream_terminal_loop(mode: i64, cursor: V, state: V, step: V) -> V {
    native_stream_defer(vec![V::Int(mode), cursor, state, step], |captures, _| {
        V::bind(
            stream_pull(&captures[1]),
            V::fun(captures.to_vec(), |captures, value| {
                let V::Int(mode) = captures[0] else {
                    unreachable!()
                };
                if value.tagged("std/prelude::Nothing") {
                    let result = captures[2].read_reference();
                    let result = if mode == 0 {
                        let mut values = result.list_values();
                        values.reverse();
                        V::Array(Shared::new(values))
                    } else {
                        result
                    };
                    return V::bind(
                        stream_close(&captures[1]),
                        V::fun(vec![result], |captures, _| stream_pure(captures[0].clone())),
                    );
                }
                let value = value.payload();
                let resume = V::fun(captures.to_vec(), |captures, _| {
                    let V::Int(mode) = captures[0] else {
                        unreachable!()
                    };
                    stream_terminal_loop(
                        mode,
                        captures[1].clone(),
                        captures[2].clone(),
                        captures[3].clone(),
                    )
                });
                match mode {
                    0 => {
                        captures[2]
                            .write_reference(V::list_cons(value, captures[2].read_reference()));
                    }
                    1 => captures[2].write_reference(
                        captures[3].apply(captures[2].read_reference()).apply(value),
                    ),
                    2 => return V::bind(captures[3].apply(value), resume),
                    _ => unreachable!(),
                }
                resume.apply(V::Unit)
            }),
        )
    })
}
pub fn native_stream_operation(name: &'static str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/stream::bufferCapacity", [V::Int(value)]) => {
            if *value > 0 {
                native_right(V::adt("std/stream::BufferCapacity", V::Int(*value)))
            } else {
                native_left(V::adt(
                    "std/stream::NonPositiveBufferCapacity",
                    V::Int(*value),
                ))
            }
        }
        ("std/stream::singleton", [value]) => native_stream_singleton(value.clone()),
        ("std/stream::fromArray", [values]) => V::fun(vec![values.clone()], |captures, _| {
            V::fun(
                vec![captures[0].clone(), stream_cell(V::Int(0))],
                |captures, _| {
                    V::leaf(captures.to_vec(), |captures, _| {
                        let V::Array(values) = &captures[0] else {
                            panic!("Rust ABI: Stream Array")
                        };
                        let V::Int(index) = captures[1].read_reference() else {
                            unreachable!()
                        };
                        Ok(match values.get().get(index as usize) {
                            Some(value) => {
                                captures[1].write_reference(V::Int(index + 1));
                                stream_item(value.clone())
                            }
                            None => stream_done(),
                        })
                    })
                },
            )
        }),
        ("std/stream::fromEffect", [effect]) => V::fun(vec![effect.clone()], |captures, _| {
            V::fun(
                vec![stream_cell(V::Bool(false)), captures[0].clone()],
                |captures, _| {
                    native_stream_defer(captures.to_vec(), |captures, _| {
                        if captures[0].read_reference().boolean() {
                            return stream_pure(stream_done());
                        }
                        captures[0].write_reference(V::Bool(true));
                        V::bind(
                            captures[1].clone(),
                            V::fun(vec![], |_, value| stream_pure(stream_item(value))),
                        )
                    })
                },
            )
        }),
        ("std/stream::unfold", [step, initial]) => {
            V::fun(vec![step.clone(), initial.clone()], |captures, _| {
                V::fun(
                    vec![captures[0].clone(), stream_cell(captures[1].clone())],
                    |captures, _| {
                        V::leaf(captures.to_vec(), |captures, _| {
                            let result = captures[0].apply(captures[1].read_reference());
                            if result.tagged("std/prelude::Nothing") {
                                return Ok(result);
                            }
                            let pair = result.payload();
                            captures[1].write_reference(pair.index(1));
                            Ok(stream_item(pair.index(0)))
                        })
                    },
                )
            })
        }
        (
            "std/stream::map"
            | "std/stream::filter"
            | "std/stream::filterMap"
            | "std/stream::mapError",
            [callback, source],
        ) => stream_transform(name, callback.clone(), source.clone()),
        ("std/stream::flatMap", [callback, source]) => {
            native_stream_bind(callback.clone(), source.clone())
        }
        ("std/stream::take" | "std/stream::drop", [count, source]) => V::fun(
            vec![
                V::Bool(name == "std/stream::take"),
                count.clone(),
                source.clone(),
            ],
            |captures, _| {
                V::bind(
                    stream_open(&captures[2]),
                    V::fun(captures[..2].to_vec(), |captures, source| {
                        stream_pure(stream_count_cursor(
                            captures[0].boolean(),
                            stream_cell(captures[1].clone()),
                            source,
                        ))
                    }),
                )
            },
        ),
        ("std/stream::chunk", [V::Int(size), source]) => {
            if *size <= 0 {
                return native_left(V::adt("std/collection::NonPositiveSize", V::Int(*size)));
            }
            native_right(V::fun(
                vec![V::Int(*size), source.clone()],
                |captures, _| {
                    V::bind(
                        stream_open(&captures[1]),
                        V::fun(vec![captures[0].clone()], |captures, source| {
                            stream_pure(stream_chunk_cursor(
                                captures[0].clone(),
                                stream_cell(V::Bool(false)),
                                source,
                            ))
                        }),
                    )
                },
            ))
        }
        ("std/stream::concat", [suffix, prefix]) => {
            V::fun(vec![suffix.clone(), prefix.clone()], |captures, _| {
                V::bind(
                    stream_open(&captures[1]),
                    V::fun(vec![captures[0].clone()], |captures, prefix| {
                        stream_pure(stream_concat_cursor(
                            stream_cell(captures[0].clone()),
                            stream_cell(prefix),
                        ))
                    }),
                )
            })
        }
        ("std/stream::runCollect", [source]) => {
            stream_terminal(0, source.clone(), V::Unit, V::Unit)
        }
        ("std/stream::runFold", [initial, step, source]) => {
            stream_terminal(1, source.clone(), initial.clone(), step.clone())
        }
        ("std/stream::runForEach", [step, source]) => {
            stream_terminal(2, source.clone(), V::Unit, step.clone())
        }
        _ => panic!("Rust ABI: unsupported Stream operation"),
    }
}
fn stream_terminal(mode: i64, source: V, initial: V, step: V) -> V {
    V::effect(
        Instruction::Scoped,
        vec![native_stream_defer(
            vec![V::Int(mode), source, initial, step],
            |captures, _| {
                let V::Int(mode) = captures[0] else {
                    unreachable!()
                };
                let initial = if mode == 0 {
                    V::ListEmpty
                } else {
                    captures[2].clone()
                };
                V::bind(
                    stream_open(&captures[1]),
                    V::fun(
                        vec![V::Int(mode), initial, captures[3].clone()],
                        |captures, cursor| {
                            let V::Int(mode) = captures[0] else {
                                unreachable!()
                            };
                            stream_terminal_loop(
                                mode,
                                cursor,
                                stream_cell(captures[1].clone()),
                                captures[2].clone(),
                            )
                        },
                    ),
                )
            },
        )],
    )
}

#[cfg(test)]
mod stream_tests {
    use super::*;

    fn range(limit: i64, calls: V) -> V {
        native_stream_operation(
            "std/stream::unfold",
            vec![
                V::fun(vec![V::Int(limit), calls], |captures, state| {
                    let V::Int(state) = state else { unreachable!() };
                    let V::Int(calls) = captures[1].read_reference() else {
                        unreachable!()
                    };
                    captures[1].write_reference(V::Int(calls + 1));
                    if state
                        >= match captures[0] {
                            V::Int(limit) => limit,
                            _ => unreachable!(),
                        }
                    {
                        stream_done()
                    } else {
                        stream_item(V::Tuple(Shared::new(vec![
                            V::Int(state),
                            V::Int(state + 1),
                        ])))
                    }
                }),
                V::Int(0),
            ],
        )
    }

    #[test]
    fn unfold_fold_retains_bounded_state_and_effect_frames() {
        for count in [1_000, 100_000] {
            let calls = stream_cell(V::Int(0));
            let source = range(count, calls.clone());
            let mapped = native_monad_map(8, &V::fun(vec![], |_, value| value), source);
            let bound = native_stream_bind(
                V::fun(vec![], |_, value| native_stream_singleton(value)),
                mapped,
            );
            let step = V::fun(vec![], |_, total| {
                V::fun(vec![total], |captures, value| {
                    binary("+", captures[0].clone(), value)
                })
            });
            let effect =
                native_stream_operation("std/stream::runFold", vec![V::Int(0), step, bound]);
            assert_eq!(calls.read_reference().render(), "0");
            let baseline = LIVE.with(Cell::get);
            PEAK.with(|peak| peak.set(baseline));
            let mut context = NativeContext::default();
            let result = context
                .root(effect.clone())
                .unwrap_or_else(|_| panic!("fold failed"));
            assert_eq!(result.render(), (count * (count - 1) / 2).to_string());
            assert_eq!(calls.read_reference().render(), (count + 1).to_string());
            assert!(
                PEAK.with(Cell::get) < baseline + 150,
                "peak={} baseline={baseline}",
                PEAK.with(Cell::get)
            );
            assert!(context.peak_frames < 16, "{}", context.peak_frames);
            drop((effect, result, calls));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn take_issues_only_demanded_pulls_and_terminal_restarts() {
        let calls = stream_cell(V::Int(0));
        let source = range(100_000, calls.clone());
        let take = native_stream_operation("std/stream::take", vec![V::Int(3), source]);
        let effect = native_stream_operation("std/stream::runCollect", vec![take]);
        let mut context = NativeContext::default();
        for expected in [3, 6] {
            assert_eq!(
                context
                    .root(effect.clone())
                    .unwrap_or_else(|_| panic!("collect failed"))
                    .render(),
                "[0, 1, 2]"
            );
            assert_eq!(calls.read_reference().render(), expected.to_string());
        }
        let zero = native_stream_operation(
            "std/stream::take",
            vec![V::Int(-1), range(10, calls.clone())],
        );
        context
            .root(native_stream_operation(
                "std/stream::runCollect",
                vec![zero],
            ))
            .unwrap_or_else(|_| panic!("empty failed"));
        assert_eq!(calls.read_reference().render(), "6");
        drop((effect, calls));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn terminal_closes_resources_after_failure_and_callback_defect() {
        for defect in [false, true] {
            let released = stream_cell(V::Int(0));
            let acquire = V::effect(
                Instruction::AcquireRelease,
                vec![
                    stream_pure(V::Int(7)),
                    V::fun(vec![released.clone()], |captures, _| {
                        V::leaf(captures.to_vec(), |captures, _| {
                            captures[0].write_reference(V::Int(1));
                            Ok(V::Unit)
                        })
                    }),
                ],
            );
            let source = if defect {
                native_stream_operation("std/stream::fromEffect", vec![acquire])
            } else {
                let fail = V::bind(
                    acquire,
                    V::fun(vec![], |_, _| {
                        V::leaf(vec![], |_, _| Err(NativeExit::failure(V::text("failed"))))
                    }),
                );
                native_stream_operation("std/stream::fromEffect", vec![fail])
            };
            let callback = V::fun(vec![], |_, _| panic!("mapper defect"));
            let source = if defect {
                native_monad_map(8, &callback, source)
            } else {
                source
            };
            let terminal = native_stream_operation("std/stream::runCollect", vec![source]);
            let mut context = NativeContext::default();
            let result = context.root(terminal);
            assert!(if defect {
                matches!(result, Err(NativeExit::Defect(_)))
            } else {
                matches!(result, Err(NativeExit::Failure(_)))
            });
            assert_eq!(released.read_reference().render(), "1");
            drop((result, callback, released));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
