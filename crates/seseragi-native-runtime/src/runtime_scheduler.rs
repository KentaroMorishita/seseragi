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
    // Emitted only after the explicit finite native Clock binding is proved.
    ClockSleep,
    QueueBounded,
    QueueUnbounded,
    QueueOffer,
    QueueTake,
    QueueTryOffer,
    QueueTryTake,
    QueueSize,
    QueueClose,
    SemaphoreMake,
    SemaphoreAcquire,
    SemaphoreRelease,
    SemaphoreAvailable,
    // Waits until every other runnable Fiber has blocked or yielded.
    Settle,
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
            "std/queue::bounded" => Self::QueueBounded,
            "std/queue::unbounded" => Self::QueueUnbounded,
            "std/queue::offer" => Self::QueueOffer,
            "std/queue::take" => Self::QueueTake,
            "std/queue::tryOffer" => Self::QueueTryOffer,
            "std/queue::tryTake" => Self::QueueTryTake,
            "std/queue::size" => Self::QueueSize,
            "std/queue::close" => Self::QueueClose,
            "std/semaphore::make" => Self::SemaphoreMake,
            "std/semaphore::acquire" => Self::SemaphoreAcquire,
            "std/semaphore::release" => Self::SemaphoreRelease,
            "std/semaphore::available" => Self::SemaphoreAvailable,
            _ => return None,
        })
    }
}
pub struct NativeFiber {
    task: usize,
    outcome: RefCell<Option<NativeOutcome>>,
}
#[derive(Default)]
pub struct NativeDeferred {
    result: RefCell<Option<Result<V, V>>>,
}
impl TraceValue for NativeFiber {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        let outcome = self
            .outcome
            .try_borrow_mut()
            .map_err(|_| "borrowed native Fiber")?;
        match outcome.as_ref() {
            Some(Ok(value)) => visit(value),
            Some(Err(NativeExit::Failure(error))) => visit(&error.value),
            _ => {}
        }
        Ok(())
    }
    fn sever_reference(&self) -> Option<V> {
        match self.outcome.replace(None) {
            Some(Ok(value)) => Some(value),
            Some(Err(NativeExit::Failure(error))) => Some(error.value),
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
    pub fn fiber(&self) -> &NativeFiber {
        let Self::Fiber(value) = self else {
            panic!("Rust ABI: expected Fiber")
        };
        value.get()
    }
    pub fn deferred(&self) -> &NativeDeferred {
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
        Err(NativeExit::Failure(value)) => Ok(V::adt("std/effect::FiberFailed", value.value)),
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
    // A Queue or Semaphore keeps its own FIFO of waiting tasks.
    Sync(V),
    Settle,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WaitKey {
    Fiber(usize),
    Deferred(usize),
    Sync(usize),
    Settle,
}
impl NativeWait {
    fn key(&self) -> WaitKey {
        match self {
            Self::Fiber(value, _) => WaitKey::Fiber(value.fiber().task),
            Self::Deferred(value) => WaitKey::Deferred(value.node_id().expect("Deferred node")),
            Self::Sync(value) => WaitKey::Sync(value.node_id().expect("synchronization node")),
            Self::Settle => WaitKey::Settle,
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
                .map(|v| v.map_err(NativeExit::failure)),
            Self::Sync(_) | Self::Settle => None,
        }
    }
}
struct NativeTask {
    machine: NativeMachine,
    handle: V,
    waiting: Option<NativeWait>,
    file: Option<NativeFileTask>,
    stdin: Option<NativeStdinTask>,
    child: Option<NativeChildTask>,
    http: Option<NativeHttpTask>,
    timer: Option<NativeTimer>,
    process_signal: Option<usize>,
}
#[derive(Default)]
struct NativeScheduler {
    tasks: BTreeMap<usize, NativeTask>,
    roots: BTreeMap<usize, Rc<RefCell<NativeProcess>>>,
    ready: std::collections::VecDeque<usize>,
    // Fibers that called `yieldNow`. Like the TS runtime, where a yield waits
    // for a later macrotask, they resume only after every other runnable Fiber
    // has reached a wait. A budget keeps a Fiber that never waits from
    // starving them.
    yielded: std::collections::VecDeque<usize>,
    yield_wait: usize,
    waiters: BTreeMap<WaitKey, Vec<usize>>,
    // Tasks waiting until every other runnable Fiber has blocked or yielded.
    // The yield budget also bounds this wait, so a Fiber that never waits
    // cannot keep them suspended.
    settling: Vec<usize>,
    settle_wait: usize,
    // A program WASM run has no host: no clock, file, process, stdin or
    // signal. Those turns end as Defects instead of starting a host task.
    hostless: bool,
    // Test roots that have not had a turn yet. On the TS target a test body
    // starts as soon as its case starts, before any timer can fire.
    unstarted: std::collections::BTreeSet<usize>,
}
// The Defect for a turn a hostless run cannot perform, or the turn itself.
fn native_hostless_turn(machine: &mut NativeMachine, turn: NativeTurn) -> NativeTurn {
    let message = match &turn {
        NativeTurn::ProcessSignalPull(_) => "program ABI: process signals are unavailable in WASM",
        NativeTurn::ChildProcess(..) => "program ABI: child processes are unavailable in WASM",
        NativeTurn::FileSystem(..)
        | NativeTurn::Http(..)
        | NativeTurn::Stdin(..)
        | NativeTurn::Operation(ConcurrentOperation::ClockSleep, _) => {
            "program ABI: unadmitted host operation"
        }
        _ => return turn,
    };
    machine.resume(Err(NativeExit::defects(vec![message.into()])));
    NativeTurn::Runnable
}
// How many turns of other Fibers a yielded Fiber waits at most.
const NATIVE_YIELD_BUDGET: usize = 10_000;
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
        // Fibers share their root's process; independent test roots do not.
        if !self
            .roots
            .values()
            .any(|process| Rc::ptr_eq(process, &context.process))
        {
            self.roots.insert(id, context.process.clone());
        }
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
                stdin: None,
                child: None,
                http: None,
                timer: None,
                process_signal: None,
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
            if let Some((id, completed_context, outcome)) = scheduler.tick(5_000) {
                if id == root_id {
                    *context = completed_context;
                    debug_assert!(scheduler.tasks.is_empty(), "root scope awaits all children");
                    return outcome;
                }
            }
            if scheduler.idle() {
                scheduler.park_until_ready(None);
            }
        }
    }
    // No Fiber can take a turn until a wait, timer or host task completes.
    fn idle(&self) -> bool {
        self.ready.is_empty() && self.yielded.is_empty() && self.settling.is_empty()
    }
    fn next_runnable(&mut self) -> Option<usize> {
        if !self.yielded.is_empty() {
            if self.ready.is_empty() || self.yield_wait >= NATIVE_YIELD_BUDGET {
                self.yield_wait = 0;
                return self.yielded.pop_front();
            }
            self.yield_wait += 1;
        }
        self.ready.pop_front()
    }
    fn park_until_ready(&self, maximum: Option<std::time::Duration>) {
        for process in self.roots.values() {
            process.borrow().force_if_expired(true);
        }
        let mut timeout = self.next_timer_wait();
        if self.roots.values().any(|process| process.borrow().active())
            || self
                .tasks
                .values()
                .any(|task| task.stdin.is_some() || task.child.is_some())
        {
            timeout = Some(
                timeout
                    .unwrap_or(std::time::Duration::MAX)
                    .min(std::time::Duration::from_millis(10)),
            );
        }
        if let Some(maximum) = maximum {
            timeout = Some(timeout.unwrap_or(std::time::Duration::MAX).min(maximum));
        }
        if let Some(timeout) = timeout {
            std::thread::park_timeout(timeout);
        } else {
            std::thread::park();
        }
    }
    fn tick(&mut self, cleanup_ms: u64) -> Option<(usize, NativeContext, NativeOutcome)> {
        let processes = self
            .roots
            .iter()
            .map(|(root, process)| (*root, process.clone()))
            .collect::<Vec<_>>();
        for (root, process) in processes {
            let cancel = process.borrow_mut().poll();
            if cancel {
                self.cancel(root);
            }
        }
        self.poll_process_signals();
        self.poll_files();
        self.poll_http();
        self.poll_stdin();
        self.poll_children();
        self.poll_timers();
        self.resume_settled();
        let Some(id) = self.next_runnable() else {
            for process in self.roots.values() {
                process.borrow().force_if_expired(true);
            }
            return None;
        };
        let mut task = self.tasks.remove(&id).expect("runnable native task");
        if !self.unstarted.is_empty() {
            self.unstarted.remove(&id);
        }
        let process = task.machine.context.process.clone();
        let turn = task.machine.turn();
        let turn = if self.hostless {
            native_hostless_turn(&mut task.machine, turn)
        } else {
            turn
        };
        let yielded = matches!(&turn, NativeTurn::Operation(ConcurrentOperation::Yield, _));
        let asynchronous = matches!(
            &turn,
            NativeTurn::Runnable
                | NativeTurn::Operation(..)
                | NativeTurn::FileSystem(..)
                | NativeTurn::Http(..)
                | NativeTurn::Stdin(..)
                | NativeTurn::ProcessSignalPull(..)
                | NativeTurn::ChildProcess(..)
        );
        match turn {
            NativeTurn::Runnable => self.ready.push_back(id),
            NativeTurn::Operation(ConcurrentOperation::Settle, _) => {
                task.waiting = Some(NativeWait::Settle);
                self.settling.push(id);
            }
            NativeTurn::Operation(operation, captures) => {
                self.operation(id, &mut task, operation, captures)
            }
            NativeTurn::FileSystem(operation, captures) => {
                match NativeFileTask::start(operation, captures) {
                    Ok(file) => task.file = Some(file),
                    Err(exit) => {
                        task.machine.resume(Err(exit));
                        self.ready.push_back(id);
                    }
                }
            }
            NativeTurn::Http(captures) => match NativeHttpTask::start(captures) {
                Ok(http) => task.http = Some(http),
                Err(exit) => {
                    task.machine.resume(Err(exit));
                    self.ready.push_back(id);
                }
            },
            NativeTurn::ChildProcess(operation, captures) => {
                match NativeChildTask::start(operation, captures) {
                    Ok(child) => task.child = Some(child),
                    Err(exit) => {
                        task.machine.resume(Err(exit));
                        self.ready.push_back(id);
                    }
                }
            }
            NativeTurn::Stdin(operation, captures) => {
                match NativeStdinTask::start(
                    task.machine.context.stdin.clone(),
                    operation,
                    &captures[0],
                ) {
                    Ok(stdin) => task.stdin = Some(stdin),
                    Err(exit) => {
                        task.machine.resume(Err(exit));
                        self.ready.push_back(id);
                    }
                }
            }
            NativeTurn::ProcessSignalPull(subscription) => {
                if let Some(value) = process.borrow_mut().pull(subscription) {
                    task.machine.resume(Ok(value));
                    self.ready.push_back(id);
                } else {
                    task.process_signal = Some(subscription);
                }
            }
            NativeTurn::CloseScope(original, previous_mask) => {
                if task.machine.context.scopes.len() == 1 {
                    if let Some(environment) = &task.machine.context.test {
                        let mut environment = environment.borrow_mut();
                        if environment.root == id && environment.cleanup.is_none() {
                            let cancelling = matches!(original, Err(NativeExit::Defect(_)));
                            let reason = if cancelling {
                                "after defect"
                            } else {
                                "during cleanup"
                            };
                            environment.cleanup = Some((
                                std::time::Instant::now()
                                    + native_test_cleanup_delay(cleanup_ms, cancelling),
                                reason,
                            ));
                        }
                    }
                }
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
                    self.cancel(child.fiber().task);
                }
                task.machine.state = Some(task.machine.context.cleanup(
                    &mut task.machine.stack,
                    original,
                    Vec::new(),
                    previous_mask,
                ));
                self.ready.push_back(id);
            }
            NativeTurn::Complete(outcome) => {
                task.handle.fiber().outcome.replace(Some(outcome.clone()));
                self.wake(WaitKey::Fiber(id));
                self.roots.remove(&id);
                return Some((id, task.machine.context, outcome));
            }
        }
        let suspended = yielded
            || task.waiting.is_some()
            || task.file.is_some()
            || task.http.is_some()
            || task.stdin.is_some()
            || task.child.is_some()
            || task.timer.is_some()
            || task.process_signal.is_some();
        self.tasks.insert(id, task);
        if asynchronous {
            process.borrow().force_if_expired(suspended);
        }
        None
    }
    // The next turn is one the TS target runs as a microtask, so no timer
    // can fire before it. A Fiber resuming from `yieldNow` is a macrotask
    // there, and a due timer may fire before it.
    fn spawn_test(&mut self, context: NativeContext, plan: V) -> usize {
        let root = self.spawn(context, plan).fiber().task;
        self.unstarted.insert(root);
        root
    }
    fn started(&self, root: usize) -> bool {
        !self.unstarted.contains(&root)
    }
    fn microtask_pending(&self) -> bool {
        !self.ready.is_empty() && (self.yielded.is_empty() || self.yield_wait < NATIVE_YIELD_BUDGET)
    }
    fn discard_test(&mut self, environment: &Rc<RefCell<NativeTestEnvironment>>) {
        let removed = self
            .tasks
            .iter()
            .filter_map(|(id, task)| {
                task.machine
                    .context
                    .test
                    .as_ref()
                    .is_some_and(|test| Rc::ptr_eq(test, environment))
                    .then_some(*id)
            })
            .collect::<std::collections::BTreeSet<_>>();
        for id in &removed {
            let task = self.tasks.remove(id).unwrap();
            task.handle
                .fiber()
                .outcome
                .replace(Some(Err(NativeExit::Cancelled)));
            self.wake(WaitKey::Fiber(*id));
        }
        self.ready.retain(|id| !removed.contains(id));
        self.yielded.retain(|id| !removed.contains(id));
        self.roots.retain(|id, _| !removed.contains(id));
        self.waiters.retain(|_, waiters| {
            waiters.retain(|id| !removed.contains(id));
            !waiters.is_empty()
        });
    }
    fn cancel(&mut self, id: usize) {
        let Some(task) = self.tasks.get_mut(&id) else {
            return;
        };
        task.machine.context.cancelled = true;
        if task.machine.context.masked {
            return;
        }
        if task.timer.take().is_some() {
            task.machine.resume(Err(NativeExit::Cancelled));
            self.ready.push_back(id);
        }
        if task.stdin.take().is_some() {
            task.machine.resume(Err(NativeExit::Cancelled));
            self.ready.push_back(id);
        }
        if task.process_signal.take().is_some() {
            task.machine.resume(Err(NativeExit::Cancelled));
            self.ready.push_back(id);
        }
        if let Some(child) = &mut task.child {
            child.cancelled = true;
        }
        if let Some(http) = &task.http {
            http.cancel();
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
            if let NativeWait::Sync(value) = wait {
                self.detach_sync(id, &value);
            } else if matches!(wait, NativeWait::Settle) {
                self.settling.retain(|other| *other != id);
            }
        }
    }
    fn poll_http(&mut self) {
        for (id, task) in &mut self.tasks {
            let Some(http) = &mut task.http else {
                continue;
            };
            if let Some(outcome) = http.poll() {
                task.http = None;
                task.machine.resume(outcome);
                self.ready.push_back(*id);
            }
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
    fn poll_children(&mut self) {
        for (id, task) in &mut self.tasks {
            if let Some(outcome) = task.child.as_mut().and_then(NativeChildTask::poll) {
                task.child = None;
                task.machine.resume(outcome);
                self.ready.push_back(*id);
            }
        }
    }
    fn poll_stdin(&mut self) {
        for (id, task) in &mut self.tasks {
            if let Some(outcome) = task.stdin.as_mut().and_then(NativeStdinTask::poll) {
                task.stdin = None;
                task.machine.resume(outcome);
                self.ready.push_back(*id);
            }
        }
    }
    fn poll_process_signals(&mut self) {
        for (id, task) in &mut self.tasks {
            let Some(subscription) = task.process_signal else {
                continue;
            };
            let value = task.machine.context.process.borrow_mut().pull(subscription);
            if let Some(value) = value {
                task.process_signal = None;
                task.machine.resume(Ok(value));
                self.ready.push_back(*id);
            }
        }
    }
    // Settled tasks resume ahead of every Fiber that only yielded, as a
    // TypeScript continuation runs before a `yieldNow` timer.
    fn resume_settled(&mut self) {
        if self.settling.is_empty() {
            return;
        }
        if !self.ready.is_empty() && self.settle_wait < NATIVE_YIELD_BUDGET {
            self.settle_wait += 1;
            return;
        }
        self.settle_wait = 0;
        for id in std::mem::take(&mut self.settling).into_iter().rev() {
            if let Some(task) = self.tasks.get_mut(&id) {
                if matches!(task.waiting, Some(NativeWait::Settle)) {
                    task.waiting = None;
                    task.machine.resume(Ok(V::Unit));
                    self.ready.push_front(id);
                }
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
            ConcurrentOperation::QueueBounded
            | ConcurrentOperation::QueueUnbounded
            | ConcurrentOperation::QueueOffer
            | ConcurrentOperation::QueueTake
            | ConcurrentOperation::QueueTryOffer
            | ConcurrentOperation::QueueTryTake
            | ConcurrentOperation::QueueSize
            | ConcurrentOperation::QueueClose
            | ConcurrentOperation::SemaphoreMake
            | ConcurrentOperation::SemaphoreAcquire
            | ConcurrentOperation::SemaphoreRelease
            | ConcurrentOperation::SemaphoreAvailable
            | ConcurrentOperation::Settle => match self.sync_operation(id, task, operation, &c) {
                Some(result) => result,
                None => return,
            },
            ConcurrentOperation::ClockSleep => {
                if let Some(environment) = &task.machine.context.test {
                    environment.borrow_mut().instant += native_duration_nanoseconds(&c[0])
                        .expect("canonical test Duration")
                        as u128;
                    task.machine.resume(Ok(V::Unit));
                    self.ready.push_back(id);
                    return;
                }

                match NativeTimer::start(&c[0]) {
                    Ok(timer) => task.timer = Some(timer),
                    Err(exit) => {
                        task.machine.resume(Err(exit));
                        self.ready.push_back(id);
                    }
                }
                return;
            }
            ConcurrentOperation::Fork => {
                let child = self.spawn(
                    NativeContext {
                        gc_every: task.machine.context.gc_every,
                        stdin: task.machine.context.stdin.clone(),
                        random: task.machine.context.random.clone(),
                        test: task.machine.context.test.clone(),
                        temporaries: task.machine.context.temporaries.clone(),
                        process: task.machine.context.process.clone(),
                        environment: task.machine.context.environment.clone(),
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
            ConcurrentOperation::Yield => {
                task.machine.resume(Ok(V::Unit));
                self.yielded.push_back(id);
                return;
            }
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
    #[test]
    fn a_test_root_counts_as_started_only_after_its_first_turn() {
        enable_cycle_collection();
        let mut scheduler = NativeScheduler::default();
        let root = scheduler.spawn_test(
            NativeContext::default(),
            native_operation("effect.succeed", vec![V::Unit]),
        );
        assert!(!scheduler.started(root));
        let finished = scheduler.tick(0);
        assert!(scheduler.started(root));
        let finished = finished
            .or_else(|| (0..8).find_map(|_| scheduler.tick(0)))
            .expect("finished test root");
        assert_eq!(finished.0, root);
        assert!(finished.2.is_ok());
    }
    #[test]
    fn a_timer_may_fire_only_when_no_microtask_is_pending() {
        let mut scheduler = NativeScheduler::default();
        assert!(!scheduler.microtask_pending());
        scheduler.ready.push_back(1);
        assert!(scheduler.microtask_pending());
        // A yielded Fiber waits while others run, until its budget runs out.
        scheduler.yielded.push_back(2);
        scheduler.yield_wait = NATIVE_YIELD_BUDGET - 1;
        assert!(scheduler.microtask_pending());
        scheduler.yield_wait = NATIVE_YIELD_BUDGET;
        assert!(!scheduler.microtask_pending());
        scheduler.ready.clear();
        scheduler.yield_wait = 0;
        assert!(!scheduler.microtask_pending());
    }
    #[test]
    fn tick_resumes_signal_waiters_in_their_own_root_process() {
        let _access = native_process_signal_host::TEST_ACCESS.lock().unwrap();
        enable_cycle_collection();
        let mut scheduler = NativeScheduler::default();
        let mut roots = Vec::new();
        for _ in 0..2 {
            let context = NativeContext::default();
            let process = context.process.clone();
            process.borrow_mut().cursors.insert(
                0,
                NativeProcessSignalCursor {
                    handlers: vec![],
                    queue: Default::default(),
                },
            );
            let handle = scheduler.spawn(
                context,
                V::effect(Instruction::ProcessSignalPull, vec![V::Int(0)]),
            );
            roots.push((handle, process));
        }
        for _ in 0..64 {
            if scheduler
                .tasks
                .values()
                .all(|task| task.process_signal == Some(0))
            {
                break;
            }
            assert!(scheduler.tick(5_000).is_none());
        }
        assert_eq!(scheduler.roots.len(), 2);
        assert!(scheduler
            .tasks
            .values()
            .all(|task| task.process_signal == Some(0)));
        for (position, signal) in [(1, 5), (0, 4)] {
            let (handle, process) = &roots[position];
            process
                .borrow_mut()
                .cursors
                .get_mut(&0)
                .unwrap()
                .queue
                .push_back(signal);
            let mut complete = false;
            for _ in 0..64 {
                if let Some((id, _context, outcome)) = scheduler.tick(5_000) {
                    assert_eq!(id, handle.fiber().task);
                    let value = outcome.ok().expect("signal pull succeeded");
                    assert!(value.payload().tagged(NATIVE_PROCESS_SIGNALS[signal]));
                    complete = true;
                    break;
                }
            }
            assert!(complete, "tick must poll suspended signal pulls");
            assert!(!scheduler.roots.contains_key(&handle.fiber().task));
        }
        assert!(scheduler.tasks.is_empty());
        assert!(scheduler.roots.is_empty());
        drop((roots, scheduler));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

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
                V::Int(1) => Err(NativeExit::failure(V::Int(7))),
                _ => Err(NativeExit::defects(vec!["body".into()])),
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
        assert!(
            matches!(run(plan), Err(NativeExit::Failure(error)) if matches!(error.value, V::Int(8)))
        );
        let defect = V::leaf(vec![], |_, _| {
            Err(NativeExit::defects(vec!["child defect".into()]))
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
    #[test]
    fn a_yield_lets_other_fibers_run_until_they_wait() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let gate = deferred();
        // Several Effect steps before the child reaches its wait.
        let child = then(
            done(V::Unit),
            then(
                done(V::Unit),
                then(
                    done(V::Unit),
                    then(
                        append(&log, "child;"),
                        op("std/deferred::await", vec![gate.clone()]),
                    ),
                ),
            ),
        );
        let plan = V::bind(
            op("std/effect::fork", vec![child]),
            V::fun(vec![log.clone()], |c, fiber| {
                then(
                    op("std/effect::yieldNow", vec![V::Unit]),
                    then(
                        op("std/effect::interrupt", vec![fiber]),
                        append(&c[0], "parent;"),
                    ),
                )
            }),
        );
        run(plan).ok().unwrap();
        assert_eq!(log.read_reference().render(), "child;parent;");
        drop((log, gate));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    fn spin(count: V) -> V {
        let step = V::leaf(vec![count.clone()], |c, _| {
            let V::Int(n) = c[0].read_reference() else {
                unreachable!()
            };
            c[0].write_reference(V::Int(n + 1));
            Ok(V::Unit)
        });
        V::bind(step, V::fun(vec![count], |c, _| spin(c[0].clone())))
    }
    #[test]
    fn a_fiber_that_never_waits_does_not_starve_a_settling_fiber() {
        enable_cycle_collection();
        let count = V::reference(V::Int(0));
        let settle = V::effect(Instruction::Concurrent(ConcurrentOperation::Settle), vec![]);
        let plan = V::bind(
            op("std/effect::fork", vec![spin(count.clone())]),
            V::fun(vec![settle], |c, fiber| {
                then(c[0].clone(), op("std/effect::interrupt", vec![fiber]))
            }),
        );
        run(plan).ok().unwrap();
        let V::Int(steps) = count.read_reference() else {
            unreachable!()
        };
        assert!(
            steps > 0 && (steps as usize) <= NATIVE_YIELD_BUDGET + 1,
            "{steps}"
        );
        drop(count);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
    #[test]
    fn a_fiber_that_never_waits_does_not_starve_a_yielded_fiber() {
        enable_cycle_collection();
        let count = V::reference(V::Int(0));
        let plan = V::bind(
            op("std/effect::fork", vec![spin(count.clone())]),
            V::fun(vec![], |_, fiber| {
                then(
                    op("std/effect::yieldNow", vec![V::Unit]),
                    op("std/effect::interrupt", vec![fiber]),
                )
            }),
        );
        run(plan).ok().unwrap();
        let V::Int(steps) = count.read_reference() else {
            unreachable!()
        };
        assert!(
            steps > 0 && (steps as usize) <= NATIVE_YIELD_BUDGET,
            "{steps}"
        );
        drop(count);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
