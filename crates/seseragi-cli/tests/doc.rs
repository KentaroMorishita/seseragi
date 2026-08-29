use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_PROJECT: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

struct LockedProject {
    root: PathBuf,
    path: PathBuf,
}

impl LockedProject {
    fn doc_tests() -> Self {
        fn copy_directory(source: &Path, destination: &Path) {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let from = entry.path();
                let to = destination.join(entry.file_name());
                if from.is_dir() {
                    copy_directory(&from, &to);
                } else {
                    fs::copy(from, to).unwrap();
                }
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let sequence = NEXT_PROJECT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "seseragi-doc-tests-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        let path = root.join("project");
        copy_directory(
            &repository_root().join("examples/spec/fixtures/projects/doc-tests"),
            &path,
        );
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(output.status.success());
        Self { root, path }
    }

    fn source(&self) -> PathBuf {
        self.path.join("src/math.ssrg")
    }

    fn run(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("doc")
            .arg(&self.path)
            .arg("--test")
            .output()
            .unwrap()
    }

    fn update_lock(&self) {
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["lock", "update"])
            .arg(&self.path)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
}

impl Drop for LockedProject {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn runs_the_doc_tests_product_fixture_without_rewriting_source() {
    let project = LockedProject::doc_tests();
    let before = fs::read(project.source()).unwrap();
    let output = project.run();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        fs::read_to_string(project.path.join("expected.stdout")).unwrap()
    );
    assert_eq!(fs::read(project.source()).unwrap(), before);
}

#[test]
fn reports_stdout_and_diagnostic_code_mismatches_with_original_locations() {
    let project = LockedProject::doc_tests();
    let source = fs::read_to_string(project.source())
        .unwrap()
        .replace("// 3", "// 4")
        .replace("compile_fail SES-T0101", "compile_fail SES-T9999");
    fs::write(project.source(), source).unwrap();
    project.update_lock();
    let output = project.run();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(stdout.contains("FAIL math::add#2 run"));
    assert!(stdout.contains("FAIL math::add#3 compile_fail SES-T9999"));
    assert!(stdout.ends_with("1 passed; 2 failed\n"));
    assert!(stderr.contains("math.ssrg:"));
    assert!(stderr.contains("stdout mismatch"));
    assert!(stderr.contains("expected error[SES-T9999] was not produced"));
    assert!(stderr.contains("SES-T0101"));
    assert!(stderr.contains("math.ssrg:22:"));
}

#[test]
fn rejects_run_blocks_without_a_selected_target_before_reporting_blocks() {
    let project = LockedProject::doc_tests();
    let manifest = fs::read_to_string(project.path.join("seseragi.toml"))
        .unwrap()
        .replace("\n[test]\ntarget = \"test-js\"\n", "\n");
    fs::write(project.path.join("seseragi.toml"), manifest).unwrap();
    project.update_lock();
    let output = project.run();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("doc run target is required"));
}

#[test]
fn help_exposes_the_document_test_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("seseragi doc [path/to/package] --test"));
}
