#[path = "../../test-support/native_cache.rs"]
mod native_cache;

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
            "seseragi-time-calendar-{}-{}",
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
        native_cache::configure(command);
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
        seseragi_driver::CompileInput::new("time-calendar.ssrg", "proof/time-calendar", text)
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
fn calendar_matches_strict_ts_bun_node_and_native_in_both_profiles() {
    let scratch = Scratch::new();
    let source = scratch.0.join("main.ssrg");
    let original = include_str!("../../../examples/spec/fixtures/rust-backend/time-calendar.ssrg");
    let expected =
        include_bytes!("../../../examples/spec/fixtures/rust-backend/time-calendar.stdout");
    fs::write(&source, original).unwrap();
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let native = scratch.0.join(format!("native-{profile}"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--profile", profile, "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(native.join("rust-backend.json")).unwrap()).unwrap();
        assert!(metadata["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "time-calendar"));
        assert!(metadata.get("providers").is_none());
        let generated = fs::read_to_string(native.join("program.rs")).unwrap();
        assert!(generated.contains("use seseragi_native_runtime::*;"));
        assert!(!generated.contains("pub enum V"));
        let output = successful(bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .current_dir(&scratch.0)
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        ));
        assert_eq!(output.stdout, expected, "native {profile}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));

        let ts = scratch.0.join(format!("ts-{profile}"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&ts),
            &scratch.0,
        ));
        check_raw_strict(original, profile, value, &ts, &scratch.0);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(ts.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(&scratch.0)
                    .env("PATH", ""),
                &scratch.0,
            ));
            assert_eq!(output.stdout, expected, "{engine} {profile}");
            assert!(output.stderr.is_empty(), "{engine} {profile}: {output:?}");
        }
    }
    assert_eq!(fs::read_to_string(source).unwrap(), original);
}

#[test]
fn timezones_match_strict_ts_bun_node_and_native_in_both_profiles() {
    let scratch = Scratch::new();
    let source = scratch.0.join("timezones.ssrg");
    let original = include_str!("../../../examples/spec/fixtures/rust-backend/timezones.ssrg");
    let expected = include_bytes!("../../../examples/spec/fixtures/rust-backend/timezones.stdout");
    fs::write(&source, original).unwrap();
    for (profile, value) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let native = scratch.0.join(format!("native-{profile}"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--profile", profile, "--out-dir"])
                .arg(&native),
            &scratch.0,
        ));
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(native.join("rust-backend.json")).unwrap()).unwrap();
        assert!(metadata["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "time-zones"));
        let output = successful(bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .current_dir(&scratch.0)
                .env("PATH", "")
                .env("TZ", "Pacific/Honolulu")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &scratch.0,
        ));
        assert_eq!(output.stdout, expected, "native {profile}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
        let ts = scratch.0.join(format!("ts-{profile}"));
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&ts),
            &scratch.0,
        ));
        check_raw_strict(original, profile, value, &ts, &scratch.0);
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(ts.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(&scratch.0)
                    .env("PATH", "")
                    .env("TZ", "Asia/Tokyo"),
                &scratch.0,
            ));
            assert_eq!(output.stdout, expected, "{engine} {profile}");
            assert!(output.stderr.is_empty(), "{engine} {profile}: {output:?}");
        }
    }
    assert_eq!(fs::read_to_string(source).unwrap(), original);
}
