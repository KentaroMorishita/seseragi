// `zip`, `merge` and `buffer` pull each source in its own branch Fiber, as
// the TypeScript runtime pulls each branch in its own execution. A branch
// opens its source, then pulls once per demand and sends every outcome to a
// results Queue. Resources acquired while pulling belong to the branch and
// are released when the branch closes. The branch's last finalizer reports
// how it ended, after its cursor has closed, so a Defect never strands the
// consumer. Messages are `(side, outcome, token)` tuples.
const BRANCH_OPENED: &str = "\0stream.branch.opened";
const BRANCH_ITEM: &str = "\0stream.branch.item";
const BRANCH_DONE: &str = "\0stream.branch.done";
const BRANCH_FAIL: &str = "\0stream.branch.fail";
const BRANCH_ENDED: &str = "\0stream.branch.ended";

fn branch_succeed(value: V) -> V {
    native_operation("effect.succeed", vec![value])
}
fn branch_then(first: V, next: V) -> V {
    V::bind(first, V::fun(vec![next], |c, _| c[0].clone()))
}
fn branch_message(side: &V, outcome: V, token: V) -> V {
    V::Tuple(Shared::new(vec![side.clone(), outcome, token]))
}
fn branch_offer(results: &V, side: &V, outcome: V, token: V) -> V {
    native_operation(
        "std/queue::offer",
        vec![branch_message(side, outcome, token), results.clone()],
    )
}
// Runs `cleanup` when `effect` ends with a failure, a Defect or cancellation.
fn branch_on_error(effect: V, cleanup: V) -> V {
    V::effect(
        Instruction::Bracket,
        vec![
            V::leaf(vec![], |_, _| Ok(stream_cell(V::Bool(false)))),
            V::fun(vec![effect], |c, finished| {
                V::bind(
                    c[0].clone(),
                    V::fun(vec![finished], |c, value| {
                        V::leaf(vec![c[0].clone(), value], |c, _| {
                            c[0].write_reference(V::Bool(true));
                            Ok(c[1].clone())
                        })
                    }),
                )
            }),
            V::fun(vec![cleanup], |c, finished| {
                if finished.read_reference().boolean() {
                    branch_succeed(V::Unit)
                } else {
                    c[0].clone()
                }
            }),
        ],
    )
}
// Runs `first`, then `second` even when `first` fails.
fn branch_both(first: V, second: V) -> V {
    V::effect(
        Instruction::Bracket,
        vec![
            branch_succeed(V::Unit),
            V::fun(vec![first], |c, _| c[0].clone()),
            V::fun(vec![second], |c, _| c[0].clone()),
        ],
    )
}
// The branch Fiber. `demand` is the Effect that waits for the next pull; its
// result travels with an item as the token. `buffered` branches stop after
// the source ends or fails; other branches stay open until their consumer
// closes them.
fn branch_body(source: V, side: V, demand: V, results: V, buffered: bool) -> V {
    let outcome = stream_cell(V::Unit);
    let finish = V::fun(
        vec![
            results.clone(),
            side.clone(),
            outcome.clone(),
            V::Bool(buffered),
        ],
        |c, _| {
            let outcome = c[2].read_reference();
            let outcome = if matches!(outcome, V::Unit) {
                V::adt(BRANCH_ENDED, V::Unit)
            } else {
                outcome
            };
            let report = branch_offer(&c[0], &c[1], outcome.clone(), V::Unit);
            // A buffered failure or Defect discards the values that wait.
            if c[3].boolean() && !outcome.tagged(BRANCH_DONE) {
                branch_then(branch_drain(c[0].clone()), report)
            } else {
                report
            }
        },
    );
    let body = V::bind(
        V::effect(Instruction::Attempt, vec![stream_open(&source)]),
        V::fun(
            vec![side, demand, results, outcome, V::Bool(buffered)],
            |c, opened| {
                if opened.tagged("std/prelude::Left") {
                    c[3].write_reference(V::adt(BRANCH_FAIL, opened.payload()));
                    return branch_succeed(V::Unit);
                }
                let cursor = opened.payload();
                let close = V::fun(vec![], |_, cursor| stream_close(&cursor));
                branch_then(
                    V::effect(
                        Instruction::AcquireRelease,
                        vec![branch_succeed(cursor.clone()), close],
                    ),
                    branch_then(
                        branch_offer(&c[2], &c[0], V::adt(BRANCH_OPENED, V::Unit), V::Unit),
                        branch_loop(
                            cursor,
                            c[0].clone(),
                            c[1].clone(),
                            c[2].clone(),
                            c[3].clone(),
                            c[4].clone(),
                        ),
                    ),
                )
            },
        ),
    );
    V::effect(
        Instruction::Scoped,
        vec![branch_then(
            V::effect(
                Instruction::AcquireRelease,
                vec![branch_succeed(V::Unit), finish],
            ),
            body,
        )],
    )
}
fn branch_drain(results: V) -> V {
    V::bind(
        native_operation("std/queue::tryTake", vec![results.clone()]),
        V::fun(vec![results], |c, taken| {
            if taken.tagged("std/prelude::Right") && taken.payload().tagged("std/prelude::Just") {
                branch_drain(c[0].clone())
            } else {
                branch_succeed(V::Unit)
            }
        }),
    )
}
fn branch_loop(cursor: V, side: V, demand: V, results: V, outcome: V, buffered: V) -> V {
    V::bind(
        demand.clone(),
        V::fun(
            vec![cursor, side, demand, results, outcome, buffered],
            |c, token| {
                V::bind(
                    V::effect(Instruction::Attempt, vec![stream_pull(&c[0])]),
                    V::fun(
                        vec![
                            c[0].clone(),
                            c[1].clone(),
                            c[2].clone(),
                            c[3].clone(),
                            c[4].clone(),
                            c[5].clone(),
                            token,
                        ],
                        |c, pulled| {
                            let buffered = c[5].boolean();
                            let outcome = if pulled.tagged("std/prelude::Left") {
                                V::adt(BRANCH_FAIL, pulled.payload())
                            } else if pulled.payload().tagged("std/prelude::Nothing") {
                                V::adt(BRANCH_DONE, V::Unit)
                            } else {
                                V::adt(BRANCH_ITEM, pulled.payload().payload())
                            };
                            if buffered && !outcome.tagged(BRANCH_ITEM) {
                                c[4].write_reference(outcome);
                                return branch_succeed(V::Unit);
                            }
                            branch_then(
                                branch_offer(&c[3], &c[1], outcome, c[6].clone()),
                                branch_loop(
                                    c[0].clone(),
                                    c[1].clone(),
                                    c[2].clone(),
                                    c[3].clone(),
                                    c[4].clone(),
                                    c[5].clone(),
                                ),
                            )
                        },
                    ),
                )
            },
        ),
    )
}
// A branch handle: [Fiber, demand Queue, state]. State 0 is open; 1 means
// closed or ended, so closing it again does nothing.
fn branch_handle(fiber: V, demands: V) -> V {
    V::Tuple(Shared::new(vec![fiber, demands, stream_cell(V::Int(0))]))
}
fn branch_close(branch: &V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![branch.clone()], |c, _| {
            let state = c[0].index(2);
            if matches!(state.read_reference(), V::Int(1)) {
                return branch_succeed(V::Unit);
            }
            state.write_reference(V::Int(1));
            native_operation("std/effect::interrupt", vec![c[0].index(0)])
        })],
    )
}
// Interprets one branch outcome as a pull result: Just a value, Nothing at
// the end, the typed failure, or the Defect that ended the branch.
fn branch_result(branch: &V, outcome: V) -> V {
    if outcome.tagged(BRANCH_ITEM) {
        return branch_succeed(stream_item(outcome.payload()));
    }
    if outcome.tagged(BRANCH_DONE) {
        return branch_succeed(stream_done());
    }
    if outcome.tagged(BRANCH_FAIL) {
        return native_operation("effect.fail", vec![outcome.payload()]);
    }
    assert!(outcome.tagged(BRANCH_ENDED), "Rust ABI: stream branch outcome");
    branch.index(2).write_reference(V::Int(1));
    V::bind(
        native_operation("std/effect::join", vec![branch.index(0)]),
        V::fun(vec![], |_, _| branch_succeed(stream_done())),
    )
}
// Forks a branch and waits until it has opened its source. An open failure
// or Defect is raised here, after the branch has ended.
fn branch_open(source: V, side: i64, results: V, demand: Option<V>) -> V {
    V::bind(
        native_operation("std/queue::unbounded", vec![V::Unit]),
        V::fun(
            vec![source, V::Int(side), results, demand.unwrap_or(V::Unit)],
            |c, demands| {
                let buffered = !matches!(c[3], V::Unit);
                let demand = if buffered {
                    c[3].clone()
                } else {
                    native_operation("std/queue::take", vec![demands.clone()])
                };
                let body = branch_body(c[0].clone(), c[1].clone(), demand, c[2].clone(), buffered);
                V::bind(
                    native_operation("std/effect::fork", vec![body]),
                    V::fun(vec![c[2].clone(), demands], |c, fiber| {
                        let branch = branch_handle(fiber, c[1].clone());
                        V::bind(
                            native_operation("std/queue::take", vec![c[0].clone()]),
                            V::fun(vec![branch], |c, message| {
                                let outcome = message.index(1);
                                if outcome.tagged(BRANCH_OPENED) {
                                    return branch_succeed(c[0].clone());
                                }
                                V::bind(
                                    branch_result(&c[0], outcome),
                                    V::fun(vec![], |_, _| {
                                        native_effect_die(V::text("stream branch ended early"))
                                    }),
                                )
                            }),
                        )
                    }),
                )
            },
        ),
    )
}
fn branch_demand(branch: &V) -> V {
    native_operation("std/queue::offer", vec![V::Unit, branch.index(1)])
}
fn branch_side(message: &V) -> usize {
    match message.index(0) {
        V::Int(side) => side as usize,
        _ => panic!("Rust ABI: stream branch side"),
    }
}
// Opens `left` and then `right` as branches that share one results Queue.
// `kind` 0 builds a zip cursor and 1 a merge cursor.
fn branch_open_pair(kind: i64, left: V, right: V) -> V {
    V::bind(
        native_operation("std/queue::unbounded", vec![V::Unit]),
        V::fun(vec![left, right, V::Int(kind)], |c, results| {
            V::bind(
                branch_open(c[0].clone(), 0, results.clone(), None),
                V::fun(vec![c[1].clone(), results, c[2].clone()], |c, left| {
                    V::bind(
                        branch_on_error(
                            branch_open(c[0].clone(), 1, c[1].clone(), None),
                            branch_close(&left),
                        ),
                        V::fun(vec![left, c[1].clone(), c[2].clone()], |c, right| {
                            let cursor = if matches!(c[2], V::Int(0)) {
                                native_stream_zip_cursor(c[0].clone(), right, c[1].clone())
                            } else {
                                native_stream_merge_cursor(c[0].clone(), right, c[1].clone())
                            };
                            branch_succeed(cursor)
                        }),
                    )
                }),
            )
        }),
    )
}
fn branch_close_pair(left: &V, right: &V) -> V {
    branch_both(branch_close(left), branch_close(right))
}
fn native_stream_zip_cursor(left: V, right: V, results: V) -> V {
    let closed = stream_cell(V::Bool(false));
    let close = V::fun(
        vec![left.clone(), right.clone(), closed.clone()],
        |c, _| {
            V::effect(
                Instruction::Defer,
                vec![V::fun(c.to_vec(), |c, _| {
                    c[2].write_reference(V::Bool(true));
                    branch_close_pair(&c[0], &c[1])
                })],
            )
        },
    );
    let pull = V::fun(vec![left, right, results, closed, close.clone()], |c, _| {
        if c[3].read_reference().boolean() {
            return stream_pure(stream_done());
        }
        let both = V::Tuple(Shared::new(vec![c[0].clone(), c[1].clone()]));
        let collect = V::bind(
            native_operation("std/queue::take", vec![c[2].clone()]),
            V::fun(vec![both.clone(), c[2].clone()], |c, first| {
                let branch = c[0].index(branch_side(&first));
                V::bind(
                    branch_result(&branch, first.index(1)),
                    V::fun(vec![c[0].clone(), c[1].clone(), first], |c, first_value| {
                        V::bind(
                            native_operation("std/queue::take", vec![c[1].clone()]),
                            V::fun(vec![c[0].clone(), c[2].clone(), first_value], |c, second| {
                                let branch = c[0].index(branch_side(&second));
                                V::bind(
                                    branch_result(&branch, second.index(1)),
                                    V::fun(c.to_vec(), |c, second_value| {
                                        let (left, right) = if branch_side(&c[1]) == 0 {
                                            (c[2].clone(), second_value)
                                        } else {
                                            (second_value, c[2].clone())
                                        };
                                        branch_succeed(V::Tuple(Shared::new(vec![left, right])))
                                    }),
                                )
                            }),
                        )
                    }),
                )
            }),
        );
        let step = branch_then(
            branch_demand(&c[0]),
            branch_then(
                branch_demand(&c[1]),
                V::bind(
                    collect,
                    V::fun(vec![c[4].clone()], |c, pair| {
                        let (left, right) = (pair.index(0), pair.index(1));
                        if left.tagged("std/prelude::Nothing") || right.tagged("std/prelude::Nothing") {
                            return branch_then(c[0].apply(V::Unit), stream_pure(stream_done()));
                        }
                        stream_pure(stream_item(V::Tuple(Shared::new(vec![
                            left.payload(),
                            right.payload(),
                        ]))))
                    }),
                ),
            ),
        );
        branch_on_error(step, c[4].apply(V::Unit))
    });
    stream_cursor(pull, close)
}
// Merge state: [left, right, results, closed, pending flags, done flags,
// close, slots]. A slot holds an outcome that arrived but was not chosen yet.
fn native_stream_merge_cursor(left: V, right: V, results: V) -> V {
    let closed = stream_cell(V::Bool(false));
    let close = V::fun(
        vec![left.clone(), right.clone(), closed.clone()],
        |c, _| {
            V::effect(
                Instruction::Defer,
                vec![V::fun(c.to_vec(), |c, _| {
                    c[2].write_reference(V::Bool(true));
                    branch_close_pair(&c[0], &c[1])
                })],
            )
        },
    );
    let pair = |value: V| V::Tuple(Shared::new(vec![value.clone(), value]));
    let state = V::Tuple(Shared::new(vec![
        left,
        right,
        results,
        closed,
        stream_cell(pair(V::Bool(false))),
        stream_cell(pair(V::Bool(false))),
        close.clone(),
        stream_cell(pair(V::Unit)),
    ]));
    let pull = V::fun(vec![state], |c, _| {
        if c[0].index(3).read_reference().boolean() {
            return stream_pure(stream_done());
        }
        let close = c[0].index(6).apply(V::Unit);
        branch_on_error(native_stream_merge_step(c[0].clone()), close)
    });
    stream_cursor(pull, close)
}
fn branch_flags(cell: &V) -> [bool; 2] {
    let flags = cell.read_reference();
    [flags.index(0).boolean(), flags.index(1).boolean()]
}
fn branch_set_slot(cell: &V, side: usize, value: V) {
    let current = cell.read_reference();
    let mut values = vec![current.index(0), current.index(1)];
    values[side] = value;
    cell.write_reference(V::Tuple(Shared::new(values)));
}
fn branch_set_flag(cell: &V, side: usize, value: bool) {
    branch_set_slot(cell, side, V::Bool(value));
}
fn branch_settle() -> V {
    V::effect(
        Instruction::Concurrent(ConcurrentOperation::Settle),
        vec![],
    )
}
// Moves every outcome that has already arrived into its branch's slot.
fn native_stream_merge_collect(state: V) -> V {
    V::bind(
        native_operation("std/queue::tryTake", vec![state.index(2)]),
        V::fun(vec![state], |c, taken| {
            if !(taken.tagged("std/prelude::Right") && taken.payload().tagged("std/prelude::Just")) {
                return branch_succeed(V::Unit);
            }
            native_stream_merge_store(&c[0], taken.payload().payload());
            native_stream_merge_collect(c[0].clone())
        }),
    )
}
fn native_stream_merge_store(state: &V, message: V) {
    let side = branch_side(&message);
    branch_set_flag(&state.index(4), side, false);
    branch_set_slot(&state.index(7), side, message.index(1));
}
// Asks every branch that is neither finished nor already asked. After the
// other Fibers settle, a failure wins, then the left value, then the right
// one, as TypeScript scans its settled branches in order.
fn native_stream_merge_step(state: V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![state], |c, _| {
            let state = &c[0];
            let pending = state.index(4);
            let done = branch_flags(&state.index(5));
            let slots = state.index(7).read_reference();
            let mut demands = Vec::new();
            for side in 0..2 {
                if !done[side]
                    && !branch_flags(&pending)[side]
                    && matches!(slots.index(side), V::Unit)
                {
                    branch_set_flag(&pending, side, true);
                    demands.push(branch_demand(&state.index(side)));
                }
            }
            let waiting = (0..2).any(|side| !matches!(slots.index(side), V::Unit));
            if !branch_flags(&pending).contains(&true) && !waiting {
                return branch_then(state.index(6).apply(V::Unit), stream_pure(stream_done()));
            }
            let gather = branch_then(
                branch_settle(),
                V::bind(
                    native_stream_merge_collect(state.clone()),
                    V::fun(vec![state.clone()], |c, _| {
                        let slots = c[0].index(7).read_reference();
                        if (0..2).any(|side| !matches!(slots.index(side), V::Unit)) {
                            return native_stream_merge_choose(c[0].clone());
                        }
                        V::bind(
                            native_operation("std/queue::take", vec![c[0].index(2)]),
                            V::fun(vec![c[0].clone()], |c, message| {
                                native_stream_merge_store(&c[0], message);
                                branch_then(
                                    branch_settle(),
                                    branch_then(
                                        native_stream_merge_collect(c[0].clone()),
                                        native_stream_merge_choose(c[0].clone()),
                                    ),
                                )
                            }),
                        )
                    }),
                ),
            );
            demands
                .into_iter()
                .rev()
                .fold(gather, |next, demand| branch_then(demand, next))
        })],
    )
}
fn native_stream_merge_choose(state: V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![state], |c, _| {
            let state = &c[0];
            let slots = state.index(7).read_reference();
            let failed = |outcome: &V| outcome.tagged(BRANCH_FAIL) || outcome.tagged(BRANCH_ENDED);
            let side = (0..2)
                .find(|side| failed(&slots.index(*side)))
                .or_else(|| (0..2).find(|side| !matches!(slots.index(*side), V::Unit)))
                .expect("merge slot");
            let outcome = slots.index(side);
            branch_set_slot(&state.index(7), side, V::Unit);
            V::bind(
                branch_result(&state.index(side), outcome),
                V::fun(vec![state.clone(), V::Int(side as i64)], |c, value| {
                    if value.tagged("std/prelude::Just") {
                        return stream_pure(value);
                    }
                    let V::Int(side) = c[1] else { unreachable!() };
                    branch_set_flag(&c[0].index(5), side as usize, true);
                    native_stream_merge_step(c[0].clone())
                }),
            )
        })],
    )
}
// `buffer` pulls ahead of its consumer while fewer than `capacity` values wait.
// A branch failure discards the waiting values, as in TypeScript.
// State: [branch, results, closed, failed, finished, failure].
fn native_stream_buffer(capacity: V, source: V) -> V {
    V::fun(vec![capacity, source], |c, _| {
        let capacity = c[0].clone();
        V::bind(
            native_operation("std/semaphore::make", vec![capacity]),
            V::fun(vec![c[1].clone()], |c, space| {
                V::bind(
                    native_operation("std/queue::unbounded", vec![V::Unit]),
                    V::fun(vec![c[0].clone(), space], |c, results| {
                        let demand = native_operation("std/semaphore::acquire", vec![c[1].clone()]);
                        V::bind(
                            branch_open(c[0].clone(), 0, results.clone(), Some(demand)),
                            V::fun(vec![results], |c, branch| {
                                stream_pure(native_stream_buffer_cursor(branch, c[0].clone()))
                            }),
                        )
                    }),
                )
            }),
        )
    })
}
fn native_stream_buffer_cursor(branch: V, results: V) -> V {
    let state = V::Tuple(Shared::new(vec![
        branch,
        results,
        stream_cell(V::Bool(false)),
        stream_cell(V::Unit),
    ]));
    let close = V::fun(vec![state.clone()], |c, _| {
        V::effect(
            Instruction::Defer,
            vec![V::fun(c.to_vec(), |c, _| {
                c[0].index(2).write_reference(V::Bool(true));
                branch_close(&c[0].index(0))
            })],
        )
    });
    let pull = V::fun(vec![state], |c, _| native_stream_buffer_pull(c[0].clone()));
    stream_cursor(pull, close)
}
fn native_stream_buffer_pull(state: V) -> V {
    V::effect(
        Instruction::Defer,
        vec![V::fun(vec![state], |c, _| {
            let state = &c[0];
            if state.index(2).read_reference().boolean() {
                return stream_pure(stream_done());
            }
            let ended = state.index(3).read_reference();
            if !matches!(ended, V::Unit) {
                return branch_result(&state.index(0), ended);
            }
            V::bind(
                native_operation("std/queue::take", vec![state.index(1)]),
                V::fun(vec![state.clone()], |c, message| {
                    let outcome = message.index(1);
                    if outcome.tagged(BRANCH_ITEM) {
                        return branch_then(
                            native_operation("std/semaphore::release", vec![message.index(2)]),
                            branch_result(&c[0].index(0), outcome),
                        );
                    }
                    // The end, a failure or a Defect is final; later pulls
                    // report it again without waiting.
                    c[0].index(3).write_reference(outcome.clone());
                    branch_result(&c[0].index(0), outcome)
                }),
            )
        })],
    )
}

#[cfg(test)]
mod stream_branch_tests {
    use super::*;

    fn array(values: &[i64]) -> V {
        native_stream_operation(
            "std/stream::fromArray",
            vec![V::Array(Shared::new(
                values.iter().map(|value| V::Int(*value)).collect(),
            ))],
        )
    }
    fn collect(source: V) -> NativeOutcome {
        NativeContext::default().root(native_stream_operation(
            "std/stream::runCollect",
            vec![source],
        ))
    }

    #[test]
    fn zip_pairs_until_the_shorter_side_ends() {
        enable_cycle_collection();
        let zipped = native_stream_operation("std/stream::zip", vec![array(&[1, 2]), array(&[3, 4, 5])]);
        match collect(zipped) {
            Ok(value) => assert_eq!(value.render(), "[(3, 1), (4, 2)]"),
            Err(NativeExit::Cancelled) => panic!("cancelled"),
            Err(NativeExit::Defect(errors)) => panic!("defect {:?}", errors.iter().map(|e| e.value.clone()).collect::<Vec<_>>()),
            Err(NativeExit::Failure(_)) => panic!("failure"),
        }
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
