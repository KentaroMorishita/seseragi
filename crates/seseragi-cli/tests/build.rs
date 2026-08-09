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
fn rejects_unsupported_dom_before_single_file_and_project_builds() {
    let fixtures = repository_root().join("crates/seseragi-cli/tests/fixtures");
    let directory = test_directory("target-mismatch");
    for (index, path) in [
        fixtures.join("target-mismatch.ssrg"),
        fixtures.join("target-mismatch-project"),
    ]
    .into_iter()
    .enumerate()
    {
        let output_directory = directory.join(format!("artifact-{index}"));
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(path)
            .arg("--out-dir")
            .arg(&output_directory)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("seseragi: target mismatch before execution"));
        assert!(stderr.contains("required capabilities: dom"));
        assert!(stderr.contains("selected target: process"));
        assert!(stderr.contains("available target contracts: browser"));
        assert!(!stderr.contains("runtime defect"));
        assert!(!output_directory.exists());
    }
    fs::remove_dir_all(directory).unwrap();
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
        "node_modules/@seseragi/runtime/src/browser/dom.ts",
        "node_modules/@seseragi/runtime/src/browser/ime-input.ts",
        "node_modules/@seseragi/runtime/src/effect.ts",
    ] {
        assert!(first_files.contains_key(required), "{required}");
    }
    let browser_dom = fs::read_to_string(
        output_directory.join("node_modules/@seseragi/runtime/src/browser/dom.ts"),
    )
    .unwrap();
    assert!(browser_dom.contains("from \"../html\""));
    assert!(browser_dom.contains("from \"../signal\""));
    assert!(!browser_dom.contains("apps/playground"));

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

    fs::write(
        output_directory.join("browser-consumer.ts"),
        fs::read_to_string(
            repository_root().join("runtime/ts/fixtures/browser-dom-consumer/main.ts"),
        )
        .unwrap(),
    )
    .unwrap();
    let bundle = Command::new("bun")
        .args([
            "build",
            "browser-consumer.ts",
            "--outdir",
            "browser-consumer-dist",
        ])
        .current_dir(&output_directory)
        .output()
        .unwrap();
    assert_eq!(
        bundle.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&bundle.stderr)
    );
    assert!(output_directory
        .join("browser-consumer-dist/browser-consumer.js")
        .is_file());

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
fn builds_a_nested_multi_import_package_that_matches_run() {
    let package = repository_root().join("examples/spec/fixtures/projects/cli-build-nested");
    let directory = test_directory("package");
    let output_directory = directory.join("artifact");

    let first = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");

    let files = files_in(&output_directory);
    let module_root = "dist/packages/fixture/cli-build-nested/0.0.0";
    for required in [
        ".seseragi-build.json",
        "entry.ts",
        "node_modules/@seseragi/runtime/package.json",
        &format!("{module_root}/main.ts"),
        &format!("{module_root}/math/score.ts"),
        &format!("{module_root}/math/score.ts.map"),
        &format!("{module_root}/math/score.generated-module.json"),
        &format!("{module_root}/text/label.ts"),
        &format!("{module_root}/text/label.ts.map"),
        &format!("{module_root}/text/label.generated-module.json"),
    ] {
        assert!(files.contains_key(required), "{required}");
    }
    let manifest = fs::read_to_string(output_directory.join(".seseragi-build.json")).unwrap();
    assert!(manifest.contains("fixture/cli-build-nested@0.0.0::math/score"));
    assert!(manifest.contains("fixture/cli-build-nested@0.0.0::text/label"));
    assert!(manifest.contains("\"entryModule\""));

    let run = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("run")
        .arg(&package)
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

    let second = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(files_in(&output_directory), files);
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
fn reports_package_compile_diagnostics_without_creating_output() {
    let directory = test_directory("package-diagnostics");
    let package = directory.join("package");
    let source_directory = package.join("src");
    let output_directory = directory.join("artifact");
    fs::create_dir_all(&source_directory).unwrap();
    fs::write(
        package.join("seseragi.toml"),
        "[package]\nname = \"fixture/build-diagnostics\"\nversion = \"0.0.0\"\nlanguage = \">=0.1.0 <0.2.0\"\n\n[run]\nentry = \"main\"\ntarget = \"test-js\"\n",
    )
    .unwrap();
    fs::write(
        source_directory.join("main.ssrg"),
        "pub effect fn main = println missing\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
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
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("seseragi build path/to/package [--out-dir path/to/dist]"));
}

#[test]
fn reports_version_commit_and_channel() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--version")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(&format!("seseragi {}", env!("CARGO_PKG_VERSION"))));
    assert!(stdout.contains("commit "));
    assert!(stdout.contains("target "));
    assert!(stdout.contains("development") || stdout.contains("release"));
}
