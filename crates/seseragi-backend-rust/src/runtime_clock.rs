// Exact opaque Duration values. The two private Int limbs are each safe Ints;
// no language Int/Float is substituted for the nominal Duration owner.
fn native_duration(nanoseconds: u64) -> V {
    assert!(nanoseconds <= i64::MAX as u64);
    V::adt(
        "std/time::Duration",
        V::Tuple(Shared::new(vec![
            V::Int((nanoseconds / 1_000_000_000) as i64),
            V::Int((nanoseconds % 1_000_000_000) as i64),
        ])),
    )
}
fn native_duration_nanoseconds(value: &V) -> Result<u64, String> {
    let V::Adt(value) = value else {
        return Err("native Clock: expected canonical Duration".into());
    };
    if value.get().0 != "std/time::Duration" {
        return Err("native Clock: invalid Duration owner".into());
    }
    let V::Tuple(parts) = &value.get().1 else {
        return Err("native Clock: invalid Duration payload".into());
    };
    let [V::Int(seconds), V::Int(nanos)] = parts.get().as_slice() else {
        return Err("native Clock: invalid Duration limbs".into());
    };
    if *seconds < 0 || !(0..1_000_000_000).contains(nanos) {
        return Err("native Clock: invalid Duration range".into());
    }
    (*seconds as u64)
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(*nanos as u64))
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or_else(|| "native Clock: Duration outside supported range".into())
}
fn native_clock_operation(name: &str, arguments: Vec<V>) -> V {
    match (name, arguments.as_slice()) {
        ("std/time::milliseconds", [V::Int(value)]) => {
            let value = checked_int_value(*value);
            if value < 0 {
                native_left(V::adt("std/time::NegativeDuration", V::Int(value)))
            } else if let Some(nanos) = (value as u64)
                .checked_mul(1_000_000)
                .filter(|n| *n <= i64::MAX as u64)
            {
                native_right(native_duration(nanos))
            } else {
                native_left(V::adt("std/time::DurationOutsideRange", V::Unit))
            }
        }
        ("std/time::NegativeDuration", [value]) => {
            V::adt("std/time::NegativeDuration", value.clone())
        }
        ("std/time::DurationOutsideRange", []) => V::adt("std/time::DurationOutsideRange", V::Unit),
        ("std/clock::sleep", [_]) => V::effect(
            Instruction::Concurrent(ConcurrentOperation::ClockSleep),
            arguments,
        ),
        _ => panic!("Rust ABI: unadmitted native Clock operation"),
    }
}

thread_local! {
    static NATIVE_TIMERS: Cell<usize> = const { Cell::new(0) };
    static NATIVE_TIMERS_PEAK: Cell<usize> = const { Cell::new(0) };
    static NATIVE_TIMERS_CREATED: Cell<usize> = const { Cell::new(0) };
}
struct NativeTimer {
    deadline: std::time::Instant,
}
impl NativeTimer {
    fn start(value: &V) -> Result<Self, NativeExit> {
        let nanos = native_duration_nanoseconds(value)
            .map_err(|message| NativeExit::defects(vec![message]))?;
        let deadline = std::time::Instant::now()
            .checked_add(std::time::Duration::from_nanos(nanos))
            .ok_or_else(|| {
                NativeExit::defects(vec![
                    "native Clock: host deadline is outside its range".into()
                ])
            })?;
        NATIVE_TIMERS.with(|live| {
            live.set(live.get() + 1);
            NATIVE_TIMERS_PEAK.with(|peak| peak.set(peak.get().max(live.get())));
        });
        NATIVE_TIMERS_CREATED.with(|n| n.set(n.get() + 1));
        Ok(Self { deadline })
    }
}
impl Drop for NativeTimer {
    fn drop(&mut self) {
        NATIVE_TIMERS.with(|live| live.set(live.get() - 1));
    }
}
fn run_native_clock_entry(action: impl FnOnce() -> V, tracing: bool) -> i32 {
    run_native_clock_entry_with_show(action, tracing, |value| V::text(&value.render()))
}
fn run_native_clock_entry_with_show(
    action: impl FnOnce() -> V,
    tracing: bool,
    show: fn(V) -> V,
) -> i32 {
    assert_eq!(
        NATIVE_TIMERS.with(Cell::get),
        0,
        "previous native timer retained"
    );
    NATIVE_TIMERS_PEAK.with(|n| n.set(0));
    NATIVE_TIMERS_CREATED.with(|n| n.set(0));
    let code = run_native_entry_with_show(action, tracing, show);
    if std::env::var_os("SESERAGI_NATIVE_METRICS").is_some() {
        eprintln!(
            "native clock timers-live={} timers-peak={} timers-created={}",
            NATIVE_TIMERS.with(Cell::get),
            NATIVE_TIMERS_PEAK.with(Cell::get),
            NATIVE_TIMERS_CREATED.with(Cell::get)
        );
    }
    assert_eq!(
        NATIVE_TIMERS.with(Cell::get),
        0,
        "native timers released after cleanup"
    );
    code
}

impl NativeScheduler {
    fn poll_timers(&mut self) {
        // Do not read a host clock for programs with no pending timer.
        if !self.tasks.values().any(|task| task.timer.is_some()) {
            return;
        }
        let now = std::time::Instant::now();
        let mut ready = self
            .tasks
            .iter()
            .filter_map(|(id, task)| {
                let timer = task.timer.as_ref()?;
                (timer.deadline <= now).then_some((timer.deadline, *id))
            })
            .collect::<Vec<_>>();
        ready.sort();
        for (_, id) in ready {
            let task = self.tasks.get_mut(&id).expect("registered native timer");
            drop(task.timer.take());
            task.machine.resume(
                if task.machine.context.cancelled && !task.machine.context.masked {
                    Err(NativeExit::Cancelled)
                } else {
                    Ok(V::Unit)
                },
            );
            self.ready.push_back(id);
        }
    }
    fn next_timer_wait(&self) -> Option<std::time::Duration> {
        self.tasks
            .values()
            .filter_map(|task| task.timer.as_ref().map(|t| t.deadline))
            .min()
            .map(|deadline| deadline.saturating_duration_since(std::time::Instant::now()))
    }
}

#[cfg(test)]
mod clock_tests {
    use super::*;
    #[test]
    fn duration_conversion_keeps_exact_range_and_typed_failures() {
        for value in [0, 1, 9_223_372_036_854] {
            let result = native_clock_operation("std/time::milliseconds", vec![V::Int(value)]);
            assert!(result.tagged("std/prelude::Right"));
            assert_eq!(
                native_duration_nanoseconds(&result.payload()).unwrap(),
                value as u64 * 1_000_000
            );
        }
        let negative = native_clock_operation("std/time::milliseconds", vec![V::Int(-1)]);
        assert!(negative.payload().tagged("std/time::NegativeDuration"));
        let large =
            native_clock_operation("std/time::milliseconds", vec![V::Int(9_223_372_036_855)]);
        assert!(large.payload().tagged("std/time::DurationOutsideRange"));
        assert_eq!(
            native_duration_nanoseconds(&native_duration(i64::MAX as u64)).unwrap(),
            i64::MAX as u64
        );
        assert!(native_duration_nanoseconds(&V::adt("local::Duration", V::Unit)).is_err());
        drop((negative, large));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn sleep_creation_is_cold_and_repeated_execution_releases_timers() {
        let plan = native_clock_operation("std/clock::sleep", vec![native_duration(0)]);
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        for _ in 0..2 {
            assert!(NativeContext::default().root(plan.clone()).is_ok());
            assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        }
        drop(plan);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    fn then(first: V, next: V) -> V {
        V::bind(first, V::fun(vec![next], |c, _| c[0].clone()))
    }
    fn sleep(millis: u64) -> V {
        native_clock_operation(
            "std/clock::sleep",
            vec![native_duration(millis * 1_000_000)],
        )
    }
    fn append(log: &V, text: &str) -> V {
        V::leaf(vec![log.clone(), V::text(text)], |c, _| {
            c[0].write_reference(binary("+", c[0].read_reference(), c[1].clone()));
            Ok(V::Unit)
        })
    }
    fn await_body_timer() -> V {
        // Observe registration, rather than race two wall-clock deadlines.
        V::effect(
            Instruction::Defer,
            vec![V::fun(vec![], |_, _| {
                if NATIVE_TIMERS.with(Cell::get) == 1 {
                    native_operation("effect.succeed", vec![V::Unit])
                } else {
                    then(
                        native_operation("std/effect::yieldNow", vec![V::Unit]),
                        await_body_timer(),
                    )
                }
            })],
        )
    }
    #[test]
    fn sibling_cancellation_removes_registered_body_timer_and_waits_for_release() {
        enable_cycle_collection();
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        NATIVE_TIMERS_CREATED.with(|n| n.set(0));
        let log = V::reference(V::text(""));
        let ready = V::Deferred(Shared::new(NativeDeferred::default()));
        let signal = native_operation("std/deferred::succeed", vec![V::Unit, ready.clone()]);
        let release = V::fun(vec![then(sleep(0), append(&log, "R"))], |c, _| c[0].clone());
        let acquire = native_operation(
            "std/effect::acquireRelease",
            vec![append(&log, "A"), release],
        );
        // Ten days cannot expire within the outer test's bounded lifetime.
        let child = then(acquire, then(signal, sleep(864_000_000)));
        let parent = V::bind(
            native_operation("std/effect::fork", vec![child]),
            V::fun(vec![ready.clone(), log.clone()], |c, child| {
                then(
                    native_operation("std/deferred::await", vec![c[0].clone()]),
                    then(
                        await_body_timer(),
                        then(
                            append(&c[1], "S"),
                            then(
                                native_operation("std/effect::interrupt", vec![child]),
                                append(&c[1], "C"),
                            ),
                        ),
                    ),
                )
            }),
        );
        assert!(NativeContext {
            gc_every: 1,
            ..NativeContext::default()
        }
        .root(parent)
        .is_ok());
        assert_eq!(log.read_reference().render(), "ASRC");
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        assert_eq!(NATIVE_TIMERS_CREATED.with(Cell::get), 2);
        drop((log, ready));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn acquisition_handoff_registers_release_and_masked_timer_keeps_its_waiter() {
        enable_cycle_collection();
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        // Directly cancel an already suspended masked machine, without any
        // host-time race. Its timer must remain registered and resume normally.
        let mut scheduler = NativeScheduler::default();
        let handle = scheduler.spawn(
            NativeContext {
                masked: true,
                ..NativeContext::default()
            },
            sleep(0),
        );
        let id = handle.fiber().task;
        let mut task = scheduler.tasks.remove(&id).unwrap();
        let NativeTurn::Operation(ConcurrentOperation::ClockSleep, captures) = task.machine.turn()
        else {
            panic!("the real machine must suspend on its Clock operation");
        };
        scheduler.operation(id, &mut task, ConcurrentOperation::ClockSleep, captures);
        assert!(task.machine.state.is_none());
        scheduler.tasks.insert(id, task);
        scheduler.ready.clear();
        scheduler.cancel(id);
        assert!(scheduler.tasks[&id].machine.context.cancelled);
        assert!(scheduler.tasks[&id].machine.context.masked);
        assert!(scheduler.tasks[&id].timer.is_some());
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 1);
        scheduler.poll_timers();
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        assert_eq!(scheduler.ready.iter().copied().collect::<Vec<_>>(), [id]);
        assert!(matches!(
            scheduler.tasks[&id].machine.state,
            Some(NativeState::Return(Ok(V::Unit)))
        ));
        drop((scheduler, handle));
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        NATIVE_TIMERS_CREATED.with(|n| n.set(0));
        let log = V::reference(V::text(""));
        let request = V::leaf(vec![log.clone()], |c, context| {
            context.cancelled = true;
            c[0].write_reference(V::text("A"));
            Ok(V::Unit)
        });
        // Successful acquisition registers release before observing this request.
        let acquire = request;
        let release = V::fun(vec![then(sleep(0), append(&log, "R"))], |c, _| c[0].clone());
        let plan = native_operation("std/effect::acquireRelease", vec![acquire, release]);
        assert!(matches!(
            NativeContext {
                gc_every: 1,
                ..NativeContext::default()
            }
            .root(plan),
            Err(NativeExit::Cancelled)
        ));
        assert_eq!(log.read_reference().render(), "AR");
        assert_eq!(NATIVE_TIMERS_CREATED.with(Cell::get), 1);
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        drop(log);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn invalid_duration_is_a_cold_defect_and_still_closes_scope() {
        enable_cycle_collection();
        let released = V::reference(V::Bool(false));
        let release = V::fun(vec![released.clone()], |c, _| {
            native_operation("std/ref::set", vec![V::Bool(true), c[0].clone()])
        });
        let acquire = native_operation(
            "std/effect::acquireRelease",
            vec![native_operation("effect.succeed", vec![V::Unit]), release],
        );
        let bad =
            native_clock_operation("std/clock::sleep", vec![V::adt("other::Duration", V::Unit)]);
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        let result =
            NativeContext::default().root(V::bind(acquire, V::fun(vec![bad], |c, _| c[0].clone())));
        assert!(
            matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["native Clock: invalid Duration owner"])
        );
        assert_eq!(released.read_reference().render(), "True");
        assert_eq!(NATIVE_TIMERS.with(Cell::get), 0);
        drop((result, released));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
