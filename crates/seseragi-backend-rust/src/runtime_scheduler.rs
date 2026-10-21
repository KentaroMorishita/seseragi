// The scheduler owns suspended machines. Managed values never cross a thread;
// continuations and scope finalizers remain collector roots while waiting.
#[derive(Clone, Copy)]
enum ConcurrentOperation {
    Fork,
    Supervise,
    AwaitFiber,
    PollFiber,
    Join,
    Interrupt,
    Yield,
    MakeDeferred,
    AwaitDeferred,
    PollDeferred,
    CompleteDeferred,
    SucceedDeferred,
    FailDeferred,
}
impl ConcurrentOperation {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "std/effect::fork" => Self::Fork,
            "std/effect::await" => Self::AwaitFiber,
            "std/effect::poll" => Self::PollFiber,
            "std/effect::join" => Self::Join,
            "std/effect::interrupt" => Self::Interrupt,
            "std/effect::yieldNow" => Self::Yield,
            "std/deferred::make" => Self::MakeDeferred,
            "std/deferred::await" => Self::AwaitDeferred,
            "std/deferred::poll" => Self::PollDeferred,
            "std/deferred::complete" => Self::CompleteDeferred,
            "std/deferred::succeed" => Self::SucceedDeferred,
            "std/deferred::fail" => Self::FailDeferred,
            _ => return None,
        })
    }
}
struct NativeFiber {
    task: usize,
    outcome: RefCell<Option<NativeOutcome>>,
}
#[derive(Default)]
struct NativeDeferred {
    result: RefCell<Option<Result<V, V>>>,
}
impl TraceValue for NativeFiber {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        let outcome = self
            .outcome
            .try_borrow_mut()
            .map_err(|_| "borrowed native Fiber")?;
        match outcome.as_ref() {
            Some(Ok(value) | Err(NativeExit::Failure(value))) => visit(value),
            _ => {}
        }
        Ok(())
    }
    fn sever_reference(&self) -> Option<V> {
        match self.outcome.replace(None) {
            Some(Ok(value) | Err(NativeExit::Failure(value))) => Some(value),
            _ => None,
        }
    }
}
impl TraceValue for NativeDeferred {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        let result = self
            .result
            .try_borrow_mut()
            .map_err(|_| "borrowed native Deferred")?;
        if let Some(Ok(value) | Err(value)) = result.as_ref() {
            visit(value);
        }
        Ok(())
    }
    fn sever_reference(&self) -> Option<V> {
        self.result.replace(None).map(|result| match result {
            Ok(value) | Err(value) => value,
        })
    }
}
impl V {
    fn fiber(&self) -> &NativeFiber {
        let Self::Fiber(value) = self else {
            panic!("Rust ABI: expected Fiber")
        };
        value.get()
    }
    fn deferred(&self) -> &NativeDeferred {
        let Self::Deferred(value) = self else {
            panic!("Rust ABI: expected Deferred")
        };
        value.get()
    }
}
fn native_either(value: Result<V, V>) -> V {
    match value {
        Ok(value) => V::adt("std/prelude::Right", value),
        Err(value) => V::adt("std/prelude::Left", value),
    }
}
fn native_fiber_exit(outcome: NativeOutcome) -> NativeOutcome {
    match outcome {
        Ok(value) => Ok(V::adt("std/effect::FiberSucceeded", value)),
        Err(NativeExit::Failure(value)) => Ok(V::adt("std/effect::FiberFailed", value)),
        Err(NativeExit::Cancelled) => Ok(V::adt("std/effect::FiberCancelled", V::Unit)),
        Err(defect) => Err(defect),
    }
}
#[derive(Clone, Copy)]
enum FiberWait {
    Await,
    Join,
    Interrupt,
}
impl FiberWait {
    fn outcome(self, result: NativeOutcome) -> NativeOutcome {
        match self {
            Self::Await => native_fiber_exit(result),
            Self::Join => result,
            Self::Interrupt => match result {
                Err(NativeExit::Defect(errors)) => Err(NativeExit::Defect(errors)),
                _ => Ok(V::Unit),
            },
        }
    }
}
enum NativeWait {
    Fiber(V, FiberWait),
    Deferred(V),
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WaitKey {
    Fiber(usize),
    Deferred(usize),
}
impl NativeWait {
    fn key(&self) -> WaitKey {
        match self {
            Self::Fiber(value, _) => WaitKey::Fiber(value.fiber().task),
            Self::Deferred(value) => WaitKey::Deferred(value.node_id().expect("Deferred node")),
        }
    }
    fn ready(&self) -> Option<NativeOutcome> {
        match self {
            Self::Fiber(value, mode) => value
                .fiber()
                .outcome
                .borrow()
                .clone()
                .map(|v| mode.outcome(v)),
            Self::Deferred(value) => value
                .deferred()
                .result
                .borrow()
                .clone()
                .map(|v| v.map_err(NativeExit::Failure)),
        }
    }
}
struct NativeTask {
    machine: NativeMachine,
    handle: V,
    waiting: Option<NativeWait>,
    file: Option<NativeFileTask>,
}
#[derive(Default)]
struct NativeScheduler {
    tasks: BTreeMap<usize, NativeTask>,
    ready: std::collections::VecDeque<usize>,
    waiters: BTreeMap<WaitKey, Vec<usize>>,
}
thread_local! {
    // Handles retained across independent root executions cannot address a
    // later task through a recycled scheduler-local index.
    static NEXT_NATIVE_TASK: Cell<usize> = const { Cell::new(0) };
}
impl NativeScheduler {
    fn spawn(&mut self, context: NativeContext, plan: V) -> V {
        let id = NEXT_NATIVE_TASK.with(|next| {
            let id = next.get();
            next.set(id.checked_add(1).expect("native task identity exhausted"));
            id
        });
        let handle = V::Fiber(Shared::new(NativeFiber {
            task: id,
            outcome: RefCell::new(None),
        }));
        self.tasks.insert(
            id,
            NativeTask {
                machine: NativeMachine::new(context, plan),
                handle: handle.clone(),
                waiting: None,
                file: None,
            },
        );
        self.ready.push_back(id);
        handle
    }
    fn run(context: &mut NativeContext, plan: V) -> NativeOutcome {
        let mut scheduler = Self::default();
        let root = scheduler.spawn(std::mem::take(context), plan);
        let root_id = root.fiber().task;
        loop {
            scheduler.poll_files();
            let Some(id) = scheduler.ready.pop_front() else {
                // A program awaiting an uncompleted Deferred stays pending.
                // No artificial typed failure or Defect is invented for it.
                std::thread::park();
                continue;
            };
            let mut task = scheduler.tasks.remove(&id).expect("runnable native task");
            match task.machine.turn() {
                NativeTurn::Runnable => scheduler.ready.push_back(id),
                NativeTurn::Operation(operation, captures) => {
                    scheduler.operation(id, &mut task, operation, captures)
                }
                NativeTurn::FileSystem(operation, captures) => {
                    match NativeFileTask::start(operation, captures) {
                        Ok(file) => task.file = Some(file),
                        Err(exit) => {
                            task.machine.resume(Err(exit));
                            scheduler.ready.push_back(id);
                        }
                    }
                }
                NativeTurn::CloseScope(original, previous_mask) => {
                    let scope = task
                        .machine
                        .context
                        .scopes
                        .last_mut()
                        .expect("closing scope");
                    // Completed children have already left their scope; their
                    // stored failure is observed only through await/join.
                    scope.finalizers.retain(|plan| {
                        let (instruction, captures) = plan.plan();
                        !matches!(
                            instruction,
                            Instruction::Concurrent(ConcurrentOperation::Supervise)
                        ) || captures[0].fiber().outcome.borrow().is_none()
                    });
                    let children = &task
                        .machine
                        .context
                        .scopes
                        .last()
                        .expect("closing scope")
                        .children;
                    for child in children {
                        scheduler.cancel(child.fiber().task);
                    }
                    task.machine.state = Some(task.machine.context.cleanup(
                        &mut task.machine.stack,
                        original,
                        Vec::new(),
                        previous_mask,
                    ));
                    scheduler.ready.push_back(id);
                }
                NativeTurn::Complete(outcome) => {
                    task.handle.fiber().outcome.replace(Some(outcome.clone()));
                    scheduler.wake(WaitKey::Fiber(id));
                    if id == root_id {
                        *context = task.machine.context;
                        debug_assert!(scheduler.tasks.is_empty(), "root scope awaits all children");
                        return outcome;
                    }
                    continue;
                }
            }
            scheduler.tasks.insert(id, task);
        }
    }
    fn cancel(&mut self, id: usize) {
        let Some(task) = self.tasks.get_mut(&id) else {
            return;
        };
        task.machine.context.cancelled = true;
        if task.machine.context.masked {
            return;
        }
        if let Some(file) = &task.file {
            // A blocking OS call has no invented abort. Observe it and join
            // its worker after close, before running language finalizers.
            file.cancel();
        }
        if let Some(wait) = task.waiting.take() {
            if let Some(waiters) = self.waiters.get_mut(&wait.key()) {
                waiters.retain(|other| *other != id);
                if waiters.is_empty() {
                    self.waiters.remove(&wait.key());
                }
            }
            task.machine.resume(Err(NativeExit::Cancelled));
            self.ready.push_back(id);
        }
    }
    fn poll_files(&mut self) {
        for (id, task) in &mut self.tasks {
            let Some(file) = &mut task.file else {
                continue;
            };
            if let Some((outcome, notes)) = file.poll() {
                for note in notes {
                    eprintln!("native filesystem: {note}");
                }
                task.file = None;
                task.machine.resume(outcome);
                self.ready.push_back(*id);
            }
        }
    }
    fn wait(&mut self, id: usize, task: &mut NativeTask, wait: NativeWait) {
        if let Some(result) = wait.ready() {
            task.machine.resume(result);
            self.ready.push_back(id);
        } else {
            self.waiters.entry(wait.key()).or_default().push(id);
            task.waiting = Some(wait);
        }
    }
    fn wake(&mut self, key: WaitKey) {
        for id in self.waiters.remove(&key).unwrap_or_default() {
            let task = self.tasks.get_mut(&id).expect("registered native waiter");
            let wait = task.waiting.take().expect("suspended native waiter");
            task.machine
                .resume(wait.ready().expect("completed native wait"));
            self.ready.push_back(id);
        }
    }
    fn operation(
        &mut self,
        id: usize,
        task: &mut NativeTask,
        operation: ConcurrentOperation,
        c: Vec<V>,
    ) {
        let result = match operation {
            ConcurrentOperation::Fork => {
                let child = self.spawn(
                    NativeContext {
                        gc_every: task.machine.context.gc_every,
                        ..NativeContext::default()
                    },
                    c[0].clone(),
                );
                let scope = task
                    .machine
                    .context
                    .scopes
                    .last_mut()
                    .expect("owning Fiber scope");
                scope.children.push(child.clone());
                scope.finalizers.push(V::effect(
                    Instruction::Concurrent(ConcurrentOperation::Supervise),
                    vec![child.clone()],
                ));
                Ok(child)
            }
            ConcurrentOperation::AwaitFiber
            | ConcurrentOperation::Join
            | ConcurrentOperation::Interrupt
            | ConcurrentOperation::Supervise => {
                let mode = match operation {
                    ConcurrentOperation::AwaitFiber => FiberWait::Await,
                    ConcurrentOperation::Join => FiberWait::Join,
                    _ => {
                        if c[0].fiber().task == id && !task.machine.context.masked {
                            task.machine.context.cancelled = true;
                            task.machine.resume(Err(NativeExit::Cancelled));
                            self.ready.push_back(id);
                            return;
                        }
                        self.cancel(c[0].fiber().task);
                        FiberWait::Interrupt
                    }
                };
                self.wait(id, task, NativeWait::Fiber(c[0].clone(), mode));
                return;
            }
            ConcurrentOperation::PollFiber => {
                let exit = c[0]
                    .fiber()
                    .outcome
                    .borrow()
                    .clone()
                    .and_then(|v| native_fiber_exit(v).ok());
                Ok(native_maybe(exit))
            }
            ConcurrentOperation::Yield => Ok(V::Unit),
            ConcurrentOperation::MakeDeferred => {
                Ok(V::Deferred(Shared::new(NativeDeferred::default())))
            }
            ConcurrentOperation::AwaitDeferred => {
                self.wait(id, task, NativeWait::Deferred(c[0].clone()));
                return;
            }
            ConcurrentOperation::PollDeferred => Ok(native_maybe(
                c[0].deferred().result.borrow().clone().map(native_either),
            )),
            ConcurrentOperation::CompleteDeferred
            | ConcurrentOperation::SucceedDeferred
            | ConcurrentOperation::FailDeferred => {
                let result = match operation {
                    ConcurrentOperation::SucceedDeferred => Ok(c[0].clone()),
                    ConcurrentOperation::FailDeferred => Err(c[0].clone()),
                    _ => {
                        if c[0].tagged("std/prelude::Right") {
                            Ok(c[0].payload())
                        } else if c[0].tagged("std/prelude::Left") {
                            Err(c[0].payload())
                        } else {
                            panic!("Rust ABI: expected Either")
                        }
                    }
                };
                let first = {
                    let mut slot = c[1].deferred().result.borrow_mut();
                    if slot.is_some() {
                        false
                    } else {
                        *slot = Some(result);
                        true
                    }
                };
                if first {
                    self.wake(WaitKey::Deferred(c[1].node_id().expect("Deferred node")));
                }
                Ok(V::Bool(first))
            }
        };
        task.machine.resume(result);
        self.ready.push_back(id);
    }
}

#[cfg(test)]
mod scheduler_tests {
    use super::*;
    fn op(name: &str, args: Vec<V>) -> V {
        native_operation(name, args)
    }
    fn done(value: V) -> V {
        op("effect.succeed", vec![value])
    }
    fn then(first: V, second: V) -> V {
        V::bind(first, V::fun(vec![second], |c, _| c[0].clone()))
    }
    fn deferred() -> V {
        V::Deferred(Shared::new(NativeDeferred::default()))
    }
    fn append(log: &V, text: &str) -> V {
        V::leaf(vec![log.clone(), V::text(text)], |c, _| {
            let text = c[0].read_reference().render() + &c[1].render();
            c[0].write_reference(V::text(&text));
            Ok(V::Unit)
        })
    }
    fn acquired(log: &V, body: V) -> V {
        let release = V::fun(vec![append(log, "R")], |c, _| c[0].clone());
        then(
            op(
                "std/effect::acquireRelease",
                vec![append(log, "A"), release],
            ),
            body,
        )
    }
    fn run(plan: V) -> NativeOutcome {
        NativeContext {
            gc_every: 1,
            ..NativeContext::default()
        }
        .root(plan)
    }
    #[test]
    fn interrupt_waits_for_one_release_and_detaches_the_deferred_waiter() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let gate = deferred();
        let ready = deferred();
        let child = acquired(
            &log,
            then(
                op("std/deferred::succeed", vec![V::Unit, ready.clone()]),
                op("std/deferred::await", vec![gate.clone()]),
            ),
        );
        let plan = V::bind(
            op("std/effect::fork", vec![child]),
            V::fun(vec![ready], |c, fiber| {
                then(
                    op("std/deferred::await", vec![c[0].clone()]),
                    then(
                        op("std/effect::interrupt", vec![fiber.clone()]),
                        then(
                            op("std/effect::interrupt", vec![fiber.clone()]),
                            op("std/effect::await", vec![fiber]),
                        ),
                    ),
                )
            }),
        );
        let exit = run(plan).ok().unwrap();
        assert!(exit.tagged("std/effect::FiberCancelled"));
        assert_eq!(log.read_reference().render(), "AR");
        assert!(gate.deferred().result.borrow().is_none());
        drop((exit, gate, log));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn scope_cancels_unjoined_children_on_success_failure_and_defect() {
        enable_cycle_collection();
        for ending in 0..3 {
            let log = V::reference(V::text(""));
            let ready = deferred();
            let child = acquired(
                &log,
                then(
                    op("std/deferred::succeed", vec![V::Unit, ready.clone()]),
                    op("std/deferred::await", vec![deferred()]),
                ),
            );
            let end = V::leaf(vec![V::Int(ending)], |c, _| match c[0] {
                V::Int(0) => Ok(V::Unit),
                V::Int(1) => Err(NativeExit::Failure(V::Int(7))),
                _ => Err(NativeExit::Defect(vec!["body".into()])),
            });
            let plan = then(
                op("std/effect::fork", vec![child]),
                then(op("std/deferred::await", vec![ready]), end),
            );
            let result = run(plan);
            assert!(match ending {
                0 => result.is_ok(),
                1 => matches!(result, Err(NativeExit::Failure(_))),
                _ => matches!(result, Err(NativeExit::Defect(_))),
            });
            assert_eq!(log.read_reference().render(), "AR");
            drop((result, log));
            collect_cycles().unwrap();
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
    #[test]
    fn cancelling_a_join_waiter_does_not_cancel_the_target() {
        enable_cycle_collection();
        let gate = deferred();
        let plan = V::bind(
            op(
                "std/effect::fork",
                vec![op("std/deferred::await", vec![gate.clone()])],
            ),
            V::fun(vec![gate], |c, target| {
                let waiter = op(
                    "std/effect::fork",
                    vec![op("std/effect::join", vec![target.clone()])],
                );
                V::bind(
                    waiter,
                    V::fun(vec![target, c[0].clone()], |c, waiter| {
                        then(
                            op("std/effect::yieldNow", vec![V::Unit]),
                            then(
                                op("std/effect::interrupt", vec![waiter]),
                                then(
                                    op("std/deferred::succeed", vec![V::Int(42), c[1].clone()]),
                                    op("std/effect::join", vec![c[0].clone()]),
                                ),
                            ),
                        )
                    }),
                )
            }),
        );
        assert_eq!(run(plan).ok().unwrap().render(), "42");
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn deferred_first_completion_wakes_fifo_and_second_completion_is_false() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let gate = deferred();
        let child1 = then(
            op("std/deferred::await", vec![gate.clone()]),
            append(&log, "1"),
        );
        let child2 = then(
            op("std/deferred::await", vec![gate.clone()]),
            append(&log, "2"),
        );
        let plan = then(
            op("std/effect::fork", vec![child1]),
            then(
                op("std/effect::fork", vec![child2]),
                then(
                    op("std/effect::yieldNow", vec![V::Unit]),
                    then(
                        op(
                            "std/deferred::complete",
                            vec![native_either(Ok(V::Int(7))), gate.clone()],
                        ),
                        then(
                            op("std/effect::yieldNow", vec![V::Unit]),
                            op("std/deferred::fail", vec![V::Int(9), gate.clone()]),
                        ),
                    ),
                ),
            ),
        );
        assert!(!run(plan).ok().unwrap().boolean());
        assert_eq!(log.read_reference().render(), "12");
        assert_eq!(
            gate.deferred()
                .result
                .borrow()
                .as_ref()
                .unwrap()
                .as_ref()
                .ok()
                .unwrap()
                .render(),
            "7"
        );
        drop((log, gate));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn failure_and_defect_remain_distinct_and_completed_children_leave_the_scope() {
        enable_cycle_collection();
        let failure = op("effect.fail", vec![V::Int(7)]);
        let plan = V::bind(
            op("std/effect::fork", vec![failure]),
            V::fun(vec![], |_, fiber| op("std/effect::await", vec![fiber])),
        );
        let exit = run(plan).ok().unwrap();
        assert!(exit.tagged("std/effect::FiberFailed"));
        assert_eq!(exit.payload().render(), "7");
        drop(exit);
        let failure = op("effect.fail", vec![V::Int(8)]);
        let plan = V::bind(
            op("std/effect::fork", vec![failure]),
            V::fun(vec![], |_, fiber| op("std/effect::join", vec![fiber])),
        );
        assert!(matches!(run(plan), Err(NativeExit::Failure(V::Int(8)))));
        let defect = V::leaf(vec![], |_, _| {
            Err(NativeExit::Defect(vec!["child defect".into()]))
        });
        let plan = V::bind(
            op("std/effect::fork", vec![defect]),
            V::fun(vec![], |_, fiber| op("std/effect::await", vec![fiber])),
        );
        let Err(NativeExit::Defect(errors)) = run(plan) else {
            panic!()
        };
        assert_eq!(errors, vec!["child defect"]);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn cancellation_is_masked_while_a_finalizer_is_suspended() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let ready = deferred();
        let releasing = deferred();
        let gate = deferred();
        let release = then(
            op("std/deferred::succeed", vec![V::Unit, releasing.clone()]),
            then(
                op("std/deferred::await", vec![gate.clone()]),
                append(&log, "R"),
            ),
        );
        let acquired = op(
            "std/effect::acquireRelease",
            vec![done(V::Unit), V::fun(vec![release], |c, _| c[0].clone())],
        );
        let child = then(
            acquired,
            then(
                op("std/deferred::succeed", vec![V::Unit, ready.clone()]),
                op("std/deferred::await", vec![deferred()]),
            ),
        );
        let completer = then(
            op("std/deferred::await", vec![releasing]),
            op("std/deferred::succeed", vec![V::Unit, gate]),
        );
        let plan = then(
            op("std/effect::fork", vec![completer]),
            V::bind(
                op("std/effect::fork", vec![child]),
                V::fun(vec![ready], |c, fiber| {
                    then(
                        op("std/deferred::await", vec![c[0].clone()]),
                        op("std/effect::interrupt", vec![fiber]),
                    )
                }),
            ),
        );
        assert!(run(plan).is_ok());
        assert_eq!(log.read_reference().render(), "R");
        drop(log);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn opaque_results_are_traced_and_unreachable_completed_cycles_are_reclaimed() {
        enable_cycle_collection();
        let gate = deferred();
        let cell = V::reference(gate.clone());
        let plan = op("std/deferred::succeed", vec![cell.clone(), gate.clone()]);
        assert!(run(plan).ok().unwrap().boolean());
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((gate, cell));
        assert!(collect_cycles().unwrap().reclaimed >= 2);
        assert_eq!(LIVE.with(Cell::get), 0);
        let cell = V::reference(V::Unit);
        let plan = V::bind(
            op("std/effect::fork", vec![done(cell.clone())]),
            V::fun(vec![cell.clone()], |c, fiber| {
                c[0].write_reference(fiber.clone());
                op("std/effect::join", vec![fiber])
            }),
        );
        let result = run(plan).ok().unwrap();
        assert_eq!(collect_cycles().unwrap().reclaimed, 0);
        drop((result, cell));
        assert!(collect_cycles().unwrap().reclaimed >= 2);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn checkpoint_queue_does_not_starve_another_runnable_fiber() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let mut long = done(V::Unit);
        for _ in 0..100 {
            long = then(
                op("std/effect::yieldNow", vec![V::Unit]),
                then(append(&log, "1"), long),
            );
        }
        let short = append(&log, "2");
        let plan = V::bind(
            op("std/effect::fork", vec![long]),
            V::fun(vec![short], |c, first| {
                V::bind(
                    op("std/effect::fork", vec![c[0].clone()]),
                    V::fun(vec![first], |c, second| {
                        then(
                            op("std/effect::join", vec![c[0].clone()]),
                            op("std/effect::join", vec![second]),
                        )
                    }),
                )
            }),
        );
        assert!(run(plan).is_ok());
        let text = log.read_reference().render();
        assert!(
            text.find('2').unwrap() < 100,
            "another Fiber must run before the long child finishes"
        );
        assert_eq!(text.chars().filter(|c| *c == '1').count(), 100);
        drop(log);
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
