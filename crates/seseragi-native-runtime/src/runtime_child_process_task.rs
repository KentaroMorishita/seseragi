struct NativeChildTask {
    operation: NativeChildOperation,
    captures: Vec<V>,
    input: Option<NativeChildWriteResult>,
    output: [Vec<u8>; 2],
    failure: Option<NativeExit>,
    cancelled: bool,
}
impl NativeChildTask {
    fn start(operation: NativeChildOperation, captures: Vec<V>) -> Result<Self, NativeExit> {
        let mut task = Self {
            operation,
            captures,
            input: None,
            output: Default::default(),
            failure: None,
            cancelled: false,
        };
        let mut state = task.captures[0].child_process().0.borrow_mut();
        let started = match operation {
            NativeChildOperation::Captured => state
                .write(native_bytes(&task.captures[1]).to_vec(), true)
                .map(Some),
            NativeChildOperation::Input => {
                let event = &task.captures[1];
                match native_error_tag(event) {
                    "WriteChildStdin" if state.stdin_closed => Err(native_child_failure(
                        "std/child-process::ChildInputAfterClose",
                        V::Unit,
                    )),
                    "WriteChildStdin" => state
                        .write(native_bytes(&event.payload()).to_vec(), false)
                        .map(Some),
                    "CloseChildStdin" => state.write(vec![], true).map(Some),
                    "SignalChild" => state.signal(&event.payload()).map(|_| None),
                    "KillChild" => state.wait().and_then(|_| {
                        if state.status.is_none() {
                            state.host_signal(9).map_err(|error| {
                                native_child_io_failure(
                                    "std/child-process::ChildTerminationFailed",
                                    error,
                                )
                            })?;
                        }
                        Ok(None)
                    }),
                    _ => panic!("Rust ABI: ChildInput owner"),
                }
            }
            _ => Ok(None),
        };
        drop(state);
        match started {
            Ok(input) => task.input = input,
            Err(error) => task.failure = Some(error),
        }
        Ok(task)
    }
    fn poll(&mut self) -> Option<NativeOutcome> {
        if matches!(self.operation, NativeChildOperation::Event) && self.failure.is_none() {
            let error = self.captures[1].read_reference();
            if !matches!(error, V::Unit) {
                self.failure = Some(NativeExit::failure(error));
            }
            if let Some(Err(NativeExit::Defect(errors))) =
                self.captures[2].fiber().outcome.borrow().as_ref()
            {
                self.failure = Some(NativeExit::Defect(errors.clone()));
            }
        }
        let child = self.captures[0].clone();
        let mut state = child.child_process().0.borrow_mut();
        if self.cancelled
            || self.failure.is_some()
            || matches!(self.operation, NativeChildOperation::Close)
        {
            return match state.stop() {
                Ok(false) => None,
                Ok(true) => Some(if self.cancelled {
                    Err(NativeExit::Cancelled)
                } else if let Some(error) = self.failure.take() {
                    Err(error)
                } else {
                    Ok(V::Unit)
                }),
                Err(error) => Some(Err(error)),
            };
        }
        let outcome = self.poll_running(&mut state);
        match outcome {
            Err(error) => {
                self.failure = Some(if matches!(self.operation, NativeChildOperation::Event) {
                    match error {
                        NativeExit::Failure(error) => {
                            NativeExit::failure(native_right(error.value))
                        }
                        other => other,
                    }
                } else {
                    error
                });
                None
            }
            Ok(result) => result,
        }
    }
    fn poll_running(
        &mut self,
        state: &mut NativeChildState,
    ) -> Result<Option<NativeOutcome>, NativeExit> {
        if let Some(input) = &self.input {
            match input.try_recv() {
                Ok(Ok(())) => self.input = None,
                Ok(Err(error)) => {
                    return Err(native_child_io_failure(
                        "std/child-process::ChildInputFailed",
                        error,
                    ))
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(native_child_io_failure(
                        "std/child-process::ChildInputFailed",
                        "child stdin worker disconnected",
                    ))
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if matches!(self.operation, NativeChildOperation::Input) {
            return Ok(self.input.is_none().then_some(Ok(V::Unit)));
        }
        state.wait()?;
        if matches!(self.operation, NativeChildOperation::Inherited) {
            return Ok(state.status.map(|status| Ok(state.exit_status(status))));
        }
        let streaming = matches!(self.operation, NativeChildOperation::Event);
        for channel in 0..2 {
            let budget = if streaming {
                state
                    .capacity
                    .saturating_sub(state.output[channel].len())
                    .min(16)
            } else {
                16
            };
            for _ in 0..budget {
                let Some(bytes) = state.read(channel)? else {
                    break;
                };
                if streaming {
                    state.output[channel].push_back(bytes);
                } else {
                    let limit = native_child_int(&self.captures[2]) as u64;
                    if self.output[channel].len() as u64 + bytes.len() as u64 > limit {
                        return Err(native_child_failure(
                            "std/child-process::ChildOutputLimitExceeded",
                            native_child_record(vec![
                                ("channel", native_child_channel(channel)),
                                ("limitBytes", self.captures[2].clone()),
                            ]),
                        ));
                    }
                    self.output[channel].extend(bytes);
                }
            }
        }
        if streaming {
            for offset in 0..2 {
                let channel = (state.next_channel + offset) % 2;
                if let Some(bytes) = state.output[channel].pop_front() {
                    state.next_channel = 1 - channel;
                    return Ok(Some(Ok(stream_item(V::adt(
                        if channel == 0 {
                            "std/child-process::ChildStdoutChunk"
                        } else {
                            "std/child-process::ChildStderrChunk"
                        },
                        V::Bytes(Shared::new(bytes)),
                    )))));
                }
            }
        }
        if state.stdout.is_some() || state.stderr.is_some() {
            return Ok(None);
        }
        let Some(status) = state.status else {
            return Ok(None);
        };
        if streaming {
            if state.emitted {
                return Ok(Some(Ok(stream_done())));
            }
            state.emitted = true;
            Ok(Some(Ok(stream_item(V::adt(
                "std/child-process::ChildExitedWith",
                state.exit_status(status),
            )))))
        } else if self.input.is_none() {
            state.input.take();
            if state
                .writer
                .as_ref()
                .is_some_and(|worker| !worker.is_finished())
            {
                return Ok(None);
            }
            if let Some(worker) = state.writer.take() {
                worker
                    .join()
                    .map_err(|_| NativeExit::defects(vec!["child stdin worker panicked".into()]))?;
            }
            Ok(Some(Ok(native_child_record(vec![
                ("status", state.exit_status(status)),
                (
                    "stdout",
                    V::Bytes(Shared::new(std::mem::take(&mut self.output[0]))),
                ),
                (
                    "stderr",
                    V::Bytes(Shared::new(std::mem::take(&mut self.output[1]))),
                ),
            ]))))
        } else {
            Ok(None)
        }
    }
}
