use std::io::{Read, Write};

// The high-level small-file operations own each handle until close completes.
// A host seam makes read/write/flush/close failure and unwind cleanup observable
// without exposing native file descriptors as language values.
trait NativeFileHost {
    type Handle;
    fn open(&mut self, path: &str, mode: Option<&str>) -> std::io::Result<Self::Handle>;
    fn read(&mut self, handle: &mut Self::Handle, bytes: &mut Vec<u8>) -> std::io::Result<()>;
    fn write(&mut self, handle: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<()>;
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
        options.open(host_path)
    }
    fn read(&mut self, handle: &mut Self::Handle, bytes: &mut Vec<u8>) -> std::io::Result<()> {
        handle.read_to_end(bytes).map(|_| ())
    }
    fn write(&mut self, handle: &mut Self::Handle, bytes: &[u8]) -> std::io::Result<()> {
        handle.write_all(bytes)
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
    let path = native_path_normalize(path);
    #[cfg(windows)]
    {
        path.replace('/', "\\")
    }
    #[cfg(not(windows))]
    {
        path
    }
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
impl<H: NativeFileHost> NativeFileGuard<'_, H> {
    fn finish(mut self, result: std::io::Result<()>) -> std::io::Result<()> {
        let closed = self.host.close(self.handle.take().unwrap());
        // A failed read/write keeps its primary typed error. A close failure
        // after successful work is itself a typed FileSystemError.
        result.and(closed)
    }
}
impl<H: NativeFileHost> Drop for NativeFileGuard<'_, H> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.host.close(handle);
        }
    }
}
fn native_read_file(host: &mut impl NativeFileHost, path: &str) -> std::io::Result<Vec<u8>> {
    let handle = host.open(path, None)?;
    let mut guard = NativeFileGuard {
        host,
        handle: Some(handle),
    };
    let mut bytes = Vec::new();
    let result = guard.host.read(guard.handle.as_mut().unwrap(), &mut bytes);
    guard.finish(result)?;
    Ok(bytes)
}
fn native_write_file(
    host: &mut impl NativeFileHost,
    path: &str,
    mode: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    let handle = host.open(path, Some(mode))?;
    let mut guard = NativeFileGuard {
        host,
        handle: Some(handle),
    };
    let result = guard
        .host
        .write(guard.handle.as_mut().unwrap(), bytes)
        .and_then(|_| guard.host.flush(guard.handle.as_mut().unwrap()));
    guard.finish(result)
}
fn native_file_error_code(error: &std::io::Error) -> &'static str {
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
fn native_file_error(operation: &str, path: V, error: std::io::Error) -> V {
    let code = native_file_error_code(&error);
    let kind = match code {
        "ENOENT" => "FileNotFound",
        "EEXIST" => "FileAlreadyExists",
        "EPERM" | "EACCES" => "PermissionDenied",
        "ENOTDIR" => "NotADirectory",
        "EISDIR" => "IsADirectory",
        "ENOTEMPTY" => "DirectoryNotEmpty",
        "ELOOP" => "SymbolicLinkLoop",
        "EXDEV" => "CrossDeviceMove",
        "EINVAL" | "ENAMETOOLONG" => "PathNotSupported",
        "ENOSYS" | "ENODEV" => "FileSystemUnavailable",
        _ => "OtherFileSystemError",
    };
    let kind = V::adt(
        &format!("std/fs::{kind}"),
        if kind == "OtherFileSystemError" {
            V::text(code)
        } else {
            V::Unit
        },
    );
    V::Record(Shared::new(BTreeMap::from([
        (
            "operation".into(),
            V::adt(&format!("std/fs::{operation}"), V::Unit),
        ),
        ("path".into(), path),
        ("otherPath".into(), native_maybe(None)),
        ("kind".into(), kind),
    ])))
}

fn native_read_text(host: &mut impl NativeFileHost, path: &V) -> NativeOutcome {
    let bytes = native_read_file(host, native_path_text(path)).map_err(|error| {
        NativeExit::Failure(V::adt(
            "std/fs::FileAccessFailure",
            native_file_error("ReadFile", path.clone(), error),
        ))
    })?;
    match String::from_utf8(bytes) {
        Ok(text) => Ok(V::text(&text)),
        Err(error) => Err(NativeExit::Failure(V::adt(
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
fn native_filesystem_operation(name: &str, arguments: Vec<V>) -> V {
    if name.starts_with("std/path::") {
        return native_path_operation(name, &arguments);
    }
    match name {
        "std/fs::Replace" | "std/fs::CreateNew" | "std/fs::Append" => V::adt(name, V::Unit),
        "std/fs::readTextUtf8" => V::leaf(arguments, |c, _| {
            native_read_text(&mut NativeStdFileHost, &c[0])
        }),
        "std/fs::writeTextUtf8" => V::leaf(arguments, |c, _| {
            let V::Adt(mode) = &c[0] else {
                panic!("Rust ABI: expected WriteMode")
            };
            let mode = match mode.get().0.as_str() {
                "std/fs::Replace" => "Replace",
                "std/fs::CreateNew" => "CreateNew",
                "std/fs::Append" => "Append",
                _ => panic!("Rust ABI: WriteMode owner"),
            };
            native_write_file(
                &mut NativeStdFileHost,
                native_path_text(&c[2]),
                mode,
                native_text(&c[1]).as_bytes(),
            )
            .map_err(|error| {
                NativeExit::Failure(native_file_error("WriteFile", c[2].clone(), error))
            })?;
            Ok(V::Unit)
        }),
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
        "std/fs::FileSystemOperation" | "std/fs::WriteMode" => {
            native_standard_tag(value, "std/fs::")
        }
        _ => panic!("Rust ABI: filesystem display identity"),
    }
}
fn native_filesystem_display_dictionary(name: &'static str, debug: bool) -> V {
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
        fn read(&mut self, _: &mut (), bytes: &mut Vec<u8>) -> std::io::Result<()> {
            if self.panic_read {
                panic!("read defect")
            }
            self.event("read")?;
            bytes.extend_from_slice(self.bytes.as_deref().unwrap_or(b"hello"));
            Ok(())
        }
        fn write(&mut self, _: &mut (), _: &[u8]) -> std::io::Result<()> {
            self.event("write")
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
        assert_eq!(host.events, ["open", "read", "close"]);
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
