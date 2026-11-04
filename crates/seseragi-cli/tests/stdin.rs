#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

struct Program {
    name: String,
    executable: PathBuf,
    argument: Option<PathBuf>,
}
impl Program {
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        if let Some(argument) = &self.argument {
            if self.name.ends_with("-bun") {
                command.arg("--no-install");
            }
            command.arg(argument);
        }
        command.env("PATH", "").env("SESERAGI_NATIVE_METRICS", "1");
        command
    }
    fn run(&self, input: &[u8], scratch: &Path) -> Output {
        let path = scratch.join("input");
        fs::write(&path, input).unwrap();
        let output = bounded(
            self.command().stdin(Stdio::from(File::open(path).unwrap())),
            scratch,
        );
        assert!(
            output.status.success(),
            "{} / input {:?}: {}\n{}",
            self.name,
            &input[..input.len().min(40)],
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        self.check_metrics(&output);
        output
    }
    fn check_metrics(&self, output: &Output) {
        if self.argument.is_none() {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
                "{}: {output:?}",
                self.name
            );
        } else {
            assert!(output.stderr.is_empty(), "{}: {output:?}", self.name);
        }
    }
}

fn programs(scratch: &Path) -> Vec<Program> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/stdin.ssrg");
    build_programs(&source, scratch)
}

fn build_programs(source: &Path, scratch: &Path) -> Vec<Program> {
    let mut programs = vec![native_program(source, scratch)];
    for profile in ["development", "release"] {
        let out = scratch.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&out),
            scratch,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for host in ["bun", "node"] {
            programs.push(Program {
                name: format!("{profile}-{host}"),
                executable: runtime(host, scratch),
                argument: Some(out.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}

fn native_program(source: &Path, scratch: &Path) -> Program {
    let input = process::copy_input(source, scratch);
    let source = input.as_path();
    let native = scratch.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
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

#[cfg(unix)]
#[test]
fn stdin_read_failures_match_typescript_and_release_the_lease() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = scratch.join("failure.ssrg");
    let fixture = include_str!("../../../examples/spec/fixtures/rust-backend/stdin.ssrg");
    let source_text = format!("{}pub effect fn main -> Unit with Stdin, Console fails String = do {{\n  let saved = readLine\n  let action = saved ()\n  report (stdin.readChunk (stdin.defaultReadSize ()) |> map byteInts)\n  report (stdin.readLine ())\n  report (stdin.readLineWith (stdin.defaultLineLimit ()))\n  report (stdin.lines (stdin.defaultLineLimit ()) |> streams.runCollect)\n  report action\n  report (saved ())\n}}\n", fixture.split("pub effect fn main").next().unwrap());
    fs::write(&source, &source_text).unwrap();
    for mut program in build_programs(&source, scratch) {
        let mut command;
        if let Some(entry) = &program.argument {
            let profile = if program.name.starts_with("development") {
                BuildProfile::Development
            } else {
                BuildProfile::Release
            };
            let compiled = compile_module(
                CompileInput::new("failure.ssrg", "single-file/main", &source_text)
                    .with_profile(profile),
            )
            .unwrap();
            let fault = entry.parent().unwrap().join("source-failure.ts");
            let script = format!("{}\nimport {{ run as runFault }} from '@seseragi/runtime/effect';\nimport {{ createByteStdin, StdinReadFailure }} from '@seseragi/runtime/stdin';\nimport {{ serviceFailure }} from '@seseragi/runtime/service';\nimport {{ liveConsole }} from '@seseragi/runtime/console';\nconst fault = createByteStdin({{ read: () => serviceFailure(StdinReadFailure) }});\nconst outcome = await runFault(main(undefined), {{ stdin: fault, console: liveConsole }});\nif (outcome.kind !== 'success') throw new Error('unexpected failure');\n", compiled.generated.typescript);
            fs::write(&fault, source_runtime_imports(&script)).unwrap();
            let bundle = fault.with_extension("mjs");
            successful(bounded(
                Command::new(runtime("bun", scratch))
                    .arg("build")
                    .arg(&fault)
                    .args(["--target", "node", "--outfile"])
                    .arg(&bundle),
                scratch,
            ));
            program.argument = Some(bundle);
            command = program.command();
            command.stdin(Stdio::null());
        } else {
            command = program.command();
            command.stdin(Stdio::from(File::open(scratch).unwrap()));
        }
        let output = bounded(&mut command, scratch);
        assert!(output.status.success(), "{}: {output:?}", program.name);
        program.check_metrics(&output);
        assert_eq!(output.stdout, b"Left StdinReadFailure\nLeft StdinReadFailure\nLeft StdinReadFailure\nLeft StdinReadFailure\nLeft StdinReadFailure\nLeft StdinReadFailure\n", "{}", program.name);
    }
}

fn source_runtime_imports(source: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../runtime/ts/src")
        .canonicalize()
        .unwrap();
    let prefix = "@seseragi/runtime";
    let mut rest = source;
    let mut output = String::new();
    while let Some(start) = rest.find(prefix) {
        let quote = rest.as_bytes()[start - 1] as char;
        assert!(matches!(quote, '\'' | '"'));
        let tail = &rest[start + prefix.len()..];
        let end = tail.find(quote).unwrap();
        let module = tail[..end].strip_prefix('/').unwrap_or("index");
        let path = root.join(module).with_extension("ts");
        output.push_str(&rest[..start - 1]);
        output.push_str(&serde_json::to_string(&path.to_string_lossy()).unwrap());
        rest = &tail[end + 1..];
    }
    output.push_str(rest);
    output
}

#[test]
fn saved_standard_stdin_functions_execute_natively_with_the_shared_cursor() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = scratch.join("saved.ssrg");
    let fixture = include_str!("../../../examples/spec/fixtures/rust-backend/stdin.ssrg")
        .replace("keep readChunk", "keep stdin.readChunk")
        .replace("keep readWith", "keep stdin.readLineWith");
    fs::write(&source, fixture).unwrap();
    let programs = [native_program(&source, scratch)];
    compare(&programs, scratch, b"mixed\n\xff\0abcd\r\nxyremaining\nlast\r", "Right Just [255, 0]\nRight Just abcd\nRight Just [120, 121]\nRight Just remaining\nRight Just last\r\nRight Nothing\n");
    compare(&programs, scratch, b"chunks\n\x00\x80\xff\r\n\xc3\x28", "created\nRight Just [0, 128, 255]\nRight Just [13, 10, 195]\nRight Just [40]\nRight Nothing\nRight Nothing\nRight Nothing\n");
}

fn compare(programs: &[Program], scratch: &Path, input: &[u8], expected: &str) {
    for program in programs {
        assert_eq!(
            String::from_utf8(program.run(input, scratch).stdout).unwrap(),
            expected,
            "{} / input length {}",
            program.name,
            input.len()
        );
    }
}

#[test]
fn saved_prelude_read_line_matches_typescript_and_remains_cold() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend/prelude-read-line-saved.ssrg");
    let programs = build_programs(&source, scratch);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let directory = scratch.join(label);
        let compiled = compile_module(
            CompileInput::new(
                "saved.ssrg",
                "single-file/main",
                &fs::read_to_string(&source).unwrap(),
            )
            .with_profile(profile),
        )
        .unwrap();
        let raw = directory.join("raw.ts");
        fs::write(&raw, source_runtime_imports(&compiled.generated.typescript)).unwrap();
        let config = directory.join("tsconfig.json");
        fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({
                "compilerOptions": {
                    "strict": true, "noEmit": true, "skipLibCheck": false,
                    "allowImportingTsExtensions": true, "target": "ES2022",
                    "module": "ESNext", "moduleResolution": "Bundler",
                    "types": ["bun", "node"], "typeRoots": [root.join("node_modules/@types")],
                    "lib": ["ES2023", "DOM", "DOM.Iterable"]
                }, "files": [raw]
            }))
            .unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", scratch))
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config),
            scratch,
        ));
    }
    let eof = "Right Nothing\n";
    compare(
        &programs,
        scratch,
        "雪🙂\r\n\nlast\r".as_bytes(),
        &format!(
            "created\nRight Just \"雪🙂\"\nRight Just \"\"\nRight Just \"last\\r\"\n{}",
            eof.repeat(5)
        ),
    );
    compare(
        &programs,
        scratch,
        b"",
        &format!("created\n{}", eof.repeat(8)),
    );
    compare(
        &programs,
        scratch,
        b"\xff\nkept\n",
        &format!(
            "created\nLeft InvalidStdinUtf8 {{ offset: 0 }}\nRight Just \"kept\"\n{}",
            eof.repeat(6)
        ),
    );
    for length in [1048576, 1048577] {
        for ending in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
            let input = [&vec![b'x'; length], ending].concat();
            let first = if length == 1048576 {
                format!("Right Just \"{}\"\n", "x".repeat(length))
            } else {
                "Left StdinLineTooLong { limitBytes: 1048576 }\n".into()
            };
            compare(
                &programs,
                scratch,
                &input,
                &format!("created\n{first}{}", eof.repeat(7)),
            );
        }
    }
    compare(
        &programs,
        scratch,
        &[vec![b'x'; 1048577], b"\nok\n".to_vec()].concat(),
        &format!(
            "created\nLeft StdinLineTooLong {{ limitBytes: 1048576 }}\nRight Just \"ok\"\n{}",
            eof.repeat(6)
        ),
    );

    for program in &programs {
        let out = scratch.join("saved.stdout");
        let err = scratch.join("saved.stderr");
        let mut child = Running(
            program
                .command()
                .stdin(Stdio::piped())
                .stdout(Stdio::from(File::create(&out).unwrap()))
                .stderr(Stdio::from(File::create(&err).unwrap()))
                .spawn()
                .unwrap(),
        );
        wait_for(&mut child, &out, "created\n");
        assert_eq!(
            fs::read_to_string(&out).unwrap(),
            "created\n",
            "{}",
            program.name
        );
        child
            .0
            .stdin
            .take()
            .unwrap()
            .write_all(b"first\nsecond\n")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "{}: saved read did not finish",
                program.name
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        let output = successful(Output {
            status,
            stdout: fs::read(&out).unwrap(),
            stderr: fs::read(&err).unwrap(),
        });
        program.check_metrics(&output);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!(
                "created\nRight Just \"first\"\nRight Just \"second\"\n{}",
                eof.repeat(6)
            ),
            "{}",
            program.name
        );
    }
}

#[test]
fn stdin_configuration_chunks_lines_and_streams_match_typescript_at_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let programs = programs(scratch);
    compare(&programs, scratch, b"config\n", "NonPositiveReadSize -1\nNonPositiveReadSize 0\nRight\nRight\nReadSizeTooLarge 1048577\nReadSizeTooLarge 9007199254740991\nNonPositiveLineLimit -1\nNonPositiveLineLimit 0\nRight\nRight\nLineLimitTooLarge 67108865\nLineLimitTooLarge 9007199254740991\n");
    compare(&programs, scratch, b"chunks\n\x00\x80\xff\r\n\xc3\x28", "created\nRight Just [0, 128, 255]\nRight Just [13, 10, 195]\nRight Just [40]\nRight Nothing\nRight Nothing\nRight Nothing\n");
    compare(&programs, scratch, b"chunks\n", "created\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\n");
    compare(&programs, scratch, b"mixed\n\xff\0abcd\r\nxyremaining\nlast\r", "Right Just [255, 0]\nRight Just abcd\nRight Just [120, 121]\nRight Just remaining\nRight Just last\r\nRight Nothing\n");
    compare(&programs, scratch, "limited\n雪\r\n\n\u{feff}x\nlastx\r\nx\r".as_bytes(), "Right Just 雪\nRight Just \nRight Just x\nLeft StdinLineTooLong { limitBytes: 4 }\nRight Just x\r\nRight Nothing\n");
    compare(
        &programs,
        scratch,
        b"lines\na\r\n\nb\nlast",
        "created\nRight [\"a\", \"\", \"b\", \"last\"]\nRight []\nRight Nothing\n",
    );
    compare(
        &programs,
        scratch,
        b"take\na\nb\nrest\n",
        "created\nRight [\"a\"]\nRight [\"b\"]\nRight Just rest\n",
    );
    compare(
        &programs,
        scratch,
        b"zero\nkept\n",
        "created\nRight []\nRight []\nRight Just kept\n",
    );
    compare(&programs, scratch, b"lines\n12345\nok\nlast", "created\nLeft StdinLineTooLong { limitBytes: 4 }\nRight [\"ok\", \"last\"]\nRight Nothing\n");
    compare(
        &programs,
        scratch,
        b"lines\n\xff\nok\n",
        "created\nLeft InvalidStdinUtf8 { offset: 6 }\nRight [\"ok\"]\nRight Nothing\n",
    );

    for invalid in [
        vec![0x80],
        vec![0xc0, 0xaf],
        vec![0xc2],
        vec![0xc3, 0x28],
        vec![0xe0, 0x80, 0x80],
        vec![0xed, 0xa0, 0x80],
        vec![0xe2, 0x82],
        vec![0xf0, 0x80, 0x80, 0x80],
        vec![0xf4, 0x90, 0x80, 0x80],
        vec![0xf5, 0x80, 0x80, 0x80],
    ] {
        let input = [b"limited\nok\n".as_slice(), &invalid, b"\nlast\n"].concat();
        compare(&programs, scratch, &input, "Right Just ok\nLeft InvalidStdinUtf8 { offset: 11 }\nRight Just last\nRight Nothing\nRight Nothing\nRight Nothing\n");
    }
    compare(&programs, scratch, b"limited\n\xffxxxx\nok\n", "Left StdinLineTooLong { limitBytes: 4 }\nRight Just ok\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\n");
    for command in ["default-chunks", "maximum-chunks"] {
        for length in [0, 65535, 65536, 65537, 131073] {
            let prefix = format!("{command}\n");
            let input = [prefix.as_bytes(), &vec![0xff; length]].concat();
            let expected = format!("Right {length}\nRight Nothing\nRight Nothing\n");
            compare(&programs, scratch, &input, &expected);
        }
    }
    let mut input = b"limited\n".to_vec();
    input.extend(vec![0xff; 131073]);
    input.extend_from_slice(b"\nok\n");
    compare(&programs, scratch, &input, "Left StdinLineTooLong { limitBytes: 4 }\nRight Just ok\nRight Nothing\nRight Nothing\nRight Nothing\nRight Nothing\n");
    for length in [65535, 65536, 65537, 1048576, 1048577] {
        for ending in [b"\n".as_slice(), b"\r\n".as_slice(), b"".as_slice()] {
            let input = [b"defaults\n".as_slice(), &vec![b'x'; length], ending].concat();
            let first = if length <= 1048576 {
                format!("Right Just {}\n", "x".repeat(length))
            } else {
                "Left StdinLineTooLong { limitBytes: 1048576 }\n".into()
            };
            compare(
                &programs,
                scratch,
                &input,
                &format!("{first}Right Nothing\nRight Nothing\n"),
            );
        }
    }
}

struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
fn wait_for(child: &mut Running, output: &Path, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let text = fs::read_to_string(output).unwrap();
        if text.ends_with(expected) {
            return;
        }
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "process exited: {text}"
        );
        assert!(Instant::now() < deadline, "timed out: {text}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn concurrent_and_cancelled_stdin_reads_match_typescript_without_closing_cursor() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let programs = programs(scratch);
    for program in &programs {
        for chunk in [false, true] {
            for cancel in [false, true] {
                let out = scratch.join("interactive.stdout");
                let err = scratch.join("interactive.stderr");
                let mut child = Running(
                    program
                        .command()
                        .stdin(Stdio::piped())
                        .stdout(Stdio::from(File::create(&out).unwrap()))
                        .stderr(Stdio::from(File::create(&err).unwrap()))
                        .spawn()
                        .unwrap(),
                );
                child
                    .0
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(
                        format!(
                            "{}-{}\n",
                            if cancel { "cancel" } else { "concurrent" },
                            if chunk { "chunk" } else { "line" }
                        )
                        .as_bytes(),
                    )
                    .unwrap();
                wait_for(&mut child, &out, "resume\n");
                let prefix = if cancel {
                    "cancelled\nresume\n"
                } else {
                    "Left ConcurrentStdinRead\nLeft ConcurrentStdinRead\nLeft ConcurrentStdinRead\nresume\n"
                };
                assert_eq!(
                    fs::read_to_string(&out).unwrap(),
                    prefix,
                    "{} / chunk={chunk} / cancel={cancel}",
                    program.name
                );
                child.0.stdin.take().unwrap().write_all(b"data\n").unwrap();
                let deadline = Instant::now() + Duration::from_secs(30);
                let status = loop {
                    if let Some(status) = child.0.try_wait().unwrap() {
                        break status;
                    }
                    assert!(
                        Instant::now() < deadline,
                        "{}: pending read did not finish",
                        program.name
                    );
                    std::thread::sleep(Duration::from_millis(10));
                };
                let output = Output {
                    status,
                    stdout: fs::read(&out).unwrap(),
                    stderr: fs::read(&err).unwrap(),
                };
                let output = successful(output);
                program.check_metrics(&output);
                let suffix = if cancel {
                    "Right Just data\n"
                } else if chunk {
                    "Just [100, 97, 116, 97, 10]\nRight Nothing\n"
                } else {
                    "Just data\nRight Nothing\n"
                };
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap(),
                    format!("{prefix}{suffix}"),
                    "{} / chunk={chunk} / cancel={cancel}",
                    program.name
                );
            }
        }
    }
}
