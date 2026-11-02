use std::io::{Read, Write};

// The high-level small-file operations own each handle until close completes.
// A host seam makes read/write/flush/close failure and unwind cleanup observable
// without exposing native file descriptors as language values.
trait NativeFileHost {
    type Handle;
    fn open(&mut self, path: &str, mode: Option<&str>) -> std::io::Result<Self::Handle>;
    fn read(&mut self, handle: &mut Self::Handle, bytes: &mut [u8]) -> std::io::Result<usize>;
    fn write(&mut self, handle: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<usize>;
    fn flush(&mut self, handle: &mut Self::Handle) -> std::io::Result<()>;
    fn close(&mut self, handle: Self::Handle) -> std::io::Result<()>;
}
struct NativeStdFileHost;
impl NativeFileHost for NativeStdFileHost {
    type Handle = std::fs::File;
    fn open(&mut self, path: &str, mode: Option<&str>) -> std::io::Result<Self::Handle> {
        let host_path = native_host_path(path);
        let mut options = std::fs::OpenOptions::new();
        match mode {
            None => {
                options.read(true);
            }
            Some("Replace") => {
                options.write(true).create(true).truncate(true);
            }
            Some("CreateNew") => {
                options.write(true).create_new(true);
            }
            Some("Append") => {
                options.append(true).create(true);
            }
            _ => panic!("Rust ABI: invalid WriteMode"),
        }
        let result = options.open(&host_path);
        // Windows reports ERROR_ACCESS_DENIED for opening a directory as a
        // file. Match the standard Node/Bun provider's IsADirectory result;
        // ordinary permission errors retain their primary classification.
        #[cfg(windows)]
        if let Err(error) = &result {
            if error.raw_os_error() == Some(5)
                && std::fs::metadata(&host_path).is_ok_and(|metadata| metadata.is_dir())
            {
                return Err(std::io::ErrorKind::IsADirectory.into());
            }
        }
        result
    }
    fn read(&mut self, handle: &mut Self::Handle, bytes: &mut [u8]) -> std::io::Result<usize> {
        handle.read(bytes)
    }
    fn write(&mut self, handle: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<usize> {
        handle.write(bytes)
    }
    fn flush(&mut self, handle: &mut Self::Handle) -> std::io::Result<()> {
        handle.sync_all()
    }
    fn close(&mut self, handle: Self::Handle) -> std::io::Result<()> {
        native_close_file(handle)
    }
}
fn native_host_path(path: &str) -> String {
    // Match std/fs's public wrapper before reaching the provider. This is
    // lexical normalization, not symlink resolution or a security boundary.
    native_provider_path(&native_path_normalize(path))
}
fn native_provider_path(path: &str) -> String {
    #[cfg(windows)]
    {
        path.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path.into()
    }
}

fn native_inspect_file(
    operation: NativeFileOperation,
    path: &str,
) -> std::io::Result<NativeFilePayload> {
    match operation {
        NativeFileOperation::Exists => match std::fs::metadata(native_provider_path(path)) {
            Ok(_) => Ok(NativeFilePayload::Exists(true)),
            Err(error) if native_file_error_code(&error) == "ENOENT" => {
                Ok(NativeFilePayload::Exists(false))
            }
            Err(error) => Err(error),
        },
        NativeFileOperation::Metadata => {
            std::fs::metadata(native_host_path(path)).map(NativeFilePayload::Metadata)
        }
        NativeFileOperation::SymlinkMetadata => {
            std::fs::symlink_metadata(native_host_path(path)).map(NativeFilePayload::Metadata)
        }
        NativeFileOperation::Canonicalize => {
            let path = native_provider_path(path);
            // macOS realpath can discard a trailing slash or file/.. before
            // checking directory traversal. Match the default TS host (Bun).
            std::fs::metadata(&path)?;
            let path = std::fs::canonicalize(path)?;
            let path = path.to_string_lossy();
            #[cfg(windows)]
            let path = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
                format!("//{unc}")
            } else {
                path.strip_prefix(r"\\?\").unwrap_or(&path).into()
            };
            Ok(NativeFilePayload::CanonicalPath(path.replace('\\', "/")))
        }
        _ => panic!("Rust ABI: invalid filesystem inspection"),
    }
}

fn native_file_instant(time: std::io::Result<std::time::SystemTime>) -> V {
    native_maybe(time.ok().map(|time| {
        let nanos = match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos() as i128,
            Err(error) => -(error.duration().as_nanos() as i128),
        };
        // The TS provider uses numeric stat milliseconds, then truncates their
        // product with one million. Preserve that rounding at both conversions.
        let milliseconds = nanos.div_euclid(1_000_000_000) as f64 * 1_000.0
            + nanos.rem_euclid(1_000_000_000) as f64 / 1_000_000.0;
        let rounded = (milliseconds * 1_000_000.0).trunc() as i128;
        // The TS provider accepts only unsigned timestamp strings.
        assert!(rounded >= 0, "filesystem metadata is invalid");
        V::adt(
            "std/time::Instant",
            native_big_int_parse(&rounded.to_string()).payload(),
        )
    }))
}
fn native_file_metadata(metadata: std::fs::Metadata) -> V {
    let file_type = metadata.file_type();
    let file_type = if file_type.is_file() {
        "std/fs::RegularFile"
    } else if file_type.is_dir() {
        "std/fs::Directory"
    } else if file_type.is_symlink() {
        "std/fs::SymbolicLink"
    } else {
        "std/fs::OtherFileType"
    };
    let created = metadata.created();
    #[cfg(unix)]
    let created = created.or_else(|_| {
        use std::os::unix::fs::MetadataExt;
        let nanos = metadata.ctime() as i128 * 1_000_000_000 + metadata.ctime_nsec() as i128;
        let magnitude = nanos.unsigned_abs();
        let duration = std::time::Duration::new(
            (magnitude / 1_000_000_000) as u64,
            (magnitude % 1_000_000_000) as u32,
        );
        Ok(if nanos < 0 {
            std::time::UNIX_EPOCH - duration
        } else {
            std::time::UNIX_EPOCH + duration
        })
    });
    V::Record(Shared::new(BTreeMap::from([
        ("fileType".into(), V::adt(file_type, V::Unit)),
        (
            "sizeBytes".into(),
            checked_int(i64::try_from(metadata.len()).expect("file size fits Int")),
        ),
        ("modified".into(), native_file_instant(metadata.modified())),
        ("created".into(), native_file_instant(created)),
    ])))
}
#[cfg(unix)]
fn native_close_file(file: std::fs::File) -> std::io::Result<()> {
    use std::os::fd::IntoRawFd;
    unsafe extern "C" {
        fn close(fd: i32) -> i32;
    }
    // Transfer the sole owner before closing. Never retry close after an error:
    // the OS may already have released and reused the descriptor.
    let descriptor = file.into_raw_fd();
    if unsafe { close(descriptor) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}
#[cfg(windows)]
fn native_close_file(file: std::fs::File) -> std::io::Result<()> {
    use std::os::windows::io::IntoRawHandle;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    let handle = file.into_raw_handle();
    if unsafe { CloseHandle(handle) } != 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}
#[cfg(not(any(unix, windows)))]
fn native_close_file(file: std::fs::File) -> std::io::Result<()> {
    drop(file);
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "explicit file close is unavailable",
    ))
}
struct NativeFileGuard<'a, H: NativeFileHost> {
    host: &'a mut H,
    handle: Option<H::Handle>,
}
impl<H: NativeFileHost> Drop for NativeFileGuard<'_, H> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.host.close(handle);
        }
    }
}
fn native_file_checkpoint(
    cancelled: &std::sync::atomic::AtomicBool,
) -> Result<(), NativeFileFault> {
    if cancelled.load(std::sync::atomic::Ordering::Acquire) {
        Err(NativeFileFault::Cancelled)
    } else {
        Ok(())
    }
}
fn native_run_file_request(
    host: &mut impl NativeFileHost,
    request: NativeFileRequest,
    cancelled: &std::sync::atomic::AtomicBool,
) -> NativeFileReply {
    if matches!(request, NativeFileRequest::Path { .. }) {
        return native_run_path_request(request, cancelled);
    }
    let mut guard = NativeFileGuard { host, handle: None };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        native_file_checkpoint(cancelled)?;
        if let NativeFileRequest::Inspect { operation, path } = &request {
            return native_inspect_file(*operation, path).map_err(NativeFileFault::Io);
        }
        let (path, mode) = match &request {
            NativeFileRequest::Read { path } => (path, None),
            NativeFileRequest::Write { path, mode, .. } => (path, Some(*mode)),
            NativeFileRequest::Inspect { .. }
            | NativeFileRequest::Session { .. }
            | NativeFileRequest::Path { .. } => unreachable!(),
        };
        guard.handle = Some(guard.host.open(path, mode).map_err(NativeFileFault::Io)?);
        native_file_checkpoint(cancelled)?;
        match request {
            NativeFileRequest::Read { .. } => {
                let mut bytes = Vec::new();
                let mut buffer = vec![0; 64 * 1024];
                loop {
                    native_file_checkpoint(cancelled)?;
                    let size = match guard.host.read(guard.handle.as_mut().unwrap(), &mut buffer) {
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        result => result.map_err(NativeFileFault::Io)?,
                    };
                    if size > buffer.len() {
                        return Err(NativeFileFault::Defect(
                            "file host returned an invalid read length".into(),
                        ));
                    }
                    if size == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buffer[..size]);
                }
                Ok(NativeFilePayload::Bytes(bytes))
            }
            NativeFileRequest::Write { bytes, .. } => {
                for chunk in bytes.chunks(64 * 1024) {
                    let mut offset = 0;
                    while offset < chunk.len() {
                        native_file_checkpoint(cancelled)?;
                        let size = match guard
                            .host
                            .write(guard.handle.as_mut().unwrap(), &chunk[offset..])
                        {
                            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                                continue
                            }
                            result => result.map_err(NativeFileFault::Io)?,
                        };
                        if size == 0 {
                            return Err(NativeFileFault::Io(std::io::ErrorKind::WriteZero.into()));
                        }
                        if size > chunk.len() - offset {
                            return Err(NativeFileFault::Defect(
                                "file host returned an invalid write length".into(),
                            ));
                        }
                        offset += size;
                    }
                }
                native_file_checkpoint(cancelled)?;
                guard
                    .host
                    .flush(guard.handle.as_mut().unwrap())
                    .map_err(NativeFileFault::Io)?;
                Ok(NativeFilePayload::Unit)
            }
            NativeFileRequest::Inspect { .. }
            | NativeFileRequest::Session { .. }
            | NativeFileRequest::Path { .. } => unreachable!(),
        }
    }))
    .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error))));
    let close = match guard.handle.take() {
        Some(handle) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            guard.host.close(handle).map_err(NativeFileFault::Io)
        }))
        .unwrap_or_else(|error| Err(NativeFileFault::Defect(native_panic_message(error)))),
        None => Ok(()),
    };
    NativeFileReply { result, close }
}
#[cfg(test)]
fn native_test_file_result(reply: NativeFileReply) -> std::io::Result<NativeFilePayload> {
    match reply.result.and_then(|value| reply.close.map(|()| value)) {
        Ok(value) => Ok(value),
        Err(NativeFileFault::Io(error)) => Err(error),
        Err(NativeFileFault::Defect(error)) => panic!("{error}"),
        Err(NativeFileFault::Cancelled) => {
            panic!("unexpected cancellation in synchronous host test")
        }
    }
}
#[cfg(test)]
fn native_read_file(host: &mut impl NativeFileHost, path: &str) -> std::io::Result<Vec<u8>> {
    let reply = native_run_file_request(
        host,
        NativeFileRequest::Read { path: path.into() },
        &std::sync::atomic::AtomicBool::new(false),
    );
    let NativeFilePayload::Bytes(bytes) = native_test_file_result(reply)? else {
        panic!("read payload")
    };
    Ok(bytes)
}
#[cfg(test)]
fn native_write_file(
    host: &mut impl NativeFileHost,
    path: &str,
    mode: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mode = match mode {
        "Replace" => "Replace",
        "CreateNew" => "CreateNew",
        "Append" => "Append",
        _ => panic!("test mode"),
    };
    let reply = native_run_file_request(
        host,
        NativeFileRequest::Write {
            path: path.into(),
            mode,
            bytes: bytes.to_vec(),
        },
        &std::sync::atomic::AtomicBool::new(false),
    );
    native_test_file_result(reply).map(|_| ())
}
fn native_file_error_code(error: &std::io::Error) -> &'static str {
    if error
        .get_ref()
        .is_some_and(|error| error.is::<NativeFileProviderError>())
    {
        return "FILESYSTEM_ERROR";
    }
    #[cfg(windows)]
    match error.raw_os_error() {
        Some(206) => return "ENAMETOOLONG",
        Some(1921) => return "ELOOP",
        _ => (),
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Some(code) = error.raw_os_error() {
        let common = match code {
            1 => Some("EPERM"),
            2 => Some("ENOENT"),
            4 => Some("EINTR"),
            5 => Some("EIO"),
            6 => Some("ENXIO"),
            9 => Some("EBADF"),
            13 => Some("EACCES"),
            17 => Some("EEXIST"),
            18 => Some("EXDEV"),
            19 => Some("ENODEV"),
            20 => Some("ENOTDIR"),
            21 => Some("EISDIR"),
            22 => Some("EINVAL"),
            23 => Some("ENFILE"),
            24 => Some("EMFILE"),
            27 => Some("EFBIG"),
            28 => Some("ENOSPC"),
            30 => Some("EROFS"),
            32 => Some("EPIPE"),
            _ => None,
        };
        if let Some(code) = common {
            return code;
        }
        #[cfg(target_os = "linux")]
        match code {
            36 => return "ENAMETOOLONG",
            38 => return "ENOSYS",
            39 => return "ENOTEMPTY",
            40 => return "ELOOP",
            95 => return "ENOTSUP",
            122 => return "EDQUOT",
            _ => (),
        }
        #[cfg(target_os = "macos")]
        match code {
            45 => return "ENOTSUP",
            62 => return "ELOOP",
            63 => return "ENAMETOOLONG",
            66 => return "ENOTEMPTY",
            69 => return "EDQUOT",
            78 => return "ENOSYS",
            _ => (),
        }
    }
    use std::io::ErrorKind;
    match error.kind() {
        ErrorKind::NotFound => "ENOENT",
        ErrorKind::AlreadyExists => "EEXIST",
        ErrorKind::PermissionDenied => "EACCES",
        ErrorKind::NotADirectory => "ENOTDIR",
        ErrorKind::IsADirectory => "EISDIR",
        ErrorKind::DirectoryNotEmpty => "ENOTEMPTY",
        ErrorKind::CrossesDevices => "EXDEV",
        ErrorKind::InvalidInput => "EINVAL",
        ErrorKind::Unsupported => "ENOSYS",
        _ => "UNKNOWN",
    }
}
fn native_file_error(operation: &'static str, path: V, error: std::io::Error) -> V {
    native_file_error_with_other(operation, path, None, error)
}
fn native_file_error_with_other(
    operation: &'static str,
    path: V,
    other: Option<V>,
    error: std::io::Error,
) -> V {
    let code = native_file_error_code(&error);
    let kind = match code {
        "ENOENT" => "std/fs::FileNotFound",
        "EEXIST" => "std/fs::FileAlreadyExists",
        "EPERM" | "EACCES" => "std/fs::PermissionDenied",
        "ENOTDIR" => "std/fs::NotADirectory",
        "EISDIR" => "std/fs::IsADirectory",
        "ENOTEMPTY" => "std/fs::DirectoryNotEmpty",
        "ELOOP" => "std/fs::SymbolicLinkLoop",
        "EXDEV" => "std/fs::CrossDeviceMove",
        "EINVAL" | "ENAMETOOLONG" => "std/fs::PathNotSupported",
        "ENOSYS" | "ENODEV" => "std/fs::FileSystemUnavailable",
        _ => "std/fs::OtherFileSystemError",
    };
    let kind = V::adt(
        kind,
        if kind == "std/fs::OtherFileSystemError" {
            V::text(code)
        } else {
            V::Unit
        },
    );
    V::Record(Shared::new(BTreeMap::from([
        ("operation".into(), V::adt(operation, V::Unit)),
        ("path".into(), path),
        ("otherPath".into(), native_maybe(other)),
        ("kind".into(), kind),
    ])))
}

#[cfg(test)]
fn native_read_text(host: &mut impl NativeFileHost, path: &V) -> NativeOutcome {
    let bytes = native_read_file(host, native_path_text(path)).map_err(|error| {
        NativeExit::failure(V::adt(
            "std/fs::FileAccessFailure",
            native_file_error("std/fs::ReadFile", path.clone(), error),
        ))
    })?;
    native_decode_text(bytes)
}
fn native_decode_text(bytes: Vec<u8>) -> NativeOutcome {
    match String::from_utf8(bytes) {
        Ok(text) => Ok(V::text(&text)),
        Err(error) => Err(NativeExit::failure(V::adt(
            "std/fs::FileUtf8Failure",
            V::adt(
                "std/text::InvalidUtf8",
                V::Record(Shared::new(BTreeMap::from([(
                    "offset".into(),
                    V::Int(error.utf8_error().valid_up_to() as i64),
                )]))),
            ),
        ))),
    }
}
pub fn native_filesystem_operation(name: &'static str, arguments: Vec<V>) -> V {
    if name.starts_with("std/path::") {
        return native_path_operation(name, &arguments);
    }
    if let Some(operation) = NativePathOperation::named(name) {
        return native_path_effect(operation, arguments);
    }
    match name {
        "std/fs::list" => native_directory_list(arguments[0].clone()),
        "std/fs::withTemporaryDirectory" | "std/fs::withTemporaryFile" => native_with_temporary(
            arguments[0].clone(),
            arguments[1].clone(),
            name == "std/fs::withTemporaryFile",
        ),
        "std/fs::Replace"
        | "std/fs::CreateNew"
        | "std/fs::Append"
        | "std/fs::RegularFile"
        | "std/fs::Directory"
        | "std/fs::SymbolicLink"
        | "std/fs::OtherFileType" => V::adt(name, V::Unit),
        "std/fs::exists" => V::effect(
            Instruction::FileSystem(NativeFileOperation::Exists),
            arguments,
        ),
        "std/fs::metadata" => V::effect(
            Instruction::FileSystem(NativeFileOperation::Metadata),
            arguments,
        ),
        "std/fs::symlinkMetadata" => V::effect(
            Instruction::FileSystem(NativeFileOperation::SymlinkMetadata),
            arguments,
        ),
        "std/fs::canonicalize" => V::effect(
            Instruction::FileSystem(NativeFileOperation::Canonicalize),
            arguments,
        ),
        "std/fs::readTextUtf8" => V::effect(
            Instruction::FileSystem(NativeFileOperation::ReadText),
            arguments,
        ),
        "std/fs::writeTextUtf8" => V::effect(
            Instruction::FileSystem(NativeFileOperation::WriteText),
            arguments,
        ),
        "std/fs::readBytes" => V::effect(
            Instruction::FileSystem(NativeFileOperation::ReadBytes),
            arguments,
        ),
        "std/fs::writeBytes" => V::effect(
            Instruction::FileSystem(NativeFileOperation::WriteBytes),
            arguments,
        ),
        "std/fs::readChunks" => native_file_read_chunks(arguments[0].clone(), arguments[1].clone()),
        "std/fs::writeChunks" => native_file_write_chunks(
            arguments[0].clone(),
            arguments[1].clone(),
            arguments[2].clone(),
        ),
        _ => panic!("Rust ABI: unsupported filesystem operation"),
    }
}

fn native_json_string(value: &str) -> String {
    let mut result = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0c}' => result.push_str("\\f"),
            c if c < ' ' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}
fn native_standard_tag(value: &V, owner: &str) -> String {
    let V::Adt(value) = value else {
        panic!("Rust ABI: expected canonical variant")
    };
    value
        .get()
        .0
        .strip_prefix(owner)
        .expect("Rust ABI: variant owner")
        .into()
}
fn native_filesystem_display(name: &str, value: &V) -> String {
    match name {
        "std/path::Path" => native_json_string(native_path_text(value)),
        "std/path::PathError" => {
            let tag = native_standard_tag(value, "std/path::");
            match tag.as_str() {
                "PathContainsNul" | "PathContainsBackslash" => format!(
                    "{tag} {{ offset: {} }}",
                    value.payload().field("offset").render()
                ),
                "InvalidPathSegment" => format!(
                    "{tag} {}",
                    native_json_string(native_text(&value.payload()))
                ),
                "EmptyPath" | "InvalidDriveRoot" | "InvalidUncRoot" | "AbsoluteChildPath" => tag,
                _ => panic!("Rust ABI: PathError variant"),
            }
        }
        "std/fs::FileSystemError" => {
            let other = value.field("otherPath");
            let other = if other.tagged("std/prelude::Nothing") {
                "Nothing".into()
            } else {
                assert!(other.tagged("std/prelude::Just"));
                format!(
                    "Just {}",
                    native_json_string(native_path_text(&other.payload()))
                )
            };
            format!(
                "FileSystemError {{ operation: {}, path: {}, otherPath: {other}, kind: {} }}",
                native_filesystem_display("std/fs::FileSystemOperation", &value.field("operation")),
                native_json_string(native_path_text(&value.field("path"))),
                native_filesystem_display("std/fs::FileSystemErrorKind", &value.field("kind"))
            )
        }
        "std/fs::FileTextError" => {
            let tag = native_standard_tag(value, "std/fs::");
            let child = match tag.as_str() {
                "FileAccessFailure" => "std/fs::FileSystemError",
                "FileUtf8Failure" => "std/text::Utf8DecodeError",
                _ => panic!("Rust ABI: FileTextError variant"),
            };
            format!(
                "{tag} {}",
                native_filesystem_display(child, &value.payload())
            )
        }
        "std/fs::FileMetadata" => format!(
            "FileMetadata {{ fileType: {}, sizeBytes: {}, modified: {}, created: {} }}",
            native_filesystem_display("std/fs::FileType", &value.field("fileType")),
            value.field("sizeBytes").render(),
            native_standard_tag(&value.field("modified"), "std/prelude::"),
            native_standard_tag(&value.field("created"), "std/prelude::")
        ),
        "std/fs::DirectoryEntry" => {
            let file_type = value.field("fileType");
            let file_type = if file_type.tagged("std/prelude::Nothing") {
                "Nothing".into()
            } else {
                format!(
                    "Just {}",
                    native_filesystem_display("std/fs::FileType", &file_type.payload())
                )
            };
            format!(
                "DirectoryEntry {{ name: {}, path: {}, fileType: {file_type} }}",
                native_json_string(native_text(&value.field("name"))),
                native_json_string(native_path_text(&value.field("path"))),
            )
        }
        "std/text::Utf8DecodeError" => {
            assert!(value.tagged("std/text::InvalidUtf8"));
            format!(
                "InvalidUtf8 {{ offset: {} }}",
                value.payload().field("offset").render()
            )
        }
        "std/fs::FileSystemErrorKind" => {
            let tag = native_standard_tag(value, "std/fs::");
            if tag == "OtherFileSystemError" {
                format!(
                    "{tag} {}",
                    native_json_string(native_text(&value.payload()))
                )
            } else {
                tag
            }
        }
        "std/fs::FileSystemOperation" | "std/fs::WriteMode" | "std/fs::FileType" => {
            native_standard_tag(value, "std/fs::")
        }
        _ => panic!("Rust ABI: filesystem display identity"),
    }
}
pub fn native_filesystem_display_dictionary(name: &'static str, debug: bool) -> V {
    let method = if debug { "debug" } else { "show" };
    V::dict(vec![(
        method,
        V::fun(vec![V::text(name)], |c, value| {
            native_display(|| V::text(&native_filesystem_display(native_text(&c[0]), &value)))
        }),
    )])
}

#[cfg(test)]
mod filesystem_tests {
    use super::*;
    #[derive(Default)]
    struct Host {
        events: Vec<&'static str>,
        fail: Option<&'static str>,
        panic_read: bool,
        live: usize,
        bytes: Option<Vec<u8>>,
        read_finished: bool,
    }
    impl Host {
        fn event(&mut self, name: &'static str) -> std::io::Result<()> {
            self.events.push(name);
            if self.fail == Some(name) {
                Err(std::io::Error::new(std::io::ErrorKind::Other, name))
            } else {
                Ok(())
            }
        }
    }
    impl NativeFileHost for Host {
        type Handle = ();
        fn open(&mut self, _: &str, _: Option<&str>) -> std::io::Result<()> {
            self.event("open")?;
            self.live += 1;
            Ok(())
        }
        fn read(&mut self, _: &mut (), bytes: &mut [u8]) -> std::io::Result<usize> {
            if self.panic_read {
                panic!("read defect")
            }
            self.event("read")?;
            if self.read_finished {
                return Ok(0);
            }
            self.read_finished = true;
            let content = self.bytes.as_deref().unwrap_or(b"hello");
            bytes[..content.len()].copy_from_slice(content);
            Ok(content.len())
        }
        fn write(&mut self, _: &mut (), bytes: &[u8]) -> std::io::Result<usize> {
            self.event("write")?;
            Ok(bytes.len())
        }
        fn flush(&mut self, _: &mut ()) -> std::io::Result<()> {
            self.event("flush")
        }
        fn close(&mut self, _: ()) -> std::io::Result<()> {
            self.live -= 1;
            self.event("close")
        }
    }
    #[test]
    fn file_instants_preserve_stat_rounding_epoch_sign_and_large_nanoseconds() {
        use std::time::{Duration, UNIX_EPOCH};
        for (time, expected) in [
            (UNIX_EPOCH, "0"),
            (UNIX_EPOCH + Duration::from_nanos(1), "1"),
            (UNIX_EPOCH - Duration::from_nanos(1), "0"),
            (
                UNIX_EPOCH + Duration::new(1_793_586_967, 539_172_942),
                "1793586967539172864",
            ),
            (
                UNIX_EPOCH + Duration::new(10_000_000_000, 0),
                "10000000000000000000",
            ),
        ] {
            let maybe = native_file_instant(Ok(time));
            assert!(maybe.tagged("std/prelude::Just"));
            let instant = maybe.payload();
            assert!(instant.tagged("std/time::Instant"));
            assert!(matches!(instant.payload(), V::BigInt(_)));
            assert_eq!(instant.payload().render(), expected);
        }
        assert!(
            native_file_instant(Err(std::io::ErrorKind::Unsupported.into()))
                .tagged("std/prelude::Nothing")
        );
        let invalid = catch_native(|| {
            Ok(native_file_instant(Ok(
                UNIX_EPOCH - Duration::from_nanos(1_000_000_001)
            )))
        });
        assert!(
            matches!(invalid, Err(NativeExit::Defect(ref errors)) if errors == &["filesystem metadata is invalid"])
        );
    }

    #[test]
    fn inspections_do_not_open_file_handles_and_cancellation_discards_the_result() {
        let path = std::env::temp_dir().to_str().unwrap().to_owned();
        for operation in [
            NativeFileOperation::Exists,
            NativeFileOperation::Metadata,
            NativeFileOperation::SymlinkMetadata,
            NativeFileOperation::Canonicalize,
        ] {
            for cancelled in [false, true] {
                let mut host = Host::default();
                let reply = native_run_file_request(
                    &mut host,
                    NativeFileRequest::Inspect {
                        operation,
                        path: path.clone(),
                    },
                    &std::sync::atomic::AtomicBool::new(cancelled),
                );
                assert!(host.events.is_empty());
                assert_eq!(host.live, 0);
                assert!(reply.close.is_ok());
                if cancelled {
                    assert!(matches!(reply.result, Err(NativeFileFault::Cancelled)));
                } else {
                    assert!(reply.result.is_ok());
                    let (outcome, notes) = operation.outcome(&[native_path(&path)], reply, true);
                    assert!(matches!(outcome, Err(NativeExit::Cancelled)));
                    assert!(notes.is_empty());
                }
            }
        }
    }
    #[test]
    fn small_file_ownership_closes_once_after_success_failure_and_unwind() {
        for operation in ["read", "write"] {
            for fail in [
                None,
                Some("open"),
                Some(operation),
                Some("flush"),
                Some("close"),
            ] {
                let mut host = Host {
                    fail,
                    ..Host::default()
                };
                let result = if operation == "read" {
                    native_read_file(&mut host, ".").map(|_| ())
                } else {
                    native_write_file(&mut host, ".", "Replace", b"x")
                };
                assert_eq!(host.live, 0);
                assert_eq!(
                    host.events.iter().filter(|e| **e == "close").count(),
                    usize::from(fail != Some("open"))
                );
                assert_eq!(
                    result.is_err(),
                    fail.is_some() && (operation == "write" || fail != Some("flush"))
                );
                if let Some(failure) = fail.filter(|_| result.is_err()) {
                    assert_eq!(result.unwrap_err().to_string(), failure);
                }
            }
        }
        let mut host = Host {
            panic_read: true,
            ..Host::default()
        };
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| native_read_file(
                &mut host, "."
            )))
            .is_err()
        );
        assert_eq!(host.events, ["open", "close"]);
        assert_eq!(host.live, 0);
    }
    #[test]
    fn decoding_waits_for_close_and_keeps_byte_offsets_and_primary_failure() {
        let path = native_path("input.txt");
        let mut host = Host {
            bytes: Some(vec![0xc3, 0xa9, 0xff]),
            ..Host::default()
        };
        let Err(NativeExit::Failure(error)) = native_read_text(&mut host, &path) else {
            panic!("expected UTF8 failure")
        };
        assert!(error.tagged("std/fs::FileUtf8Failure"));
        assert_eq!(error.payload().payload().field("offset").render(), "2");
        assert_eq!(host.events, ["open", "read", "read", "close"]);
        assert_eq!(host.live, 0);
        let mut host = Host {
            bytes: Some(vec![0xff]),
            fail: Some("close"),
            ..Host::default()
        };
        let Err(NativeExit::Failure(error)) = native_read_text(&mut host, &path) else {
            panic!("expected close failure")
        };
        assert!(error.tagged("std/fs::FileAccessFailure"));
        assert_eq!(host.live, 0);
    }
}
