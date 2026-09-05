use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static NEXT_LOCKED_PROJECT_ID: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

struct LockedProject {
    root: PathBuf,
    path: PathBuf,
}

impl LockedProject {
    fn copy(source: &Path) -> Self {
        fn copy_directory(source: &Path, destination: &Path) {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let from = entry.path();
                let to = destination.join(entry.file_name());
                if from.is_dir() {
                    copy_directory(&from, &to);
                } else if entry.file_name() != "seseragi.lock" {
                    fs::copy(from, to).unwrap();
                }
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_LOCKED_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "seseragi-run-locked-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        let path = root.join("project");
        copy_directory(source, &path);
        let updated = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            updated.status.success(),
            "{}",
            String::from_utf8_lossy(&updated.stderr)
        );
        Self { root, path }
    }
}

impl Deref for LockedProject {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl AsRef<Path> for LockedProject {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<OsStr> for LockedProject {
    fn as_ref(&self) -> &OsStr {
        self.path.as_os_str()
    }
}

impl Drop for LockedProject {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn assert_target_mismatch(output: &std::process::Output) {
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in ["SES-K0203 provider.target-mismatch", "browser"] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in {stderr}"
        );
    }
    if stderr.contains("target mismatch before execution") {
        for expected in [
            "required capabilities: dom",
            "selected target: process",
            "selected target capabilities: console, logger, stdin, process",
            "missing capabilities: dom",
            "available target contracts: browser",
        ] {
            assert!(
                stderr.contains(expected),
                "missing {expected:?} in {stderr}"
            );
        }
    } else {
        for expected in [
            "standard module `std/web/dom`",
            "target: bun-process",
            "compatible targets: browser",
        ] {
            assert!(
                stderr.contains(expected),
                "missing {expected:?} in {stderr}"
            );
        }
    }
    assert!(!stderr.contains("runtime defect"));
}

#[test]
fn rejects_unsupported_dom_before_single_file_and_project_execution() {
    let fixtures = repository_root().join("crates/seseragi-cli/tests/fixtures");
    let file = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(fixtures.join("target-mismatch.ssrg"))
        .output()
        .unwrap();
    assert_target_mismatch(&file);
    for project in [
        fixtures.join("target-mismatch-project"),
        repository_root().join("examples/spec/fixtures/projects/std-parity-target"),
    ] {
        let project = LockedProject::copy(&project);
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&project)
            .output()
            .unwrap();
        assert_target_mismatch(&output);
    }
}

#[test]
fn rejects_browser_only_file_module_at_the_import_on_process() {
    let project = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/file-target-mismatch"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for expected in [
        "SES-K0203 provider.target-mismatch",
        "std/web/file",
        "target `bun-process`",
        "src/main.ssrg:0..36",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected:?} in {stderr}"
        );
    }
}

#[test]
fn runs_the_phase_one_program_without_fixture_metadata() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"rock\nscissors\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Player 1 wins!\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_stdin_lines_project_with_console_and_structured_logging() {
    let package =
        LockedProject::copy(&repository_root().join("examples/spec/fixtures/projects/stdin-lines"));
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&fs::read(package.join("input.txt")).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        fs::read(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(
        output.stderr,
        fs::read(package.join("expected.stderr")).unwrap()
    );
}

#[test]
fn renders_typed_failure_and_preserves_the_program_exit_class() {
    let root = repository_root();
    let program = root.join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "UnknownHand lizard\n"
    );
}

#[test]
fn renders_unhandled_typed_failure_as_one_json_runtime_diagnostic() {
    let program = repository_root()
        .join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(program)
        .args(["--diagnostic-format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["schema"], 1);
    assert_eq!(diagnostic["kind"], "TypedFailure");
    assert_eq!(diagnostic["phase"], serde_json::Value::Null);
    assert_eq!(diagnostic["message"], "UnknownHand lizard");
    assert_eq!(diagnostic["groups"], serde_json::json!([]));
}

#[test]
fn runs_the_manifest_discovered_split_phase_one_program() {
    let package = LockedProject::copy(
        &repository_root()
            .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split"),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(".")
        .current_dir(&package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"paper\nrock\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Player 1 wins!\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn renders_a_split_package_typed_failure() {
    let package = LockedProject::copy(
        &repository_root()
            .join("examples/spec/artifacts/project-schema-1/rock-paper-scissors-cli-split"),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"lizard\n").unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "InputFailure UnknownHand lizard\n"
    );
}

#[test]
fn runs_a_local_path_dependency_package() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/package-path-dependency-basic"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_portable_standard_parity_package() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/std-parity-portable"),
    );
    let expected = std::fs::read_to_string(package.join("expected.stdout")).unwrap();
    let source_entry = package.join("src/main.ssrg");
    for entry in [package.as_ref(), source_entry.as_path()] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(entry)
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

#[test]
fn runs_imported_derived_json_codecs() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/imported-derived-json-codecs"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_inline_polymorphic_inference() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/inline-polymorphic-inference"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_imported_derived_structural_instances() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/imported-derived-structural"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_temporal_control() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/effect-temporal-control"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_reproducible_random_seed() {
    for fixture in ["random-seed", "random-shuffle"] {
        let package = LockedProject::copy(
            &repository_root()
                .join("examples/spec/fixtures/projects")
                .join(fixture),
        );
        let first = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&package)
            .output()
            .unwrap();
        let second = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&package)
            .output()
            .unwrap();

        let expected = std::fs::read_to_string(package.join("expected.stdout")).unwrap();
        for output in [first, second] {
            assert_eq!(
                output.status.code(),
                Some(0),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
            assert_eq!(String::from_utf8_lossy(&output.stderr), "");
        }
    }
}

#[test]
fn command_line_random_seed_overrides_the_manifest_for_one_run() {
    let package =
        LockedProject::copy(&repository_root().join("examples/spec/fixtures/projects/random-seed"));
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&package)
            .args(["--random-seed", "17"])
            .output()
            .unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    assert_ne!(
        first.stdout,
        fs::read(package.join("expected.stdout")).unwrap(),
        "the manifest seed must not win over the invocation"
    );
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");
    assert_eq!(String::from_utf8_lossy(&second.stderr), "");
}

#[test]
fn runs_pinned_timezones_with_explicit_dst_resolution() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/timezones-dst"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_filesystem_temporary_cleanup() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/filesystem-temporary-cleanup"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_captured_child_process_from_relative_and_absolute_package_paths() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/child-process-captured"),
    );
    let source = std::fs::read_to_string(package.join("src/main.ssrg")).unwrap();
    assert!(!source.contains("runtime-bun"));
    assert!(!source.contains("node:child_process"));

    let expected = std::fs::read_to_string(package.join("expected.stdout")).unwrap();
    assert_child_process_run(&package, Path::new("."), &expected);
    assert_child_process_run(&package.root, Path::new("./project"), &expected);
    assert_child_process_run(&repository_root(), &package, &expected);
}

fn assert_child_process_run(current_directory: &Path, package: &Path, expected: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .current_dir(current_directory)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_concurrency_primitives() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/effect-concurrency-primitives"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_tail_recursive_queue_worker_to_completion() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/effect-tail-recursion"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[cfg(unix)]
fn interrupt_process_target(mut child: std::process::Child) -> std::process::Output {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let children = Command::new("pgrep")
            .args(["-P", &child.id().to_string()])
            .output()
            .unwrap();
        if children.status.success() {
            if let Some(pid) = String::from_utf8_lossy(&children.stdout)
                .split_whitespace()
                .next()
            {
                std::thread::sleep(Duration::from_millis(500));
                let interrupted = Command::new("kill").args(["-INT", pid]).status().unwrap();
                assert!(interrupted.success());
                return child.wait_with_output().unwrap();
            }
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "Seseragi process target did not start\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(unix)]
#[test]
fn forwards_interrupt_to_the_process_signal_stream() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/process-shutdown-forward"),
    );
    let child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = interrupt_process_target(child);

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        fs::read(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn command_line_signal_policy_overrides_the_manifest_for_one_run() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/process-shutdown-forward"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .args([
            "--signal-mode",
            "cancel",
            "--shutdown-grace-ms",
            "0",
            "--diagnostic-format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["kind"], "TypedFailure");
    assert_eq!(diagnostic["phase"], serde_json::Value::Null);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("ReservedProcessSignal Interrupt"));
}

#[cfg(unix)]
#[test]
fn cancels_the_root_effect_and_preserves_the_signal_exit_status() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/process-shutdown-cancel"),
    );
    let child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let output = interrupt_process_target(child);

    assert_eq!(output.status.code(), Some(130));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_effect_resource_scope() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/effect-resource-scope"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_stream_cold_resource() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/stream-cold-resource"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_typeclass_operator_parity_project() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/typeclass-operator-parity"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_standard_evidence_parity_project() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/standard-evidence-parity"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn diagnoses_missing_signal_monad_operator_instance() {
    let source = repository_root()
        .join("crates/seseragi-cli/tests/fixtures/typeclass-signal-monad-negative.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(source)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[SES-T0201]"), "{stderr}");
    assert!(stderr.contains("no Monad instance matches"), "{stderr}");
    assert!(!stderr.contains("runtime defect"), "{stderr}");
}

#[test]
fn diagnoses_intentionally_unavailable_float_eq_evidence() {
    let source = repository_root()
        .join("crates/seseragi-cli/tests/fixtures/standard-evidence-float-eq-negative.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(source)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[SES-T0201]"), "{stderr}");
    assert!(
        stderr.contains("no Eq instance matches the inferred call arguments"),
        "{stderr}"
    );
    assert!(!stderr.contains("runtime defect"), "{stderr}");
}

#[test]
fn diagnoses_intentionally_unavailable_float_hash_evidence() {
    let source = repository_root()
        .join("crates/seseragi-cli/tests/fixtures/standard-evidence-float-hash-negative.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(source)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[SES-T0201]"), "{stderr}");
    assert!(
        stderr.contains("no Hash instance matches the inferred call arguments"),
        "{stderr}"
    );
    assert!(!stderr.contains("runtime defect"), "{stderr}");
}

#[test]
fn runs_effect_stream_simultaneous_failure() {
    let package = LockedProject::copy(
        &repository_root()
            .join("examples/spec/fixtures/projects/effect-stream-simultaneous-failure"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_small_response_http_client_surface_from_seseragi_source() {
    let listener = TcpListener::bind(("127.0.0.1", 41287)).unwrap();
    let server = std::thread::spawn(move || {
        let (mut connection, _) = listener.accept().unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let count = connection.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            let Some(head_end) = request.windows(4).position(|part| part == b"\r\n\r\n") else {
                continue;
            };
            let head = String::from_utf8_lossy(&request[..head_end + 4]);
            let content_length = head
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|value| value.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= head_end + 4 + content_length {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request);
        assert!(
            request.starts_with("POST /upload HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains("x-seseragi: small-response\r\n"),
            "{request}"
        );
        assert!(request.ends_with("seseragi"), "{request}");
        connection
            .write_all(
                b"HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nContent-Length: 5\r\nConnection: close\r\n\r\nready",
            )
            .unwrap();
    });

    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/provider-http-client-e2e"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_streaming_http_events_with_demand_and_trailers() {
    let listener = TcpListener::bind(("127.0.0.1", 41288)).unwrap();
    let server = std::thread::spawn(move || {
        let (mut connection, _) = listener.accept().unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|part| part == b"\r\n\r\n") {
            let count = connection.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
        }
        let request = String::from_utf8_lossy(&request);
        assert!(request.starts_with("GET /stream HTTP/1.1\r\n"), "{request}");
        connection
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nTrailer: x-end\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n1\r\na\r\n",
            )
            .unwrap();
        connection.flush().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        connection.write_all(b"2\r\nbc\r\n0\r\nx-").unwrap();
        connection.flush().unwrap();
        std::thread::sleep(Duration::from_millis(100));
        connection.write_all(b"end: yes\r\n\r\n").unwrap();
    });

    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/http-stream-events"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    server.join().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_a_real_bun_http_provider_from_seseragi_source() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/provider-http-server-e2e"),
    );
    let source = std::fs::read_to_string(package.join("src/main.ssrg")).unwrap();
    assert!(!source.contains("runtime-bun"));
    assert!(!source.contains("seseragi/runtime-bun#http-server"));
    run_http_server_fixture(
        &package,
        b"{\"name\":\"Mio\"}",
        "201",
        "{\"id\":42,\"name\":\"Mio\"}",
    );
    run_http_server_fixture(&package, b"not json", "400", "invalid json");
    run_http_server_fixture(&package, &[0xff], "400", "invalid utf-8");
}

#[test]
fn runs_websocket_client_and_server_from_seseragi_source() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/provider-websocket-e2e"),
    );
    let source = std::fs::read_to_string(package.join("src/main.ssrg")).unwrap();
    assert!(!source.contains("runtime-bun"));
    assert!(!source.contains("Bun.serve"));

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_sse_server_and_client_from_seseragi_source() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/sse-server-client-e2e"),
    );
    let source = std::fs::read_to_string(package.join("src/main.ssrg")).unwrap();
    assert!(!source.contains("runtime-bun"));
    assert!(!source.contains("EventSource"));

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn run_http_server_fixture(
    package: &Path,
    body: &[u8],
    expected_status: &str,
    expected_body: &str,
) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let connection = (0..2400).find_map(|_| match TcpStream::connect(("127.0.0.1", 41286)) {
        Ok(stream) => Some(stream),
        Err(_) => {
            std::thread::sleep(Duration::from_millis(25));
            None
        }
    });
    let Some(mut connection) = connection else {
        let _ = child.kill();
        let output = child.wait_with_output().unwrap();
        panic!(
            "Seseragi HTTP provider did not start\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    let mut request = format!(
        "POST /users?source=e2e HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(body);
    connection.write_all(&request).unwrap();
    let mut response = String::new();
    connection.read_to_string(&mut response).unwrap();
    assert!(
        response.starts_with(&format!("HTTP/1.1 {expected_status}")),
        "{response}"
    );
    if expected_status == "201" {
        assert!(
            response
                .to_ascii_lowercase()
                .contains("content-type: application/json; charset=utf-8"),
            "{response}"
        );
        assert!(
            response
                .to_ascii_lowercase()
                .contains("x-seseragi-method: post"),
            "{response}"
        );
    }
    assert!(response.contains(expected_body), "{response}");

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_only_entry_reachable_modules_in_a_local_project() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/entry-rooted-runtime"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_foreign_typescript_project_fixtures() {
    for fixture in [
        "foreign-failure-phases",
        "foreign-pure-load",
        "foreign-task-load",
        "foreign-task-single-flight",
    ] {
        let package = LockedProject::copy(
            &repository_root()
                .join("examples/spec/fixtures/projects")
                .join(fixture),
        );
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&package)
            .output()
            .unwrap();

        assert_eq!(
            output.status.code(),
            Some(0),
            "{fixture}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            std::fs::read_to_string(package.join("expected.stdout")).unwrap(),
            "{fixture}"
        );
        assert_eq!(String::from_utf8_lossy(&output.stderr), "", "{fixture}");
    }
}

#[test]
fn runs_source_map_rejection_as_a_typed_foreign_failure() {
    let fixture = repository_root().join("examples/spec/fixtures/projects/source-map-rejection");
    let package = LockedProject::copy(&fixture);
    let expectation: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(fixture.join("project.expect.json")).unwrap())
            .unwrap();
    let arguments = expectation["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|argument| argument.as_str().unwrap())
        .collect::<Vec<_>>();
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .args(arguments)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        std::fs::read_to_string(package.join("expected.stderr")).unwrap()
    );
}

#[test]
fn renders_runtime_defects_as_json_without_changing_the_exit_class() {
    let package = LockedProject::copy(
        &repository_root().join("crates/seseragi-cli/tests/fixtures/runtime-defect-project"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .args(["--diagnostic-format", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(70));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let diagnostic: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["schema"], 1);
    assert_eq!(diagnostic["kind"], "Defect");
    assert_eq!(diagnostic["phase"], serde_json::Value::Null);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("foreign pure binding explode threw"));
    assert!(diagnostic["groups"].is_array());
}

#[test]
fn invocation_target_overrides_the_manifest_without_rewriting_it() {
    let package = LockedProject::copy(
        &repository_root().join("crates/seseragi-cli/tests/fixtures/run-option-project"),
    );
    let manifest_before = fs::read(package.join("seseragi.toml")).unwrap();
    let unsupported = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(unsupported.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unsupported.stderr)
        .contains("run does not support the `web` target"));

    let overridden = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["run", "--target", "process"])
        .arg(&package)
        .output()
        .unwrap();
    assert_eq!(
        overridden.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&overridden.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&overridden.stdout),
        "process override\n"
    );
    assert_eq!(
        fs::read(package.join("seseragi.toml")).unwrap(),
        manifest_before
    );
}

#[test]
fn single_file_and_package_compile_diagnostics_share_the_json_envelope() {
    let fixture = repository_root()
        .join("crates/seseragi-cli/tests/fixtures/typeclass-signal-monad-negative.ssrg");
    let single = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&fixture)
        .args(["--diagnostic-format", "json"])
        .output()
        .unwrap();

    let package = LockedProject::copy(
        &repository_root().join("crates/seseragi-cli/tests/fixtures/compile-diagnostic-project"),
    );
    let project = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .args(["--target", "process", "--diagnostic-format", "json"])
        .output()
        .unwrap();

    for output in [single, project] {
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(envelope["schema"], 1);
        assert_eq!(envelope["toolVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(envelope["languageVersion"], "0.1.0");
        assert_eq!(
            envelope["unicodeVersion"],
            seseragi_release::UNICODE_VERSION
        );
        let diagnostics = envelope["diagnostics"].as_array().unwrap();
        assert!(!diagnostics.is_empty());
        assert!(diagnostics
            .iter()
            .any(|document| document["diagnostics"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["code"] == "SES-T0201")));
    }
}

#[test]
fn rejects_invalid_run_options_before_execution_and_documents_the_contract() {
    let source = repository_root()
        .join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    for arguments in [
        vec!["--diagnostic-format", "human"],
        vec!["--target", "browser"],
        vec!["--random-seed", "not-an-integer"],
        vec!["--signal-mode", "forward", "--shutdown-grace-ms", "1"],
        vec!["--target", "process", "--target", "process"],
        vec!["--unknown"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("run")
            .arg(&source)
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        assert!(String::from_utf8_lossy(&output.stderr).starts_with("seseragi: "));
    }

    let help = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&help.stdout);
    for expected in [
        "--target process|web",
        "--diagnostic-format text|json",
        "--signal-mode cancel|forward",
        "--shutdown-grace-ms ms",
        "--hash-seed entropy|int",
        "--random-seed entropy|int",
    ] {
        assert!(stdout.contains(expected), "missing {expected:?} in help");
    }
}

#[test]
fn runs_logical_conditions_with_branch_values_and_short_circuiting() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/logical-short-circuit"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_canonical_reduce_with_curried_lambdas() {
    let package = LockedProject::copy(
        &repository_root().join("examples/spec/fixtures/projects/prelude-reduce-lambda"),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        std::fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn runs_the_postgres_package_and_preserves_driver_failure_as_typed() {
    let package = repository_root().join("examples/spec/fixtures/projects/postgres-application");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DriverFailure DriverError"), "{stderr}");
    assert!(stderr.contains("operation: query"), "{stderr}");
    assert!(stderr.contains("code: ECONNREFUSED"), "{stderr}");
    assert!(!stderr.contains("runtime defect"), "{stderr}");
    assert!(!stderr.contains("runtime-postgres#pg"), "{stderr}");
}

#[test]
fn runs_the_sqlite_package_with_commit_rollback_and_cleanup() {
    let package = repository_root().join("examples/spec/fixtures/projects/sqlite-application");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(package)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn reports_compiler_diagnostics_with_source_ranges() {
    let program = repository_root()
        .join("examples/spec/artifacts/semantic-diagnostics-schema-1/unknown-pure-name/main.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&program)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(&format!("{}:1:", program.display())));
    assert!(stderr.contains("error[SES-N0001]: Name could not be resolved"));
    assert!(stderr.contains("pub fn useMissing value: Int -> Int = missing"));
    assert!(stderr.contains("= help:"));
    assert!(!stderr.contains("name.unresolved"));
}

#[test]
fn formats_a_file_and_supports_check_mode() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("seseragi-format-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("main.ssrg");
    std::fs::write(
        &source_path,
        "pub fn identity value: Int -> Int =   \r\n      value   \r\n",
    )
    .unwrap();

    let before = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(before.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&before.stderr).contains("not canonically formatted"));

    let formatted = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("format")
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(formatted.status.code(), Some(0));
    assert_eq!(
        std::fs::read_to_string(&source_path).unwrap(),
        "pub fn identity value: Int -> Int = value\n"
    );

    let after = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(&source_path)
        .output()
        .unwrap();
    assert_eq!(after.status.code(), Some(0));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn phase_one_goal_program_passes_format_check() {
    let program = repository_root()
        .join("examples/spec/artifacts/schema-1/rock-paper-scissors-cli/main.ssrg");
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["format", "--check"])
        .arg(program)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
