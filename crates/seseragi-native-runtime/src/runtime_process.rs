fn native_process_arguments(arguments: impl IntoIterator<Item = std::ffi::OsString>) -> V {
    // The TypeScript hosts decode OS strings with replacement characters.
    V::Array(Shared::new(
        arguments
            .into_iter()
            .map(|argument| V::text(&argument.to_string_lossy()))
            .collect(),
    ))
}

fn native_process_environment(name: &V) -> NativeOutcome {
    let text = native_text(name);
    if text.is_empty() || text.contains('\0') {
        return Err(NativeExit::failure(V::adt(
            "std/process::InvalidEnvironmentName",
            name.clone(),
        )));
    }
    Ok(native_maybe(
        std::env::var_os(text).map(|value| V::text(&value.to_string_lossy())),
    ))
}

fn native_process_directory(directory: std::io::Result<std::path::PathBuf>) -> NativeOutcome {
    if let Ok(directory) = directory {
        let text = directory.to_string_lossy().replace('\\', "/");
        if native_path_validation(&text).is_none() {
            return Ok(native_path(&text));
        }
    }
    Err(NativeExit::failure(V::adt(
        "std/process::CurrentDirectoryUnavailable",
        V::Unit,
    )))
}

pub fn native_process_operation(name: &str, argument: V) -> V {
    match name {
        "std/process::signals" => native_process_signals(argument),
        "std/process::arguments" => {
            assert!(matches!(argument, V::Unit));
            V::leaf(vec![], |_, _| {
                Ok(native_process_arguments(std::env::args_os().skip(1)))
            })
        }
        "std/process::environment" => V::leaf(vec![argument], |captures, _| {
            native_process_environment(&captures[0])
        }),
        "std/process::currentDirectory" => {
            assert!(matches!(argument, V::Unit));
            V::leaf(vec![], |_, _| {
                native_process_directory(std::env::current_dir())
            })
        }
        _ => panic!("Rust ABI: unsupported Process operation"),
    }
}

#[cfg(test)]
mod process_tests {
    use super::*;

    #[test]
    fn arguments_keep_order_empty_values_and_unicode() {
        let values = native_process_arguments(["", "--flag", "雪🙂", "a\nb"].map(Into::into));
        let V::Array(values) = values else { panic!() };
        assert_eq!(
            values.get().iter().map(native_text).collect::<Vec<_>>(),
            ["", "--flag", "雪🙂", "a\nb"]
        );
        drop(values);
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn invalid_environment_names_fail_only_when_the_plan_runs() {
        for name in ["", "a\0b", "\0", "雪🙂\0"] {
            let plan = native_process_operation("std/process::environment", V::text(name));
            for _ in 0..2 {
                let Err(NativeExit::Failure(error)) = NativeContext::default().root(plan.clone())
                else {
                    panic!("expected InvalidEnvironmentName")
                };
                assert!(error.value.tagged("std/process::InvalidEnvironmentName"));
                assert_eq!(native_text(&error.value.payload()), name);
            }
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[test]
    fn directories_use_portable_separators_and_typed_failures() {
        for (input, expected) in [
            ("/雪🙂/a", "/雪🙂/a"),
            ("C:\\a\\b", "C:/a/b"),
            ("\\\\server\\share\\a", "//server/share/a"),
        ] {
            let path = native_process_directory(Ok(input.into())).unwrap_or_else(|_| panic!());
            assert_eq!(native_path_text(&path), expected);
        }
        for directory in [
            Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
            Ok("".into()),
            Ok("/a\0b".into()),
        ] {
            let Err(NativeExit::Failure(error)) = native_process_directory(directory) else {
                panic!("expected CurrentDirectoryUnavailable")
            };
            assert!(error
                .value
                .tagged("std/process::CurrentDirectoryUnavailable"));
            assert!(matches!(error.value.payload(), V::Unit));
        }
        assert_eq!(LIVE.with(Cell::get), 0);
    }

    #[cfg(unix)]
    #[test]
    fn host_bytes_use_the_same_replacement_characters_as_typescript() {
        use std::os::unix::ffi::OsStringExt;
        let bytes = std::ffi::OsString::from_vec(b"bad\xff\xed\xa0\x80".to_vec());
        let V::Array(arguments) = native_process_arguments([bytes.clone()]) else {
            panic!()
        };
        assert_eq!(native_text(&arguments.get()[0]), "bad����");
        let path = native_process_directory(Ok(std::path::PathBuf::from(bytes)))
            .unwrap_or_else(|_| panic!());
        assert_eq!(native_path_text(&path), "bad����");
        drop((arguments, path));
        assert_eq!(LIVE.with(Cell::get), 0);
    }
}
