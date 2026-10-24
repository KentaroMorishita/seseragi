use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use seseragi_runtime::{BuildTarget, DiagnosticFormat, ProcessRunOptions};
use sha2::{Digest, Sha256};

fn successful(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn engine(name: &str) -> PathBuf {
    let found = successful(
        Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(name)
            .output()
            .unwrap(),
    );
    String::from_utf8(found.stdout)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .into()
}

fn verify_artifact(
    site: &Path,
    proof: &Path,
    root: &Path,
    engines: &[PathBuf; 2],
    code: i32,
    expected: &[u8],
    json: bool,
) -> Option<String> {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(site.join("artifact-manifest.json")).unwrap()).unwrap();
    let files = manifest["files"].as_array().unwrap();
    for file in files {
        let bytes = fs::read(site.join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            file["sha256"].as_str().unwrap()
        );
    }
    let strict_failure = if manifest["profile"] == "development" {
        let raw: Vec<_> = files
            .iter()
            .map(|file| file["path"].as_str().unwrap())
            .filter(|file| {
                file.ends_with(".ts")
                    && !file.starts_with("node_modules/")
                    && !file.starts_with("runtime/")
            })
            .collect();
        // Typecheck the original program together with its real bootstrap.
        // Excluding entry.ts would miss the regression this test covers.
        assert!(raw.contains(&"entry.ts"));
        assert!(raw.contains(&"main.ts"));
        fs::create_dir_all(proof).unwrap();
        let config = serde_json::json!({
            "compilerOptions": {
                "strict": true, "skipLibCheck": false, "noEmit": true,
                "target": "ES2022", "module": "ESNext",
                "moduleResolution": "Bundler", "allowImportingTsExtensions": true,
                "types": ["bun", "node"],
                "typeRoots": [root.join("node_modules/@types")],
                "lib": ["ES2023", "DOM", "DOM.Iterable"]
            },
            "files": raw.iter().map(|file| site.join(file)).collect::<Vec<_>>()
        });
        // A proof-only tsconfig must not change normal bundle resolution.
        fs::write(proof.join("tsconfig.json"), config.to_string()).unwrap();
        let checked = Command::new(&engines[1])
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(proof.join("tsconfig.json"))
            .output()
            .unwrap();
        (!checked.status.success()).then(|| {
            format!(
                "{}: {}{}",
                site.display(),
                String::from_utf8_lossy(&checked.stdout),
                String::from_utf8_lossy(&checked.stderr)
            )
        })
    } else {
        assert_eq!(manifest["profile"], "release");
        assert!(!site.join("entry.ts").exists());
        None
    };
    for engine in engines {
        let executed = Command::new(engine)
            .arg(site.join(manifest["entry"].as_str().unwrap()))
            .current_dir(proof.parent().unwrap())
            .env("PATH", "")
            .output()
            .unwrap();
        assert_eq!(executed.status.code(), Some(code));
        assert_eq!(executed.stdout, expected);
        if code == 0 {
            assert!(executed.stderr.is_empty());
        } else {
            let error = String::from_utf8(executed.stderr).unwrap();
            assert!(error.contains("Int overflow"), "{error}");
            if json {
                let diagnostic: serde_json::Value = serde_json::from_str(error.trim()).unwrap();
                assert_eq!(diagnostic["kind"], "Defect");
                assert_eq!(diagnostic["schema"], 1);
            } else {
                assert!(error.contains("during module initialization"), "{error}");
            }
        }
    }
    strict_failure
}

#[test]
fn original_process_program_and_bootstrap_are_strict_and_keep_initialization_defects() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-process-entry-strict-")
        .tempdir()
        .unwrap();
    let engines = [engine("bun"), engine("node")];
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let fixtures = root.join("examples/spec/fixtures/rust-backend");
    let expected = fs::read(fixtures.join("process-bootstrap-strict.stdout")).unwrap();
    let mut strict_failures = Vec::new();
    for (name, code, expected) in [
        ("process-bootstrap-strict", 0, expected.as_slice()),
        (
            "process-bootstrap-initialization-defect",
            70,
            b"".as_slice(),
        ),
    ] {
        let source = fixtures.join(format!("{name}.ssrg"));
        let native = scratch.path().join(format!("{name}-native"));
        successful(
            Command::new(cli)
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native)
                .output()
                .unwrap(),
        );
        let native = Command::new(native.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        assert_eq!(native.status.code(), Some(code));
        assert_eq!(native.stdout, expected);
        assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));
        for (profile, build_profile) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let text = scratch.path().join(format!("{name}-{profile}-text"));
            successful(
                Command::new(cli)
                    .arg("build")
                    .arg(&source)
                    .args(["--profile", profile, "--out-dir"])
                    .arg(&text)
                    .output()
                    .unwrap(),
            );
            let source_text = fs::read_to_string(&source).unwrap();
            let compiled = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    source.to_str().unwrap(),
                    "artifact/process-entry-strict",
                    &source_text,
                )
                .with_profile(build_profile),
            )
            .unwrap();
            let json = scratch.path().join(format!("{name}-{profile}-json"));
            seseragi_runtime::build_main_with_options(
                &compiled,
                &json,
                BuildTarget::Process,
                ProcessRunOptions {
                    diagnostic_format: DiagnosticFormat::Json,
                    ..ProcessRunOptions::default()
                },
            )
            .unwrap();
            for (site, json) in [(&text, false), (&json, true)] {
                let proof = scratch
                    .path()
                    .join(format!("proof-{name}-{profile}-{json}"));
                if let Some(error) =
                    verify_artifact(site, &proof, &root, &engines, code, expected, json)
                {
                    strict_failures.push(error);
                }
            }
        }
    }
    assert!(strict_failures.is_empty(), "{}", strict_failures.join("\n"));
}
