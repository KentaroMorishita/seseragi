use std::{fs, path::Path, process::Command};

fn successful(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn optional_record_patterns_match_shared_stdout_on_bun_node_and_native() {
    check_fixture("optional-record-patterns");
}

#[test]
fn optional_field_access_matches_strict_typescript_and_native() {
    check_fixture("strict-optional-field-access");
}

fn check_fixture(name: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let source = root.join(format!("examples/spec/fixtures/rust-backend/{name}.ssrg"));
    let expected = fs::read(source.with_extension("stdout")).unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let cli = env!("CARGO_BIN_EXE_seseragi");
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let directory = scratch.path().join(profile);
        successful(
            Command::new(cli)
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&directory),
        );
        let text = fs::read_to_string(&source).unwrap();
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(&format!("{name}.ssrg"), "fixture/queries", &text)
                .with_profile(build_profile),
        )
        .unwrap();
        fs::write(directory.join("raw.ts"), &compiled.generated.typescript).unwrap();
        let mut config = serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "types": ["bun", "node"], "typeRoots": [root.join("node_modules/@types")],
                "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023", "DOM", "DOM.Iterable"], "allowImportingTsExtensions": true
            },
            "files": [directory.join("raw.ts")]
        });
        for exact in [false, true] {
            config["compilerOptions"]["exactOptionalPropertyTypes"] = exact.into();
            fs::write(
                directory.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            successful(
                Command::new("node")
                    .arg(root.join("node_modules/typescript/bin/tsc"))
                    .arg("--project")
                    .arg(directory.join("tsconfig.json")),
            );
        }
        if name == "strict-optional-field-access" {
            fs::write(
                directory.join("invalid.ts"),
                format!(
                    "{}\ndescribe({{ note: 42 }});\nprototype({{ toString: 42 }});\n",
                    compiled.generated.typescript
                ),
            )
            .unwrap();
            config["files"] = serde_json::json!([directory.join("invalid.ts")]);
            fs::write(
                directory.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            let rejected = Command::new("node")
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(directory.join("tsconfig.json"))
                .output()
                .unwrap();
            assert!(!rejected.status.success(), "{rejected:?}");
            assert_eq!(
                String::from_utf8_lossy(&rejected.stdout)
                    .matches("error TS2322")
                    .count(),
                2,
                "{rejected:?}"
            );
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(directory.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for engine in ["bun", "node"] {
            let output = successful(
                Command::new(engine).arg(directory.join(manifest["entry"].as_str().unwrap())),
            );
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{output:?}");
        }
    }
    let output = successful(Command::new(cli).arg("run").arg(&source));
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty(), "{output:?}");
    let directory = scratch.path().join("native");
    successful(
        Command::new(cli)
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&directory),
    );
    let output = successful(&mut Command::new(directory.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    })));
    assert_eq!(output.stdout, expected, "native");
    assert!(output.stderr.is_empty(), "{output:?}");
}
