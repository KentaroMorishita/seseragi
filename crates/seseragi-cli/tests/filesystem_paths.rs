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
    fn run(&self, cwd: &Path, command: &str, text: &str, other: &str, scratch: &Path) -> Output {
        self.run_with_temp_variable(
            cwd,
            command,
            text,
            other,
            scratch,
            if cfg!(windows) { "TEMP" } else { "TMPDIR" },
        )
    }
    fn run_with_temp_variable(
        &self,
        cwd: &Path,
        command: &str,
        text: &str,
        other: &str,
        scratch: &Path,
        variable: &str,
    ) -> Output {
        let input = scratch.join("input");
        fs::write(&input, format!("{command}\n{text}\n{other}\n")).unwrap();
        let temporary = scratch.join("temporaries");
        fs::create_dir_all(&temporary).unwrap();
        let mut process = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            process.arg(argument);
        }
        let output = bounded(
            process
                .current_dir(cwd)
                .stdin(Stdio::from(File::open(input).unwrap()))
                .env("PATH", "")
                .env("TMPDIR", "")
                .env("TMP", "")
                .env("TEMP", "")
                .env(variable, &temporary)
                .env("SESERAGI_NATIVE_METRICS", "1"),
            scratch,
        );
        if self.argument.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        }
        output
    }
}
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/filesystem-paths.ssrg")
}
fn native_program(source: &Path, scratch: &Path) -> Program {
    let input = process::copy_input(source, scratch);
    let source = input.as_path();
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
fn programs(scratch: &Path) -> Vec<Program> {
    let mut programs = vec![native_program(&fixture(), scratch)];
    for profile in ["development", "release"] {
        let output = scratch.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(fixture())
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

#[test]
fn native_paths_keep_saved_standard_references_partial_application_and_duplicate_requirements() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = fs::read_to_string(fixture()).unwrap()
        .replace("keep storedDirectory", "keep fs.withTemporaryDirectory")
        .replace("= storedFile text", "= fs.withTemporaryFile text")
        .replace("let atomic = storedAtomic content", "let atomic = fs.writeAtomic content")
        .replace("let move = storedMove destination", "let move = fs.move destination")
        .replace("keep storedCreates", "keep fs.createDirectories")
        .replace("keep storedCreate", "keep fs.createDirectory")
        .replace("keep storedRemoveDirectory", "keep fs.removeDirectory")
        .replace("keep storedRemove", "keep fs.removeFile")
        .replace("keep storedList", "keep fs.list")
        .replace("if file then storedFile prefix use else storedDirectory prefix use", "if file then fs.withTemporaryFile prefix use else fs.withTemporaryDirectory prefix use");
    let path = scratch.join("saved.ssrg");
    fs::write(&path, source).unwrap();
    let program = native_program(&path, scratch);
    for (index, (command, path, other, expected)) in [
        ("createDirectory", "new", "", "Right ()\n"),
        ("createDirectories", "new/nested", "", "Right ()\n"),
        ("removeFile", "file", "", "Right ()\n"),
        ("removeDirectory", "directory", "", "Right ()\n"),
        ("move", "file", "new", "Right ()\n"),
        ("atomic", "file", "bytes", "Right ()\n"),
        ("list", "directory", "", "Right []\nRight []\n"),
        (
            "temporary-directory",
            "seseragi-saved-",
            "success",
            "Right used\n",
        ),
        (
            "temporary-file",
            "seseragi-saved-",
            "success",
            "Right []:temporary 😀\n",
        ),
        ("cancel-directory", "seseragi-saved-", "", "cancelled\n"),
        ("cancel-file", "seseragi-saved-", "", "cancelled\n"),
    ]
    .into_iter()
    .enumerate()
    {
        let cwd = scratch.join(format!("saved-{index}"));
        seed(&cwd);
        let output = successful(program.run(&cwd, command, path, other, scratch));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            expected,
            "{command}"
        );
        assert_eq!(
            fs::read_dir(scratch.join("temporaries")).unwrap().count(),
            0
        );
    }
}
fn failure(operation: &str, path: &str, other: Option<&str>, kind: &str) -> String {
    let other = other.map_or("Nothing".into(), |path| {
        format!("Just {}", serde_json::to_string(path).unwrap())
    });
    format!(
        "FileSystemError {{ operation: {operation}, path: {}, otherPath: {other}, kind: {kind} }}",
        serde_json::to_string(path).unwrap()
    )
}
fn seed(cwd: &Path) {
    fs::create_dir(cwd).unwrap();
    fs::write(cwd.join("file"), b"original").unwrap();
    fs::create_dir(cwd.join("directory")).unwrap();
    fs::create_dir(cwd.join("nonempty")).unwrap();
    fs::write(cwd.join("nonempty/child"), b"nested").unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("file", cwd.join("link")).unwrap();
        std::os::unix::fs::symlink("directory", cwd.join("directory-link")).unwrap();
        std::os::unix::fs::symlink("missing", cwd.join("dangling")).unwrap();
        std::os::unix::fs::symlink("loop", cwd.join("loop")).unwrap();
    }
}
fn snapshot(path: &Path) -> Vec<(PathBuf, String, Vec<u8>)> {
    fn collect(root: &Path, path: &Path, values: &mut Vec<(PathBuf, String, Vec<u8>)>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let kind = entry.file_type().unwrap();
            let (name, content) = if kind.is_symlink() {
                (
                    "link",
                    fs::read_link(&path)
                        .unwrap()
                        .to_string_lossy()
                        .as_bytes()
                        .to_vec(),
                )
            } else if kind.is_dir() {
                ("directory", vec![])
            } else {
                ("file", fs::read(&path).unwrap())
            };
            values.push((
                path.strip_prefix(root).unwrap().into(),
                name.into(),
                content,
            ));
            if kind.is_dir() {
                collect(root, &path, values);
            }
        }
    }
    let mut values = Vec::new();
    collect(path, path, &mut values);
    values.sort();
    values
}

#[test]
fn directory_mutations_atomic_writes_and_scoped_temporaries_match_both_ts_profiles_and_hosts() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let programs = programs(scratch);
    let mut cases = vec![
        ("createDirectory", "new", "", None),
        (
            "createDirectory",
            "directory",
            "",
            Some("FileAlreadyExists"),
        ),
        ("createDirectory", "file", "", Some("FileAlreadyExists")),
        ("createDirectory", "missing/child", "", Some("FileNotFound")),
        ("createDirectory", "file/child", "", Some("NotADirectory")),
        (
            "createDirectory",
            "absent/../directory",
            "",
            Some("FileAlreadyExists"),
        ),
        ("createDirectories", "new/nested/deep", "", None),
        ("createDirectories", "directory", "", None),
        ("createDirectories", ".", "", None),
        ("createDirectories", "file", "", Some("FileAlreadyExists")),
        ("createDirectories", "file/child", "", Some("NotADirectory")),
        ("removeFile", "file", "", None),
        ("removeFile", "missing", "", Some("FileNotFound")),
        ("removeFile", "file/child", "", Some("NotADirectory")),
        (
            "removeFile",
            "directory",
            "",
            Some(if cfg!(target_os = "linux") {
                "IsADirectory"
            } else {
                "PermissionDenied"
            }),
        ),
        ("removeDirectory", "directory", "", None),
        ("removeDirectory", "nonempty", "", Some("DirectoryNotEmpty")),
        ("removeDirectory", "file", "", Some("NotADirectory")),
        ("removeDirectory", "missing", "", Some("FileNotFound")),
        ("removeDirectory", "file/child", "", Some("NotADirectory")),
        ("move", "file", "new", None),
        ("move", "file", "file", Some("FileAlreadyExists")),
        ("move", "file", "directory", Some("FileAlreadyExists")),
        ("move", "missing", "file", Some("FileAlreadyExists")),
        ("move", "missing", "new", Some("FileNotFound")),
        ("move", "file", "missing/child", Some("FileNotFound")),
        ("move", "file", "file/child", Some("NotADirectory")),
        ("move", "nonempty", "new", None),
        ("move", "directory", "new", None),
        (
            "move",
            "missing/../file",
            "./directory",
            Some("FileAlreadyExists"),
        ),
        ("atomic", "new", "bytes", None),
        ("atomic", "file", "bytes", None),
        ("atomic", "file", "empty", None),
        ("atomic", "absent/../file", "bytes", None),
        (
            "atomic",
            "directory",
            "bytes",
            Some(if cfg!(windows) {
                "PermissionDenied"
            } else {
                "IsADirectory"
            }),
        ),
        (
            "atomic",
            "nonempty",
            "bytes",
            Some(if cfg!(windows) {
                "PermissionDenied"
            } else {
                "IsADirectory"
            }),
        ),
        ("atomic", "missing/child", "bytes", Some("FileNotFound")),
        ("atomic", "file/child", "bytes", Some("NotADirectory")),
    ];
    #[cfg(unix)]
    cases.extend([
        ("createDirectory", "dangling", "", Some("FileAlreadyExists")),
        (
            "createDirectories",
            "loop/child",
            "",
            Some("SymbolicLinkLoop"),
        ),
        ("removeFile", "directory-link", "", None),
        (
            "removeDirectory",
            "directory-link",
            "",
            Some("NotADirectory"),
        ),
        ("move", "file", "dangling", Some("FileAlreadyExists")),
        ("move", "dangling", "new", None),
        ("atomic", "link", "bytes", None),
        ("atomic", "dangling", "bytes", None),
        ("atomic", "loop", "bytes", None),
    ]);
    let long = "a".repeat(300);
    cases.extend([
        (
            "createDirectory",
            long.as_str(),
            "",
            Some("PathNotSupported"),
        ),
        (
            "createDirectories",
            long.as_str(),
            "",
            Some("PathNotSupported"),
        ),
        ("removeFile", long.as_str(), "", Some("PathNotSupported")),
        (
            "removeDirectory",
            long.as_str(),
            "",
            Some("PathNotSupported"),
        ),
        ("move", "file", long.as_str(), Some("PathNotSupported")),
        ("atomic", long.as_str(), "bytes", Some("PathNotSupported")),
    ]);
    for program in &programs {
        #[cfg(windows)]
        let variables = ["TEMP", "TMP"].as_slice();
        #[cfg(not(windows))]
        let variables = ["TMPDIR", "TMP", "TEMP"].as_slice();
        for variable in variables {
            for command in ["temporary-directory", "temporary-file"] {
                let cwd = scratch.join(format!("{}-{command}-{variable}", program.name));
                seed(&cwd);
                let output = successful(program.run_with_temp_variable(
                    &cwd,
                    command,
                    "seseragi-environment-",
                    "success",
                    scratch,
                    variable,
                ));
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    if command == "temporary-file" {
                        "Right []:temporary 😀\n"
                    } else {
                        "Right used\n"
                    }
                );
                let observed = fs::read_to_string(cwd.join("observed")).unwrap();
                assert!(Path::new(&observed).starts_with(scratch.join("temporaries")));
                assert!(!Path::new(&observed).exists());
                assert_eq!(
                    fs::read_dir(scratch.join("temporaries")).unwrap().count(),
                    0
                );
            }
        }
        for (index, &(command, path, other, error)) in cases.iter().enumerate() {
            let cwd = scratch.join(format!("{}-mutation-{index}", program.name));
            seed(&cwd);
            let before = snapshot(&cwd);
            let output = successful(program.run(&cwd, command, path, other, scratch));
            let expected = if let Some(kind) = error {
                let operation = match command {
                    "atomic" => "WriteFile",
                    "move" => "MovePath",
                    "createDirectory" | "createDirectories" => "CreateDirectory",
                    _ => "RemovePath",
                };
                format!(
                    "Left {}\n",
                    failure(operation, path, (command == "move").then_some(other), kind)
                )
            } else {
                "Right ()\n".into()
            };
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{} / {command} / {path} / {other}",
                program.name
            );
            if error.is_some() {
                assert_eq!(
                    snapshot(&cwd),
                    before,
                    "{} / {command} changed existing entries",
                    program.name
                );
            } else {
                let normalized = if path == "absent/../file" {
                    "file"
                } else {
                    path
                };
                match command {
                    "atomic" => {
                        assert_eq!(
                            fs::read(cwd.join(normalized)).unwrap(),
                            if other == "empty" {
                                vec![]
                            } else {
                                vec![0, 128, 255, 13, 10, 1]
                            }
                        );
                        if matches!(path, "link" | "dangling" | "loop") {
                            assert_eq!(fs::read(cwd.join("file")).unwrap(), b"original");
                        }
                    }
                    "move" => {
                        assert!(fs::symlink_metadata(cwd.join(path)).is_err());
                        assert!(fs::symlink_metadata(cwd.join(other)).is_ok());
                        if path == "file" {
                            assert_eq!(fs::read(cwd.join(other)).unwrap(), b"original");
                        }
                        if path == "nonempty" {
                            assert_eq!(fs::read(cwd.join(other).join("child")).unwrap(), b"nested");
                        }
                    }
                    "createDirectory" | "createDirectories" => assert!(cwd.join(path).is_dir()),
                    _ => assert!(fs::symlink_metadata(cwd.join(path)).is_err()),
                }
            }
            assert!(!fs::read_dir(&cwd).unwrap().any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".seseragi-atomic-")));
        }
        for (index, (command, path, kind)) in [
            ("list", "entries", None),
            ("list", "directory", None),
            ("list-zero", "entries", None),
            ("list-negative", "entries", None),
            ("list-one", "single", None),
            ("list", "missing", Some("FileNotFound")),
            ("list-zero", "missing", Some("FileNotFound")),
            ("list", "file", Some("NotADirectory")),
            ("list", "file/child", Some("NotADirectory")),
        ]
        .into_iter()
        .enumerate()
        {
            let cwd = scratch.join(format!("{}-list-{index}", program.name));
            seed(&cwd);
            fs::create_dir(cwd.join("entries")).unwrap();
            fs::write(cwd.join("entries/😀.txt"), []).unwrap();
            fs::create_dir(cwd.join("entries/nested")).unwrap();
            fs::create_dir(cwd.join("single")).unwrap();
            fs::write(cwd.join("single/file"), []).unwrap();
            let mut entries = vec![("nested", "Directory"), ("😀.txt", "RegularFile")];
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink("missing", cwd.join("entries/link")).unwrap();
                use std::os::unix::ffi::OsStrExt;
                unsafe extern "C" {
                    fn mkfifo(path: *const std::ffi::c_char, mode: u32) -> i32;
                }
                let fifo = std::ffi::CString::new(cwd.join("entries/fifo").as_os_str().as_bytes())
                    .unwrap();
                assert_eq!(unsafe { mkfifo(fifo.as_ptr(), 0o600) }, 0);
                entries.extend([("link", "SymbolicLink"), ("fifo", "OtherFileType")]);
            }
            let output = successful(program.run(&cwd, command, path, "", scratch));
            let expected = if let Some(kind) = kind {
                format!("Left {}", failure("OpenDirectory", path, None, kind))
            } else if command == "list-zero" || command == "list-negative" || path == "directory" {
                "Right []".into()
            } else {
                if path == "single" {
                    entries = vec![("file", "RegularFile")];
                }
                let mut values: Vec<_> = entries
                    .iter()
                    .map(|(name, kind)| {
                        let entry = format!(
                            "DirectoryEntry {{ name: {}, path: {}, fileType: Just {kind} }}",
                            serde_json::to_string(name).unwrap(),
                            serde_json::to_string(&format!("{path}/{name}")).unwrap()
                        );
                        format!("{entry} | {entry}")
                    })
                    .collect();
                values.sort();
                format!("Right [{}]", values.join(", "))
            };
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                format!("{expected}\n{expected}\n"),
                "{} / {command} / {path}",
                program.name
            );
        }
        for command in ["temporary-directory", "temporary-file"] {
            for (index, prefix) in ["", ".", "..", "a/b", "a\\b", "a\0b", long.as_str()]
                .into_iter()
                .enumerate()
            {
                let cwd = scratch.join(format!("{}-{command}-invalid-{index}", program.name));
                seed(&cwd);
                let output = successful(program.run(&cwd, command, prefix, "success", scratch));
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    format!(
                        "Left Left {}\n",
                        failure("CreateTemporary", ".", None, "PathNotSupported")
                    ),
                    "{} / {command} / {prefix:?}",
                    program.name
                );
                assert!(!cwd.join("observed").exists());
                assert_eq!(
                    fs::read_dir(scratch.join("temporaries")).unwrap().count(),
                    0
                );
            }
            for mode in [
                "success",
                "fail",
                "remove",
                "move-source",
                "move-destination",
                "defect",
            ] {
                let cwd = scratch.join(format!("{}-{command}-{mode}", program.name));
                seed(&cwd);
                let output = program.run(&cwd, command, "日本😀-", mode, scratch);
                let observed = fs::read_to_string(cwd.join("observed")).unwrap();
                if mode == "defect" {
                    assert_eq!(output.status.code(), Some(70));
                    assert!(
                        String::from_utf8_lossy(&output.stderr)
                            .contains("Seseragi Defect: callback defect\n"),
                        "{}: {output:?}",
                        program.name
                    );
                } else {
                    let output = successful(output);
                    let expected = match mode {
                        "fail" => "Left Right callback failed\n".into(),
                        "move-source" | "move-destination" => {
                            let (path, other) = if mode == "move-source" {
                                (observed.clone(), "moved".into())
                            } else if command == "temporary-file" {
                                ("file".into(), observed.clone())
                            } else {
                                (format!("{observed}/nested"), observed.clone())
                            };
                            format!(
                                "Left Right {}\n",
                                failure("MovePath", &path, Some(&other), "PermissionDenied")
                            )
                        }
                        _ if command == "temporary-directory" => "Right used\n".into(),
                        "remove" => "Right temporary 😀\n".into(),
                        _ => "Right []:temporary 😀\n".into(),
                    };
                    assert_eq!(
                        String::from_utf8(output.stdout).unwrap(),
                        expected,
                        "{} / {command} / {mode}",
                        program.name
                    );
                }
                assert!(
                    !Path::new(&observed).exists(),
                    "{} / {command} / {mode}",
                    program.name
                );
                assert_eq!(
                    fs::read_dir(scratch.join("temporaries")).unwrap().count(),
                    0
                );
                assert_eq!(fs::read(cwd.join("file")).unwrap(), b"original");
                assert!(!cwd.join("moved").exists());
            }
        }
        for command in ["cancel-directory", "cancel-file"] {
            let cwd = scratch.join(format!("{}-{command}", program.name));
            seed(&cwd);
            let output = successful(program.run(&cwd, command, "seseragi-cancel-", "", scratch));
            assert_eq!(output.stdout, b"cancelled\n");
            let observed = fs::read_to_string(cwd.join("observed")).unwrap();
            assert!(!Path::new(&observed).exists());
            assert_eq!(
                fs::read_dir(scratch.join("temporaries")).unwrap().count(),
                0
            );
        }
        let cwd = scratch.join(format!("{}-replaced-temporary", program.name));
        seed(&cwd);
        let output = program.run(
            &cwd,
            "temporary-file",
            "seseragi-replaced-",
            "replace",
            scratch,
        );
        let observed = fs::read_to_string(cwd.join("observed")).unwrap();
        assert_eq!(
            output.status.code(),
            Some(70),
            "{}: {output:?}",
            program.name
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "Left Left {}\n",
                failure("RemovePath", &observed, None, "PermissionDenied")
            ),
            "{}",
            program.name
        );
        assert_eq!(fs::read(&observed).unwrap(), b"");
        assert_eq!(
            fs::read_dir(scratch.join("temporaries")).unwrap().count(),
            1
        );
        fs::remove_dir_all(Path::new(&observed).parent().unwrap()).unwrap();
    }
}

#[test]
fn filesystem_paths_original_generated_typescript_is_strict_in_both_profiles() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = fs::read_to_string(fixture()).unwrap();
    let mut files = Vec::new();
    for (name, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let compiled = compile_module(
            CompileInput::new("main.ssrg", "single-file/main", &source).with_profile(profile),
        )
        .unwrap();
        let path = scratch.path().join(format!("{name}.ts"));
        fs::write(&path, compiled.generated.typescript).unwrap();
        files.push(path);
    }
    let config = scratch.path().join("tsconfig.json");
    fs::write(&config, serde_json::to_vec(&serde_json::json!({
        "compilerOptions":{"noEmit":true,"strict":true,"skipLibCheck":false,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","typeRoots":[root.join("node_modules/@types")],"paths":{"@seseragi/runtime/*":[root.join("runtime/ts/src/*").to_string_lossy()]}},
        "files":files,
    })).unwrap()).unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch.path()))
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        scratch.path(),
    ));
}
