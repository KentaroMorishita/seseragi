#[derive(Clone, Copy)]
enum NativeFileAction {
    OpenRead,
    OpenWrite,
    Read,
    Write,
    Flush,
    Close,
    CloseWrite,
}
pub struct NativeFileResource(std::sync::Arc<std::sync::Mutex<Option<std::fs::File>>>);
#[derive(Debug)]
struct NativeFileProviderError(&'static str);
impl std::fmt::Display for NativeFileProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}
impl std::error::Error for NativeFileProviderError {}
fn native_file_provider_error(message: &'static str) -> std::io::Error {
    std::io::Error::other(NativeFileProviderError(message))
}
impl TraceValue for NativeFileResource {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}
fn native_write_mode(mode: &V) -> &'static str {
    let V::Adt(mode) = mode else {
        panic!("Rust ABI: expected WriteMode")
    };
    match mode.get().0 {
        "std/fs::Replace" => "Replace",
        "std/fs::CreateNew" => "CreateNew",
        "std/fs::Append" => "Append",
        _ => panic!("Rust ABI: WriteMode owner"),
    }
}
fn native_run_file_action<H: NativeFileHost>(
    host: &mut H,
    handle: &mut Option<H::Handle>,
    action: NativeFileAction,
    path: &str,
    mode: Option<&str>,
    bytes: &[u8],
    limit: usize,
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<NativeFilePayload, NativeFileFault> {
    native_file_checkpoint(cancelled)?;
    match action {
        NativeFileAction::OpenRead | NativeFileAction::OpenWrite => {
            assert!(handle.is_none(), "Rust ABI: file already open");
            *handle = Some(host.open(path, mode).map_err(NativeFileFault::Io)?);
        }
        NativeFileAction::Read => {
            assert!(limit > 0, "Rust ABI: nonpositive read capacity");
            let mut bytes = Vec::new();
            bytes.try_reserve_exact(limit).map_err(|_| {
                NativeFileFault::Io(native_file_provider_error(
                    "filesystem read capacity exceeds host memory limits",
                ))
            })?;
            bytes.resize(limit, 0);
            let size = loop {
                native_file_checkpoint(cancelled)?;
                match host.read(handle.as_mut().expect("open file"), &mut bytes) {
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    result => break result.map_err(NativeFileFault::Io)?,
                }
            };
            assert!(
                size <= bytes.len(),
                "file host returned an invalid read length"
            );
            bytes.truncate(size);
            return Ok(NativeFilePayload::Bytes(bytes));
        }
        NativeFileAction::Write => {
            let mut offset = 0;
            while offset < bytes.len() {
                native_file_checkpoint(cancelled)?;
                let size = match host.write(handle.as_mut().expect("open file"), &bytes[offset..]) {
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    result => result.map_err(NativeFileFault::Io)?,
                };
                if size == 0 {
                    return Err(NativeFileFault::Io(native_file_provider_error(
                        "filesystem write made no progress",
                    )));
                }
                assert!(
                    size <= bytes.len() - offset,
                    "file host returned an invalid write length"
                );
                offset += size;
            }
        }
        NativeFileAction::Flush => host
            .flush(handle.as_mut().expect("open file"))
            .map_err(NativeFileFault::Io)?,
        NativeFileAction::Close | NativeFileAction::CloseWrite => {
            if let Some(handle) = handle.take() {
                host.close(handle).map_err(NativeFileFault::Io)?;
            }
        }
    }
    Ok(NativeFilePayload::Unit)
}
fn native_run_file_session(
    request: NativeFileRequest,
    cancelled: &std::sync::atomic::AtomicBool,
) -> NativeFileReply {
    let NativeFileRequest::Session {
        file,
        action,
        path,
        mode,
        bytes,
        limit,
    } = request
    else {
        unreachable!()
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut handle = file.lock().unwrap_or_else(|error| error.into_inner());
        native_run_file_action(
            &mut NativeStdFileHost,
            &mut handle,
            action,
            &path,
            mode,
            &bytes,
            limit,
            cancelled,
        )
    }))
    .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error))));
    NativeFileReply {
        result,
        close: Ok(()),
    }
}
fn native_file_action(action: NativeFileAction, file: V, path: V, data: V) -> V {
    V::effect(
        Instruction::FileSystem(NativeFileOperation::Session(action)),
        vec![file, path, data],
    )
}
fn native_file_cleanup(close: V) -> V {
    V::effect(
        Instruction::Recover,
        vec![
            V::fun(vec![], |_, error| {
                V::leaf(vec![error], |captures, _| {
                    Err(NativeExit::defects(vec![format!(
                        "filesystem cleanup failed: {}",
                        captures[0].render()
                    )]))
                })
            }),
            close,
        ],
    )
}
fn native_file_acquire(path: V, mode: V) -> V {
    let opened = stream_cell(V::Bool(false));
    let writing = !matches!(mode, V::Unit);
    let allocate = V::leaf(vec![], |_, _| {
        Ok(V::File(Shared::new(NativeFileResource(Default::default()))))
    });
    let registered = V::effect(
        Instruction::AcquireRelease,
        vec![
            allocate,
            V::fun(
                vec![path.clone(), opened.clone(), V::Bool(writing)],
                |captures, file| {
                    let close = native_file_action(
                        if captures[2].boolean() {
                            NativeFileAction::CloseWrite
                        } else {
                            NativeFileAction::Close
                        },
                        file,
                        captures[0].clone(),
                        V::Unit,
                    );
                    if !(captures[2].boolean() && captures[1].read_reference().boolean()) {
                        return native_file_cleanup(close);
                    }
                    V::effect(
                        Instruction::Recover,
                        vec![V::fun(vec![], |_, _| stream_pure(V::Unit)), close],
                    )
                },
            ),
        ],
    );
    V::bind(
        registered,
        V::fun(vec![path, mode, opened], |captures, file| {
            let action = if matches!(captures[1], V::Unit) {
                NativeFileAction::OpenRead
            } else {
                NativeFileAction::OpenWrite
            };
            V::bind(
                native_file_action(
                    action,
                    file.clone(),
                    captures[0].clone(),
                    captures[1].clone(),
                ),
                V::fun(vec![file, captures[2].clone()], |captures, _| {
                    captures[1].write_reference(V::Bool(true));
                    stream_pure(captures[0].clone())
                }),
            )
        }),
    )
}
// Cleanup observes the same close failure without retrying the OS close.
fn native_file_stream_close(plan: V, outcome: V) -> V {
    native_stream_defer(vec![plan, outcome], |captures, _| {
        let outcome = captures[1].read_reference();
        if outcome.tagged("std/prelude::Left") {
            return V::leaf(vec![outcome.payload()], |captures, _| {
                Err(NativeExit::failure(captures[0].clone()))
            });
        }
        if matches!(outcome, V::Bool(true)) {
            return stream_pure(V::Unit);
        }
        let close = V::bind(
            captures[0].clone(),
            V::fun(vec![captures[1].clone()], |captures, _| {
                captures[0].write_reference(V::Bool(true));
                stream_pure(V::Unit)
            }),
        );
        V::effect(
            Instruction::Recover,
            vec![
                V::fun(vec![captures[1].clone()], |captures, error| {
                    captures[0].write_reference(native_left(error.clone()));
                    V::leaf(vec![error], |captures, _| {
                        Err(NativeExit::failure(captures[0].clone()))
                    })
                }),
                close,
            ],
        )
    })
}
fn native_file_read_chunks(capacity: V, path: V) -> V {
    V::fun(vec![capacity, path], |captures, _| {
        V::bind(
            native_file_acquire(captures[1].clone(), V::Unit),
            V::fun(captures.to_vec(), |captures, file| {
                let close = V::fun(
                    vec![
                        native_file_action(
                            NativeFileAction::Close,
                            file.clone(),
                            captures[1].clone(),
                            V::Unit,
                        ),
                        stream_cell(V::Unit),
                    ],
                    |captures, _| {
                        native_file_stream_close(captures[0].clone(), captures[1].clone())
                    },
                );
                let pull = V::fun(
                    vec![
                        file,
                        captures[1].clone(),
                        captures[0].payload(),
                        close.clone(),
                    ],
                    |captures, _| {
                        V::bind(
                            native_file_action(
                                NativeFileAction::Read,
                                captures[0].clone(),
                                captures[1].clone(),
                                captures[2].clone(),
                            ),
                            V::fun(captures.to_vec(), |captures, bytes| {
                                if native_bytes(&bytes).is_empty() {
                                    V::bind(
                                        captures[3].apply(V::Unit),
                                        V::fun(vec![], |_, _| stream_pure(stream_done())),
                                    )
                                } else {
                                    stream_pure(stream_item(bytes))
                                }
                            }),
                        )
                    },
                );
                V::effect(
                    Instruction::AcquireRelease,
                    vec![
                        stream_pure(stream_cursor(pull, close)),
                        V::fun(vec![], |_, cursor| {
                            native_file_cleanup(stream_close(&cursor))
                        }),
                    ],
                )
            }),
        )
    })
}
fn native_file_write_chunks(mode: V, source: V, path: V) -> V {
    let effect = native_stream_defer(vec![mode, source, path], |captures, _| {
        V::bind(
            native_file_acquire(captures[2].clone(), captures[0].clone()),
            V::fun(captures.to_vec(), |captures, file| {
                let source = native_stream_operation(
                    "std/stream::mapError",
                    vec![
                        V::fun(vec![], |_, error| native_left(error)),
                        captures[1].clone(),
                    ],
                );
                let write = V::fun(
                    vec![file.clone(), captures[2].clone()],
                    |captures, bytes| {
                        native_file_action(
                            NativeFileAction::Write,
                            captures[0].clone(),
                            captures[1].clone(),
                            bytes,
                        )
                    },
                );
                V::bind(
                    native_stream_operation("std/stream::runForEach", vec![write, source]),
                    V::fun(vec![file, captures[2].clone()], |captures, _| {
                        V::bind(
                            native_file_action(
                                NativeFileAction::Flush,
                                captures[0].clone(),
                                captures[1].clone(),
                                V::Unit,
                            ),
                            V::fun(captures.to_vec(), |captures, _| {
                                native_file_action(
                                    NativeFileAction::CloseWrite,
                                    captures[0].clone(),
                                    captures[1].clone(),
                                    V::Unit,
                                )
                            }),
                        )
                    }),
                )
            }),
        )
    });
    V::effect(
        Instruction::MapError,
        vec![
            V::fun(vec![], |_, error| {
                if error.tagged("std/prelude::Left") {
                    error
                } else {
                    native_right(error)
                }
            }),
            V::effect(Instruction::Scoped, vec![effect]),
        ],
    )
}

#[cfg(test)]
mod file_chunk_tests {
    use super::*;

    #[derive(Default)]
    struct ShortHost {
        read: usize,
        written: Vec<u8>,
        interrupted: bool,
        fail: Option<&'static str>,
        closes: usize,
    }
    impl NativeFileHost for ShortHost {
        type Handle = ();
        fn open(&mut self, _: &str, _: Option<&str>) -> std::io::Result<()> {
            Ok(())
        }
        fn read(&mut self, _: &mut (), buffer: &mut [u8]) -> std::io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            let input = [0, 128, 255, 9, 8];
            let size = (input.len() - self.read).min(2).min(buffer.len());
            buffer[..size].copy_from_slice(&input[self.read..self.read + size]);
            self.read += size;
            Ok(size)
        }
        fn write(&mut self, _: &mut (), bytes: &[u8]) -> std::io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(std::io::ErrorKind::Interrupted.into());
            }
            if self.fail == Some("write") {
                return Ok(0);
            }
            let size = bytes.len().min(2);
            self.written.extend_from_slice(&bytes[..size]);
            Ok(size)
        }
        fn flush(&mut self, _: &mut ()) -> std::io::Result<()> {
            if self.fail == Some("flush") {
                Err(std::io::ErrorKind::PermissionDenied.into())
            } else {
                Ok(())
            }
        }
        fn close(&mut self, _: ()) -> std::io::Result<()> {
            self.closes += 1;
            if self.fail == Some("close") {
                Err(std::io::ErrorKind::PermissionDenied.into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn chunk_io_keeps_short_reads_retries_partial_writes_and_classifies_each_error_stage() {
        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let mut reader = ShortHost::default();
        let mut handle = Some(());
        for expected in [vec![0, 128], vec![255, 9], vec![8], vec![]] {
            let result = native_run_file_action(
                &mut reader,
                &mut handle,
                NativeFileAction::Read,
                "input",
                None,
                &[],
                3,
                &cancelled,
            );
            let Ok(NativeFilePayload::Bytes(bytes)) = result else {
                panic!("read failed")
            };
            assert_eq!(bytes, expected);
        }
        for fail in [None, Some("write"), Some("flush"), Some("close")] {
            let mut host = ShortHost {
                fail,
                ..ShortHost::default()
            };
            let mut handle = Some(());
            for action in [
                NativeFileAction::Write,
                NativeFileAction::Flush,
                NativeFileAction::CloseWrite,
            ] {
                let result = native_run_file_action(
                    &mut host,
                    &mut handle,
                    action,
                    "output",
                    None,
                    &[0, 128, 255, 9, 8],
                    0,
                    &cancelled,
                );
                if let Err(NativeFileFault::Io(error)) = result {
                    let operation = NativeFileOperation::Session(action);
                    let exit = operation.failure(&[V::Unit, native_path("output")], error);
                    let NativeExit::Failure(error) = exit else {
                        panic!("typed I/O failure expected")
                    };
                    assert!(error.field("operation").tagged("std/fs::WriteFile"));
                    assert!(error.field("kind").tagged(if fail == Some("write") {
                        "std/fs::OtherFileSystemError"
                    } else {
                        "std/fs::PermissionDenied"
                    }));
                    if fail == Some("write") {
                        assert_eq!(
                            native_text(&error.field("kind").payload()),
                            "FILESYSTEM_ERROR"
                        );
                    }
                    break;
                }
                assert!(result.is_ok());
            }
            let _ = native_run_file_action(
                &mut host,
                &mut handle,
                NativeFileAction::Close,
                "output",
                None,
                &[],
                0,
                &cancelled,
            );
            assert_eq!(host.closes, 1);
            assert!(handle.is_none());
            if fail != Some("write") {
                assert_eq!(host.written, [0, 128, 255, 9, 8]);
            }
        }
        let captures = [V::Unit, V::Unit, native_path("output")];
        let NativeExit::Failure(error) = NativeFileOperation::WriteBytes
            .failure(&captures, std::io::ErrorKind::WriteZero.into())
        else {
            panic!("typed write failure expected")
        };
        assert!(error.field("operation").tagged("std/fs::WriteFile"));
        assert_eq!(
            native_text(&error.field("kind").payload()),
            "FILESYSTEM_ERROR"
        );
        drop(error);
        let (outcome, notes) = NativeFileOperation::WriteBytes.outcome(
            &captures,
            NativeFileReply {
                result: Ok(NativeFilePayload::Unit),
                close: Err(NativeFileFault::Io(
                    std::io::ErrorKind::PermissionDenied.into(),
                )),
            },
            false,
        );
        let Err(NativeExit::Failure(error)) = outcome else {
            panic!("typed close failure expected")
        };
        assert!(error.field("operation").tagged("std/fs::WriteFile"));
        assert!(error.field("kind").tagged("std/fs::PermissionDenied"));
        assert!(notes.is_empty());
        drop((captures, error));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "seseragi-file-chunks-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> V {
            native_path(self.0.join("file").to_str().unwrap())
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn capacity(value: i64) -> V {
        native_stream_operation("std/stream::bufferCapacity", vec![V::Int(value)]).payload()
    }
    fn success(plan: V) -> V {
        NativeContext::default()
            .root(plan)
            .unwrap_or_else(|_| panic!("unexpected file failure"))
    }

    #[test]
    fn chunk_capacity_keeps_nonpositive_errors_and_the_safe_int_boundary() {
        for value in [
            -9_007_199_254_740_991,
            -1,
            0,
            1,
            3,
            65536,
            9_007_199_254_740_991,
        ] {
            let result = native_stream_operation("std/stream::bufferCapacity", vec![V::Int(value)]);
            assert_eq!(result.tagged("std/prelude::Right"), value > 0);
            if value <= 0 {
                assert!(result
                    .payload()
                    .tagged("std/stream::NonPositiveBufferCapacity"));
            }
            assert_eq!(result.payload().payload().render(), value.to_string());
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn stream_cleanup_reobserves_a_failed_close_without_retrying_the_host() {
        let calls = stream_cell(V::Int(0));
        let plan = V::leaf(vec![calls.clone()], |captures, _| {
            let V::Int(count) = captures[0].read_reference() else {
                unreachable!()
            };
            captures[0].write_reference(V::Int(count + 1));
            Err(NativeExit::failure(V::text("close failed")))
        });
        let outcome = stream_cell(V::Unit);
        let cursor = stream_cursor(
            V::fun(vec![plan.clone(), outcome.clone()], |captures, _| {
                native_file_stream_close(captures[0].clone(), captures[1].clone())
            }),
            V::fun(vec![plan, outcome], |captures, _| {
                native_file_stream_close(captures[0].clone(), captures[1].clone())
            }),
        );
        let source = V::fun(vec![cursor], |captures, _| {
            V::effect(
                Instruction::AcquireRelease,
                vec![
                    stream_pure(captures[0].clone()),
                    V::fun(vec![], |_, cursor| {
                        native_file_cleanup(stream_close(&cursor))
                    }),
                ],
            )
        });
        let result = NativeContext::default().root(native_stream_operation(
            "std/stream::runCollect",
            vec![source],
        ));
        assert!(
            matches!(result, Err(NativeExit::Defect(ref errors)) if errors.iter().any(|error| error.contains("close failed")))
        );
        assert_eq!(calls.read_reference().render(), "1");
        drop((calls, result));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn file_streams_are_cold_reopen_and_close_before_early_terminal_returns() {
        let scratch = Scratch::new();
        let path = scratch.path();
        let source = native_file_read_chunks(capacity(3), path.clone());
        let terminal = native_stream_operation(
            "std/stream::runCollect",
            vec![native_stream_operation(
                "std/stream::take",
                vec![V::Int(1), source.clone()],
            )],
        );
        assert!(!scratch.0.join("file").exists());
        std::fs::write(scratch.0.join("file"), [0, 128, 255, 1, 2]).unwrap();
        for bytes in [vec![0, 128, 255], vec![9, 8]] {
            let result = success(terminal.clone());
            assert_eq!(native_bytes(&result.index(0)), bytes);
            std::fs::remove_file(scratch.0.join("file")).unwrap();
            std::fs::write(scratch.0.join("file"), [9, 8]).unwrap();
        }
        let empty = success(native_stream_operation(
            "std/stream::runCollect",
            vec![native_stream_operation(
                "std/stream::take",
                vec![V::Int(0), source.clone()],
            )],
        ));
        assert_eq!(empty.render(), "[]");
        std::fs::remove_file(scratch.0.join("file")).unwrap();
        let missing = NativeContext::default().root(native_stream_operation(
            "std/stream::runCollect",
            vec![native_stream_operation(
                "std/stream::take",
                vec![V::Int(0), source.clone()],
            )],
        ));
        assert!(
            matches!(missing, Err(NativeExit::Failure(ref error)) if error.field("kind").tagged("std/fs::FileNotFound"))
        );
        drop((source, terminal, path, empty, missing));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn chunk_write_preserves_source_failures_defects_cancellation_and_partial_output() {
        let scratch = Scratch::new();
        for ending in 0..3 {
            let path = scratch.path();
            let prefix = native_stream_singleton(V::Bytes(Shared::new(vec![0, 128, 255])));
            let suffix = native_stream_operation(
                "std/stream::fromEffect",
                vec![V::leaf(
                    vec![V::Int(ending)],
                    |captures, context| match captures[0] {
                        V::Int(0) => Err(NativeExit::failure(native_right(V::text("upstream")))),
                        V::Int(1) => panic!("upstream defect"),
                        _ => {
                            context.cancelled = true;
                            Err(NativeExit::Cancelled)
                        }
                    },
                )],
            );
            let source = native_stream_operation("std/stream::concat", vec![suffix, prefix]);
            let result = NativeContext::default().root(native_file_write_chunks(
                V::adt("std/fs::Replace", V::Unit),
                source,
                path,
            ));
            match ending {
                0 => assert!(
                    matches!(result, Err(NativeExit::Failure(ref error)) if error.tagged("std/prelude::Left") && error.payload().tagged("std/prelude::Right") && native_text(&error.payload().payload()) == "upstream")
                ),
                1 => assert!(
                    matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["upstream defect"])
                ),
                _ => assert!(matches!(result, Err(NativeExit::Cancelled))),
            }
            assert_eq!(
                std::fs::read(scratch.0.join("file")).unwrap(),
                [0, 128, 255]
            );
            std::fs::remove_file(scratch.0.join("file")).unwrap();
            drop(result);
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn open_resources_close_after_callback_failure_defect_and_cancellation() {
        let scratch = Scratch::new();
        for ending in 0..3 {
            let slot = stream_cell(V::Unit);
            let acquire = native_file_acquire(scratch.path(), V::adt("std/fs::Replace", V::Unit));
            let plan = V::bind(
                acquire,
                V::fun(vec![slot.clone(), V::Int(ending)], |captures, file| {
                    captures[0].write_reference(file);
                    V::leaf(
                        vec![captures[1].clone()],
                        |captures, context| match captures[0] {
                            V::Int(0) => Err(NativeExit::failure(V::text("failed"))),
                            V::Int(1) => panic!("callback defect"),
                            _ => {
                                context.cancelled = true;
                                Err(NativeExit::Cancelled)
                            }
                        },
                    )
                }),
            );
            let result = NativeContext::default().root(plan);
            assert!(result.is_err());
            let resource = slot.read_reference();
            let V::File(file) = &resource else {
                panic!("missing file resource")
            };
            assert!(file.get().0.lock().unwrap().is_none());
            std::fs::remove_file(scratch.0.join("file")).unwrap();
            drop((slot, resource, result));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
