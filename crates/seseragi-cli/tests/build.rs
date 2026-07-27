use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn test_directory(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "seseragi-build-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

fn files_in(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(relative, fs::read(path).unwrap());
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn builds_a_reproducible_single_file_program_that_matches_run() {
    let source = repository_root().join("examples/samples/hello-world/main.ssrg");
    let directory = test_directory("execution");
    let output_directory = directory.join("artifact");

    let first = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&source)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");
    assert!(String::from_utf8_lossy(&first.stdout).contains("Built"));

    let first_files = files_in(&output_directory);
    for required in [
        ".seseragi-build.json",
        "entry.ts",
        "generated-module.json",
        "main.ts",
        "main.ts.map",
        "node_modules/@seseragi/runtime/package.json",
        "node_modules/@seseragi/runtime/src/effect.ts",
    ] {
        assert!(first_files.contains_key(required), "{required}");
    }

    let run = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&source)
        .output()
        .unwrap();
    let built = Command::new("bun")
        .args(["run", "entry.ts"])
        .current_dir(&output_directory)
        .output()
        .unwrap();
    assert_eq!(built.status.code(), run.status.code());
    assert_eq!(built.stdout, run.stdout);
    assert_eq!(built.stderr, run.stderr);

    fs::write(output_directory.join("stale.txt"), "stale").unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&source)
        .args(["--out-dir"])
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(files_in(&output_directory), first_files);

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn uses_dist_as_the_default_output_directory() {
    let source = repository_root().join("examples/samples/hello-world/main.ssrg");
    let directory = test_directory("default");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&source)
        .current_dir(&directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(directory.join("dist/entry.ts").is_file());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn refuses_to_clean_an_unmanaged_output_directory() {
    let source = repository_root().join("examples/samples/hello-world/main.ssrg");
    let directory = test_directory("unmanaged");
    let output_directory = directory.join("artifact");
    fs::create_dir(&output_directory).unwrap();
    fs::write(output_directory.join("keep.txt"), "keep").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&source)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("refusing to clean unmanaged build output"));
    assert_eq!(
        fs::read_to_string(output_directory.join("keep.txt")).unwrap(),
        "keep"
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reports_compile_diagnostics_without_creating_output() {
    let source = repository_root()
        .join("examples/spec/artifacts/semantic-diagnostics-schema-1/unknown-pure-name/main.ssrg");
    let directory = test_directory("diagnostics");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&source)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[SES-N0001]: Name could not be resolved"));
    assert!(!output_directory.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn documents_build_in_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--help")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("seseragi build path/to/app.ssrg [--out-dir path/to/dist]"));
}
