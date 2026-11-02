fn native_child_int(value: &V) -> i64 {
    let V::Int(value) = value else {
        panic!("Rust ABI: expected Int")
    };
    *value
}
type NativeChildWriteResult = std::sync::mpsc::Receiver<Result<(), String>>;
struct NativeChildWrite {
    bytes: Vec<u8>,
    close: bool,
    result: std::sync::mpsc::Sender<Result<(), String>>,
}
struct NativeChildState {
    child: std::process::Child,
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
    status: Option<std::process::ExitStatus>,
    input: Option<std::sync::mpsc::Sender<NativeChildWrite>>,
    writer: Option<std::thread::JoinHandle<()>>,
    stdin_closed: bool,
    grace: std::time::Duration,
    stopping: Option<std::time::Instant>,
    output: [std::collections::VecDeque<Vec<u8>>; 2],
    capacity: usize,
    next_channel: usize,
    emitted: bool,
    #[cfg(not(unix))]
    signaled: Option<i32>,
}
pub struct NativeChildResource(RefCell<NativeChildState>);
impl TraceValue for NativeChildResource {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}
fn native_child_array(value: &V) -> &[V] {
    let V::Array(values) = value else {
        panic!("Rust ABI: expected Array")
    };
    values.get()
}
fn native_child_failure(tag: &'static str, payload: V) -> NativeExit {
    NativeExit::failure(V::adt(tag, payload))
}
fn native_child_io_failure(tag: &'static str, error: impl std::fmt::Display) -> NativeExit {
    native_child_failure(tag, V::text(&error.to_string()))
}
impl NativeChildResource {
    fn start(command: &V, inherited: bool) -> Result<Self, NativeExit> {
        let fields = native_child_command(command);
        let executable = &fields["executable"];
        let spawn_error = |detail: String| {
            native_child_failure(
                "std/child-process::ChildSpawnFailed",
                native_child_record(vec![
                    ("executable", executable.clone()),
                    ("detail", V::text(&detail)),
                ]),
            )
        };
        let mut environment: BTreeMap<std::ffi::OsString, std::ffi::OsString> =
            if fields["clear"].boolean() {
                BTreeMap::new()
            } else {
                std::env::vars_os().collect()
            };
        for (name, value) in fields["environment"].record_fields() {
            #[cfg(windows)]
            environment.retain(|key, _| !key.to_string_lossy().eq_ignore_ascii_case(name));
            if matches!(value, V::Unit) {
                environment.remove(std::ffi::OsStr::new(name));
            } else {
                environment.insert(name.into(), native_text(value).into());
            }
        }
        let searching = executable.tagged("std/child-process::SearchPath");
        if searching
            && !environment.keys().any(|key| {
                if cfg!(windows) {
                    key.to_string_lossy().eq_ignore_ascii_case("PATH")
                } else {
                    key == "PATH"
                }
            })
        {
            return Err(spawn_error(
                "SearchPath executable requires PATH in the child environment".into(),
            ));
        }
        let path = executable.payload();
        let path = if searching {
            native_text(&path)
        } else {
            native_path_text(&path)
        };
        let application_directory = std::env::var("SESERAGI_APPLICATION_ROOT")
            .ok()
            .filter(|directory| !directory.is_empty());
        let directory = if matches!(fields["directory"], V::Unit) {
            application_directory.as_deref()
        } else {
            Some(native_path_text(&fields["directory"]))
        };
        let path: std::ffi::OsString = if !searching && !std::path::Path::new(path).is_absolute() {
            let root = std::env::current_dir().map_err(|error| spawn_error(error.to_string()))?;
            let root = directory.map_or_else(|| root.clone(), |directory| root.join(directory));
            let resolved = root.join(path);
            let text = resolved.to_string_lossy();
            #[cfg(windows)]
            let text = text.replace('\\', "/");
            native_path_normalize(&text).into()
        } else {
            path.into()
        };
        let mut builder = std::process::Command::new(path);
        builder
            .args(
                native_child_array(&fields["arguments"])
                    .iter()
                    .map(native_text),
            )
            .env_clear()
            .envs(environment);
        if let Some(directory) = directory {
            builder.current_dir(directory);
        }
        if !inherited {
            builder
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
        }
        let mut child = builder
            .spawn()
            .map_err(|error| spawn_error(error.to_string()))?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let resource = Self(RefCell::new(NativeChildState {
            child,
            stdout,
            stderr,
            status: None,
            input: None,
            writer: None,
            stdin_closed: false,
            grace: std::time::Duration::from_millis(native_child_int(&fields["grace"]) as u64),
            stopping: None,
            output: Default::default(),
            capacity: usize::try_from(native_child_int(&fields["buffer"])).unwrap_or(usize::MAX),
            next_channel: 0,
            emitted: false,
            #[cfg(not(unix))]
            signaled: None,
        }));
        if let Some(stdin) = stdin {
            let (send, receive) = std::sync::mpsc::channel::<NativeChildWrite>();
            let worker = std::thread::Builder::new()
                .name("seseragi-child-stdin".into())
                .spawn(move || {
                    let mut stdin = Some(stdin);
                    for request in receive {
                        let result = match &mut stdin {
                            Some(stdin) => std::io::Write::write_all(stdin, &request.bytes)
                                .map_err(|error| error.to_string()),
                            None if request.bytes.is_empty() => Ok(()),
                            None => Err("child stdin is closed".into()),
                        };
                        if request.close {
                            stdin.take();
                        }
                        let _ = request.result.send(result);
                    }
                })
                .map_err(|error| spawn_error(error.to_string()))?;
            let mut state = resource.0.borrow_mut();
            state.input = Some(send);
            state.writer = Some(worker);
        }
        Ok(resource)
    }
}
impl V {
    fn child_process(&self) -> &NativeChildResource {
        let Self::ChildProcess(child) = self else {
            panic!("Rust ABI: expected child process resource")
        };
        child.get()
    }
}
impl NativeChildState {
    fn write(&mut self, bytes: Vec<u8>, close: bool) -> Result<NativeChildWriteResult, NativeExit> {
        if self.stdin_closed && !bytes.is_empty() {
            return Err(native_child_failure(
                "std/child-process::ChildInputAfterClose",
                V::Unit,
            ));
        }
        let (send, receive) = std::sync::mpsc::channel();
        if self.stdin_closed {
            let _ = send.send(Ok(()));
        } else {
            self.input
                .as_ref()
                .ok_or_else(|| {
                    native_child_io_failure(
                        "std/child-process::ChildInputFailed",
                        "child stdin is closed",
                    )
                })?
                .send(NativeChildWrite {
                    bytes,
                    close,
                    result: send,
                })
                .map_err(|error| {
                    native_child_io_failure("std/child-process::ChildInputFailed", error)
                })?;
        }
        self.stdin_closed |= close;
        Ok(receive)
    }
    fn wait(&mut self) -> Result<(), NativeExit> {
        if self.status.is_none() {
            self.status = self.child.try_wait().map_err(|error| {
                native_child_io_failure("std/child-process::ChildWaitFailed", error)
            })?;
        }
        Ok(())
    }
    fn signal(&mut self, signal: &V) -> Result<(), NativeExit> {
        let signal_number = match native_error_tag(signal) {
            "Interrupt" => 2,
            "Terminate" => 15,
            "Hangup" => 1,
            "Quit" => 3,
            "User1" => {
                if cfg!(target_os = "macos") {
                    30
                } else {
                    10
                }
            }
            "User2" => {
                if cfg!(target_os = "macos") {
                    31
                } else {
                    12
                }
            }
            _ => panic!("Rust ABI: ProcessSignal owner"),
        };
        #[cfg(windows)]
        if !matches!(signal_number, 2 | 15) {
            return Err(native_child_failure(
                "std/child-process::UnsupportedChildSignal",
                signal.clone(),
            ));
        }
        self.wait()?;
        if self.status.is_some() {
            return Ok(());
        }
        self.host_signal(signal_number)
            .map_err(|error| native_child_io_failure("std/child-process::ChildInputFailed", error))
    }
    fn host_signal(&mut self, signal: i32) -> std::io::Result<()> {
        #[cfg(unix)]
        {
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            if unsafe { kill(self.child.id() as i32, signal) } == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
        #[cfg(not(unix))]
        {
            self.child.kill()?;
            self.signaled = Some(signal);
            Ok(())
        }
    }
    fn exit_status(&self, status: std::process::ExitStatus) -> V {
        #[cfg(windows)]
        if let Some(signal) = self.signaled {
            return match signal {
                2 => V::adt(
                    "std/child-process::ChildSignaled",
                    V::adt("std/process::Interrupt", V::Unit),
                ),
                15 => V::adt(
                    "std/child-process::ChildSignaled",
                    V::adt("std/process::Terminate", V::Unit),
                ),
                _ => V::adt("std/child-process::ChildHostTerminated", V::text("SIGKILL")),
            };
        }
        native_child_status(status)
    }
    fn stop(&mut self) -> Result<bool, NativeExit> {
        if self.stopping.is_none() {
            self.input.take();
            self.output.iter_mut().for_each(|queue| queue.clear());
            self.wait()?;
            if self.status.is_none() {
                let _ = self.host_signal(15);
            }
            self.stopping = Some(std::time::Instant::now());
        }
        for channel in 0..2 {
            for _ in 0..16 {
                if self.read(channel).ok().flatten().is_none() {
                    break;
                }
            }
        }
        self.wait()?;
        if self.status.is_none() && self.stopping.unwrap().elapsed() >= self.grace {
            self.child.kill().map_err(|error| {
                native_child_io_failure("std/child-process::ChildTerminationFailed", error)
            })?;
            self.wait()?;
        }
        if self.status.is_none() {
            return Ok(false);
        }
        self.stdout.take();
        self.stderr.take();
        if self
            .writer
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
        {
            return Ok(false);
        }
        if let Some(worker) = self.writer.take() {
            worker
                .join()
                .map_err(|_| NativeExit::defects(vec!["child stdin worker panicked".into()]))?;
        }
        Ok(true)
    }
    fn read(&mut self, channel: usize) -> Result<Option<Vec<u8>>, NativeExit> {
        let mut bytes = vec![0; 65_536];
        let count = if channel == 0 {
            match &mut self.stdout {
                Some(pipe) => native_child_read(pipe, &mut bytes),
                None => return Ok(None),
            }
        } else {
            match &mut self.stderr {
                Some(pipe) => native_child_read(pipe, &mut bytes),
                None => return Ok(None),
            }
        }
        .map_err(|error| {
            native_child_failure(
                "std/child-process::ChildOutputReadFailed",
                native_child_record(vec![
                    ("channel", native_child_channel(channel)),
                    ("detail", V::text(&error.to_string())),
                ]),
            )
        })?;
        match count {
            Some(0) => {
                if channel == 0 {
                    self.stdout.take();
                } else {
                    self.stderr.take();
                }
                Ok(None)
            }
            Some(count) => {
                bytes.truncate(count);
                Ok(Some(bytes))
            }
            None => Ok(None),
        }
    }
}
impl Drop for NativeChildResource {
    fn drop(&mut self) {
        let state = self.0.get_mut();
        state.input.take();
        state.stdout.take();
        state.stderr.take();
        if state.status.is_none() {
            let _ = state.child.kill();
            let _ = state.child.wait();
        }
        if let Some(writer) = state.writer.take() {
            let _ = writer.join();
        }
    }
}
fn native_child_channel(channel: usize) -> V {
    V::adt(
        if channel == 0 {
            "std/child-process::ChildStdout"
        } else {
            "std/child-process::ChildStderr"
        },
        V::Unit,
    )
}
