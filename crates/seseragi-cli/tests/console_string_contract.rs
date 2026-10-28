use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-console-contract-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedChild {
    child: Child,
    stdout: PathBuf,
    stderr: PathBuf,
}
impl OwnedChild {
    fn start(command: &mut Command, directory: &Path) -> Self {
        let stdout = directory.join("stdout");
        let stderr = directory.join("stderr");
        command
            .stdout(Stdio::from(File::create(&stdout).unwrap()))
            .stderr(Stdio::from(File::create(&stderr).unwrap()));
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        Self {
            child: command.spawn().unwrap(),
            stdout,
            stderr,
        }
    }
    fn wait(&mut self, timeout: Duration) -> Output {
        let deadline = Instant::now() + timeout;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "deadline for owned PID {}\n{}",
                self.child.id(),
                fs::read_to_string(&self.stderr).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        self.child.wait().unwrap();
        Output {
            status,
            stdout: fs::read(&self.stdout).unwrap(),
            stderr: fs::read(&self.stderr).unwrap(),
        }
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            #[cfg(unix)]
            unsafe {
                unsafe extern "C" {
                    fn kill(pid: i32, signal: i32) -> i32;
                }
                // Only this still-live child's dedicated process group.
                kill(-(self.child.id() as i32), 15);
                // Do not reap the group leader during the grace interval. Its
                // reserved PID prevents group-ID reuse before the final kill;
                // compiler descendants may outlive a terminated parent.
                std::thread::sleep(Duration::from_secs(2));
                kill(-(self.child.id() as i32), 9);
            }
            #[cfg(windows)]
            {
                if let Ok(mut terminator) = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &self.child.id().to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    let until = Instant::now() + Duration::from_secs(5);
                    while Instant::now() < until {
                        if terminator.try_wait().ok().flatten().is_some() {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let _ = terminator.kill();
                    let _ = terminator.wait();
                }
            }
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn bounded(command: &mut Command, directory: &Path) -> Output {
    OwnedChild::start(command, directory).wait(Duration::from_secs(120))
}
fn runtime(name: &str, directory: &Path) -> PathBuf {
    let output = successful(bounded(
        Command::new(if cfg!(windows) { "where" } else { "which" }).arg(name),
        directory,
    ));
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}

fn root() -> PathBuf {
    // Node24 cannot open script entries through Windows verbatim paths from
    // fs::canonicalize. Cargo's manifest path is absolute; use lexical parents.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root above the crate")
        .to_path_buf()
}

fn check_raw_strict(
    text: &str,
    profile: &str,
    value: seseragi_driver::BuildProfile,
    output: &Path,
    scratch: &Path,
) {
    let raw = seseragi_driver::compile_module(
        seseragi_driver::CompileInput::new("console.ssrg", "proof/console", text)
            .with_profile(value),
    )
    .unwrap();
    fs::write(output.join("raw.ts"), raw.generated.typescript).unwrap();
    let mut files = vec![output.join("raw.ts")];
    if profile == "development" {
        files.push(output.join("entry.ts"));
    }
    let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"],"allowImportingTsExtensions":true},"files":files});
    fs::write(
        output.join("tsconfig.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    successful(bounded(
        Command::new(runtime("node", scratch))
            .arg(root().join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(output.join("tsconfig.json")),
        scratch,
    ));
}

#[test]
fn string_console_calls_keep_raw_strict_ts_native_and_scope_lifetimes() {
    let fixture =
        include_str!("../../../examples/spec/fixtures/rust-backend/console-string-contract.ssrg");
    let prefix = "String:hello\nTrue\n()\nsaved:String\nacquired\n";
    assert_eq!(fixture.matches("body \"ready\"").count(), 1);
    assert_eq!(fixture.matches("output \"雫🙂\"").count(), 1);
    for (label, text, code, expected) in [
        (
            "success",
            fixture.to_owned(),
            0,
            format!("{prefix}cold\n雫🙂\n雫🙂\nreleased\ndone\n"),
        ),
        (
            "failure",
            fixture.replace("body \"ready\"", "body \"fail\""),
            0,
            format!("{prefix}cold\n雫🙂\n雫🙂\nreleased\nFailure: body failure\n"),
        ),
        (
            "defect",
            fixture.replace("body \"ready\"", "body \"defect\""),
            70,
            format!("{prefix}cold\n雫🙂\n雫🙂\nreleased\n"),
        ),
        (
            "argument-defect",
            fixture.replace("output \"雫🙂\"", "output (show (9007199254740991 + 1))"),
            70,
            format!("{prefix}released\n"),
        ),
    ] {
        let scratch = Scratch::new();
        let source = scratch.0.join(format!("{label}.ssrg"));
        fs::write(&source, &text).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.0.join(profile);
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&source)
                    .args(["--target", "process", "--profile", profile, "--out-dir"])
                    .arg(&output),
                &scratch.0,
            ));
            check_raw_strict(&text, profile, value, &output, &scratch.0);
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for engine in ["bun", "node"] {
                let result = bounded(
                    Command::new(runtime(engine, &scratch.0))
                        .arg(output.join(manifest["entry"].as_str().unwrap()))
                        .env("PATH", ""),
                    &scratch.0,
                );
                assert_eq!(
                    result.status.code(),
                    Some(code),
                    "{label}/{profile}/{engine}: {result:?}"
                );
                assert_eq!(
                    result.stdout,
                    expected.as_bytes(),
                    "{label}/{profile}/{engine}"
                );
                if code == 0 {
                    assert!(
                        result.stderr.is_empty(),
                        "{label}/{profile}/{engine}: {result:?}"
                    );
                } else {
                    assert_eq!(
                        result.stderr, b"seseragi: runtime defect\n",
                        "{label}/{profile}/{engine}"
                    );
                }
            }
        }
        let native = scratch.0.join("native");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let result = bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        );
        assert_eq!(result.status.code(), Some(code), "{label}: {result:?}");
        assert_eq!(result.stdout, expected.as_bytes(), "{label}: {result:?}");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
        if label == "defect" {
            assert!(stderr.contains("body defect"), "{stderr}");
        }
    }
}

#[test]
fn console_rejects_non_string_before_lint_build_native_and_run_outputs() {
    for import in ["", "import * as console from \"std/console\"\n"] {
        let name = if import.is_empty() {
            "println"
        } else {
            "console.println"
        };
        let scratch = Scratch::new();
        let source = scratch.0.join("invalid.ssrg");
        fs::write(
            &source,
            format!("{import}pub effect fn main = do {{ println \"unreachable\"; {name} True }}\n"),
        )
        .unwrap();
        for command in ["lint", "build", "run", "native"] {
            let output = scratch.0.join(command);
            let mut process = Command::new(env!("CARGO_BIN_EXE_seseragi"));
            if command == "native" {
                process.args(["experimental", "build-native"]);
            } else {
                process.arg(command);
            }
            process.arg(&source);
            if command == "native" {
                process.args(["--entry", "main"]);
            }
            if matches!(command, "build" | "native") {
                process.arg("--out-dir").arg(&output);
            }
            let result = bounded(&mut process, &scratch.0);
            assert!(!result.status.success(), "{command}: {result:?}");
            assert!(result.stdout.is_empty(), "{command}: {result:?}");
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("SES-T0101"),
                "{command}: {result:?}"
            );
            assert!(!output.exists(), "{command} emitted a rejected program");
        }
    }
}
