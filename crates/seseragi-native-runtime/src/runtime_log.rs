fn native_log_event(event: &V) -> String {
    let level = match native_error_tag(&event.field("level")) {
        "LogTrace" => "trace",
        "LogDebug" => "debug",
        "LogInfo" => "info",
        "LogWarn" => "warn",
        "LogFailure" => "failure",
        _ => panic!("Rust ABI: LogLevel variant"),
    };
    let mut output = format!(
        "{{\"level\":\"{level}\",\"message\":{},\"fields\":[",
        native_json_string(native_text(&event.field("message")))
    );
    let fields = event.field("fields");
    let mut cursor = &fields;
    let mut separator = "";
    while let Some(cell) = cursor.list_cell() {
        output.push_str(separator);
        output.push('[');
        output.push_str(&native_json_string(native_text(&cell.head.index(0))));
        output.push(',');
        let value = cell.head.index(1);
        let V::Adt(value) = value else {
            panic!("Rust ABI: LogValue")
        };
        let text = match value.get() {
            ("std/log::LogString", V::Text(value)) => native_json_string(value.get()),
            ("std/log::LogInt", V::Int(value)) => value.to_string(),
            ("std/log::LogFloat", V::Float(value)) => {
                if value.is_finite() {
                    native_number_text(*value)
                } else {
                    "null".into()
                }
            }
            ("std/log::LogBool", V::Bool(value)) => value.to_string(),
            _ => panic!("Rust ABI: LogValue payload"),
        };
        output.push_str(&text);
        output.push(']');
        separator = ",";
        cursor = &cell.tail;
    }
    output.push_str("]}\n");
    output
}

fn native_log_io_message(error: &std::io::Error) -> String {
    use std::io::ErrorKind;
    let description = match error.kind() {
        ErrorKind::BrokenPipe => Some("EPIPE: broken pipe, write"),
        ErrorKind::StorageFull => Some("ENOSPC: no space left on device, write"),
        ErrorKind::PermissionDenied => Some("EACCES: permission denied, write"),
        ErrorKind::InvalidInput => Some("EINVAL: invalid argument, write"),
        ErrorKind::WouldBlock => Some("EAGAIN: resource temporarily unavailable, write"),
        ErrorKind::FileTooLarge => Some("EFBIG: file too large, write"),
        _ => None,
    };
    #[cfg(unix)]
    let description = description.or(match error.raw_os_error() {
        Some(9) => Some("EBADF: bad file descriptor, write"),
        Some(5) => Some("EIO: i/o error, write"),
        _ => None,
    });
    description
        .map(str::to_owned)
        .unwrap_or_else(|| error.to_string())
}

fn write_native_log(writer: &mut impl std::io::Write, event: &V) -> NativeOutcome {
    writer
        .write_all(native_log_event(event).as_bytes())
        .and_then(|_| writer.flush())
        .map(|_| V::Unit)
        .map_err(|error| {
            NativeExit::failure(V::Record(Shared::new(BTreeMap::from([
                ("kind".into(), V::text("log-error")),
                ("message".into(), V::text(&native_log_io_message(&error))),
            ]))))
        })
}

fn native_log_stderr(event: &V) -> NativeOutcome {
    let stderr = std::io::stderr();
    #[cfg(unix)]
    {
        struct Stderr;
        impl std::io::Write for Stderr {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                unsafe extern "C" {
                    #[link_name = "write"]
                    fn write_stderr(fd: i32, bytes: *const u8, length: usize) -> isize;
                }
                let length = bytes.len().min(isize::MAX as usize);
                let written = unsafe { write_stderr(2, bytes.as_ptr(), length) };
                if written < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(written as usize)
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        // Keep stderr writes serialized while preserving bad-descriptor errors.
        let _lock = stderr.lock();
        write_native_log(&mut Stderr, event)
    }
    #[cfg(not(unix))]
    write_native_log(&mut stderr.lock(), event)
}

pub fn native_log_operation(name: &str, mut arguments: Vec<V>) -> V {
    let tag = match name {
        "std/log::LogTrace" => "std/log::LogTrace",
        "std/log::LogDebug" => "std/log::LogDebug",
        "std/log::LogInfo" => "std/log::LogInfo",
        "std/log::LogWarn" => "std/log::LogWarn",
        "std/log::LogFailure" => "std/log::LogFailure",
        "std/log::LogString" => "std/log::LogString",
        "std/log::LogInt" => "std/log::LogInt",
        "std/log::LogFloat" => "std/log::LogFloat",
        "std/log::LogBool" => "std/log::LogBool",
        "std/log::log" => return V::leaf(arguments, |captures, _| native_log_stderr(&captures[0])),
        _ => panic!("Rust ABI: unsupported Logger operation"),
    };
    V::adt(tag, arguments.pop().unwrap_or(V::Unit))
}

#[cfg(test)]
mod log_tests {
    use super::*;

    fn event(fields: Vec<V>) -> V {
        V::Record(Shared::new(BTreeMap::from([
            ("level".into(), V::adt("std/log::LogInfo", V::Unit)),
            ("message".into(), V::text("雪🙂\n\0\u{1f}")),
            ("fields".into(), V::list_from_values(fields)),
        ])))
    }

    #[test]
    fn json_spelling_preserves_order_duplicates_and_nonfinite_numbers() {
        let fields = [
            V::adt("std/log::LogString", V::text("\"\\\u{b}")),
            V::adt("std/log::LogInt", V::Int(-9007199254740991)),
            V::adt("std/log::LogFloat", V::Float(-0.0)),
            V::adt("std/log::LogFloat", V::Float(1e21)),
            V::adt("std/log::LogFloat", V::Float(f64::NAN)),
            V::adt("std/log::LogFloat", V::Float(f64::INFINITY)),
            V::adt("std/log::LogFloat", V::Float(f64::NEG_INFINITY)),
            V::adt("std/log::LogBool", V::Bool(false)),
        ]
        .into_iter()
        .map(|value| V::Tuple(Shared::new(vec![V::text(""), value])))
        .collect();
        let mut output = Vec::new();
        assert!(matches!(
            write_native_log(&mut output, &event(fields)),
            Ok(V::Unit)
        ));
        assert_eq!(String::from_utf8(output).unwrap(), "{\"level\":\"info\",\"message\":\"雪🙂\\n\\u0000\\u001f\",\"fields\":[[\"\",\"\\\"\\\\\\u000b\"],[\"\",-9007199254740991],[\"\",0],[\"\",1e+21],[\"\",null],[\"\",null],[\"\",null],[\"\",false]]}\n");
    }

    #[test]
    fn write_and_flush_errors_keep_the_log_failure_shape_and_message() {
        struct Broken {
            flush: bool,
        }
        impl std::io::Write for Broken {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.flush {
                    Ok(bytes.len())
                } else {
                    Err(std::io::Error::other("sink unavailable 雪"))
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::other("sink unavailable 雪"))
            }
        }
        for flush in [false, true] {
            let Err(NativeExit::Failure(error)) =
                write_native_log(&mut Broken { flush }, &event(vec![]))
            else {
                panic!("expected LogError")
            };
            assert_eq!(error.field("kind").render(), "log-error");
            assert_eq!(error.field("message").render(), "sink unavailable 雪");
            assert_eq!(
                native_standard_error_text("std/log::LogError", &error, false, None),
                "LogError: sink unavailable 雪"
            );
            assert_eq!(
                native_standard_error_text("std/log::LogError", &error, true, None),
                "LogError { message: \"<redacted>\" }"
            );
        }
    }

    #[test]
    fn large_field_lists_are_rendered_iteratively() {
        let value = event(
            (0..10_000)
                .map(|index| {
                    V::Tuple(Shared::new(vec![
                        V::text("n"),
                        V::adt("std/log::LogInt", V::Int(index)),
                    ]))
                })
                .collect(),
        );
        let text = native_log_event(&value);
        assert_eq!(text.matches("[\"n\",").count(), 10_000);
        assert!(text.ends_with("[\"n\",9999]]}\n"));
        drop(value);
        clear_dictionaries();
        collect_cycles().unwrap();
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[cfg(unix)]
    #[test]
    fn operating_system_write_errors_use_the_ts_provider_messages() {
        for (code, expected) in [
            (9, "EBADF: bad file descriptor, write"),
            (5, "EIO: i/o error, write"),
            (32, "EPIPE: broken pipe, write"),
            (28, "ENOSPC: no space left on device, write"),
        ] {
            assert_eq!(
                native_log_io_message(&std::io::Error::from_raw_os_error(code)),
                expected
            );
        }
    }
}
