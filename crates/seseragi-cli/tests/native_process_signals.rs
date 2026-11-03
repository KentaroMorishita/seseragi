#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use seseragi_driver::{compile_module, BuildProfile, CompileInput};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Program {
    name: String,
    executable: PathBuf,
    entry: Option<PathBuf>,
}

fn programs(directory: &Path, forward: bool, grace_ms: u64) -> Vec<Program> {
    fs::create_dir_all(directory.join("src")).unwrap();
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/process-signals.ssrg");
    fs::write(directory.join("src/main.ssrg"), source).unwrap();
    let mode = if forward { "forward" } else { "cancel" };
    let grace = if forward {
        String::new()
    } else {
        format!("shutdown_grace_ms = {grace_ms}\n")
    };
    fs::write(directory.join("seseragi.toml"), format!("[package]\nname = \"fixture/process-signals\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\nsignal_mode = \"{mode}\"\n{grace}")).unwrap();
    let native = directory.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(directory),
        directory,
    ));
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(directory)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native)
            .args([
                "--provider",
                "std/clock::Clock=seseragi/runtime-native#clock-sleep",
            ]),
        directory,
    ));
    let mut programs = vec![Program {
        name: "native".into(),
        executable: native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        entry: None,
    }];
    for (label, profile) in [
        ("development", BuildProfile::Development),
        ("release", BuildProfile::Release),
    ] {
        let dist = directory.join(label);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(directory)
                .args(["--target", "process", "--profile", label, "--out-dir"])
                .arg(&dist),
            directory,
        ));
        let raw = dist.join("original.ts");
        let generated = compile_module(
            CompileInput::new("main.ssrg", "fixture/process-signals/main", source)
                .with_profile(profile),
        )
        .unwrap()
        .generated
        .typescript;
        fs::write(&raw, generated).unwrap();
        let config = dist.join("tsconfig.json");
        fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({
                "compilerOptions": {
                    "strict": true, "noEmit": true, "skipLibCheck": false,
                    "allowImportingTsExtensions": true, "target": "ES2022",
                    "module": "ESNext", "moduleResolution": "Bundler",
                    "types": ["bun", "node"], "typeRoots": [root().join("node_modules/@types")],
                    "baseUrl": root(), "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                    "lib": ["ES2023", "DOM", "DOM.Iterable"]
                }, "files": [raw]
            }))
            .unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", directory))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(config),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            programs.push(Program {
                name: format!("{label}-{engine}"),
                executable: runtime(engine, directory),
                entry: Some(dist.join(manifest["entry"].as_str().unwrap())),
            });
        }
    }
    programs
}

struct Running {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl Running {
    fn start(program: &Program, directory: &Path, mode: &str) -> Self {
        let stdout = directory.join("signal-stdout");
        let stderr = directory.join("signal-stderr");
        let mut command = Command::new(&program.executable);
        if let Some(entry) = &program.entry {
            command.arg(entry);
        } else {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        command
            .arg(mode)
            .current_dir(directory)
            .env("PATH", "")
            .stdout(Stdio::from(File::create(&stdout).unwrap()))
            .stderr(Stdio::from(File::create(&stderr).unwrap()));
        Self {
            child: command.spawn().unwrap(),
            stdout,
            stderr,
        }
    }
    fn until(&mut self, text: &str) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let stdout = fs::read_to_string(&self.stdout).unwrap();
            if stdout.contains(text) {
                return;
            }
            assert!(
                self.child.try_wait().unwrap().is_none(),
                "child exited before {text}: {stdout}\n{}",
                fs::read_to_string(&self.stderr).unwrap()
            );
            assert!(
                Instant::now() < deadline,
                "child did not print {text}: {stdout}"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    #[cfg(unix)]
    fn signal(&mut self, signal: &str) {
        assert!(self.child.try_wait().unwrap().is_none());
        assert!(Command::new("/bin/kill")
            .args(["-s", signal, &self.child.id().to_string()])
            .status()
            .unwrap()
            .success());
    }
    fn finish(&mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "signal process did not exit\n{}\n{}",
                fs::read_to_string(&self.stdout).unwrap(),
                fs::read_to_string(&self.stderr).unwrap()
            );
            std::thread::sleep(Duration::from_millis(5));
        };
        self.child.wait().unwrap();
        Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

fn check(
    program: &Program,
    output: Output,
    code: i32,
    stdout: &str,
    error: Option<&str>,
    forced: bool,
) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}: {output:?}",
        program.name
    );
    assert_eq!(
        output.stdout,
        stdout.as_bytes(),
        "{}: {output:?}",
        program.name
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    if let Some(error) = error {
        assert!(stderr.contains(error), "{}: {stderr}", program.name);
    } else if program.entry.is_some() {
        assert!(stderr.is_empty(), "{}: {stderr}", program.name);
    }
    if program.entry.is_none() && !forced {
        assert!(
            stderr.contains("managed-live=0"),
            "{}: {stderr}",
            program.name
        );
    }
}

#[test]
#[cfg(unix)]
fn signal_streams_match_strict_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap().join("forward");
    let programs = programs(&directory, true, 1000);
    for program in &programs {
        for (signal, value) in [
            ("INT", "Interrupt"),
            ("TERM", "Terminate"),
            ("HUP", "Hangup"),
            ("QUIT", "Quit"),
            ("USR1", "User1"),
            ("USR2", "User2"),
        ] {
            let mut running = Running::start(program, &directory, "one");
            running.until("ready\n");
            running.signal(signal);
            check(
                program,
                running.finish(),
                0,
                &format!("ready\n{value}\nroot:start\nroot:done\n"),
                None,
                false,
            );
        }
        for (mode, expected) in [
            ("cold", "cold\nroot:start\nroot:done\n"),
            (
                "values",
                "[Interrupt, Terminate, Hangup, Quit, User1, User2]\nroot:start\nroot:done\n",
            ),
            ("zero", "[]\nroot:start\nroot:done\n"),
        ] {
            let mut running = Running::start(program, &directory, mode);
            check(program, running.finish(), 0, expected, None, false);
        }
        for (mode, expected) in [
            (
                "broadcast",
                "ready\n[User1]\n[User1]\nroot:start\nroot:done\n",
            ),
            (
                "cancel-subscriber",
                "FiberCancelled\nready\n[User1]\nroot:start\nroot:done\n",
            ),
            ("failure", "ready\nUser1\nroot:start\nroot:done\n"),
            ("defect", "ready\nUser1\nroot:start\nroot:done\n"),
        ] {
            let mut running = Running::start(program, &directory, mode);
            running.until("ready\n");
            running.signal("USR1");
            let (code, error) = match mode {
                "failure" => (1, Some("signal callback failed")),
                "defect" => (70, Some("Seseragi Defect: signal callback defect")),
                _ => (0, None),
            };
            check(program, running.finish(), code, expected, error, false);
        }
        let mut running = Running::start(program, &directory, "replay");
        running.until("ready\n");
        running.signal("USR1");
        running.until("next\n");
        running.signal("USR2");
        check(
            program,
            running.finish(),
            0,
            "ready\n[User1]\nnext\n[User2]\nroot:start\nroot:done\n",
            None,
            false,
        );

        let mut running = Running::start(program, &directory, "queue");
        running.until("ready\n");
        running.signal("USR1");
        running.until("User1\n");
        for signal in ["USR2", "HUP", "USR2", "HUP"] {
            running.signal(signal);
            std::thread::sleep(Duration::from_millis(25));
        }
        check(
            program,
            running.finish(),
            0,
            "ready\nUser1\n[User1, User2, Hangup]\nroot:start\nroot:done\n",
            None,
            false,
        );

        let mut ignored = Running::start(program, &directory, "slow");
        ignored.until("ready\n");
        ignored.signal("INT");
        std::thread::sleep(Duration::from_millis(30));
        assert!(ignored.child.try_wait().unwrap().is_none());
        ignored.signal("TERM");
        check(program, ignored.finish(), 143, "ready\n", None, true);
    }
}

#[test]
#[cfg(unix)]
fn shutdown_and_reserved_signal_errors_match_ts_bun_node_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path()).unwrap().join("cancel");
    let built_programs = programs(&directory, false, 1000);
    for program in &built_programs {
        for (mode, error) in [
            ("reserved-int", "ReservedProcessSignal Interrupt"),
            ("reserved-term", "ReservedProcessSignal Terminate"),
            ("zero", "ReservedProcessSignal Interrupt"),
        ] {
            let mut running = Running::start(program, &directory, mode);
            check(
                program,
                running.finish(),
                1,
                "root:start\nroot:done\n",
                Some(error),
                false,
            );
        }
        let mut cold = Running::start(program, &directory, "cold");
        check(
            program,
            cold.finish(),
            0,
            "cold\nroot:start\nroot:done\n",
            None,
            false,
        );
        for (signal, code) in [("INT", 130), ("TERM", 143)] {
            let mut running = Running::start(program, &directory, "wait");
            running.until("ready\n");
            running.signal(signal);
            check(
                program,
                running.finish(),
                code,
                "ready\nchild:start\nchild:done\nroot:start\nroot:done\n",
                None,
                false,
            );
        }
        let mut running = Running::start(program, &directory, "slow");
        running.until("ready\n");
        running.signal("INT");
        running.until("root:start\n");
        running.signal("TERM");
        check(
            program,
            running.finish(),
            143,
            "ready\nroot:start\n",
            None,
            true,
        );
    }
    for grace_ms in [50, 0] {
        let deadline_directory = scratch.path().join(format!("deadline-{grace_ms}"));
        for program in programs(&deadline_directory, false, grace_ms) {
            let mut running = Running::start(&program, &deadline_directory, "slow");
            running.until("ready\n");
            running.signal("TERM");
            let output = running.finish();
            assert_eq!(
                output.status.code(),
                Some(143),
                "{}: {output:?}",
                program.name
            );
            assert_eq!(
                output.stdout, b"ready\nroot:start\n",
                "{}: {output:?}",
                program.name
            );
            if grace_ms == 0 {
                let mut running = Running::start(&program, &deadline_directory, "sync");
                running.until("ready\n");
                running.signal("INT");
                check(
                    &program,
                    running.finish(),
                    130,
                    "ready\nroot:start\nroot:done\n",
                    None,
                    false,
                );
            }
        }
    }
}

#[cfg(windows)]
#[test]
fn unsupported_signals_precede_reserved_signals_in_ts_and_native() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = fs::canonicalize(scratch.path())
        .unwrap()
        .join("unsupported");
    for program in programs(&directory, false, 1000) {
        for (mode, error) in [
            ("reserved-int", "UnsupportedProcessSignal User1"),
            ("reserved-term", "ReservedProcessSignal Terminate"),
            ("zero", "UnsupportedProcessSignal Hangup"),
        ] {
            let mut running = Running::start(&program, &directory, mode);
            check(
                &program,
                running.finish(),
                1,
                "root:start\nroot:done\n",
                Some(error),
                false,
            );
        }
        let mut cold = Running::start(&program, &directory, "cold");
        check(
            &program,
            cold.finish(),
            0,
            "cold\nroot:start\nroot:done\n",
            None,
            false,
        );
    }
}
