use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
        "seseragi-dts-{name}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

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

fn fixture(name: &str) -> PathBuf {
    repository_root()
        .join("examples/spec/fixtures/projects")
        .join(name)
}

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn executes_every_dts_fixture_through_the_cli_product_route() {
    for (name, output) in [
        ("dts-basic-conversion", "fixture-api"),
        ("dts-callback-during-call", "callback-api"),
        ("dts-declaration-merge", "merge-api"),
        ("dts-generated-name", "naming-api"),
        ("dts-namespace-runtime", "analytics"),
        ("dts-overload-selection", "parser-api"),
    ] {
        let directory = test_directory(name);
        let package = directory.join("package");
        copy_directory(&fixture(name), &package);
        let converted = run(&["dts", "convert", package.to_str().unwrap()]);
        assert_eq!(
            converted.status.code(),
            Some(0),
            "{name}: {}",
            String::from_utf8_lossy(&converted.stderr)
        );
        assert_eq!(
            fs::read_to_string(
                package
                    .join(".seseragi/generated")
                    .join(format!("{output}.ssrg"))
            )
            .unwrap(),
            fs::read_to_string(package.join("expected").join(format!("{output}.ssrg"))).unwrap(),
            "{name}"
        );
        fs::remove_dir_all(directory).unwrap();
    }

    for (name, code) in [
        ("dts-callback-missing-release", "SES-F0102"),
        ("dts-unsupported-any", "SES-F0101"),
    ] {
        let directory = test_directory(name);
        let package = directory.join("package");
        copy_directory(&fixture(name), &package);
        let converted = run(&["dts", "convert", package.to_str().unwrap()]);
        assert_eq!(converted.status.code(), Some(1), "{name}");
        assert!(String::from_utf8_lossy(&converted.stderr).contains(code));
        assert!(!package.join(".seseragi/generated").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}

#[test]
fn converts_a_configured_entry_and_writes_the_three_deterministic_artifacts() {
    let directory = test_directory("success");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);

    let first = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");
    assert_eq!(
        String::from_utf8_lossy(&first.stdout),
        "converted dts entry `api` to .seseragi/generated/fixture-api.ssrg\n"
    );

    let generated = package.join(".seseragi/generated");
    assert_eq!(
        fs::read_to_string(generated.join("fixture-api.ssrg")).unwrap(),
        fs::read_to_string(package.join("expected/fixture-api.ssrg")).unwrap()
    );
    let metadata: Value =
        serde_json::from_slice(&fs::read(generated.join("fixture-api.binding.json")).unwrap())
            .unwrap();
    assert_eq!(metadata["schema"], 1);
    assert_eq!(metadata["kind"], "seseragi-typescript-binding");
    assert_eq!(metadata["entry"], "api");
    assert_eq!(metadata["specifier"], "fixture-api");
    assert_eq!(metadata["hostModule"]["specifier"], "fixture-api");
    assert_eq!(metadata["hostModule"]["exactIdentity"], "fixture-api@1.0.0");
    assert_eq!(metadata["evaluation"], "task");
    assert_eq!(metadata["symbols"].as_array().unwrap().len(), 4);
    assert_eq!(metadata["generator"]["name"], "seseragi-dts");
    assert_eq!(metadata["inputDigest"].as_str().unwrap().len(), 64);
    assert_eq!(metadata["settingsDigest"].as_str().unwrap().len(), 64);

    let report: Value =
        serde_json::from_slice(&fs::read(generated.join("fixture-api.report.json")).unwrap())
            .unwrap();
    assert_eq!(report["schema"], 1);
    assert_eq!(report["entry"], "api");
    assert_eq!(report["added"].as_array().unwrap().len(), 4);
    assert_eq!(report["changed"].as_array().unwrap().len(), 0);
    assert_eq!(report["removed"].as_array().unwrap().len(), 0);
    assert_eq!(report["unsupported"].as_array().unwrap().len(), 0);

    let before = fs::read(generated.join("fixture-api.binding.json")).unwrap();
    let second = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "api",
    ]);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(
        fs::read(generated.join("fixture-api.binding.json")).unwrap(),
        before
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn converts_all_entries_or_only_the_selected_entry_in_stable_id_order() {
    let directory = test_directory("entry-selection");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);
    fs::write(
        package.join("host/extra.d.ts"),
        "export declare function label(value: string): string;\n",
    )
    .unwrap();
    fs::write(
        package.join("seseragi.bindings.toml"),
        concat!(
            "schema = 1\n\n",
            "[entries.alpha]\n",
            "declaration = \"host/extra.d.ts\"\n",
            "specifier = \"extra-api\"\n",
            "output = \"extra-api\"\n",
            "evaluation = \"task\"\n\n",
            "[entries.api]\n",
            "declaration = \"host/index.d.ts\"\n",
            "specifier = \"fixture-api\"\n",
            "output = \"fixture-api\"\n",
            "evaluation = \"task\"\n",
        ),
    )
    .unwrap();
    let host_manifest: Value =
        serde_json::from_slice(&fs::read(package.join("host/package.json")).unwrap()).unwrap();
    let mut host_manifest = host_manifest.as_object().unwrap().clone();
    host_manifest.insert("name".to_owned(), Value::String("host-app".to_owned()));
    host_manifest.insert(
        "dependencies".to_owned(),
        serde_json::json!({ "extra-api": "1.0.0", "fixture-api": "1.0.0" }),
    );
    fs::write(
        package.join("host/package.json"),
        serde_json::to_vec_pretty(&host_manifest).unwrap(),
    )
    .unwrap();
    for package_name in ["extra-api", "fixture-api"] {
        let module = package.join("host/node_modules").join(package_name);
        fs::create_dir_all(&module).unwrap();
        fs::write(
            module.join("package.json"),
            format!("{{\"name\":\"{package_name}\",\"version\":\"1.0.0\"}}\n"),
        )
        .unwrap();
    }

    let selected = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "api",
    ]);
    assert_eq!(selected.status.code(), Some(0));
    let generated = package.join(".seseragi/generated");
    assert!(generated.join("fixture-api.ssrg").is_file());
    assert!(!generated.join("extra-api.ssrg").exists());

    let all = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(
        all.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&all.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&all.stdout),
        concat!(
            "converted dts entry `alpha` to .seseragi/generated/extra-api.ssrg\n",
            "converted dts entry `api` to .seseragi/generated/fixture-api.ssrg\n",
        )
    );
    assert!(generated.join("extra-api.ssrg").is_file());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reports_precise_conversion_errors_without_replacing_any_entry_artifact() {
    let directory = test_directory("atomic-error");
    let package = directory.join("package");
    copy_directory(&fixture("dts-unsupported-any"), &package);
    let generated = package.join(".seseragi/generated");
    fs::create_dir_all(&generated).unwrap();
    for suffix in ["ssrg", "binding.json", "report.json"] {
        fs::write(
            generated.join(format!("unsafe-api.{suffix}")),
            b"previous\n",
        )
        .unwrap();
    }

    let output = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("host/index.d.ts:38:41: error[SES-F0101]"));
    assert!(stderr.contains("`any` requires an explicit unsafe fallback"));
    for suffix in ["ssrg", "binding.json", "report.json"] {
        assert_eq!(
            fs::read(generated.join(format!("unsafe-api.{suffix}"))).unwrap(),
            b"previous\n"
        );
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_unknown_entries_and_stale_bindings_before_build_output() {
    let directory = test_directory("stale");
    let package = directory.join("package");
    copy_directory(&fixture("dts-basic-conversion"), &package);

    let unknown = run(&[
        "dts",
        "convert",
        package.to_str().unwrap(),
        "--entry",
        "missing",
    ]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("binding entry `missing` does not exist")
    );

    assert_eq!(
        run(&["dts", "convert", package.to_str().unwrap()])
            .status
            .code(),
        Some(0)
    );
    fs::create_dir_all(package.join("src")).unwrap();
    fs::write(
        package.join("src/main.ssrg"),
        "pub effect fn main -> Unit with Console fails ConsoleError = println \"ready\"\n",
    )
    .unwrap();
    fs::write(
        package.join("seseragi.toml"),
        fs::read_to_string(package.join("seseragi.toml")).unwrap()
            + "\n[run]\nentry = \"main\"\ntarget = \"process\"\n",
    )
    .unwrap();
    let lock = run(&["lock", "update", package.to_str().unwrap()]);
    assert_eq!(
        lock.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    fs::write(
        package.join("host/index.d.ts"),
        fs::read_to_string(package.join("host/index.d.ts")).unwrap()
            + "\nexport declare function newlyAdded(): string;\n",
    )
    .unwrap();

    let artifact = directory.join("artifact");
    let build = run(&[
        "build",
        package.to_str().unwrap(),
        "--out-dir",
        artifact.to_str().unwrap(),
    ]);
    assert_eq!(build.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&build.stderr);
    assert!(stderr.contains("SES-F0103"));
    assert!(stderr.contains("run `seseragi dts convert`"));
    assert!(!artifact.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn runs_a_generated_binding_through_the_normal_foreign_typescript_pipeline() {
    let directory = test_directory("dts-namespace-runtime-pipeline");
    let package = directory.join("package");
    copy_directory(&fixture("dts-namespace-runtime"), &package);

    let converted = run(&["dts", "convert", package.to_str().unwrap()]);
    assert_eq!(
        converted.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&converted.stderr)
    );
    let lock = run(&["lock", "update", package.to_str().unwrap()]);
    assert_eq!(
        lock.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    let executed = run(&["run", package.to_str().unwrap()]);
    assert_eq!(
        executed.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(
        fs::read_to_string(package.join("expected.stdout")).unwrap(),
        String::from_utf8_lossy(&executed.stdout)
    );
    fs::remove_dir_all(directory).unwrap();
}
