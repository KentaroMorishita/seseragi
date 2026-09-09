use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}
fn command(args: &[&str], cwd: &Path) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn temporary(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "seseragi-production-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy(&entry.path(), &target);
        } else if entry.file_name() != "seseragi.lock" {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
fn manifest(output: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap()).unwrap()
}
fn execute(output: &Path) -> std::process::Output {
    let output = Command::new("bun")
        .arg(manifest(output)["entry"].as_str().unwrap())
        .current_dir(output)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
#[test]
fn release_removes_dead_modules_and_declarations_before_bundling() {
    let temp = temporary("reachability");
    let package = temp.join("package");
    copy(
        &root().join("examples/spec/fixtures/projects/production-reachability"),
        &package,
    );
    command(&["lock", "update"], &package);
    for profile in ["development", "release"] {
        command(
            &["build", ".", "--profile", profile, "--out-dir", profile],
            &package,
        );
        let output = package.join(profile);
        assert_eq!(String::from_utf8(execute(&output).stdout).unwrap(), "42\n");
        let inventory = manifest(&output);
        let modules = inventory["generatedModules"].as_array().unwrap();
        assert_eq!(
            modules
                .iter()
                .any(|module| module["module"].as_str().unwrap().ends_with("::unused")),
            profile == "development"
        );
        let source = fs::read_to_string(output.join(if profile == "release" {
            "entry.js"
        } else {
            "dist/packages/fixture/production-reachability/0.0.0/values.ts"
        }))
        .unwrap();
        assert_eq!(source.contains("unusedExport"), profile == "development");
        assert_eq!(source.contains("unusedPrivate"), profile == "development");
        if profile == "development" {
            assert!(source.contains("helper") && source.contains("countdown"));
        } else {
            for name in ["helper", "countdown"] {
                assert!(inventory["reachability"]["retained"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry["declaration"] == name));
            }
        }
        if profile == "release" {
            assert!(
                inventory["reachability"]["eliminatedDeclarations"]
                    .as_u64()
                    .unwrap()
                    >= 3
            );
        }
    }
    let before = manifest(&package.join("release"));
    fs::write(
        package.join("src/unreferenced.ssrg"),
        "pub let anotherDeadValue = 99999\n",
    )
    .unwrap();
    let values = package.join("src/values.ssrg");
    let mut text = fs::read_to_string(&values).unwrap();
    text = format!("import * as math from \"std/math\"\n{text}");
    text.push_str("\npub fn anotherDeadExport value: Float -> Float = math.sin value\n");
    fs::write(values, text).unwrap();
    command(&["lock", "update"], &package);
    command(
        &["build", ".", "--profile", "release", "--out-dir", "release"],
        &package,
    );
    let after = manifest(&package.join("release"));
    assert_eq!(before["generatedModules"], after["generatedModules"]);
    assert_eq!(before["sizes"], after["sizes"]);
    assert_eq!(before["runtimeRetention"], after["runtimeRetention"]);
    assert_eq!(before["files"], after["files"]);
    assert_eq!(
        before["reachability"]["retained"],
        after["reachability"]["retained"]
    );
    assert_eq!(execute(&package.join("release")).stdout, b"42\n");
    // Make the previously dead math feature observable. Its new runtime cost
    // must now be attributable to a retained declaration and runtime module.
    fs::write(package.join("src/main.ssrg"), "import { answer, anotherDeadExport } from \"./values\"\npub effect fn main = do {\n  println $ show (answer 6)\n  println $ show (anotherDeadExport 0.5)\n}\n").unwrap();
    command(&["lock", "update"], &package);
    command(
        &["build", ".", "--profile", "release", "--out-dir", "release"],
        &package,
    );
    let needed = manifest(&package.join("release"));
    assert!(needed["reachability"]["retained"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["declaration"] == "anotherDeadExport"));
    assert!(needed["runtimeRetention"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["module"] == "@seseragi/runtime/math"));
    assert_ne!(after["runtimeRetention"], needed["runtimeRetention"]);
    assert!(
        needed["sizes"]["minifiedJavascriptBytes"].as_u64().unwrap()
            > after["sizes"]["minifiedJavascriptBytes"].as_u64().unwrap()
    );
    assert_ne!(execute(&package.join("release")).stdout, b"42\n");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn release_application_matches_development_for_existing_semantic_fixtures() {
    let temp = temporary("parity");
    for fixture in [
        "cli-build-nested",
        "performance-release-shapes",
        "performance-stack-safety",
    ] {
        let package = temp.join(fixture);
        copy(
            &root().join("examples/spec/fixtures/projects").join(fixture),
            &package,
        );
        command(&["lock", "update"], &package);
        let mut outputs = Vec::new();
        for profile in ["development", "release"] {
            command(
                &["build", ".", "--profile", profile, "--out-dir", profile],
                &package,
            );
            outputs.push(execute(&package.join(profile)).stdout);
        }
        assert_eq!(outputs[0], outputs[1], "{fixture}");
    }
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn bundled_process_retains_failure_dictionary_and_excludes_unrelated_runtime() {
    let temp = temporary("runtime-retention");
    fs::write(temp.join("main.ssrg"), "pub type Failure deriving Show =\n  | Broken\npub effect fn main -> Unit\nfails Failure = fail Broken\n").unwrap();
    command(
        &[
            "build",
            "main.ssrg",
            "--profile",
            "release",
            "--out-dir",
            "release",
        ],
        &temp,
    );
    let output = temp.join("release");
    let inventory = manifest(&output);
    assert_eq!(inventory["entry"], "entry.js");
    assert!(!output.join("node_modules").exists());
    let runtime = inventory["runtimeRetention"].as_array().unwrap();
    for required in [
        "@seseragi/runtime/hash",
        "@seseragi/runtime/unicode-version",
    ] {
        assert!(
            runtime.iter().any(|module| module["module"] == required),
            "{required}"
        );
    }
    for forbidden in [
        "/dom",
        "/random",
        "/provider-sqlite",
        "/provider-postgres",
        "/test",
    ] {
        assert!(
            !runtime
                .iter()
                .any(|module| module["module"].as_str().unwrap().ends_with(forbidden)),
            "{forbidden}"
        );
    }
    let result = Command::new("bun")
        .arg("entry.js")
        .current_dir(&output)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&result.stderr).contains("Broken"));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn release_minifies_and_composes_optional_source_maps() {
    let temp = temporary("source-maps");
    let package = temp.join("package");
    copy(
        &root().join("examples/spec/fixtures/projects/production-reachability"),
        &package,
    );
    let source = package.join("src/values.ssrg");
    let text = fs::read_to_string(&source).unwrap() + "\n// 雪とUnicode source content\n";
    fs::write(source, text).unwrap();
    command(&["lock", "update"], &package);
    for policy in ["emit", "omit"] {
        command(
            &[
                "build",
                ".",
                "--profile",
                "release",
                "--source-map",
                policy,
                "--out-dir",
                policy,
            ],
            &package,
        );
        let output = package.join(policy);
        let inventory = manifest(&output);
        assert_eq!(inventory["sourceMap"]["policy"], policy);
        assert!(
            inventory["sizes"]["minifiedJavascriptBytes"]
                .as_u64()
                .unwrap()
                < inventory["sizes"]["bundledJavascriptBytes"]
                    .as_u64()
                    .unwrap()
        );
        assert_eq!(execute(&output).stdout, b"42\n");
        if policy == "emit" {
            let map: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("entry.js.map")).unwrap()).unwrap();
            assert!(map["sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source
                    .as_str()
                    .unwrap()
                    .starts_with("seseragi://fixture/production-reachability")));
            assert!(map["sourcesContent"]
                .as_array()
                .unwrap()
                .iter()
                .any(|source| source.as_str().unwrap().contains("雪とUnicode")));
            assert!(map["sources"]
                .as_array()
                .unwrap()
                .iter()
                .all(|source| !source.as_str().unwrap().starts_with('/')));
        } else {
            assert!(!output.join("entry.js.map").exists());
            assert!(!fs::read_to_string(output.join("entry.js"))
                .unwrap()
                .contains("sourceMappingURL"));
        }
    }
    fs::remove_dir_all(temp).unwrap();
}
