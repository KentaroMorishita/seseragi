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
            "seseragi-fibers-{}-{}",
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
struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
// Files keep output collection bounded even if an inherited stream stays open.
// Every child is killed and reaped on deadline or assertion unwind.
fn bounded(command: &mut Command, cwd: &Path) -> Output {
    let stdout = cwd.join("command.stdout");
    let stderr = cwd.join("command.stderr");
    command
        .stdout(Stdio::from(File::create(&stdout).unwrap()))
        .stderr(Stdio::from(File::create(&stderr).unwrap()));
    let mut child = Reap(command.spawn().unwrap());
    let deadline = Instant::now() + Duration::from_secs(120);
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "deadline for PID {}: {command:?}\n{}",
            child.0.id(),
            fs::read_to_string(&stderr).unwrap()
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    child.0.wait().unwrap();
    Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
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
fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}
fn runtime(name: &str) -> PathBuf {
    let output = Command::new(if cfg!(windows) { "where" } else { "which" })
        .arg(name)
        .output()
        .unwrap();
    assert!(output.status.success());
    PathBuf::from(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
}
fn compare(source: &str, stdout: &str, trace: Option<&str>, exit: i32, error: Option<&str>) {
    let scratch = Scratch::new();
    let input = scratch.0.join("main.ssrg");
    fs::write(&input, source).unwrap();
    let native = scratch.0.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![(
        native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }),
        None,
    )];
    for profile in ["development", "release"] {
        let dist = scratch.0.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(&input)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&dist),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(dist.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for name in ["bun", "node"] {
            programs.push((
                runtime(name),
                Some(dist.join(manifest["entry"].as_str().unwrap())),
            ));
        }
    }
    for (i, (executable, argument)) in programs.into_iter().enumerate() {
        let cwd = scratch.0.join(format!("run-{i}"));
        fs::create_dir(&cwd).unwrap();
        let native = argument.is_none();
        let mut command = Command::new(executable);
        if let Some(argument) = argument {
            command.arg(argument);
        }
        command
            .current_dir(&cwd)
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1");
        let output = bounded(&mut command, &cwd);
        assert_eq!(output.status.code(), Some(exit), "{output:?}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if native {
            assert!(stderr.contains("managed-live=0"), "{output:?}");
        }
        if let Some(error) = error {
            assert!(
                stderr.contains(if exit == 70 && !native {
                    "runtime defect"
                } else {
                    error
                }),
                "{output:?}"
            );
        }
        if let Some(trace) = trace {
            assert_eq!(fs::read_to_string(cwd.join("trace.txt")).unwrap(), trace);
        }
    }
}
#[test]
fn interrupted_child_releases_once_before_its_exit_is_observed() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/fiber-finalizer.ssrg"),
        "",
        Some(include_str!(
            "../../../examples/spec/fixtures/rust-backend/fiber-finalizer.trace"
        )),
        0,
        None,
    );
}
#[test]
fn deferred_completion_order_and_saved_operations_match_on_native_bun_and_node() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/deferred-order.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/deferred-order.stdout"),
        None,
        0,
        None,
    );
}
#[test]
fn fiber_results_and_generic_join_preserve_all_exit_channels() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/fiber-results.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/fiber-results.stdout"),
        None,
        0,
        None,
    );
}
#[test]
fn parent_scope_closes_children_on_success_failure_and_defect() {
    let source = include_str!("../../../examples/spec/fixtures/rust-backend/fiber-finalizer.ssrg");
    let prefix = &source[..source.find("  effects.interrupt worker").unwrap()];
    for (ending, exit, error) in [
        ("append \"parent\\n\"", 0, None),
        ("effects.fail \"parent-failure\"", 1, Some("parent-failure")),
        (
            "append (show (9007199254740991 + 1))",
            70,
            Some("Int overflow"),
        ),
    ] {
        let input = format!("{prefix}  {ending}\n}}\n");
        compare(
            &input,
            "",
            Some(if exit == 0 {
                "acquire\nparent\nrelease\n"
            } else {
                "acquire\nrelease\n"
            }),
            exit,
            error,
        );
    }
}

#[test]
fn fiber_generated_typescript_is_strict_in_both_profiles() {
    use seseragi_driver::{compile_module, BuildProfile, CompileInput};
    let scratch = Scratch::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for name in ["fiber-finalizer", "fiber-results", "deferred-order"] {
        let source = fs::read_to_string(
            root.join(format!("examples/spec/fixtures/rust-backend/{name}.ssrg")),
        )
        .unwrap();
        for (label, profile) in [
            ("development", BuildProfile::Development),
            ("release", BuildProfile::Release),
        ] {
            let compiled = compile_module(
                CompileInput::new("main.ssrg", "single-file/main", &source).with_profile(profile),
            )
            .unwrap();
            let path = scratch.0.join(format!("{name}-{label}.ts"));
            fs::write(&path, compiled.generated.typescript).unwrap();
            files.push(path);
        }
    }
    let config = scratch.0.join("tsconfig.json");
    fs::write(&config,serde_json::to_vec(&serde_json::json!({
        "compilerOptions":{"noEmit":true,"strict":true,"skipLibCheck":true,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","typeRoots":[root.join("node_modules/@types")],"paths":{"@seseragi/runtime/*":[root.join("runtime/ts/src/*").to_string_lossy()]}},"files":files
    })).unwrap()).unwrap();
    successful(bounded(
        Command::new(runtime("node"))
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        &scratch.0,
    ));
}
