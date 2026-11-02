#[path = "../../test-support/native_runtime.rs"]
mod link;
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    link::configure(&mut command);
    command
}

struct Program {
    name: String,
    executable: PathBuf,
    argument: Option<PathBuf>,
}

impl Program {
    fn run(&self, cwd: &Path, input: &Path, logs: &Path) -> Output {
        let mut command = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            command.arg(argument);
        }
        let output = bounded(
            command
                .current_dir(cwd)
                .stdin(Stdio::from(File::open(input).unwrap()))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            logs,
        );
        if self.argument.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        }
        successful(output)
    }
}

fn native_program(source: &Path, scratch: &Path) -> Program {
    let native = scratch.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        scratch,
    ));
    Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        argument: None,
    }
}

fn programs(source: &Path, scratch: &Path) -> Vec<Program> {
    let mut programs = vec![native_program(source, scratch)];
    for profile in ["development", "release"] {
        let output = scratch.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            scratch,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for host in ["bun", "node"] {
            programs.push(Program {
                name: format!("{profile}-{host}"),
                executable: runtime(host, scratch),
                argument: Some(output.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}

fn error(operation: &str, path: &str, kind: &str) -> String {
    format!(
        "FileSystemError {{ operation: {operation}, path: {}, otherPath: Nothing, kind: {kind} }}",
        serde_json::to_string(path).unwrap()
    )
}
fn ints(bytes: &[u8]) -> String {
    format!(
        "[{}]",
        bytes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn twice(value: &str) -> String {
    format!("{value}\n{value}\n")
}

#[test]
fn binary_and_chunked_files_match_ts_for_data_modes_errors_and_demand_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/filesystem-bytes.ssrg");
    let programs = programs(&source, scratch);
    let input = scratch.join("input");
    let binary = vec![0, 1, 127, 128, 255, 0xc3, 0x28, 13, 10, 0];
    for program in &programs {
        let cwd = scratch.join(format!("data-{}", program.name));
        fs::create_dir(&cwd).unwrap();
        fs::create_dir(cwd.join("directory")).unwrap();
        fs::write(cwd.join("binary"), &binary).unwrap();
        fs::write(cwd.join("empty"), []).unwrap();
        fs::write(cwd.join("file"), b"parent").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("binary", cwd.join("link")).unwrap();
            std::os::unix::fs::symlink("missing", cwd.join("dangling")).unwrap();
            std::os::unix::fs::symlink("loop", cwd.join("loop")).unwrap();
        }
        let mut paths = vec![
            ("binary".to_string(), Some(binary.clone()), None),
            ("empty".to_string(), Some(vec![]), None),
            ("./binary".to_string(), Some(binary.clone()), None),
            ("missing/../binary".to_string(), Some(binary.clone()), None),
            ("missing".to_string(), None, Some("FileNotFound")),
            ("directory".to_string(), None, Some("IsADirectory")),
            ("file/child".to_string(), None, Some("NotADirectory")),
            ("a".repeat(300), None, Some("PathNotSupported")),
        ];
        #[cfg(unix)]
        paths.extend([
            ("link".into(), Some(binary.clone()), None),
            ("dangling".into(), None, Some("FileNotFound")),
            ("loop".into(), None, Some("SymbolicLinkLoop")),
        ]);
        for (path, bytes, failure) in paths {
            for command in ["read", "small", "one", "zero", "negative"] {
                fs::write(&input, format!("{command}\n{path}\n")).unwrap();
                let expected = if let Some(kind) = failure {
                    if matches!(command, "zero" | "negative")
                        && kind == "IsADirectory"
                        && !cfg!(windows)
                    {
                        twice("Right []")
                    } else {
                        twice(&format!("Left {}", error("ReadFile", &path, kind)))
                    }
                } else {
                    let bytes = bytes.as_deref().unwrap();
                    let value = match command {
                        "read" => ints(bytes),
                        "zero" | "negative" => "[]".into(),
                        "one" => {
                            if bytes.is_empty() {
                                "[]".into()
                            } else {
                                format!("[{}]", ints(&bytes[..1]))
                            }
                        }
                        _ => format!(
                            "[{}]",
                            bytes.chunks(3).map(ints).collect::<Vec<_>>().join(", ")
                        ),
                    };
                    twice(&format!("Right {value}"))
                };
                let output = program.run(&cwd, &input, scratch);
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    expected,
                    "{} / {command} / {path}",
                    program.name
                );
            }
        }
        for length in [65535, 65536, 65537, 131073] {
            let bytes = (0..length).map(|i| (i % 256) as u8).collect::<Vec<_>>();
            fs::write(cwd.join("boundary"), &bytes).unwrap();
            for command in ["read", "large"] {
                fs::write(&input, format!("{command}\nboundary\n")).unwrap();
                let value = if command == "read" {
                    ints(&bytes)
                } else {
                    format!(
                        "[{}]",
                        bytes.chunks(65536).map(ints).collect::<Vec<_>>().join(", ")
                    )
                };
                assert_eq!(
                    String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
                    twice(&format!("Right {value}")),
                    "{} / {command} / {length}",
                    program.name
                );
            }
        }
        for (command, payload, tagged) in [
            ("write", vec![0, 128, 255, 13, 10, 254, 1], false),
            (
                "write-chunks",
                vec![0, 128, 255, 13, 10, 254, 1, 0, 128, 255, 13, 10, 254, 1],
                true,
            ),
        ] {
            for path in ["new", "existing", "directory", "absent/child", "file/child"] {
                if path == "new" {
                    let _ = fs::remove_file(cwd.join(path));
                }
                if path == "existing" {
                    fs::write(cwd.join(path), [42; 100]).unwrap();
                }
                fs::write(&input, format!("{command}\n{path}\n")).unwrap();
                let expected = if matches!(path, "new" | "existing") {
                    let failure = error("WriteFile", path, "FileAlreadyExists");
                    format!(
                        "Right ()\nRight ()\nLeft {}\n{}",
                        if tagged {
                            format!("Right {failure}")
                        } else {
                            failure
                        },
                        twice(&format!("Right {}", ints(&payload)))
                    )
                } else {
                    let kind = match path {
                        "directory" => "IsADirectory",
                        "absent/child" => "FileNotFound",
                        _ => "NotADirectory",
                    };
                    let failure = error("WriteFile", path, kind);
                    let failure = if tagged {
                        format!("Right {failure}")
                    } else {
                        failure
                    };
                    let create_failure = if path == "directory" {
                        let failure = error("WriteFile", path, "FileAlreadyExists");
                        if tagged {
                            format!("Right {failure}")
                        } else {
                            failure
                        }
                    } else {
                        failure.clone()
                    };
                    let read_kind = match path {
                        "directory" => "IsADirectory",
                        "absent/child" => "FileNotFound",
                        _ => "NotADirectory",
                    };
                    format!(
                        "{}{}",
                        format!("Left {failure}\nLeft {failure}\nLeft {create_failure}\n"),
                        twice(&format!("Left {}", error("ReadFile", path, read_kind)))
                    )
                };
                assert_eq!(
                    String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
                    expected,
                    "{} / {command} / {path}",
                    program.name
                );
                if matches!(path, "new" | "existing") {
                    assert_eq!(fs::read(cwd.join(path)).unwrap(), payload);
                }
            }
        }
        for path in ["partial", "directory"] {
            fs::write(&input, format!("failed\n{path}\n")).unwrap();
            let expected = if path == "partial" {
                "Left Left source failed\n".into()
            } else {
                format!("Left Right {}\n", error("WriteFile", path, "IsADirectory"))
            };
            assert_eq!(
                String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
                expected,
                "{} / failed / {path}",
                program.name
            );
            if path == "partial" {
                assert_eq!(fs::read(cwd.join(path)).unwrap(), [0, 128, 255]);
            }
        }
        for command in [
            "empty-bytes",
            "empty-chunks",
            "create-bytes",
            "create-chunks",
            "append-bytes",
            "append-chunks",
        ] {
            for existing in [false, true] {
                let path = "once";
                let _ = fs::remove_file(cwd.join(path));
                if existing {
                    fs::write(cwd.join(path), [9, 8, 7, 6, 5]).unwrap();
                }
                fs::write(&input, format!("{command}\n{path}\n")).unwrap();
                let rejected = existing && command.starts_with("create");
                let bytes = if rejected {
                    vec![9, 8, 7, 6, 5]
                } else if command.starts_with("empty") {
                    vec![]
                } else if existing {
                    vec![9, 8, 7, 6, 5, 0, 128, 255]
                } else {
                    vec![0, 128, 255]
                };
                let status = if rejected {
                    format!(
                        "Left {}{}",
                        if command.ends_with("chunks") {
                            "Right "
                        } else {
                            ""
                        },
                        error("WriteFile", path, "FileAlreadyExists")
                    )
                } else {
                    "Right ()".into()
                };
                assert_eq!(
                    String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
                    format!("{status}\n{}", twice(&format!("Right {}", ints(&bytes)))),
                    "{} / {command} / existing={existing}",
                    program.name
                );
                assert_eq!(fs::read(cwd.join(path)).unwrap(), bytes);
            }
        }
        for path in ["binary", "empty", "boundary", "missing", "directory"] {
            fs::write(cwd.join("copy.bin"), b"old output").unwrap();
            fs::write(&input, format!("copy\n{path}\n")).unwrap();
            let (expected, bytes) = match path {
                "missing" | "directory" => (
                    format!(
                        "Left {} {}\n",
                        if path == "missing" || cfg!(windows) {
                            "Right"
                        } else {
                            "Left"
                        },
                        error(
                            "ReadFile",
                            path,
                            if path == "missing" {
                                "FileNotFound"
                            } else {
                                "IsADirectory"
                            }
                        )
                    ),
                    vec![],
                ),
                _ => ("Right ()\n".into(), fs::read(cwd.join(path)).unwrap()),
            };
            assert_eq!(
                String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
                expected,
                "{} / copy / {path}",
                program.name
            );
            assert_eq!(fs::read(cwd.join("copy.bin")).unwrap(), bytes);
        }
        fs::remove_file(cwd.join("copy.bin")).unwrap();
        fs::create_dir(cwd.join("copy.bin")).unwrap();
        fs::write(&input, "copy\nmissing\n").unwrap();
        assert_eq!(
            String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
            format!(
                "Left Right {}\n",
                error("WriteFile", "copy.bin", "IsADirectory")
            ),
            "{} / destination before source",
            program.name
        );
        fs::write(&input, "cancel\ncancelled\n").unwrap();
        assert_eq!(
            program.run(&cwd, &input, scratch).stdout,
            b"cancelled\n",
            "{} / cancellation",
            program.name
        );
        assert_eq!(fs::read(cwd.join("cancelled")).unwrap(), [0, 128, 255]);
        fs::write(&input, "huge\nbinary\n").unwrap();
        assert_eq!(
            String::from_utf8(program.run(&cwd, &input, scratch).stdout).unwrap(),
            twice(&format!(
                "Left {}",
                error(
                    "ReadFile",
                    "binary",
                    "OtherFileSystemError \"FILESYSTEM_ERROR\""
                )
            )),
            "{} / allocation boundary",
            program.name
        );
        assert_eq!(fs::read(cwd.join("binary")).unwrap(), binary);
    }
}

#[test]
fn binary_filesystem_original_typescript_is_strict_in_both_profiles() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-bytes.ssrg");
    let mut files = Vec::new();
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled = compile_module(
            CompileInput::new("main.ssrg", "single-file/main", source).with_profile(profile),
        )
        .unwrap();
        let path = scratch.join(format!("{name}.ts"));
        fs::write(&path, compiled.generated.typescript).unwrap();
        files.push(path);
    }
    let config = scratch.join("tsconfig.json");
    fs::write(&config, serde_json::to_vec(&serde_json::json!({
        "compilerOptions":{"noEmit":true,"strict":true,"skipLibCheck":false,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","typeRoots":[root.join("node_modules/@types")],"paths":{"@seseragi/runtime/*":[root.join("runtime/ts/src/*").to_string_lossy()]}},
        "files":files,
    })).unwrap()).unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch))
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        scratch,
    ));
}

#[test]
fn native_binary_filesystem_keeps_saved_and_partially_applied_standard_functions() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-bytes.ssrg")
        .replace("keep storedRead", "keep fs.readBytes")
        .replace("let saved = \\path: paths.Path -> fs.readChunks capacity path", "let saved = fs.readChunks capacity")
        .replace("keep storedWrite", "keep fs.writeBytes")
        .replace("let savedChunks = keep storedChunks", "let savedChunks: fs.WriteMode -> streams.Stream<{},String,bytes.Bytes> -> paths.Path -> Effect<{fileSystem:fs.FileSystem},Either<String,fs.FileSystemError>,Unit> = keep fs.writeChunks");
    let path = scratch.join("saved.ssrg");
    fs::write(&path, source).unwrap();
    let program = native_program(&path, scratch);
    let input = scratch.join("input");
    fs::write(scratch.join("binary"), [0, 128, 255, 1]).unwrap();
    for (command, path, expected) in [
        ("read", "binary", twice("Right [0, 128, 255, 1]")),
        ("small", "binary", twice("Right [[0, 128, 255], [1]]")),
        (
            "write",
            "bytes",
            format!(
                "Right ()\nRight ()\nLeft {}\n{}",
                error("WriteFile", "bytes", "FileAlreadyExists"),
                twice("Right [0, 128, 255, 13, 10, 254, 1]")
            ),
        ),
        (
            "write-chunks",
            "chunks",
            format!(
                "Right ()\nRight ()\nLeft Right {}\n{}",
                error("WriteFile", "chunks", "FileAlreadyExists"),
                twice("Right [0, 128, 255, 13, 10, 254, 1, 0, 128, 255, 13, 10, 254, 1]")
            ),
        ),
    ] {
        fs::write(&input, format!("{command}\n{path}\n")).unwrap();
        assert_eq!(
            String::from_utf8(program.run(scratch, &input, scratch).stdout).unwrap(),
            expected,
            "{command}"
        );
    }
}
