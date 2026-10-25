use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn copy_project(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_project(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn strict_config(root: &Path, files: &[PathBuf]) -> serde_json::Value {
    serde_json::json!({"compilerOptions": {
        "strict": true, "skipLibCheck": false, "noEmit": true,
        "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
        "allowImportingTsExtensions": true, "types": ["bun", "node"],
        "typeRoots": [root.join("node_modules/@types")],
        "lib": ["ES2023", "DOM", "DOM.Iterable"],
        "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
    }, "files": files})
}

#[test]
fn saved_nominal_return_lambda_keeps_primitive_identity_and_scoped_effects() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-nominal-lambda-")
        .tempdir()
        .unwrap();
    let fixture =
        root.join("examples/spec/fixtures/rust-backend-projects/nominal-lambda-primitive-identity");
    let expected = fs::read(fixture.join("expected.stdout")).unwrap();
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let engines = ["bun", "node"].map(|name| {
        let found = success(
            Command::new(if cfg!(windows) { "where" } else { "which" })
                .arg(name)
                .output()
                .unwrap(),
        );
        PathBuf::from(
            String::from_utf8(found.stdout)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
    });
    for (label, code, expected) in [
        ("cold-replay", 0, expected.as_slice()),
        ("scoped-defect", 70, b"40\nacquired\nreleased\n".as_slice()),
        ("implicit-unit-stage", 0, b"7:9\n".as_slice()),
    ] {
        let project = scratch.path().join(label).join("project");
        let selected_fixture = if label == "implicit-unit-stage" {
            root.join("examples/spec/fixtures/rust-backend-projects/implicit-unit-functions")
        } else {
            fixture.clone()
        };
        copy_project(&selected_fixture, &project);
        if code == 70 {
            let file = project.join("src/main.ssrg");
            let source = fs::read_to_string(&file).unwrap();
            assert_eq!(source.matches("Foreign inner -> inner").count(), 1);
            assert_eq!(source.matches("effects.acquireRelease (effects.succeed ()) (\\_ -> refs.set True released)").count(), 1);
            fs::write(file, source
                .replace("Foreign inner -> inner", "Foreign inner -> inner + 9007199254740991")
                .replace("effects.acquireRelease (effects.succeed ()) (\\_ -> refs.set True released)",
                    "effects.acquireRelease (do { println \"acquired\"; effects.succeed () }) (\\_: Unit -> do { refs.set True released; effects.recover (\\_: ConsoleError -> effects.succeed ()) (println \"released\") })")).unwrap();
        }
        success(
            Command::new(cli)
                .args(["lock", "update"])
                .arg(&project)
                .output()
                .unwrap(),
        );
        let native = scratch.path().join(format!("{label}-native"));
        success(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&project)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native)
                .output()
                .unwrap(),
        );
        let result = Command::new(native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .current_dir(scratch.path())
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        assert_eq!(result.status.code(), Some(code), "{result:?}");
        assert_eq!(result.stdout, expected);
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("managed-live=0"),
            "{result:?}"
        );
        if code == 70 {
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("Seseragi Int overflow"),
                "{result:?}"
            );
        }
        let loaded = seseragi_project::load_local_project(&project).unwrap();
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.path().join(format!("{label}-{profile}"));
            success(
                Command::new(cli)
                    .arg("build")
                    .arg(&project)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&output)
                    .output()
                    .unwrap(),
            );
            let proof = scratch.path().join(format!("{label}-{profile}-proof"));
            fs::create_dir(&proof).unwrap();
            // Validate each profile's original driver TypeScript without rewriting it.
            let compiled =
                seseragi_driver::compile_local_project_with_profile(&loaded, Some(value)).unwrap();
            let raw = proof.join("raw");
            fs::create_dir(&raw).unwrap();
            let mut files = Vec::new();
            for module in compiled.compiled.modules.values() {
                let relative = Path::new(&module.generated.metadata.outputs.typescript);
                assert!(
                    !relative.is_absolute()
                        && !relative
                            .components()
                            .any(|p| p == std::path::Component::ParentDir)
                );
                let file = raw.join(relative);
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, &module.generated.typescript).unwrap();
                files.push(file);
            }
            assert!(files.len() >= 2);
            if profile == "development" {
                assert!(output.join("entry.ts").is_file());
                files.push(output.join("entry.ts"));
            }
            fs::write(
                proof.join("tsconfig.json"),
                strict_config(&root, &files).to_string(),
            )
            .unwrap();
            success(
                Command::new(&engines[1])
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(proof.join("tsconfig.json"))
                    .output()
                    .unwrap(),
            );
            for engine in &engines {
                let result = Command::new(engine)
                    .arg(output.join("entry.js"))
                    .current_dir(&output)
                    .output()
                    .unwrap();
                assert_eq!(
                    result.status.code(),
                    Some(code),
                    "{label}/{profile}: {result:?}"
                );
                assert_eq!(result.stdout, expected, "{label}/{profile}");
                if code == 0 {
                    assert!(result.stderr.is_empty(), "{result:?}");
                } else {
                    assert!(
                        String::from_utf8_lossy(&result.stderr)
                            .contains("seseragi: runtime defect"),
                        "{result:?}"
                    );
                }
            }
        }
    }
}
