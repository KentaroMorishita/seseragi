#[derive(Clone, Copy)]
enum NativePathOperation {
    WriteAtomic,
    CreateDirectory,
    CreateDirectories,
    RemoveFile,
    RemoveDirectory,
    Move,
    OpenDirectory,
    ReadDirectory,
    CloseDirectory,
    CreateTemporary(bool),
    CleanupTemporary,
}
#[derive(Clone)]
enum NativePathHandle {
    Directory(std::sync::Arc<std::sync::Mutex<Option<std::fs::ReadDir>>>),
    Temporary(NativeTemporaryHandle),
}
pub struct NativePathResource(NativePathHandle);
impl TraceValue for NativePathResource {
    fn edges(&self, _: &mut dyn FnMut(&V)) -> Result<(), &'static str> {
        Ok(())
    }
}
type NativeTemporaryHandle = std::sync::Arc<std::sync::Mutex<NativeTemporaryState>>;
#[derive(Default)]
struct NativeTemporaryState {
    resource: Option<NativeTemporary>,
    cleanup: Option<Result<(), std::io::Error>>,
}
#[derive(Clone)]
struct NativeTemporary {
    path: std::path::PathBuf,
    root: std::path::PathBuf,
    identity: (u64, u64),
}
struct NativeDirectoryEntry {
    name: String,
    path: String,
    file_type: std::fs::FileType,
}
impl NativePathOperation {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "std/fs::writeAtomic" => Self::WriteAtomic,
            "std/fs::createDirectory" => Self::CreateDirectory,
            "std/fs::createDirectories" => Self::CreateDirectories,
            "std/fs::removeFile" => Self::RemoveFile,
            "std/fs::removeDirectory" => Self::RemoveDirectory,
            "std/fs::move" => Self::Move,
            _ => return None,
        })
    }
    fn request(self, captures: &[V]) -> NativeFileRequest {
        let (resource, path, other_path, bytes) = match self {
            Self::WriteAtomic => (
                None,
                native_host_path(native_path_text(&captures[1])),
                String::new(),
                native_bytes(&captures[0]).to_vec(),
            ),
            Self::Move => (
                None,
                native_host_path(native_path_text(&captures[1])),
                native_host_path(native_path_text(&captures[0])),
                vec![],
            ),
            Self::CreateDirectory
            | Self::CreateDirectories
            | Self::RemoveFile
            | Self::RemoveDirectory => (
                None,
                native_host_path(native_path_text(&captures[0])),
                String::new(),
                vec![],
            ),
            _ => {
                let V::PathResource(resource) = &captures[0] else {
                    panic!("Rust ABI: expected filesystem resource")
                };
                (
                    Some(resource.get().0.clone()),
                    if matches!(self, Self::CreateTemporary(_)) {
                        native_text(&captures[1]).into()
                    } else {
                        native_host_path(native_path_text(&captures[1]))
                    },
                    String::new(),
                    vec![],
                )
            }
        };
        NativeFileRequest::Path {
            operation: self,
            resource,
            path,
            other_path,
            bytes,
        }
    }
    fn failure(self, captures: &[V], error: std::io::Error) -> V {
        let (operation, path, other) = match self {
            Self::WriteAtomic => ("std/fs::WriteFile", captures[1].clone(), None),
            Self::CreateDirectory | Self::CreateDirectories => {
                ("std/fs::CreateDirectory", captures[0].clone(), None)
            }
            Self::RemoveFile | Self::RemoveDirectory => {
                ("std/fs::RemovePath", captures[0].clone(), None)
            }
            Self::Move => (
                "std/fs::MovePath",
                captures[1].clone(),
                Some(captures[0].clone()),
            ),
            Self::OpenDirectory | Self::ReadDirectory | Self::CloseDirectory => {
                ("std/fs::OpenDirectory", captures[1].clone(), None)
            }
            Self::CreateTemporary(_) => ("std/fs::CreateTemporary", native_path("."), None),
            Self::CleanupTemporary => ("std/fs::RemovePath", captures[1].clone(), None),
        };
        native_file_error_with_other(operation, path, other, error)
    }
}
fn native_path_effect(operation: NativePathOperation, arguments: Vec<V>) -> V {
    let effect = V::effect(
        Instruction::FileSystem(NativeFileOperation::Path(operation)),
        arguments.clone(),
    );
    if !matches!(operation, NativePathOperation::Move) {
        return effect;
    }
    V::bind(
        V::leaf(arguments, |captures, context| {
            let source = native_path_normalize(native_path_text(&captures[1]));
            let destination = native_path_normalize(native_path_text(&captures[0]));
            let blocked = context.temporaries.borrow().iter().any(|handle| {
                let state = handle.lock().unwrap_or_else(|error| error.into_inner());
                state.resource.as_ref().is_some_and(|resource| {
                    let path = resource.path.to_string_lossy().replace('\\', "/");
                    path == source || path == destination
                })
            });
            if blocked {
                Err(NativeExit::failure(NativePathOperation::Move.failure(
                    captures,
                    std::io::ErrorKind::PermissionDenied.into(),
                )))
            } else {
                Ok(V::Unit)
            }
        }),
        V::fun(vec![effect], |captures, _| captures[0].clone()),
    )
}
fn native_run_path_request(
    request: NativeFileRequest,
    cancelled: &std::sync::atomic::AtomicBool,
) -> NativeFileReply {
    let NativeFileRequest::Path {
        operation,
        resource,
        path,
        other_path,
        bytes,
    } = request
    else {
        unreachable!()
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        native_file_checkpoint(cancelled)?;
        let unit = |result: std::io::Result<()>| {
            result
                .map(|()| NativeFilePayload::Unit)
                .map_err(NativeFileFault::Io)
        };
        match operation {
            NativePathOperation::WriteAtomic => {
                native_atomic_write(&mut NativeStdFileHost, &path, &bytes, cancelled)
            }
            NativePathOperation::CreateDirectory => unit(std::fs::create_dir(&path)),
            NativePathOperation::CreateDirectories => unit(std::fs::create_dir_all(&path)),
            NativePathOperation::RemoveFile => unit(std::fs::remove_file(&path)),
            NativePathOperation::RemoveDirectory => unit(std::fs::remove_dir(&path)),
            NativePathOperation::Move => {
                match std::fs::symlink_metadata(&other_path) {
                    Ok(_) => {
                        return Err(NativeFileFault::Io(
                            std::io::ErrorKind::AlreadyExists.into(),
                        ))
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                    Err(error) => return Err(NativeFileFault::Io(error)),
                }
                native_file_checkpoint(cancelled)?;
                unit(std::fs::rename(&path, &other_path))
            }
            NativePathOperation::OpenDirectory
            | NativePathOperation::ReadDirectory
            | NativePathOperation::CloseDirectory => {
                let Some(NativePathHandle::Directory(handle)) = resource else {
                    panic!("Rust ABI: directory resource")
                };
                let mut handle = handle.lock().unwrap_or_else(|error| error.into_inner());
                match operation {
                    NativePathOperation::OpenDirectory => {
                        *handle = Some(std::fs::read_dir(&path).map_err(NativeFileFault::Io)?);
                        Ok(NativeFilePayload::Unit)
                    }
                    NativePathOperation::ReadDirectory => {
                        let entry = handle
                            .as_mut()
                            .expect("open directory")
                            .next()
                            .transpose()
                            .map_err(NativeFileFault::Io)?;
                        let entry = entry
                            .map(|entry| {
                                Ok(NativeDirectoryEntry {
                                    name: entry.file_name().to_string_lossy().into_owned(),
                                    path: native_path_normalize(&entry.path().to_string_lossy())
                                        .replace('\\', "/"),
                                    file_type: entry.file_type()?,
                                })
                            })
                            .transpose()
                            .map_err(NativeFileFault::Io)?;
                        Ok(NativeFilePayload::DirectoryEntry(entry))
                    }
                    _ => {
                        *handle = None;
                        Ok(NativeFilePayload::Unit)
                    }
                }
            }
            NativePathOperation::CreateTemporary(_) | NativePathOperation::CleanupTemporary => {
                let Some(NativePathHandle::Temporary(handle)) = resource else {
                    panic!("Rust ABI: temporary resource")
                };
                if let NativePathOperation::CreateTemporary(file) = operation {
                    let resource =
                        native_create_temporary(&path, file).map_err(NativeFileFault::Io)?;
                    let path = resource.path.to_string_lossy().replace('\\', "/");
                    let mut state = handle.lock().unwrap_or_else(|error| error.into_inner());
                    assert!(state.resource.is_none(), "temporary already created");
                    state.resource = Some(resource);
                    Ok(NativeFilePayload::TemporaryPath(path))
                } else {
                    unit(native_finish_temporary(&handle))
                }
            }
        }
    }))
    .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error))));
    NativeFileReply {
        result,
        close: Ok(()),
    }
}
fn native_copy_io_error(error: &std::io::Error) -> std::io::Error {
    error
        .raw_os_error()
        .map(std::io::Error::from_raw_os_error)
        .unwrap_or_else(|| std::io::Error::new(error.kind(), error.to_string()))
}
fn native_path_random() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut random = std::collections::hash_map::RandomState::new().build_hasher();
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    random.write_u64(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    random.finish()
}
fn native_unique_suffix() -> String {
    let mut value = native_path_random();
    (0..6)
        .map(|_| {
            let byte = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                [(value % 62) as usize];
            value /= 62;
            byte as char
        })
        .collect()
}
fn native_atomic_name() -> String {
    let high = native_path_random();
    let low = native_path_random();
    format!(
        ".seseragi-atomic-{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        high >> 32,
        (high >> 16) & 0xffff,
        (high & 0xfff) | 0x4000,
        ((low >> 48) & 0x3fff) | 0x8000,
        low & 0xffffffffffff,
    )
}
struct NativeOwnedPath {
    path: std::path::PathBuf,
    directory: bool,
    armed: bool,
}
impl Drop for NativeOwnedPath {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.directory {
            let _ = std::fs::remove_dir_all(&self.path);
        } else {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
fn native_atomic_write(
    host: &mut impl NativeFileHost,
    path: &str,
    bytes: &[u8],
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<NativeFilePayload, NativeFileFault> {
    let parent = std::path::Path::new(path)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let (temporary, file) = loop {
        native_file_checkpoint(cancelled)?;
        let temporary = parent.join(native_atomic_name());
        match host.open(
            temporary.to_str().expect("UTF-8 temporary path"),
            Some("CreateNew"),
        ) {
            Ok(file) => break (temporary, file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(NativeFileFault::Io(error)),
        }
    };
    let mut cleanup = NativeOwnedPath {
        path: temporary,
        directory: false,
        armed: true,
    };
    let mut guard = NativeFileGuard {
        host,
        handle: Some(file),
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        native_run_file_action(
            guard.host,
            &mut guard.handle,
            NativeFileAction::Write,
            path,
            None,
            bytes,
            0,
            cancelled,
        )?;
        native_run_file_action(
            guard.host,
            &mut guard.handle,
            NativeFileAction::Flush,
            path,
            None,
            &[],
            0,
            cancelled,
        )
    }))
    .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error))));
    let close = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        guard
            .host
            .close(guard.handle.take().expect("owned atomic file"))
            .map_err(NativeFileFault::Io)
    }))
    .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error))));
    result?;
    close?;
    native_file_checkpoint(cancelled)?;
    std::fs::rename(&cleanup.path, path).map_err(NativeFileFault::Io)?;
    cleanup.armed = false;
    Ok(NativeFilePayload::Unit)
}
fn native_temporary_base() -> std::path::PathBuf {
    #[cfg(windows)]
    let keys = ["TEMP", "TMP"].as_slice();
    #[cfg(not(windows))]
    let keys = ["TMPDIR", "TMP", "TEMP"].as_slice();
    if let Some(path) = keys
        .iter()
        .find_map(|key| std::env::var_os(key).filter(|value| !value.is_empty()))
    {
        return path.into();
    }
    #[cfg(windows)]
    {
        ["SystemRoot", "windir"]
            .iter()
            .find_map(|key| std::env::var_os(key).filter(|value| !value.is_empty()))
            .map_or_else(
                || std::path::PathBuf::from("undefined"),
                std::path::PathBuf::from,
            )
            .join("temp")
    }
    #[cfg(not(windows))]
    std::path::PathBuf::from("/tmp")
}
fn native_create_temporary(prefix: &str, file: bool) -> std::io::Result<NativeTemporary> {
    let root = loop {
        let root = native_temporary_base().join(format!("{prefix}{}", native_unique_suffix()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&root) {
            Ok(()) => break root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    let mut cleanup = NativeOwnedPath {
        path: root.clone(),
        directory: true,
        armed: true,
    };
    let path = if file {
        root.join(prefix)
    } else {
        root.clone()
    };
    if file {
        native_close_file(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)?,
        )?;
    }
    let identity = native_path_identity(&path)?;
    cleanup.armed = false;
    Ok(NativeTemporary {
        path,
        root,
        identity,
    })
}
#[cfg(unix)]
fn native_path_identity(path: &std::path::Path) -> std::io::Result<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = std::fs::symlink_metadata(path)?;
    Ok((metadata.dev(), metadata.ino()))
}
#[cfg(windows)]
fn native_path_identity(path: &std::path::Path) -> std::io::Result<(u64, u64)> {
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(handle: *mut std::ffi::c_void, information: *mut u32) -> i32;
    }
    // Inspect the resource itself, including directory and reparse-point identities.
    let handle = std::fs::OpenOptions::new()
        .access_mode(0)
        .custom_flags(0x02000000 | 0x00200000)
        .open(path)?;
    let mut information = [0u32; 13];
    if unsafe { GetFileInformationByHandle(handle.as_raw_handle(), information.as_mut_ptr()) } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    Ok((
        information[7] as u64,
        ((information[11] as u64) << 32) | information[12] as u64,
    ))
}
#[cfg(not(any(unix, windows)))]
fn native_path_identity(_: &std::path::Path) -> std::io::Result<(u64, u64)> {
    Err(std::io::ErrorKind::Unsupported.into())
}
fn native_cleanup_temporary(resource: &NativeTemporary) -> std::io::Result<()> {
    match native_path_identity(&resource.path) {
        Ok(identity) if identity != resource.identity => {
            return Err(std::io::ErrorKind::PermissionDenied.into())
        }
        Ok(_) => (),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
        Err(error) => return Err(error),
    }
    match std::fs::remove_dir_all(&resource.root) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
fn native_finish_temporary(handle: &NativeTemporaryHandle) -> std::io::Result<()> {
    let mut state = handle.lock().unwrap_or_else(|error| error.into_inner());
    if state.cleanup.is_none() {
        let resource = state.resource.clone();
        drop(state);
        let cleaned = resource.as_ref().map_or(Ok(()), native_cleanup_temporary);
        state = handle.lock().unwrap_or_else(|error| error.into_inner());
        state.cleanup = Some(cleaned);
    }
    match state.cleanup.as_ref().unwrap() {
        Ok(()) => Ok(()),
        Err(error) => Err(native_copy_io_error(error)),
    }
}
fn native_temporary_shutdown(context: &NativeContext) -> NativeOutcome {
    for handle in context.temporaries.borrow().iter().rev() {
        native_finish_temporary(handle).map_err(|error| {
            NativeExit::defects(vec![format!(
                "filesystem temporary cleanup failed: {error}"
            )])
        })?;
    }
    Ok(V::Unit)
}
fn native_directory_entry(entry: NativeDirectoryEntry) -> V {
    let file_type = if entry.file_type.is_file() {
        "std/fs::RegularFile"
    } else if entry.file_type.is_dir() {
        "std/fs::Directory"
    } else if entry.file_type.is_symlink() {
        "std/fs::SymbolicLink"
    } else {
        "std/fs::OtherFileType"
    };
    V::Record(Shared::new(BTreeMap::from([
        ("name".into(), V::text(&entry.name)),
        ("path".into(), native_path(&entry.path)),
        (
            "fileType".into(),
            native_maybe(Some(V::adt(file_type, V::Unit))),
        ),
    ])))
}
fn native_directory_list(path: V) -> V {
    V::fun(vec![path], |captures, _| {
        let allocate = V::leaf(vec![], |_, _| {
            Ok(V::PathResource(Shared::new(NativePathResource(
                NativePathHandle::Directory(Default::default()),
            ))))
        });
        let acquire = V::effect(
            Instruction::AcquireRelease,
            vec![
                allocate,
                V::fun(captures.to_vec(), |captures, resource| {
                    native_file_cleanup(native_path_effect(
                        NativePathOperation::CloseDirectory,
                        vec![resource, captures[0].clone()],
                    ))
                }),
            ],
        );
        V::bind(
            acquire,
            V::fun(captures.to_vec(), |captures, resource| {
                let path = captures[0].clone();
                V::bind(
                    native_path_effect(
                        NativePathOperation::OpenDirectory,
                        vec![resource.clone(), path.clone()],
                    ),
                    V::fun(vec![resource, path], |captures, _| {
                        let close = V::fun(
                            vec![
                                native_path_effect(
                                    NativePathOperation::CloseDirectory,
                                    captures.to_vec(),
                                ),
                                stream_cell(V::Unit),
                            ],
                            |captures, _| {
                                native_file_stream_close(captures[0].clone(), captures[1].clone())
                            },
                        );
                        let pull = V::fun(
                            vec![captures[0].clone(), captures[1].clone(), close.clone()],
                            |captures, _| {
                                V::bind(
                                    native_path_effect(
                                        NativePathOperation::ReadDirectory,
                                        captures[..2].to_vec(),
                                    ),
                                    V::fun(vec![captures[2].clone()], |captures, step| {
                                        if step.tagged("std/prelude::Nothing") {
                                            V::bind(
                                                captures[0].apply(V::Unit),
                                                V::fun(vec![step], |captures, _| {
                                                    stream_pure(captures[0].clone())
                                                }),
                                            )
                                        } else {
                                            stream_pure(step)
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
            }),
        )
    })
}
fn native_with_temporary(prefix: V, use_path: V, file: bool) -> V {
    native_stream_defer(vec![prefix, use_path, V::Bool(file)], |captures, _| {
        let prefix = native_text(&captures[0]);
        if prefix.is_empty() || matches!(prefix, "." | "..") || prefix.contains(['/', '\\', '\0']) {
            return V::leaf(vec![], |_, _| {
                Err(NativeExit::failure(native_left(native_file_error(
                    "std/fs::CreateTemporary",
                    native_path("."),
                    std::io::ErrorKind::InvalidInput.into(),
                ))))
            });
        }
        let cleaned = stream_cell(V::Unit);
        let allocate = V::leaf(vec![], |_, context| {
            let handle = NativeTemporaryHandle::default();
            context.temporaries.borrow_mut().push(handle.clone());
            Ok(V::PathResource(Shared::new(NativePathResource(
                NativePathHandle::Temporary(handle),
            ))))
        });
        let acquire = V::effect(
            Instruction::AcquireRelease,
            vec![
                allocate,
                V::fun(vec![cleaned.clone()], |captures, resource| {
                    let V::PathResource(handle) = &resource else {
                        unreachable!()
                    };
                    let NativePathHandle::Temporary(handle) = &handle.get().0 else {
                        unreachable!()
                    };
                    let path = handle
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .resource
                        .as_ref()
                        .map_or_else(
                            || native_path("."),
                            |resource| {
                                native_path(&resource.path.to_string_lossy().replace('\\', "/"))
                            },
                        );
                    V::effect(
                        Instruction::Recover,
                        vec![
                            V::fun(captures.to_vec(), |captures, error| {
                                captures[0].write_reference(native_left(error));
                                stream_pure(V::Unit)
                            }),
                            native_path_effect(
                                NativePathOperation::CleanupTemporary,
                                vec![resource, path],
                            ),
                        ],
                    )
                }),
            ],
        );
        let body = V::bind(
            acquire,
            V::fun(captures.to_vec(), |captures, resource| {
                let created = V::effect(
                    Instruction::MapError,
                    vec![
                        V::fun(vec![], |_, error| native_left(error)),
                        native_path_effect(
                            NativePathOperation::CreateTemporary(captures[2].boolean()),
                            vec![resource, captures[0].clone()],
                        ),
                    ],
                );
                V::bind(
                    created,
                    V::fun(vec![captures[1].clone()], |captures, path| {
                        V::effect(
                            Instruction::MapError,
                            vec![
                                V::fun(vec![], |_, error| native_right(error)),
                                native_stream_defer(
                                    vec![captures[0].clone(), path],
                                    |captures, _| captures[0].apply(captures[1].clone()),
                                ),
                            ],
                        )
                    }),
                )
            }),
        );
        V::bind(
            V::effect(Instruction::Scoped, vec![body]),
            V::fun(vec![cleaned], |captures, value| {
                let cleaned = captures[0].read_reference();
                if cleaned.tagged("std/prelude::Left") {
                    V::leaf(vec![cleaned], |captures, _| {
                        Err(NativeExit::failure(captures[0].clone()))
                    })
                } else {
                    stream_pure(value)
                }
            }),
        )
    })
}

#[cfg(test)]
mod filesystem_path_tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn windows_path_errors_keep_node_error_kinds_and_move_context() {
        for (code, kind) in [
            (206, "std/fs::PathNotSupported"),
            (1921, "std/fs::SymbolicLinkLoop"),
        ] {
            let error = NativePathOperation::Move.failure(
                &[native_path("destination"), native_path("source")],
                std::io::Error::from_raw_os_error(code),
            );
            assert!(error.field("operation").tagged("std/fs::MovePath"));
            assert_eq!(native_path_text(&error.field("path")), "source");
            assert_eq!(
                native_path_text(&error.field("otherPath").payload()),
                "destination"
            );
            assert!(error.field("kind").tagged(kind));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    struct Scratch(std::path::PathBuf);
    impl Scratch {
        fn new() -> Self {
            let resource = native_create_temporary("seseragi-path-test-", false).unwrap();
            Self(resource.root)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn success(plan: V) -> V {
        NativeContext::default()
            .root(plan)
            .unwrap_or_else(|_| panic!("unexpected failure"))
    }
    struct AtomicHost {
        fail: &'static str,
        closes: usize,
        writes: usize,
        cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }
    impl NativeFileHost for AtomicHost {
        type Handle = std::fs::File;
        fn open(&mut self, path: &str, mode: Option<&str>) -> std::io::Result<Self::Handle> {
            NativeStdFileHost.open(path, mode)
        }
        fn read(&mut self, _: &mut Self::Handle, _: &mut [u8]) -> std::io::Result<usize> {
            unreachable!()
        }
        fn write(&mut self, handle: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<usize> {
            self.writes += 1;
            match self.fail {
                "write" => Err(std::io::ErrorKind::PermissionDenied.into()),
                "zero" => Ok(0),
                "defect" => panic!("atomic write defect"),
                "interrupt" if self.writes == 1 => Err(std::io::ErrorKind::Interrupted.into()),
                _ => NativeStdFileHost.write(handle, &bytes[..bytes.len().min(3)]),
            }
        }
        fn flush(&mut self, handle: &mut Self::Handle) -> std::io::Result<()> {
            if self.fail == "cancel" {
                self.cancelled
                    .store(true, std::sync::atomic::Ordering::Release);
            }
            if self.fail == "flush" {
                Err(std::io::ErrorKind::PermissionDenied.into())
            } else {
                NativeStdFileHost.flush(handle)
            }
        }
        fn close(&mut self, handle: Self::Handle) -> std::io::Result<()> {
            self.closes += 1;
            NativeStdFileHost.close(handle)?;
            if self.fail == "close" {
                Err(std::io::ErrorKind::PermissionDenied.into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn atomic_write_preserves_existing_content_and_removes_staging_after_each_failure() {
        let scratch = Scratch::new();
        let path = scratch.0.join("target");
        let bytes: Vec<_> = (0..131073).map(|i| i as u8).collect();
        for fail in [
            "write",
            "zero",
            "defect",
            "flush",
            "close",
            "cancel",
            "interrupt",
            "",
        ] {
            std::fs::write(&path, b"original").unwrap();
            let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let mut host = AtomicHost {
                fail,
                closes: 0,
                writes: 0,
                cancelled: cancelled.clone(),
            };
            let result = native_atomic_write(&mut host, path.to_str().unwrap(), &bytes, &cancelled);
            assert_eq!(result.is_ok(), matches!(fail, "" | "interrupt"), "{fail}");
            assert_eq!(host.closes, 1, "{fail}");
            assert_eq!(
                std::fs::read(&path).unwrap(),
                if result.is_ok() {
                    &bytes[..]
                } else {
                    b"original"
                }
            );
            assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 1, "{fail}");
            if fail == "zero" {
                let Err(NativeFileFault::Io(error)) = result else {
                    panic!("expected write failure")
                };
                assert_eq!(native_file_error_code(&error), "FILESYSTEM_ERROR");
            }
        }
        let mut host = AtomicHost {
            fail: "",
            closes: 0,
            writes: 0,
            cancelled: Default::default(),
        };
        let result = native_atomic_write(
            &mut host,
            path.to_str().unwrap(),
            &[],
            &std::sync::atomic::AtomicBool::new(true),
        );
        assert!(matches!(result, Err(NativeFileFault::Cancelled)));
        assert_eq!(host.closes, 0);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 1);
    }

    #[test]
    fn temporary_scope_cleans_after_success_failure_defect_and_cancellation_and_is_cold() {
        for file in [false, true] {
            for ending in 0..4 {
                let observed = stream_cell(V::Unit);
                let callback = V::fun(vec![observed.clone(), V::Int(ending)], |captures, path| {
                    captures[0].write_reference(path.clone());
                    V::leaf(vec![path, captures[1].clone()], |captures, context| {
                        let path = std::path::Path::new(native_path_text(&captures[0]));
                        assert!(path.exists());
                        if path.is_dir() {
                            std::fs::create_dir(path.join("child")).unwrap();
                            std::fs::write(path.join("child/file"), b"nested").unwrap();
                        }
                        match captures[1] {
                            V::Int(0) => Ok(V::Int(42)),
                            V::Int(1) => {
                                Err(NativeExit::failure(native_left(V::text("user failure"))))
                            }
                            V::Int(2) => panic!("temporary callback defect"),
                            _ => {
                                context.cancelled = true;
                                Err(NativeExit::Cancelled)
                            }
                        }
                    })
                });
                let action = native_with_temporary(V::text("seseragi-scope-test-"), callback, file);
                assert!(matches!(observed.read_reference(), V::Unit));
                let mut paths = Vec::new();
                for _ in 0..2 {
                    let result = NativeContext::default().root(action.clone());
                    match ending {
                        0 => assert!(matches!(result, Ok(V::Int(42)))),
                        1 => assert!(
                            matches!(result, Err(NativeExit::Failure(ref error)) if error.tagged("std/prelude::Right") && error.payload().tagged("std/prelude::Left") && native_text(&error.payload().payload()) == "user failure")
                        ),
                        2 => assert!(
                            matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["temporary callback defect"])
                        ),
                        _ => assert!(matches!(result, Err(NativeExit::Cancelled))),
                    }
                    let path =
                        std::path::PathBuf::from(native_path_text(&observed.read_reference()));
                    assert!(!path.exists());
                    if file {
                        assert!(!path.parent().unwrap().exists());
                    }
                    paths.push(path);
                }
                assert_ne!(paths[0], paths[1]);
                drop((observed, action));
                assert_eq!(LIVE.with(Cell::get), 0);
            }
        }
    }

    #[test]
    fn temporary_cleanup_rejects_replaced_identity_and_preserves_the_primary_failure() {
        let scratch = Scratch::new();
        for primary in [false, true] {
            let observed = stream_cell(V::Unit);
            let callback = V::fun(
                vec![
                    observed.clone(),
                    V::text(scratch.0.to_str().unwrap()),
                    V::Bool(primary),
                ],
                |captures, path| {
                    captures[0].write_reference(path.clone());
                    V::leaf(
                        vec![path, captures[1].clone(), captures[2].clone()],
                        |captures, _| {
                            let path = std::path::Path::new(native_path_text(&captures[0]));
                            let old = std::path::Path::new(native_text(&captures[1])).join("old");
                            std::fs::rename(path, old).unwrap();
                            std::fs::write(path, b"replacement").unwrap();
                            if captures[2].boolean() {
                                Err(NativeExit::failure(V::text("primary")))
                            } else {
                                Ok(V::Unit)
                            }
                        },
                    )
                },
            );
            let result = NativeContext::default().root(native_with_temporary(
                V::text("seseragi-identity-test-"),
                callback,
                true,
            ));
            let Err(NativeExit::Failure(error)) = result else {
                panic!("expected failure")
            };
            if primary {
                assert!(error.tagged("std/prelude::Right"));
                assert_eq!(native_text(&error.payload()), "primary");
            } else {
                assert!(error.tagged("std/prelude::Left"));
                assert!(error
                    .payload()
                    .field("operation")
                    .tagged("std/fs::RemovePath"));
                assert!(error
                    .payload()
                    .field("kind")
                    .tagged("std/fs::PermissionDenied"));
            }
            let path = std::path::PathBuf::from(native_path_text(&observed.read_reference()));
            assert_eq!(std::fs::read(&path).unwrap(), b"replacement");
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
            std::fs::remove_file(scratch.0.join("old")).unwrap();
            drop((observed, error));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }

    #[test]
    fn invalid_temporary_prefixes_fail_before_the_callback_with_creation_context() {
        for prefix in ["", ".", "..", "a/b", "a\\b", "a\0b"] {
            let action = native_with_temporary(
                V::text(prefix),
                V::fun(vec![], |_, _| panic!("invalid prefix callback")),
                false,
            );
            let Err(NativeExit::Failure(error)) = NativeContext::default().root(action) else {
                panic!("expected failure")
            };
            assert!(error.tagged("std/prelude::Left"));
            assert!(error
                .payload()
                .field("operation")
                .tagged("std/fs::CreateTemporary"));
            assert!(error
                .payload()
                .field("kind")
                .tagged("std/fs::PathNotSupported"));
            assert_eq!(native_path_text(&error.payload().field("path")), ".");
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn directory_stream_reopens_after_early_close_and_releases_the_directory_owner() {
        let scratch = Scratch::new();
        std::fs::write(scratch.0.join("file"), b"contents").unwrap();
        let source = native_directory_list(native_path(scratch.0.to_str().unwrap()));
        let terminal = native_stream_operation(
            "std/stream::runCollect",
            vec![native_stream_operation(
                "std/stream::take",
                vec![V::Int(1), source.clone()],
            )],
        );
        for _ in 0..2 {
            let values = success(terminal.clone());
            assert_eq!(values.index(0).field("name").render(), "file");
            assert!(values
                .index(0)
                .field("fileType")
                .payload()
                .tagged("std/fs::RegularFile"));
        }
        drop((source, terminal));
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn directory_owners_close_after_consumer_failure_defect_and_cancellation() {
        let scratch = Scratch::new();
        std::fs::write(scratch.0.join("file"), b"contents").unwrap();
        for ending in 0..3 {
            let observed = stream_cell(V::Unit);
            let plan = V::bind(
                stream_open(&native_directory_list(native_path(
                    scratch.0.to_str().unwrap(),
                ))),
                V::fun(
                    vec![observed.clone(), V::Int(ending)],
                    |captures, cursor| {
                        let V::Fun(pull) = cursor.field("pull") else {
                            unreachable!()
                        };
                        let V::Fun(read) = &pull.get().captures[0] else {
                            unreachable!()
                        };
                        captures[0].write_reference(read.get().captures[0].clone());
                        V::bind(
                            stream_pull(&cursor),
                            V::fun(vec![captures[1].clone()], |captures, _| {
                                V::leaf(captures.to_vec(), |captures, context| match captures[0] {
                                    V::Int(0) => {
                                        Err(NativeExit::failure(V::text("consumer failed")))
                                    }
                                    V::Int(1) => panic!("consumer defect"),
                                    _ => {
                                        context.cancelled = true;
                                        Err(NativeExit::Cancelled)
                                    }
                                })
                            }),
                        )
                    },
                ),
            );
            let result = NativeContext::default().root(plan);
            match ending {
                0 => assert!(
                    matches!(result, Err(NativeExit::Failure(ref error)) if native_text(error) == "consumer failed")
                ),
                1 => assert!(
                    matches!(result, Err(NativeExit::Defect(ref errors)) if errors == &["consumer defect"])
                ),
                _ => assert!(matches!(result, Err(NativeExit::Cancelled))),
            }
            let resource = observed.read_reference();
            let V::PathResource(handle) = &resource else {
                panic!("missing directory resource")
            };
            let NativePathHandle::Directory(handle) = &handle.get().0 else {
                unreachable!()
            };
            assert!(handle.lock().unwrap().is_none());
            drop((resource, observed, result));
            assert_eq!(LIVE.with(Cell::get), 0);
        }
    }
}
