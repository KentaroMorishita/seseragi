// Queue and Semaphore keep their waiting Fibers in FIFO order inside the
// value. A waiter is a task identity; it is active only while that task is
// still suspended on this value, so cancelled or discarded waiters are skipped
// exactly like the TypeScript runtime's inactive entries.
pub struct NativeQueue {
    capacity: i64,
    buffer: RefCell<std::collections::VecDeque<V>>,
    closed: Cell<bool>,
    takers: RefCell<std::collections::VecDeque<usize>>,
    offerers: RefCell<std::collections::VecDeque<(usize, V)>>,
}
pub struct NativeSemaphore {
    available: Cell<i64>,
    waiters: RefCell<std::collections::VecDeque<usize>>,
}
pub struct NativePermit {
    semaphore: V,
    released: Cell<bool>,
}
impl TraceValue for NativeQueue {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        let buffer = self
            .buffer
            .try_borrow_mut()
            .map_err(|_| "borrowed native Queue")?;
        let offerers = self
            .offerers
            .try_borrow_mut()
            .map_err(|_| "borrowed native Queue")?;
        buffer.iter().for_each(&mut *visit);
        offerers.iter().for_each(|(_, value)| visit(value));
        Ok(())
    }
    fn sever_reference(&self) -> Option<V> {
        let mut values: Vec<V> = self.buffer.take().into_iter().collect();
        values.extend(self.offerers.take().into_iter().map(|(_, value)| value));
        (!values.is_empty()).then(|| V::Array(Shared::new(values)))
    }
}
impl TraceValue for NativeSemaphore {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}
impl TraceValue for NativePermit {
    fn edges(&self, visit: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        visit(&self.semaphore);
        Ok(())
    }
}
impl V {
    fn queue(&self) -> &NativeQueue {
        let Self::Queue(value) = self else {
            panic!("Rust ABI: expected Queue")
        };
        value.get()
    }
    fn semaphore(&self) -> &NativeSemaphore {
        let Self::Semaphore(value) = self else {
            panic!("Rust ABI: expected Semaphore")
        };
        value.get()
    }
    fn permit(&self) -> &NativePermit {
        let Self::Permit(value) = self else {
            panic!("Rust ABI: expected Permit")
        };
        value.get()
    }
}
fn native_queue_closed() -> NativeOutcome {
    Err(NativeExit::failure(native_queue_closed_value()))
}
fn native_queue_closed_value() -> V {
    V::adt("std/queue::QueueClosed", V::Unit)
}
fn native_new_permit(semaphore: &V) -> V {
    V::Permit(Shared::new(NativePermit {
        semaphore: semaphore.clone(),
        released: Cell::new(false),
    }))
}
// `withPermit` acquires, runs the Effect, and always releases the permit on
// success, typed failure, Defect and cancellation. It opens no scope, so a
// Fiber forked inside it still belongs to the caller, as in TypeScript.
fn native_with_permit(semaphore: V, effect: V) -> V {
    V::effect(
        Instruction::Bracket,
        vec![
            V::effect(
                Instruction::Concurrent(ConcurrentOperation::SemaphoreAcquire),
                vec![semaphore],
            ),
            V::fun(vec![effect], |c, _| c[0].clone()),
            V::fun(vec![], |_, permit| {
                V::effect(
                    Instruction::Concurrent(ConcurrentOperation::SemaphoreRelease),
                    vec![permit],
                )
            }),
        ],
    )
}
impl NativeScheduler {
    // True when `id` is still suspended on the Queue or Semaphore `node`.
    fn sync_waiting(&self, id: usize, node: usize) -> bool {
        self.tasks.get(&id).is_some_and(|task| {
            matches!(&task.waiting, Some(NativeWait::Sync(value)) if value.node_id() == Some(node))
        })
    }
    fn settle_sync(&mut self, id: usize, outcome: NativeOutcome) {
        let task = self.tasks.get_mut(&id).expect("suspended native waiter");
        task.waiting = None;
        task.machine.resume(outcome);
        self.ready.push_back(id);
    }
    fn first_taker(&self, queue: &V) -> Option<usize> {
        let node = queue.node_id().expect("Queue node");
        let mut takers = queue.queue().takers.borrow_mut();
        while let Some(id) = takers.front().copied() {
            if self.sync_waiting(id, node) {
                return Some(id);
            }
            takers.pop_front();
        }
        None
    }
    fn first_offerer(&self, queue: &V) -> Option<(usize, V)> {
        let node = queue.node_id().expect("Queue node");
        let mut offerers = queue.queue().offerers.borrow_mut();
        while let Some((id, _)) = offerers.front() {
            if self.sync_waiting(*id, node) {
                return offerers.pop_front();
            }
            offerers.pop_front();
        }
        None
    }
    fn take_taker(&mut self, queue: &V, value: V) -> bool {
        let Some(id) = self.first_taker(queue) else {
            return false;
        };
        queue.queue().takers.borrow_mut().pop_front();
        self.settle_sync(id, Ok(value));
        true
    }
    // Hands buffered values to waiting takers, admits waiting offerers while
    // there is room, and fails every waiter once the Queue is closed.
    fn drain_queue(&mut self, queue: &V) {
        let state = queue.queue();
        loop {
            if state.buffer.borrow().is_empty() || self.first_taker(queue).is_none() {
                break;
            }
            let value = state.buffer.borrow_mut().pop_front().unwrap();
            self.take_taker(queue, value);
        }
        if state.closed.get() {
            while let Some((id, _)) = self.first_offerer(queue) {
                self.settle_sync(id, native_queue_closed());
            }
            if state.buffer.borrow().is_empty() {
                while let Some(id) = self.first_taker(queue) {
                    state.takers.borrow_mut().pop_front();
                    self.settle_sync(id, native_queue_closed());
                }
            }
            return;
        }
        while let Some((id, value)) = self.first_offerer(queue) {
            if state.buffer.borrow().is_empty() && self.first_taker(queue).is_some() {
                self.take_taker(queue, value);
                self.settle_sync(id, Ok(V::Unit));
                continue;
            }
            if (state.buffer.borrow().len() as i64) < state.capacity {
                state.buffer.borrow_mut().push_back(value);
                self.settle_sync(id, Ok(V::Unit));
                continue;
            }
            state.offerers.borrow_mut().push_front((id, value));
            return;
        }
    }
    // A cancelled waiter leaves the value's FIFO; a Queue then admits the
    // next offerer exactly as the TypeScript runtime's drain does.
    fn detach_sync(&mut self, id: usize, value: &V) {
        match value {
            V::Queue(state) => {
                state.get().takers.borrow_mut().retain(|other| *other != id);
                state
                    .get()
                    .offerers
                    .borrow_mut()
                    .retain(|(other, _)| *other != id);
                self.drain_queue(value);
            }
            V::Semaphore(state) => {
                state
                    .get()
                    .waiters
                    .borrow_mut()
                    .retain(|other| *other != id);
            }
            _ => unreachable!("native synchronization wait"),
        }
    }
    fn suspend_sync(&mut self, task: &mut NativeTask, value: V) {
        task.waiting = Some(NativeWait::Sync(value));
    }
    fn release_permit(&mut self, permit: &V) {
        let permit = permit.permit();
        if permit.released.replace(true) {
            return;
        }
        let semaphore = permit.semaphore.clone();
        let node = semaphore.node_id().expect("Semaphore node");
        let state = semaphore.semaphore();
        loop {
            let next = state.waiters.borrow_mut().pop_front();
            match next {
                Some(id) if self.sync_waiting(id, node) => {
                    self.settle_sync(id, Ok(native_new_permit(&semaphore)));
                    return;
                }
                Some(_) => continue,
                None => {
                    state.available.set(state.available.get() + 1);
                    return;
                }
            }
        }
    }
    // Returns None when the task suspended on the value.
    fn sync_operation(
        &mut self,
        id: usize,
        task: &mut NativeTask,
        operation: ConcurrentOperation,
        c: &[V],
    ) -> Option<NativeOutcome> {
        Some(match operation {
            ConcurrentOperation::QueueBounded | ConcurrentOperation::QueueUnbounded => {
                let capacity = match operation {
                    ConcurrentOperation::QueueBounded => {
                        let V::Int(capacity) = c[0] else {
                            panic!("Rust ABI: Queue capacity")
                        };
                        if capacity <= 0 {
                            return Some(Err(NativeExit::failure(V::adt(
                                "std/queue::NonPositiveCapacity",
                                V::Int(capacity),
                            ))));
                        }
                        capacity
                    }
                    _ => i64::MAX,
                };
                Ok(V::Queue(Shared::new(NativeQueue {
                    capacity,
                    buffer: RefCell::default(),
                    closed: Cell::new(false),
                    takers: RefCell::default(),
                    offerers: RefCell::default(),
                })))
            }
            ConcurrentOperation::QueueOffer | ConcurrentOperation::QueueTryOffer => {
                let (value, queue) = (c[0].clone(), &c[1]);
                let attempt = matches!(operation, ConcurrentOperation::QueueTryOffer);
                let state = queue.queue();
                if state.closed.get() {
                    return Some(if attempt {
                        Ok(native_left(native_queue_closed_value()))
                    } else {
                        native_queue_closed()
                    });
                }
                let accepted = |accepted: bool| {
                    if attempt {
                        native_right(V::Bool(accepted))
                    } else {
                        V::Unit
                    }
                };
                if state.buffer.borrow().is_empty() && self.first_taker(queue).is_some() {
                    self.take_taker(queue, value);
                    return Some(Ok(accepted(true)));
                }
                if (state.buffer.borrow().len() as i64) < state.capacity {
                    state.buffer.borrow_mut().push_back(value);
                    return Some(Ok(accepted(true)));
                }
                if attempt {
                    return Some(Ok(accepted(false)));
                }
                state.offerers.borrow_mut().push_back((id, value));
                self.suspend_sync(task, queue.clone());
                return None;
            }
            ConcurrentOperation::QueueTake | ConcurrentOperation::QueueTryTake => {
                let queue = &c[0];
                let attempt = matches!(operation, ConcurrentOperation::QueueTryTake);
                let state = queue.queue();
                let found = |value: V| {
                    if attempt {
                        native_right(native_maybe(Some(value)))
                    } else {
                        value
                    }
                };
                let buffered = state.buffer.borrow_mut().pop_front();
                if let Some(value) = buffered {
                    self.drain_queue(queue);
                    return Some(Ok(found(value)));
                }
                if state.closed.get() {
                    return Some(if attempt {
                        Ok(native_left(native_queue_closed_value()))
                    } else {
                        native_queue_closed()
                    });
                }
                if let Some((offerer, value)) = self.first_offerer(queue) {
                    self.settle_sync(offerer, Ok(V::Unit));
                    return Some(Ok(found(value)));
                }
                if attempt {
                    return Some(Ok(native_right(native_maybe(None))));
                }
                state.takers.borrow_mut().push_back(id);
                self.suspend_sync(task, queue.clone());
                return None;
            }
            ConcurrentOperation::QueueSize => Ok(V::Int(c[0].queue().buffer.borrow().len() as i64)),
            ConcurrentOperation::QueueClose => {
                if !c[0].queue().closed.replace(true) {
                    self.drain_queue(&c[0]);
                }
                Ok(V::Unit)
            }
            ConcurrentOperation::SemaphoreMake => {
                let V::Int(permits) = c[0] else {
                    panic!("Rust ABI: Semaphore permits")
                };
                if permits <= 0 {
                    return Some(Err(NativeExit::failure(V::adt(
                        "std/semaphore::NonPositivePermits",
                        V::Int(permits),
                    ))));
                }
                Ok(V::Semaphore(Shared::new(NativeSemaphore {
                    available: Cell::new(permits),
                    waiters: RefCell::default(),
                })))
            }
            ConcurrentOperation::SemaphoreAcquire => {
                let semaphore = &c[0];
                let node = semaphore.node_id().expect("Semaphore node");
                let state = semaphore.semaphore();
                let waiting = {
                    let mut waiters = state.waiters.borrow_mut();
                    waiters.retain(|other| self.sync_waiting(*other, node));
                    !waiters.is_empty()
                };
                if state.available.get() > 0 && !waiting {
                    state.available.set(state.available.get() - 1);
                    return Some(Ok(native_new_permit(semaphore)));
                }
                state.waiters.borrow_mut().push_back(id);
                self.suspend_sync(task, semaphore.clone());
                return None;
            }
            ConcurrentOperation::SemaphoreRelease => {
                self.release_permit(&c[0]);
                Ok(V::Unit)
            }
            ConcurrentOperation::SemaphoreAvailable => Ok(V::Int(c[0].semaphore().available.get())),
            ConcurrentOperation::Settle => unreachable!("Settle suspends in the turn"),
            _ => unreachable!("native synchronization operation"),
        })
    }
}

#[cfg(test)]
mod queue_tests {
    use super::*;

    fn op(name: &str, args: Vec<V>) -> V {
        native_operation(name, args)
    }
    fn then(first: V, second: V) -> V {
        V::bind(first, V::fun(vec![second], |c, _| c[0].clone()))
    }
    fn append(log: &V, text: &str) -> V {
        V::leaf(vec![log.clone(), V::text(text)], |c, _| {
            let text = c[0].read_reference().render() + &c[1].render();
            c[0].write_reference(V::text(&text));
            Ok(V::Unit)
        })
    }
    fn run(plan: V) -> NativeOutcome {
        NativeContext {
            gc_every: 1,
            ..NativeContext::default()
        }
        .root(plan)
    }
    fn with_queue(capacity: i64, body: fn(&[V], V) -> V, captures: Vec<V>) -> V {
        V::bind(
            op("std/queue::bounded", vec![V::Int(capacity)]),
            V::fun(captures, body),
        )
    }

    #[test]
    fn blocked_offer_resumes_in_fifo_order_when_space_opens() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let plan = with_queue(
            1,
            |c, queue| {
                let offer = |value: i64| op("std/queue::offer", vec![V::Int(value), queue.clone()]);
                let take = || {
                    V::bind(
                        op("std/queue::take", vec![queue.clone()]),
                        V::fun(vec![c[0].clone()], |c, value| {
                            let text = c[0].read_reference().render() + &value.render();
                            c[0].write_reference(V::text(&text));
                            native_operation("effect.succeed", vec![V::Unit])
                        }),
                    )
                };
                then(
                    offer(1),
                    then(
                        op("std/effect::fork", vec![offer(2)]),
                        then(
                            op("std/effect::fork", vec![offer(3)]),
                            then(
                                op("std/effect::yieldNow", vec![V::Unit]),
                                then(take(), then(take(), take())),
                            ),
                        ),
                    ),
                )
            },
            vec![log.clone()],
        );
        assert!(run(plan).is_ok());
        assert_eq!(log.read_reference().render(), "123");
        drop(log);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn close_fails_waiting_takers_and_later_offers_but_drains_the_buffer() {
        enable_cycle_collection();
        let plan = with_queue(
            2,
            |_, queue| {
                let waiter = op("std/queue::take", vec![queue.clone()]);
                V::bind(
                    op("std/effect::fork", vec![waiter]),
                    V::fun(vec![queue], |c, fiber| {
                        let queue = c[0].clone();
                        then(
                            op("std/effect::yieldNow", vec![V::Unit]),
                            then(
                                op("std/queue::close", vec![queue.clone()]),
                                V::bind(
                                    op("std/effect::await", vec![fiber]),
                                    V::fun(vec![queue], |c, exit| {
                                        assert!(exit.tagged("std/effect::FiberFailed"));
                                        assert!(exit.payload().tagged("std/queue::QueueClosed"));
                                        op("std/queue::tryOffer", vec![V::Int(1), c[0].clone()])
                                    }),
                                ),
                            ),
                        )
                    }),
                )
            },
            vec![],
        );
        let result = run(plan).ok().unwrap();
        assert!(result.tagged("std/prelude::Left"));
        drop(result);
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn cancelled_taker_leaves_the_queue_without_losing_a_value() {
        enable_cycle_collection();
        let plan = with_queue(
            1,
            |_, queue| {
                let waiter = op("std/queue::take", vec![queue.clone()]);
                V::bind(
                    op("std/effect::fork", vec![waiter]),
                    V::fun(vec![queue], |c, fiber| {
                        let queue = c[0].clone();
                        then(
                            op("std/effect::yieldNow", vec![V::Unit]),
                            then(
                                op("std/effect::interrupt", vec![fiber]),
                                then(
                                    op("std/queue::offer", vec![V::Int(5), queue.clone()]),
                                    op("std/queue::take", vec![queue]),
                                ),
                            ),
                        )
                    }),
                )
            },
            vec![],
        );
        assert_eq!(run(plan).ok().unwrap().render(), "5");
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn with_permit_releases_after_failure_defect_and_cancellation() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let semaphore = V::Semaphore(Shared::new(NativeSemaphore {
            available: Cell::new(1),
            waiters: RefCell::default(),
        }));
        let failing = op("effect.fail", vec![V::Int(1)]);
        let defect = V::leaf(vec![], |_, _| {
            Err(NativeExit::defects(vec!["permit defect".into()]))
        });
        for body in [failing, defect] {
            let plan = native_with_permit(semaphore.clone(), body);
            assert!(run(plan).is_err());
            assert_eq!(semaphore.semaphore().available.get(), 1);
        }
        // A Fiber blocked inside withPermit is cancelled and must release.
        let blocked = native_with_permit(
            semaphore.clone(),
            then(
                append(&log, "in"),
                op(
                    "std/deferred::await",
                    vec![V::Deferred(Shared::new(NativeDeferred::default()))],
                ),
            ),
        );
        let plan = V::bind(
            op("std/effect::fork", vec![blocked]),
            V::fun(vec![semaphore.clone()], |c, fiber| {
                then(
                    op("std/effect::yieldNow", vec![V::Unit]),
                    then(
                        op("std/effect::interrupt", vec![fiber]),
                        op("std/semaphore::available", vec![c[0].clone()]),
                    ),
                )
            }),
        );
        assert_eq!(run(plan).ok().unwrap().render(), "1");
        assert_eq!(log.read_reference().render(), "in");
        drop((log, semaphore));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn released_permit_goes_to_the_oldest_waiter_and_double_release_is_ignored() {
        enable_cycle_collection();
        let log = V::reference(V::text(""));
        let semaphore = V::Semaphore(Shared::new(NativeSemaphore {
            available: Cell::new(1),
            waiters: RefCell::default(),
        }));
        let worker = |name: &str| native_with_permit(semaphore.clone(), append(&log, name));
        let plan = V::bind(
            op("std/semaphore::acquire", vec![semaphore.clone()]),
            V::fun(
                vec![worker("1"), worker("2"), semaphore.clone()],
                |c, permit| {
                    then(
                        op("std/effect::fork", vec![c[0].clone()]),
                        then(
                            op("std/effect::fork", vec![c[1].clone()]),
                            then(
                                op("std/effect::yieldNow", vec![V::Unit]),
                                then(
                                    op("std/semaphore::release", vec![permit.clone()]),
                                    then(
                                        op("std/semaphore::release", vec![permit]),
                                        then(
                                            op("std/effect::yieldNow", vec![V::Unit]),
                                            then(
                                                op("std/effect::yieldNow", vec![V::Unit]),
                                                op("std/semaphore::available", vec![c[2].clone()]),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    )
                },
            ),
        );
        assert_eq!(run(plan).ok().unwrap().render(), "1");
        assert_eq!(log.read_reference().render(), "12");
        drop((log, semaphore));
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
