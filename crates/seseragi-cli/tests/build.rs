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

fn update_lock(package: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["lock", "update"])
        .arg(package)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn locked_copy(source: &Path, parent: &Path, name: &str) -> PathBuf {
    fn copy(source: &Path, destination: &Path) {
        fs::create_dir_all(destination).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            let from = entry.path();
            let to = destination.join(entry.file_name());
            if from.is_dir() {
                copy(&from, &to);
            } else if entry.file_name() != "seseragi.lock" {
                fs::copy(from, to).unwrap();
            }
        }
    }

    let destination = parent.join(name);
    copy(source, &destination);
    update_lock(&destination);
    destination
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
fn rejects_malformed_namespaced_reduce_before_project_output() {
    let directory = test_directory("namespaced-reduce-rejection");
    let package = locked_copy(
        &repository_root().join("examples/spec/fixtures/projects/namespaced-reduce-rejection"),
        &directory,
        "package",
    );
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error[SES-P0001]: Expected an expression here"));
    assert!(stderr.contains(
        "arrays.reduce \"\" (\\current: String -> part: String -> current + part) parts"
    ));
    assert!(!stderr.contains("runtime defect"));
    assert!(!output_directory.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_unsupported_dom_before_single_file_and_project_builds() {
    let fixtures = repository_root().join("crates/seseragi-cli/tests/fixtures");
    let directory = test_directory("target-mismatch");
    let project = locked_copy(
        &fixtures.join("target-mismatch-project"),
        &directory,
        "target-mismatch-project",
    );
    for (index, path) in [fixtures.join("target-mismatch.ssrg"), project]
        .into_iter()
        .enumerate()
    {
        let output_directory = directory.join(format!("artifact-{index}"));
        let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(path)
            .args(["--target", "process"])
            .arg("--out-dir")
            .arg(&output_directory)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("SES-K0203 provider.target-mismatch"));
        if index == 0 {
            assert!(stderr.contains("target mismatch before execution"));
            assert!(stderr.contains("required capabilities: dom"));
            assert!(stderr.contains("selected target: process"));
            assert!(stderr.contains("available target contracts: browser"));
        } else {
            assert!(stderr.contains("standard module `std/web/dom`"));
            assert!(stderr.contains("target: bun-process"));
            assert!(stderr.contains("compatible targets: browser"));
        }
        assert!(!stderr.contains("runtime defect"));
        assert!(!output_directory.exists());
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn infers_web_for_a_single_file_with_a_browser_only_capability() {
    let source = repository_root().join("crates/seseragi-cli/tests/fixtures/target-mismatch.ssrg");
    let directory = test_directory("single-file-capability-target");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(source)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_directory.join("index.html").is_file());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn builds_self_contained_web_outputs_for_single_files_and_projects() {
    let root = repository_root();
    let directory = test_directory("web");
    let project = locked_copy(
        &root.join("crates/seseragi-cli/tests/fixtures/web-project"),
        &directory,
        "web-project",
    );
    for (index, source) in [
        root.join("crates/seseragi-cli/tests/fixtures/target-mismatch.ssrg"),
        project,
        locked_copy(
            &root.join("examples/spec/fixtures/projects/foreign-web-load"),
            &directory,
            "foreign-web-load",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let output_directory = directory.join(format!("artifact-{index}"));
        let first = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg("--target")
            .arg("web")
            .arg(&source)
            .arg("--out-dir")
            .arg(&output_directory)
            .output()
            .unwrap();
        assert_eq!(
            first.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );

        let first_files = files_in(&output_directory);
        assert_eq!(
            first_files.keys().cloned().collect::<Vec<_>>(),
            [
                ".seseragi-build.json",
                "artifact-manifest.json",
                "assets/app.css",
                "assets/app.js",
                "assets/app.js.map",
                "index.html",
            ]
        );
        let marker =
            String::from_utf8(first_files.get(".seseragi-build.json").unwrap().clone()).unwrap();
        assert!(marker.contains("\"target\": \"web\""));
        assert!(marker.contains("\"runtime\": \"bundled\""));
        let output_text = first_files
            .values()
            .filter_map(|content| std::str::from_utf8(content).ok())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!output_text.contains(&root.to_string_lossy().to_string()));
        assert!(!output_text.contains("apps/playground"));
        if index == 2 {
            assert!(output_text.contains("browser:"));
            assert!(output_text.contains("#app"));
        }

        let second = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("build")
            .arg(&source)
            .args(["--out-dir"])
            .arg(&output_directory)
            .args(["--target", "web"])
            .output()
            .unwrap();
        assert_eq!(second.status.code(), Some(0));
        assert_eq!(files_in(&output_directory), first_files);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn uses_the_manifest_target_unless_the_invocation_overrides_it() {
    let root = repository_root();
    let directory = test_directory("manifest-target");
    let package = locked_copy(
        &root.join("crates/seseragi-cli/tests/fixtures/web-project"),
        &directory,
        "web-project",
    );
    let manifest_path = package.join("seseragi.toml");
    let manifest = fs::read(&manifest_path).unwrap();
    let web_output = directory.join("web");

    let selected = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&web_output)
        .output()
        .unwrap();
    assert_eq!(
        selected.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    assert!(web_output.join("index.html").is_file());

    let inferred_package = directory.join("inferred-project");
    fs::create_dir_all(inferred_package.join("src")).unwrap();
    fs::write(
        inferred_package.join("seseragi.toml"),
        String::from_utf8(manifest.clone())
            .unwrap()
            .replace("target = \"web\"\n", ""),
    )
    .unwrap();
    for source in ["main.ssrg", "counter.ssrg"] {
        fs::copy(
            package.join("src").join(source),
            inferred_package.join("src").join(source),
        )
        .unwrap();
    }
    update_lock(&inferred_package);
    let inferred_output = directory.join("inferred");
    let inferred = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&inferred_package)
        .arg("--out-dir")
        .arg(&inferred_output)
        .output()
        .unwrap();
    assert_eq!(
        inferred.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&inferred.stderr)
    );
    assert!(inferred_output.join("index.html").is_file());

    let process_output = directory.join("process");
    let overridden = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .args(["--target", "process"])
        .arg("--out-dir")
        .arg(&process_output)
        .output()
        .unwrap();
    assert_eq!(overridden.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&overridden.stderr);
    assert!(
        stderr.contains("SES-K0203 provider.target-mismatch"),
        "{stderr}"
    );
    assert!(stderr.contains("standard module `std/web/dom`"), "{stderr}");
    assert!(stderr.contains("target: bun-process"), "{stderr}");
    assert!(stderr.contains("compatible targets: browser"), "{stderr}");
    assert!(!process_output.exists());
    assert_eq!(fs::read(manifest_path).unwrap(), manifest);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn builds_the_canonical_playground_web_package_directly() {
    let directory = test_directory("canonical-playground-package");
    let package = locked_copy(
        &repository_root().join("examples/samples/project-flow-app"),
        &directory,
        "project-flow-app",
    );
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_directory.join("index.html").is_file());
    let marker = fs::read_to_string(output_directory.join(".seseragi-build.json")).unwrap();
    assert!(marker.contains("\"target\": \"web\""));
    let bundle = fs::read_to_string(output_directory.join("assets/app.js")).unwrap();
    assert!(bundle.contains("Make room for the next release."));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn builds_and_executes_the_portable_standard_parity_package() {
    let directory = test_directory("std-parity-portable");
    let package = locked_copy(
        &repository_root().join("examples/spec/fixtures/projects/std-parity-portable"),
        &directory,
        "package",
    );
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let built = Command::new("bun")
        .args(["run", "entry.ts"])
        .current_dir(&output_directory)
        .output()
        .unwrap();
    assert_eq!(
        built.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&built.stdout),
        fs::read_to_string(package.join("expected.stdout")).unwrap()
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn builds_the_postgres_application_with_package_and_provider_boundaries() {
    let directory = test_directory("postgres-application");
    let package = repository_root().join("examples/spec/fixtures/projects/postgres-application");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let entry = fs::read_to_string(output_directory.join("entry.ts")).unwrap();
    assert!(entry.contains("createProviderPostgres"));
    assert!(entry.contains("seseragi/runtime-postgres#pg"));
    assert!(entry.contains("seseragi/postgres::Postgres"));
    let main = fs::read_to_string(
        output_directory.join("dist/packages/fixture/postgres-application/0.0.0/main.ts"),
    )
    .unwrap();
    let package_module =
        fs::read_to_string(output_directory.join("dist/packages/seseragi/postgres/0.1.0/lib.ts"))
            .unwrap();
    assert!(main.contains("@seseragi/runtime/postgres"));
    assert!(main.contains("_ssrg_postgres_transaction"));
    assert!(main.contains("[\"apply\"]"));
    assert!(main.contains("_ssrg_postgres_bool(\"active\")"));
    assert!(package_module.contains("instance$Functor"));
    assert!(package_module.contains("instance$Applicative"));
    assert!(!package_module.contains("map2"));
    assert!(!main.contains("_ssrg_postgres_map2"));
    assert!(output_directory
        .join("node_modules/seseragi/runtime-postgres/pg.bundle.js")
        .is_file());
    assert!(!main.contains("runtime-postgres#pg"));
    assert!(!main.contains("pg-cursor"));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn builds_the_sqlite_application_with_package_and_provider_boundaries() {
    let directory = test_directory("sqlite-application");
    let package = repository_root().join("examples/spec/fixtures/projects/sqlite-application");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let entry = fs::read_to_string(output_directory.join("entry.ts")).unwrap();
    assert!(entry.contains("createProviderSqlite"));
    assert!(entry.contains("seseragi/runtime-sqlite#bun"));
    assert!(entry.contains("seseragi/sqlite::Sqlite"));
    let main = fs::read_to_string(
        output_directory.join("dist/packages/fixture/sqlite-application/0.0.0/main.ts"),
    )
    .unwrap();
    let package_module =
        fs::read_to_string(output_directory.join("dist/packages/seseragi/sqlite/0.1.0/lib.ts"))
            .unwrap();
    assert!(main.contains("@seseragi/runtime/sqlite"));
    assert!(main.contains("_ssrg_sqlite_transaction"));
    assert!(main.contains("[\"apply\"]"));
    assert!(main.contains("_ssrg_sqlite_bool(\"active\")"));
    assert!(package_module.contains("instance$Functor"));
    assert!(package_module.contains("instance$Applicative"));
    assert!(!package_module.contains("map2"));
    assert!(!main.contains("_ssrg_sqlite_map2"));
    assert!(output_directory
        .join("node_modules/seseragi/runtime-sqlite/bun.ts")
        .is_file());
    assert!(!main.contains("runtime-sqlite#bun"));
    assert!(!main.contains("bun:sqlite"));

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_the_process_only_sqlite_provider_for_web_before_emitting_output() {
    let directory = test_directory("sqlite-web-target");
    let package = repository_root().join("examples/spec/fixtures/projects/sqlite-application");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg(&package)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("SES-K0201"), "{stderr}");
    assert!(stderr.contains("provider.missing"), "{stderr}");
    assert!(stderr.contains("seseragi/sqlite::Sqlite"), "{stderr}");
    assert!(stderr.contains("target: browser"), "{stderr}");
    assert!(!output_directory.exists());

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_unknown_build_targets_without_creating_output() {
    let source = repository_root().join("examples/samples/hello-world/main.ssrg");
    let directory = test_directory("unknown-target");
    let output_directory = directory.join("artifact");

    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(["build", "--target", "browser"])
        .arg(&source)
        .arg("--out-dir")
        .arg(&output_directory)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unknown project target `browser`; expected `process` or `web`"));
    assert!(!output_directory.exists());
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
    let marker =
        String::from_utf8(first_files.get(".seseragi-build.json").unwrap().clone()).unwrap();
    assert!(marker.contains("\"target\": \"process\""));
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
    let directory = test_directory("package");
    let package = locked_copy(
        &repository_root().join("examples/spec/fixtures/projects/cli-build-nested"),
        &directory,
        "package",
    );
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
    assert!(manifest.contains("\"target\": \"process\""));
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
    update_lock(&package);

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
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "seseragi build path/to/app.ssrg [--target process|web] [--profile development|release] [--out-dir path/to/dist]"
    ));
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "seseragi build path/to/package [--target process|web] [--profile development|release] [--out-dir path/to/dist]"
    ));
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

    let json = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .arg("--version-json")
        .output()
        .unwrap();
    assert_eq!(json.status.code(), Some(0));
    let metadata: serde_json::Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(metadata["name"], "seseragi");
    assert_eq!(metadata["version"], env!("CARGO_PKG_VERSION"));
    assert!(metadata["commit"].is_string());
    assert!(metadata["target"].is_string());
}

#[test]
fn builds_custom_document_and_public_assets_without_rewriting_generated_outputs() {
    let directory = test_directory("custom-web-assets");
    let project = locked_copy(
        &repository_root().join("examples/spec/fixtures/projects/web-assets"),
        &directory,
        "project",
    );
    let output_directory = directory.join("output");
    let build = || {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["build", "--target", "web"])
            .arg(&project)
            .arg("--out-dir")
            .arg(&output_directory)
            .output()
            .unwrap()
    };
    let result = build();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let first = files_in(&output_directory);
    let html = String::from_utf8(first["index.html"].clone()).unwrap();
    assert!(html.contains("<title>Seseragi custom document</title>"));
    assert!(html.contains("name=\"description\""));
    assert!(html.contains("./images/favicon.svg"));
    assert!(html.contains("./assets/app.js") && html.contains("./assets/app.css"));
    assert!(!html.contains("<!-- seseragi:"));
    assert_eq!(first["images/read me.txt"], b"nested public asset\n");
    assert_eq!(first["status"], b"ready\n");
    let result = build();
    assert!(result.status.success());
    assert_eq!(first, files_in(&output_directory));

    fs::remove_file(project.join("public/status")).unwrap();
    let index = project.join("web/index.html");
    fs::write(
        &index,
        fs::read_to_string(&index)
            .unwrap()
            .replace("Seseragi custom document", "Updated document"),
    )
    .unwrap();
    assert!(
        !build().status.success(),
        "asset edits invalidate the production lock"
    );
    assert_eq!(first, files_in(&output_directory));
    update_lock(&project);
    assert!(build().status.success());
    assert!(!output_directory.join("status").exists());
    assert!(fs::read_to_string(output_directory.join("index.html"))
        .unwrap()
        .contains("Updated document"));
    let previous = files_in(&output_directory);
    let old_manifest: serde_json::Value =
        serde_json::from_slice(&first["artifact-manifest.json"]).unwrap();
    let new_manifest: serde_json::Value =
        serde_json::from_slice(&previous["artifact-manifest.json"]).unwrap();
    assert_ne!(
        old_manifest["provenance"]["buildId"],
        new_manifest["provenance"]["buildId"]
    );
    fs::write(project.join("public/artifact-manifest.json"), "collision").unwrap();
    let result = build();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("reserved Web output"));
    assert_eq!(previous, files_in(&output_directory));
    fs::remove_file(project.join("public/artifact-manifest.json")).unwrap();
    fs::write(project.join("public/index.html"), "collision").unwrap();
    let result = build();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("reserved Web output"));
    assert_eq!(previous, files_in(&output_directory));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn artifact_manifest_tracks_outputs_and_is_independent_of_build_location() {
    use sha2::{Digest, Sha256};
    let directory = test_directory("artifact-manifest");
    for (target, fixture) in [
        ("web", "crates/seseragi-cli/tests/fixtures/web-project"),
        (
            "process",
            "examples/spec/fixtures/projects/cli-build-nested",
        ),
    ] {
        let package = locked_copy(&repository_root().join(fixture), &directory, target);
        for profile in ["development", "release"] {
            let mut previous = None;
            for location in ["first", "different-output"] {
                let build_package = if location == "different-output" {
                    locked_copy(
                        &package,
                        &directory,
                        &format!("relocated-{target}-{profile}"),
                    )
                } else {
                    package.clone()
                };
                let output_dir = directory.join(format!("{target}-{profile}-{location}"));
                let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("build")
                    .arg(&build_package)
                    .args(["--target", target, "--profile", profile, "--out-dir"])
                    .arg(&output_dir)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let mut files = files_in(&output_dir);
                let bytes = files.remove("artifact-manifest.json").unwrap();
                let manifest: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                let mut model: seseragi_runtime::artifact::ArtifactManifest =
                    serde_json::from_slice(&bytes).unwrap();
                let build_id = std::mem::take(&mut model.provenance.build_id);
                assert_eq!(
                    build_id,
                    format!("{:x}", Sha256::digest(serde_json::to_vec(&model).unwrap()))
                );
                assert_eq!(manifest["schema"], 1);
                assert_eq!(manifest["target"], target);
                assert_eq!(manifest["profile"], profile);
                assert!(manifest["runtimeRetention"].is_null());
                assert!(manifest["sizes"]["minifiedJavascriptBytes"].is_null());
                assert_eq!(manifest["sourceMap"]["policy"], "emit");
                assert!(manifest["generatedModules"].as_array().unwrap().len() > 1);
                let entries = manifest["files"].as_array().unwrap();
                assert_eq!(entries.len(), files.len());
                for (entry, (path, content)) in entries.iter().zip(&files) {
                    assert_eq!(entry["path"], *path);
                    assert_eq!(entry["bytes"], content.len() as u64);
                    assert_eq!(entry["sha256"], format!("{:x}", Sha256::digest(content)));
                }
                if target == "web" {
                    assert_eq!(
                        manifest["sizes"]["bundledJavascriptBytes"],
                        files["assets/app.js"].len() as u64
                    );
                } else {
                    assert!(manifest["sizes"]["bundledJavascriptBytes"].is_null());
                }
                let text = String::from_utf8(bytes.clone()).unwrap();
                assert!(!text.contains(directory.to_str().unwrap()));
                if let Some(previous) = previous {
                    assert_eq!(bytes, previous);
                }
                previous = Some(bytes);
            }
        }
    }
    fs::remove_dir_all(directory).unwrap();
}
