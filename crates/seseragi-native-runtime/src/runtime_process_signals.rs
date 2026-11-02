const NATIVE_PROCESS_SIGNALS: [&str; 6] = [
    "std/process::Interrupt",
    "std/process::Terminate",
    "std/process::Hangup",
    "std/process::Quit",
    "std/process::User1",
    "std/process::User2",
];

thread_local! {
    static NATIVE_PROCESS_OPTIONS: Cell<(bool, u64)> = const { Cell::new((false, 10_000)) };
}

pub fn configure_native_process_shutdown(forward: bool, grace_ms: u64) {
    NATIVE_PROCESS_OPTIONS.with(|options| options.set((forward, grace_ms)));
}

struct NativeProcessSignalCursor {
    handlers: Vec<native_process_signal_host::Handler>,
    queue: std::collections::VecDeque<usize>,
}

struct NativeProcessShutdown {
    _handlers: Vec<native_process_signal_host::Handler>,
    forward: bool,
    grace_ms: u64,
    first: Option<(usize, std::time::Instant)>,
}
impl Drop for NativeProcessShutdown {
    fn drop(&mut self) {
        native_process_signal_host::shutdown(false);
    }
}

#[derive(Default)]
struct NativeProcess {
    cursors: BTreeMap<usize, NativeProcessSignalCursor>,
    next: usize,
    shutdown: Option<NativeProcessShutdown>,
}
impl NativeProcess {
    fn install_shutdown(&mut self) -> Result<(), NativeExit> {
        native_process_signal_host::shutdown(true);
        let handlers = [0, 1]
            .into_iter()
            .map(native_process_signal_host::Handler::install)
            .collect::<Result<Vec<_>, _>>();
        let handlers = match handlers {
            Ok(handlers) => handlers,
            Err(error) => {
                native_process_signal_host::shutdown(false);
                return Err(error);
            }
        };
        let (forward, grace_ms) = NATIVE_PROCESS_OPTIONS.with(Cell::get);
        self.shutdown = Some(NativeProcessShutdown {
            _handlers: handlers,
            forward,
            grace_ms,
            first: None,
        });
        Ok(())
    }

    fn active(&self) -> bool {
        self.shutdown.is_some() || !self.cursors.is_empty()
    }

    fn reserved(&self) -> bool {
        self.shutdown
            .as_ref()
            .is_some_and(|shutdown| !shutdown.forward)
    }

    fn open(&mut self, watched: &[usize]) -> Result<usize, NativeExit> {
        self.dispatch_pending();
        native_process_signal_validate(
            watched,
            self.reserved(),
            native_process_signal_host::supported,
        )?;
        let handlers = watched
            .iter()
            .copied()
            .map(native_process_signal_host::Handler::install)
            .collect::<Result<Vec<_>, _>>()?;
        let id = self.next;
        self.next = self
            .next
            .checked_add(1)
            .expect("native signal cursor identity exhausted");
        self.cursors.insert(
            id,
            NativeProcessSignalCursor {
                handlers,
                queue: Default::default(),
            },
        );
        Ok(id)
    }

    fn close(&mut self, id: usize) {
        self.cursors.remove(&id);
    }

    fn enqueue(&mut self, signal: usize) {
        for cursor in self.cursors.values_mut() {
            if cursor
                .handlers
                .iter()
                .any(|handler| handler.index() == signal)
                && !cursor.queue.contains(&signal)
            {
                cursor.queue.push_back(signal);
            }
        }
    }

    fn dispatch_pending(&mut self) {
        for signal in native_process_signal_host::drain() {
            self.enqueue(signal);
        }
    }

    fn poll(&mut self) -> bool {
        if !self.active() {
            return false;
        }
        self.dispatch_pending();
        let Some(shutdown) = &mut self.shutdown else {
            return false;
        };
        let mut cancel = false;
        if shutdown.first.is_none() {
            if let Some(signal) = native_process_signal_host::first_termination() {
                shutdown.first = Some((signal, std::time::Instant::now()));
                cancel = !shutdown.forward;
            }
        }
        cancel
    }

    fn force_if_expired(&self, suspended: bool) {
        // A zero-delay host timer runs after synchronous cleanup continuations.
        if let Some(shutdown) = &self.shutdown {
            if let Some((signal, received)) = shutdown.first {
                if shutdown.forward
                    || (shutdown.grace_ms == 0 && !suspended)
                    || received.elapsed().as_millis() < u128::from(shutdown.grace_ms)
                {
                    return;
                }
                eprintln!("Seseragi forced termination: shutdown grace period expired");
                std::process::exit(if signal == 1 { 143 } else { 130 });
            }
        }
    }

    fn pull(&mut self, id: usize) -> Option<V> {
        match self.cursors.get_mut(&id) {
            None => Some(stream_done()),
            Some(cursor) => cursor
                .queue
                .pop_front()
                .map(|signal| stream_item(V::adt(NATIVE_PROCESS_SIGNALS[signal], V::Unit))),
        }
    }

    fn signal_exit_code(&self) -> Option<i32> {
        self.shutdown
            .as_ref()?
            .first
            .map(|(signal, _)| if signal == 1 { 143 } else { 130 })
    }
}

fn native_process_signal_validate(
    watched: &[usize],
    reserved: bool,
    supported: impl Fn(usize) -> bool,
) -> Result<(), NativeExit> {
    let error = watched
        .iter()
        .find(|&&signal| !supported(signal))
        .map(|&signal| ("std/process::UnsupportedProcessSignal", signal))
        .or_else(|| {
            reserved
                .then(|| watched.iter().find(|&&signal| signal < 2).copied())
                .flatten()
                .map(|signal| ("std/process::ReservedProcessSignal", signal))
        });
    if let Some((tag, signal)) = error {
        return Err(NativeExit::failure(V::adt(
            tag,
            V::adt(NATIVE_PROCESS_SIGNALS[signal], V::Unit),
        )));
    }
    Ok(())
}

fn native_process_signals(watched: V) -> V {
    // Constructing the Stream does not register a host handler.
    let requested =
        std::iter::once(watched.field("head")).chain(watched.field("tail").list_values());
    let mut unique = Vec::new();
    for signal in requested {
        let index = NATIVE_PROCESS_SIGNALS
            .iter()
            .position(|tag| signal.tagged(tag))
            .expect("Rust ABI: expected ProcessSignal");
        if !unique.contains(&index) {
            unique.push(index);
        }
    }
    V::fun(
        unique
            .into_iter()
            .map(|signal| V::Int(signal as i64))
            .collect(),
        |captures, _| {
            V::leaf(captures.to_vec(), |captures, context| {
                let watched = captures
                    .iter()
                    .map(|value| match value {
                        V::Int(signal) => *signal as usize,
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>();
                let id = context.process.borrow_mut().open(&watched)?;
                context
                    .scopes
                    .last_mut()
                    .expect("signal Stream scope")
                    .process_signals
                    .push(id);
                let id = V::Int(id as i64);
                Ok(stream_cursor(
                    V::fun(vec![id.clone()], |captures, _| {
                        V::effect(Instruction::ProcessSignalPull, captures.to_vec())
                    }),
                    V::fun(vec![id], |captures, _| {
                        V::leaf(captures.to_vec(), |captures, context| {
                            let V::Int(id) = captures[0] else {
                                unreachable!()
                            };
                            context.process.borrow_mut().close(id as usize);
                            Ok(V::Unit)
                        })
                    }),
                ))
            })
        },
    )
}

#[cfg(test)]
mod process_signal_tests {
    use super::*;

    #[test]
    fn validation_keeps_first_unsupported_before_first_reserved_signal() {
        for (watched, reserved, unsupported, tag, signal) in [
            (
                vec![0, 5, 4],
                true,
                Some(5),
                "std/process::UnsupportedProcessSignal",
                5,
            ),
            (
                vec![1, 0],
                true,
                None,
                "std/process::ReservedProcessSignal",
                1,
            ),
            (
                vec![4, 0, 1],
                true,
                None,
                "std/process::ReservedProcessSignal",
                0,
            ),
        ] {
            let Err(NativeExit::Failure(error)) =
                native_process_signal_validate(&watched, reserved, |index| {
                    Some(index) != unsupported
                })
            else {
                panic!("expected ProcessError")
            };
            assert!(error.value.tagged(tag));
            assert!(error.value.payload().tagged(NATIVE_PROCESS_SIGNALS[signal]));
        }
        assert!(native_process_signal_validate(&[0, 1, 2, 3, 4, 5], false, |_| true).is_ok());
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn signal_plans_are_cold_and_each_terminal_scope_releases_its_cursor() {
        let _access = native_process_signal_host::TEST_ACCESS.lock().unwrap();
        let watched = native_non_empty_list_parts(
            V::adt(NATIVE_PROCESS_SIGNALS[0], V::Unit),
            V::list_from_values(vec![V::adt(NATIVE_PROCESS_SIGNALS[0], V::Unit)]),
        );
        let source = native_process_signals(watched);
        let mut context = NativeContext::default();
        assert!(context.process.borrow().cursors.is_empty());
        for _ in 0..3 {
            let taken =
                native_stream_operation("std/stream::take", vec![V::Int(0), source.clone()]);
            let plan = native_stream_operation("std/stream::runCollect", vec![taken]);
            let values = context
                .root(plan)
                .unwrap_or_else(|_| panic!("take zero failed"));
            let V::Array(values) = values else { panic!() };
            assert!(values.get().is_empty());
            assert!(context.process.borrow().cursors.is_empty());
        }
        drop((source, context));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn cursors_broadcast_coalesce_pending_kinds_and_keep_first_observed_order() {
        let _access = native_process_signal_host::TEST_ACCESS.lock().unwrap();
        let mut process = NativeProcess::default();
        let first = process.open(&[0, 1]).unwrap_or_else(|_| panic!());
        let second = process.open(&[1]).unwrap_or_else(|_| panic!());
        for index in [1, 0, 1, 0] {
            process.enqueue(index);
        }
        for index in [1, 0] {
            assert!(process
                .pull(first)
                .unwrap()
                .payload()
                .tagged(NATIVE_PROCESS_SIGNALS[index]));
        }
        assert!(process.pull(first).is_none());
        assert!(process
            .pull(second)
            .unwrap()
            .payload()
            .tagged(NATIVE_PROCESS_SIGNALS[1]));
        assert!(process.pull(second).is_none());
        process.enqueue(1);
        process.close(first);
        process.close(first);
        assert!(process.pull(first).unwrap().tagged("std/prelude::Nothing"));
        assert!(process
            .pull(second)
            .unwrap()
            .payload()
            .tagged(NATIVE_PROCESS_SIGNALS[1]));
        process.close(second);
        assert!(process.cursors.is_empty());
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
