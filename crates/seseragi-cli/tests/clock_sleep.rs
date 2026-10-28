use sha2::{Digest, Sha256};
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
            "seseragi-native-clock-{}-{}",
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
        seseragi_driver::CompileInput::new("native-clock.ssrg", "proof/native-clock", text)
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

fn standalone_ts_package(source: &Path, scratch: &Path) -> PathBuf {
    // Native accepts the original standalone file with an explicit binding.
    // The ordinary TS process entry requires a resolved package provider.
    // Keep the body bytes, while deliberately using a different module identity.
    let package = scratch.join("ts-process-package");
    fs::create_dir(&package).unwrap();
    fs::create_dir(package.join("src")).unwrap();
    fs::write(package.join("src/main.ssrg"), fs::read(source).unwrap()).unwrap();
    fs::write(
        package.join("seseragi.toml"),
        concat!(
            "[package]\n",
            "name = \"acceptance/standalone-clock-oracle\"\n",
            "version = \"0.0.0\"\n",
            "language = \">=0.1.0 <0.2.0\"\n",
            "[run]\n",
            "entry = \"main\"\n",
            "target = \"process\"\n",
        ),
    )
    .unwrap();
    lock(&package, scratch);
    let parsed = seseragi_project::read_and_validate_lockfile(&package).unwrap();
    assert!(parsed
        .providers
        .iter()
        .any(|provider| provider.service == "std/clock::Clock"
            && provider.target == "bun-process"
            && provider.provider == "seseragi/runtime-bun#clock"));
    assert!(!parsed
        .providers
        .iter()
        .any(|provider| provider.provider == "seseragi/runtime-native#clock-sleep"));
    package
}

fn build_programs(
    source: &Path,
    project: bool,
    scratch: &Path,
) -> Vec<(String, PathBuf, Option<PathBuf>, bool)> {
    let native = scratch.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args([
                "--entry",
                "main",
                "--provider",
                "std/clock::Clock=seseragi/runtime-native#clock-sleep",
                "--out-dir",
            ])
            .arg(&native),
        &scratch,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    let ts_package = (!project).then(|| standalone_ts_package(source, scratch));
    let ts_input = ts_package.as_deref().unwrap_or(source);
    let frozen = ts_package.as_ref().map(|package| {
        ["seseragi.toml", "seseragi.lock", "src/main.ssrg"]
            .map(|name| (package.join(name), fs::read(package.join(name)).unwrap()))
    });
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(ts_input)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch,
        ));
        if project {
            let loaded = seseragi_project::load_local_project(&source).unwrap();
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            assert_eq!(compiled.compiled.modules.len(), 2);
            let raw = output.join("raw");
            fs::create_dir(&raw).unwrap();
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|c| c == std::path::Component::ParentDir)
                );
                let file = raw.join(relative);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            if profile == "development" {
                assert!(output.join("entry.ts").is_file());
                files.push(output.join("entry.ts"));
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"skipLibCheck":false,"allowImportingTsExtensions":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root().join("node_modules/@types")],"baseUrl":root(),"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2023","DOM","DOM.Iterable"]},"files":files});
            fs::write(
                output.join("project-raw.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(bounded(
                Command::new(runtime("node", &scratch))
                    .arg(root().join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(output.join("project-raw.json")),
                &scratch,
            ));
        } else {
            check_raw_strict(
                &fs::read_to_string(&source).unwrap(),
                profile,
                value,
                &output,
                &scratch,
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = output.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            programs.push((
                format!("{profile}-{engine}"),
                runtime(engine, &scratch),
                Some(entry.clone()),
                false,
            ));
        }
    }
    if let Some(frozen) = frozen {
        for (file, bytes) in frozen {
            assert_eq!(fs::read(&file).unwrap(), bytes, "{}", file.display());
        }
        assert_eq!(
            fs::read(ts_input.join("src/main.ssrg")).unwrap(),
            fs::read(source).unwrap()
        );
    }
    assert_eq!(programs.len(), 5);
    programs
}

const PROVIDER: &str = "std/clock::Clock=seseragi/runtime-native#clock-sleep";
const ORIGINAL: &str =
    include_str!("../../../examples/spec/fixtures/rust-backend/filesystem-delayed-transform.ssrg");
const OLD: &[u8] = b"old output survives a pending/cancelled/failed transform";
const INPUT: &[u8] = "初期値\n42\n".as_bytes();
const CREATED: &str = "created\n";
const ACQUIRED: &str = "acquired\npending\n";
const RELEASED: &str = "release-start\nreleased\n";

fn run_transform(
    programs: &[(String, PathBuf, Option<PathBuf>, bool)],
    directory: &Path,
    case: &str,
    input: Option<&[u8]>,
) {
    for (label, executable, argument, native) in programs {
        let cwd = directory.join(format!("{label}-{case}"));
        fs::create_dir(&cwd).unwrap();
        if let Some(input) = input {
            fs::write(cwd.join("input.txt"), input).unwrap();
        }
        fs::write(cwd.join("output.txt"), OLD).unwrap();
        let mut command = Command::new(executable);
        command.current_dir(&cwd).env("PATH", "");
        if let Some(argument) = argument {
            command.arg(argument);
        }
        if *native {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        let started = Instant::now();
        let result = bounded(&mut command, &cwd);
        let (code, wrote, expected_trace, reason, timers) = match case {
            "success" => (
                0,
                true,
                format!("{CREATED}{ACQUIRED}written\n{RELEASED}"),
                "",
                3,
            ),
            "replay" => (
                0,
                true,
                format!("{CREATED}{ACQUIRED}written\n{RELEASED}{ACQUIRED}written\n{RELEASED}"),
                "",
                6,
            ),
            "cancel" => (
                0,
                false,
                format!("{CREATED}{ACQUIRED}sibling\n{RELEASED}cancelled\n"),
                "",
                4,
            ),
            "defect" => (
                70,
                false,
                format!("{CREATED}{ACQUIRED}{RELEASED}"),
                "delayed body defect",
                3,
            ),
            "cleanup-defect" => (
                70,
                true,
                format!("{CREATED}{ACQUIRED}written\n{RELEASED}"),
                "delayed cleanup defect",
                3,
            ),
            "cancel-cleanup-defect" => (
                70,
                false,
                format!("{CREATED}{ACQUIRED}sibling\n{RELEASED}"),
                "delayed cleanup defect",
                4,
            ),
            "negative" => (
                1,
                false,
                format!("{CREATED}{ACQUIRED}{RELEASED}"),
                "negative delay: -1",
                2,
            ),
            "outside" => (
                1,
                false,
                format!("{CREATED}{ACQUIRED}{RELEASED}"),
                "delay outside range",
                2,
            ),
            "invalid-utf8" => (
                1,
                false,
                format!("{CREATED}{ACQUIRED}{RELEASED}"),
                "InvalidUtf8",
                3,
            ),
            "missing" => (
                1,
                false,
                format!("{CREATED}{ACQUIRED}{RELEASED}"),
                "FileNotFound",
                3,
            ),
            _ => unreachable!(),
        };
        assert_eq!(
            result.status.code(),
            Some(code),
            "{label}/{case}: {result:?}"
        );
        assert!(result.stdout.is_empty(), "{label}/{case}: {result:?}");
        assert_eq!(
            fs::read_to_string(cwd.join("trace.txt")).unwrap(),
            expected_trace,
            "{label}/{case}"
        );
        let mut processed = b"processed:\n".to_vec();
        processed.extend_from_slice(INPUT);
        assert_eq!(
            fs::read(cwd.join("output.txt")).unwrap(),
            if wrote { processed.as_slice() } else { OLD },
            "{label}/{case}"
        );
        if let Some(input) = input {
            assert_eq!(
                fs::read(cwd.join("input.txt")).unwrap(),
                input,
                "input unchanged: {label}/{case}"
            );
        } else {
            assert!(!cwd.join("input.txt").exists());
        }
        let stderr = String::from_utf8_lossy(&result.stderr);
        if *native {
            assert!(
                stderr.contains("managed-live=0"),
                "{label}/{case}: {stderr}"
            );
            assert!(stderr.contains("timers-live=0"), "{label}/{case}: {stderr}");
            assert!(
                stderr.contains(&format!("timers-created={timers}")),
                "{label}/{case}: {stderr}"
            );
            if case.starts_with("cancel") {
                let peak = stderr
                    .split("timers-peak=")
                    .nth(1)
                    .unwrap()
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                assert!(
                    peak >= 2,
                    "parent progressed while the child sleep was pending: {stderr}"
                );
            }
            if !reason.is_empty() {
                assert!(stderr.contains(reason), "{label}/{case}: {stderr}");
            }
        } else if code == 70 {
            assert_eq!(stderr, "seseragi: runtime defect\n", "{label}/{case}");
        } else if code == 0 {
            assert!(stderr.is_empty(), "{label}/{case}: {stderr}");
        } else {
            assert!(stderr.contains(reason), "{label}/{case}: {stderr}");
        }
        if case.starts_with("cancel") {
            // A five-second child deadline must not block its sibling's one-ms
            // action or cancellation. Allow generous host scheduling jitter.
            assert!(
                started.elapsed() < Duration::from_secs(4),
                "{label}/{case}: cancellation waited for the original timer"
            );
        }
    }
}

fn provider_receipt(directory: &Path) {
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("native/rust-backend.json")).unwrap())
            .unwrap();
    let providers = metadata["providers"].as_array().unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0]["target"], "native");
    assert_eq!(
        providers[0]["provider"],
        "seseragi/runtime-native#clock-sleep"
    );
    assert_eq!(providers[0]["selection"], "command-line");
    assert_eq!(
        providers[0]["contractVersion"],
        serde_json::json!({"major":1,"minor":0})
    );
    assert_eq!(
        providers[0]["contractDigest"],
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(
                "../../../examples/spec/artifacts/provider-contract-schema-1/clock/contract.json"
            ))
        )
    );
    assert_eq!(
        providers[0]["supportedOperations"],
        serde_json::json!(["std/clock::Clock#sleep"])
    );
    assert!(!providers[0]["traces"].as_array().unwrap().is_empty());
}

#[test]
fn delayed_file_transform_keeps_cold_replay_failures_cancel_and_masked_cleanup_on_all_targets() {
    let scratch = Scratch::new();
    let selector = "caseMain \"success\" 30";
    assert_eq!(ORIGINAL.matches(selector).count(), 1);
    for (case, call) in [
        ("success", selector),
        ("replay", "caseMain \"replay\" 0"),
        ("cancel", "caseMain \"cancel\" 5000"),
        ("defect", "caseMain \"defect\" 0"),
        ("cleanup-defect", "caseMain \"cleanup-defect\" 0"),
        (
            "cancel-cleanup-defect",
            "caseMain \"cancel-cleanup-defect\" 5000",
        ),
        ("negative", "caseMain \"success\" (-1)"),
        ("outside", "caseMain \"success\" 9223372036855"),
    ] {
        let directory = scratch.0.join(case);
        fs::create_dir(&directory).unwrap();
        let source = directory.join("original.ssrg");
        let bytes = ORIGINAL.replacen(selector, call, 1);
        fs::write(&source, &bytes).unwrap();
        let programs = build_programs(&source, false, &directory);
        provider_receipt(&directory);
        run_transform(&programs, &directory, case, Some(INPUT));
        if case == "success" {
            run_transform(&programs, &directory, "invalid-utf8", Some(b"\xff"));
            run_transform(&programs, &directory, "missing", None);
        }
        assert_eq!(fs::read_to_string(&source).unwrap(), bytes);
    }
}

fn copy_package(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_package(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
fn lock(package: &Path, scratch: &Path) {
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(package),
        scratch,
    ));
}

#[test]
fn normal_path_package_keeps_target_shared_lock_and_explicit_native_binding_separate() {
    let scratch = Scratch::new();
    let package = scratch.0.join("package");
    copy_package(
        &root().join("examples/spec/fixtures/rust-backend-projects/delayed-file-transform"),
        &package,
    );
    assert_eq!(
        fs::read(package.join("vendor/operation/src/operation.ssrg")).unwrap(),
        ORIGINAL.as_bytes()
    );
    lock(&package, &scratch.0);
    let frozen = [
        "seseragi.lock",
        "seseragi.toml",
        "src/main.ssrg",
        "vendor/operation/seseragi.toml",
        "vendor/operation/src/operation.ssrg",
    ]
    .map(|name| (name, fs::read(package.join(name)).unwrap()));
    let parsed = seseragi_project::read_and_validate_lockfile(&package).unwrap();
    assert!(parsed
        .providers
        .iter()
        .any(|provider| provider.service == "std/clock::Clock"
            && provider.target == "bun-process"
            && provider.provider == "seseragi/runtime-bun#clock"));
    assert!(!parsed
        .providers
        .iter()
        .any(|provider| provider.provider == "seseragi/runtime-native#clock-sleep"));
    let programs = build_programs(&package, true, &scratch.0);
    provider_receipt(&scratch.0);
    run_transform(&programs, &scratch.0, "success", Some(INPUT));
    for (name, bytes) in frozen {
        assert_eq!(
            fs::read(package.join(name)).unwrap(),
            bytes,
            "target-shared source and lock unchanged: {name}"
        );
    }
}

#[test]
fn provider_preflight_never_publishes_implicit_or_javascript_clock_selection() {
    let scratch = Scratch::new();
    let source = scratch.0.join("original.ssrg");
    fs::write(&source, ORIGINAL).unwrap();
    for (case, flags) in [
        ("missing", vec![]),
        (
            "bun",
            vec!["--provider", "std/clock::Clock=seseragi/runtime-bun#clock"],
        ),
        (
            "service",
            vec![
                "--provider",
                "std/fs::FileSystem=seseragi/runtime-native#clock-sleep",
            ],
        ),
        (
            "duplicate",
            vec!["--provider", PROVIDER, "--provider", PROVIDER],
        ),
    ] {
        let output = scratch.0.join(case).join("program.rs");
        let result = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "emit-rust"])
                .arg(&source)
                .args(["--entry", "main", "--output"])
                .arg(&output)
                .args(&flags),
            &scratch.0,
        );
        assert!(!result.status.success(), "{case}: {result:?}");
        assert!(result.stdout.is_empty(), "{case}: {result:?}");
        assert!(
            !output.parent().unwrap().exists(),
            "{case}: admission must precede output staging"
        );
        assert_eq!(fs::read(&source).unwrap(), ORIGINAL.as_bytes());
        if case == "missing" {
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(
                error.contains("SES-RUST-0001") && error.contains("original.ssrg:"),
                "{error}"
            );
        }
    }
}

#[test]
fn unused_clock_pin_keeps_existing_non_clock_entry_while_used_clock_pin_is_rejected() {
    let scratch = Scratch::new();
    for (case, body, accepted, identity) in [
        (
            "unused",
            "pub fn main -> Int = 42\n",
            true,
            "seseragi/runtime-bun#clock",
        ),
        ("used", ORIGINAL, false, "seseragi/runtime-bun#clock"),
        (
            "unused-native-pin",
            "pub fn main -> Int = 42\n",
            true,
            "seseragi/runtime-native#clock-sleep",
        ),
        (
            "used-native-pin",
            ORIGINAL,
            false,
            "seseragi/runtime-native#clock-sleep",
        ),
    ] {
        let package = scratch.0.join(case);
        fs::create_dir(&package).unwrap();
        fs::create_dir(package.join("src")).unwrap();
        fs::write(package.join("src/main.ssrg"), body).unwrap();
        fs::write(package.join("seseragi.toml"), format!("[package]\nname = \"acceptance/pinned-clock\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n[run]\nentry = \"main\"\ntarget = \"process\"\n[providers]\n\"std/clock::Clock\" = {identity:?}\n")).unwrap();
        lock(&package, &scratch.0);
        let frozen = ["seseragi.lock", "seseragi.toml", "src/main.ssrg"]
            .map(|name| (name, fs::read(package.join(name)).unwrap()));
        let output = scratch.0.join(format!("{case}-output")).join("program.rs");
        if accepted {
            fs::create_dir(output.parent().unwrap()).unwrap();
        }
        let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
        command
            .args(["experimental", "emit-rust"])
            .arg(&package)
            .args(["--entry", "main", "--output"])
            .arg(&output);
        if !accepted {
            command.args(["--provider", PROVIDER]);
        }
        let result = bounded(&mut command, &scratch.0);
        assert_eq!(result.status.success(), accepted, "{case}: {result:?}");
        if accepted {
            assert!(output.is_file());
            let metadata: serde_json::Value = serde_json::from_slice(
                &fs::read(output.with_extension("rust-backend.json")).unwrap(),
            )
            .unwrap();
            assert!(metadata.get("providers").is_none());
        } else {
            assert!(result.stdout.is_empty());
            assert!(!output.parent().unwrap().exists());
            let error = String::from_utf8_lossy(&result.stderr);
            assert!(
                error.contains("SES-RUST-0001")
                    && error.contains("seseragi.toml:")
                    && error.contains("manifest provider"),
                "{error}"
            );
        }
        for (name, bytes) in frozen {
            assert_eq!(
                fs::read(package.join(name)).unwrap(),
                bytes,
                "{case}/{name}"
            );
        }
    }
    for case in ["dependency-pin", "root-artifact", "dependency-artifact"] {
        let package = scratch.0.join(case);
        copy_package(
            &root().join("examples/spec/fixtures/rust-backend-projects/delayed-file-transform"),
            &package,
        );
        let owner = if case == "root-artifact" {
            package.clone()
        } else {
            package.join("vendor/operation")
        };
        let manifest = owner.join("seseragi.toml");
        let mut body = fs::read_to_string(&manifest).unwrap();
        let mut artifact_path = None;
        if case == "dependency-pin" {
            body.push_str("\n[providers]\n\"std/clock::Clock\" = \"seseragi/runtime-bun#clock\"\n");
        } else {
            body.push_str("\n[provider]\nartifacts = [\"clock.json\"]\n");
            let path = owner.join("clock.json");
            let mut provider: serde_json::Value = serde_json::from_slice(include_bytes!("../../../examples/spec/artifacts/provider-manifest-schema-1/bun-clock/provider.json")).unwrap();
            provider["identity"] = serde_json::Value::String(
                if case == "root-artifact" {
                    "acceptance/delayed-file-transform#clock"
                } else {
                    "acceptance/delayed-file-operation#clock"
                }
                .into(),
            );
            fs::write(&path, serde_json::to_vec_pretty(&provider).unwrap()).unwrap();
            artifact_path = Some(path);
        }
        fs::write(&manifest, body).unwrap();
        lock(&package, &scratch.0);
        seseragi_project::read_and_validate_lockfile(&package).unwrap();
        let mut frozen = [
            "seseragi.lock",
            "seseragi.toml",
            "src/main.ssrg",
            "vendor/operation/seseragi.toml",
            "vendor/operation/src/operation.ssrg",
        ]
        .map(|name| package.join(name))
        .into_iter()
        .collect::<Vec<_>>();
        frozen.extend(artifact_path);
        let frozen = frozen
            .into_iter()
            .map(|path| {
                let bytes = fs::read(&path).unwrap();
                (path, bytes)
            })
            .collect::<Vec<_>>();
        let output = scratch.0.join(format!("{case}-output")).join("program.rs");
        let result = bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "emit-rust"])
                .arg(&package)
                .args(["--entry", "main", "--provider", PROVIDER, "--output"])
                .arg(&output),
            &scratch.0,
        );
        assert!(!result.status.success(), "{case}: {result:?}");
        assert!(result.stdout.is_empty());
        assert!(!output.parent().unwrap().exists());
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains("SES-RUST-0001")
                && error.contains("seseragi.toml:")
                && error.contains("provider artifacts and dependency provider selections"),
            "{case}: {error}"
        );
        for (path, bytes) in frozen {
            assert_eq!(
                fs::read(path).unwrap(),
                bytes,
                "{case}: source and provider metadata remain unchanged"
            );
        }
    }
}
