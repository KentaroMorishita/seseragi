// Only owned host data crosses a worker boundary. Managed values, callbacks,
// evidence and Effect continuations remain on the scheduler's runtime thread.
#[derive(Clone, Copy)]
enum NativeFileOperation {
    ReadText,
    WriteText,
}
enum NativeFileRequest {
    Read {
        path: String,
    },
    Write {
        path: String,
        mode: &'static str,
        bytes: Vec<u8>,
    },
}
enum NativeFilePayload {
    Bytes(Vec<u8>),
    Unit,
}
enum NativeFileFault {
    Io(std::io::Error),
    Defect(String),
    Cancelled,
}
struct NativeFileReply {
    result: Result<NativeFilePayload, NativeFileFault>,
    close: Result<(), NativeFileFault>,
}
struct NativeFileTask {
    operation: NativeFileOperation,
    captures: Vec<V>,
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    completion: std::sync::mpsc::Receiver<NativeFileReply>,
    worker: Option<std::thread::JoinHandle<()>>,
}
struct NativeFileCompletion {
    send: Option<std::sync::mpsc::Sender<NativeFileReply>>,
    runtime: std::thread::Thread,
}
impl Drop for NativeFileCompletion {
    fn drop(&mut self) {
        // Publish disconnection before waking, including exceptional worker
        // exit. Otherwise an empty scheduler could sleep through a panic.
        drop(self.send.take());
        self.runtime.unpark();
    }
}
impl NativeFileOperation {
    fn request(self, captures: &[V]) -> NativeFileRequest {
        match self {
            Self::ReadText => NativeFileRequest::Read {
                path: native_path_text(&captures[0]).into(),
            },
            Self::WriteText => {
                let V::Adt(mode) = &captures[0] else {
                    panic!("Rust ABI: expected WriteMode")
                };
                let mode = match mode.get().0.as_str() {
                    "std/fs::Replace" => "Replace",
                    "std/fs::CreateNew" => "CreateNew",
                    "std/fs::Append" => "Append",
                    _ => panic!("Rust ABI: WriteMode owner"),
                };
                NativeFileRequest::Write {
                    path: native_path_text(&captures[2]).into(),
                    mode,
                    bytes: native_text(&captures[1]).as_bytes().to_vec(),
                }
            }
        }
    }
    fn failure(self, captures: &[V], error: std::io::Error) -> NativeExit {
        NativeExit::Failure(match self {
            Self::ReadText => V::adt(
                "std/fs::FileAccessFailure",
                native_file_error("ReadFile", captures[0].clone(), error),
            ),
            Self::WriteText => native_file_error("WriteFile", captures[2].clone(), error),
        })
    }
    fn outcome(
        self,
        captures: &[V],
        reply: NativeFileReply,
        cancelled: bool,
    ) -> (NativeOutcome, Vec<String>) {
        let NativeFileReply { result, close } = reply;
        let cancelled = cancelled || matches!(&result, Err(NativeFileFault::Cancelled));
        let mut notes = Vec::new();
        if cancelled {
            if let Err(fault) = close {
                return (
                    Err(NativeExit::Defect(vec![native_file_close_defect(fault)])),
                    notes,
                );
            }
            if let Err(NativeFileFault::Defect(error)) = result {
                notes.push(format!("late host defect after cancellation: {error}"));
            }
            return (Err(NativeExit::Cancelled), notes);
        }
        let result = match (result, close) {
            (Err(NativeFileFault::Defect(error)), Err(close)) => {
                return (
                    Err(NativeExit::Defect(vec![
                        native_file_close_defect(close),
                        error,
                    ])),
                    notes,
                );
            }
            (_, Err(NativeFileFault::Defect(error))) => {
                return (Err(NativeExit::Defect(vec![error])), notes);
            }
            (Err(primary), Err(close)) => {
                // Preserve the established small-file primary I/O failure;
                // the failed close is retained as a diagnostic, not lost.
                notes.push(native_file_close_defect(close));
                Err(primary)
            }
            (Ok(_), Err(close)) => Err(close),
            (result, Ok(())) => result,
        };
        let outcome = match result {
            Ok(NativeFilePayload::Bytes(bytes)) if matches!(self, Self::ReadText) => {
                native_decode_text(bytes)
            }
            Ok(NativeFilePayload::Unit) if matches!(self, Self::WriteText) => Ok(V::Unit),
            Ok(_) => Err(NativeExit::Defect(vec![
                "invalid native filesystem worker result".into(),
            ])),
            Err(NativeFileFault::Io(error)) => Err(self.failure(captures, error)),
            Err(NativeFileFault::Defect(error)) => Err(NativeExit::Defect(vec![error])),
            Err(NativeFileFault::Cancelled) => Err(NativeExit::Cancelled),
        };
        (outcome, notes)
    }
}
fn native_file_close_defect(fault: NativeFileFault) -> String {
    match fault {
        NativeFileFault::Io(error) => format!("file close failed: {error}"),
        NativeFileFault::Defect(error) => error,
        NativeFileFault::Cancelled => "invalid cancelled file close".into(),
    }
}
impl NativeFileTask {
    fn start(operation: NativeFileOperation, captures: Vec<V>) -> Result<Self, NativeExit> {
        let request = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            operation.request(&captures)
        }))
        .map_err(native_panic)?;
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancellation = cancelled.clone();
        let (send, completion) = std::sync::mpsc::channel();
        let completion_owner = NativeFileCompletion {
            send: Some(send),
            runtime: std::thread::current(),
        };
        let worker = std::thread::Builder::new()
            .name("seseragi-file-io".into())
            .spawn(move || {
                let mut completion = completion_owner;
                let reply = native_run_file_request(&mut NativeStdFileHost, request, &cancellation);
                // Handles are closed before publishing. The runtime always joins
                // this worker before committing the result or scope cancellation.
                let _ = completion
                    .send
                    .take()
                    .expect("owned file completion")
                    .send(reply);
            })
            .map_err(|error| {
                NativeExit::Defect(vec![format!("cannot start filesystem worker: {error}")])
            })?;
        Ok(Self {
            operation,
            captures,
            cancelled,
            completion,
            worker: Some(worker),
        })
    }
    fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Release);
    }
    fn poll(&mut self) -> Option<(NativeOutcome, Vec<String>)> {
        let reply = match self.completion.try_recv() {
            Ok(reply) => Some(reply),
            Err(std::sync::mpsc::TryRecvError::Empty) => return None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => None,
        };
        let joined = self.worker.take().expect("owned filesystem worker").join();
        let reply = match joined {
            Err(error) => NativeFileReply {
                result: Err(NativeFileFault::Defect(native_panic_message(error))),
                close: Ok(()),
            },
            Ok(()) => reply.unwrap_or_else(|| NativeFileReply {
                result: Err(NativeFileFault::Defect(
                    "filesystem worker returned no result".into(),
                )),
                close: Ok(()),
            }),
        };
        Some(self.operation.outcome(
            &self.captures,
            reply,
            self.cancelled.load(std::sync::atomic::Ordering::Acquire),
        ))
    }
}
impl Drop for NativeFileTask {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            // Exceptional scheduler unwinding must not detach a resource owner.
            // An OS call without cancellation still needs its host completion.
            self.cancel();
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod host_tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    };
    use std::time::Duration;

    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let directory = std::env::temp_dir().join(format!(
                "seseragi-host-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&directory).unwrap();
            std::fs::write(directory.join("input"), b"hello").unwrap();
            Self(directory)
        }
        fn path(&self) -> String {
            self.0.join("input").to_string_lossy().into_owned()
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    struct Joined<T>(Option<std::thread::JoinHandle<T>>);
    impl<T> Joined<T> {
        fn take(&mut self) -> T {
            self.0.take().unwrap().join().unwrap()
        }
    }
    impl<T> Drop for Joined<T> {
        fn drop(&mut self) {
            if let Some(worker) = self.0.take() {
                let _ = worker.join();
            }
        }
    }
    struct Host {
        events: Arc<Mutex<Vec<&'static str>>>,
        pause: Option<&'static str>,
        ready: Option<mpsc::Sender<()>>,
        resume: mpsc::Receiver<()>,
        fail_read: bool,
        panic_read: bool,
        fail_close: bool,
        panic_close: bool,
    }
    impl Host {
        fn event(&mut self, name: &'static str) {
            self.events.lock().unwrap().push(name);
            if self.pause == Some(name) {
                if let Some(ready) = self.ready.take() {
                    ready.send(()).unwrap();
                    self.resume
                        .recv_timeout(Duration::from_secs(5))
                        .expect("test must release host operation");
                }
            }
        }
    }
    impl NativeFileHost for Host {
        type Handle = std::fs::File;
        fn open(&mut self, path: &str, mode: Option<&str>) -> std::io::Result<Self::Handle> {
            #[cfg(not(windows))]
            let file = NativeStdFileHost.open(path, mode)?;
            #[cfg(windows)]
            let file = {
                use std::os::windows::fs::OpenOptionsExt;
                let mut options = std::fs::OpenOptions::new();
                options.share_mode(0);
                match mode {
                    None => {
                        options.read(true);
                    }
                    Some("Replace") => {
                        options.write(true).create(true).truncate(true);
                    }
                    Some("Append") => {
                        options.append(true).create(true);
                    }
                    Some("CreateNew") => {
                        options.write(true).create_new(true);
                    }
                    _ => panic!("test WriteMode"),
                }
                options.open(path)?
            };
            self.event("open");
            Ok(file)
        }
        fn read(&mut self, file: &mut Self::Handle, bytes: &mut [u8]) -> std::io::Result<usize> {
            self.event("read");
            assert!(!self.panic_read, "host read defect");
            if self.fail_read {
                return Err(std::io::ErrorKind::PermissionDenied.into());
            }
            NativeStdFileHost.read(file, bytes)
        }
        fn write(&mut self, file: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<usize> {
            self.event("write");
            NativeStdFileHost.write(file, bytes)
        }
        fn flush(&mut self, file: &mut Self::Handle) -> std::io::Result<()> {
            self.event("flush");
            NativeStdFileHost.flush(file)
        }
        fn close(&mut self, file: Self::Handle) -> std::io::Result<()> {
            self.event("close");
            native_close_file(file)?;
            assert!(!self.panic_close, "host close defect");
            if self.fail_close {
                return Err(std::io::Error::other("host close failure"));
            }
            Ok(())
        }
    }
    fn host(pause: Option<&'static str>) -> (Host, mpsc::Receiver<()>, mpsc::Sender<()>) {
        let (ready, waiting) = mpsc::channel();
        let (resume, resumed) = mpsc::channel();
        (
            Host {
                events: Arc::new(Mutex::new(Vec::new())),
                pause,
                ready: Some(ready),
                resume: resumed,
                fail_read: false,
                panic_read: false,
                fail_close: false,
                panic_close: false,
            },
            waiting,
            resume,
        )
    }
    #[test]
    fn owned_file_workers_drain_each_pending_stage_and_close_once_on_cancellation() {
        fn send<T: Send>() {}
        send::<NativeFileRequest>();
        send::<NativeFileReply>();
        for stage in ["open", "read", "write", "flush", "close"] {
            let scratch = Scratch::new();
            let (mut host, waiting, resume) = host(Some(stage));
            let events = host.events.clone();
            let writing = matches!(stage, "write" | "flush");
            let request = if writing {
                NativeFileRequest::Write {
                    path: scratch.path(),
                    mode: "Replace",
                    bytes: vec![42; 128 * 1024],
                }
            } else {
                NativeFileRequest::Read {
                    path: scratch.path(),
                }
            };
            let cancelled = Arc::new(AtomicBool::new(false));
            let cancellation = cancelled.clone();
            let mut worker = Joined(Some(std::thread::spawn(move || {
                native_run_file_request(&mut host, request, &cancellation)
            })));
            let pending = waiting.recv_timeout(Duration::from_secs(5));
            #[cfg(windows)]
            let exclusive_while_pending = std::fs::File::open(scratch.path()).is_err();
            cancelled.store(true, Ordering::Release);
            cancelled.store(true, Ordering::Release);
            let _ = resume.send(());
            let reply = worker.take();
            assert!(pending.is_ok(), "{stage}");
            #[cfg(windows)]
            assert!(
                exclusive_while_pending,
                "real exclusive OS handle must still be owned"
            );
            assert!(reply.close.is_ok());
            let captures = if writing {
                vec![
                    V::adt("std/fs::Replace", V::Unit),
                    V::text("body"),
                    native_path(&scratch.path()),
                ]
            } else {
                vec![native_path(&scratch.path())]
            };
            let operation = if writing {
                NativeFileOperation::WriteText
            } else {
                NativeFileOperation::ReadText
            };
            let (outcome, notes) = operation.outcome(&captures, reply, true);
            assert!(matches!(outcome, Err(NativeExit::Cancelled)));
            assert!(notes.is_empty());
            let events = events.lock().unwrap();
            assert_eq!(events.iter().filter(|e| **e == "close").count(), 1);
            if stage == "open" {
                assert!(!events.contains(&"read"));
            }
            if stage == "read" {
                assert_eq!(events.iter().filter(|e| **e == "read").count(), 1);
            }
            if stage == "write" {
                assert!(!events.contains(&"flush"));
                assert_eq!(std::fs::metadata(scratch.path()).unwrap().len(), 64 * 1024);
            }
            // On Windows the exclusive handle above makes this an OS close
            // observation. POSIX CLI tests independently probe live FIFO FDs.
            drop(std::fs::File::open(scratch.path()).unwrap());
            std::fs::remove_file(scratch.path()).unwrap();
        }
    }
    #[test]
    fn file_worker_failures_keep_primary_io_and_cleanup_defect_distinct() {
        for (read_failure, read_defect, close_failure, close_defect, cancel) in [
            (true, false, true, false, false),
            (false, true, false, false, false),
            (false, true, true, false, false),
            (false, false, true, false, true),
            (false, false, false, true, true),
            (false, true, false, false, true),
        ] {
            let scratch = Scratch::new();
            let (mut host, _, _) = host(None);
            host.fail_read = read_failure;
            host.panic_read = read_defect;
            host.fail_close = close_failure;
            host.panic_close = close_defect;
            let reply = native_run_file_request(
                &mut host,
                NativeFileRequest::Read {
                    path: scratch.path(),
                },
                &AtomicBool::new(false),
            );
            assert_eq!(
                host.events
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|e| **e == "close")
                    .count(),
                1
            );
            let (result, notes) = NativeFileOperation::ReadText.outcome(
                &[native_path(&scratch.path())],
                reply,
                cancel,
            );
            match result {
                Err(NativeExit::Failure(error)) => {
                    assert!(read_failure && !cancel);
                    assert!(error
                        .payload()
                        .field("kind")
                        .tagged("std/fs::PermissionDenied"));
                    assert_eq!(notes.len(), 1);
                }
                Err(NativeExit::Cancelled) => {
                    assert!(cancel && !close_failure && !close_defect);
                    assert_eq!(notes.len(), usize::from(read_defect));
                }
                Err(NativeExit::Defect(errors)) => {
                    assert!(read_defect || close_failure || close_defect);
                    assert!(errors[0].contains(if close_defect {
                        "close defect"
                    } else if close_failure {
                        "close failed"
                    } else {
                        "read defect"
                    }));
                }
                _ => panic!("wrong host failure class"),
            }
            drop(std::fs::File::open(scratch.path()).unwrap());
        }
        let scratch = Scratch::new();
        let (mut host, _, _) = host(None);
        let reply = native_run_file_request(
            &mut host,
            NativeFileRequest::Read {
                path: scratch.path(),
            },
            &AtomicBool::new(true),
        );
        assert!(matches!(reply.result, Err(NativeFileFault::Cancelled)));
        assert!(host.events.lock().unwrap().is_empty());
    }
    #[test]
    fn exceptional_worker_exit_wakes_the_runtime_and_is_joined() {
        let (waiting, ready) = mpsc::channel();
        let (resume, resumed) = mpsc::channel();
        let (done, result) = mpsc::channel();
        let runtime = std::thread::spawn(move || {
            let (send, completion) = mpsc::channel();
            let completion_owner = NativeFileCompletion {
                send: Some(send),
                runtime: std::thread::current(),
            };
            let worker = std::thread::spawn(move || {
                let _completion = completion_owner;
                resumed.recv_timeout(Duration::from_secs(5)).unwrap();
                panic!("worker wake defect");
            });
            let mut task = NativeFileTask {
                operation: NativeFileOperation::ReadText,
                captures: vec![native_path(".")],
                cancelled: Arc::new(AtomicBool::new(false)),
                completion,
                worker: Some(worker),
            };
            assert!(task.poll().is_none());
            waiting.send(()).unwrap();
            let outcome = loop {
                std::thread::park();
                if let Some((outcome, _)) = task.poll() {
                    break outcome;
                }
            };
            done.send(matches!(outcome, Err(NativeExit::Defect(ref errors)) if errors[0] == "worker wake defect")).unwrap();
            drop(task);
        });
        let observed_wait = ready.recv_timeout(Duration::from_secs(5));
        let _ = resume.send(());
        let observed_result = result.recv_timeout(Duration::from_secs(5));
        // Even a failed wake assertion must release and join our test thread.
        runtime.thread().unpark();
        runtime.join().unwrap();
        assert!(observed_wait.is_ok());
        assert_eq!(observed_result.unwrap(), true);
    }
}
