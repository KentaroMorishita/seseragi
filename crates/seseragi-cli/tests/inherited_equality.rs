use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn inherited_ord_eq_matches_raw_strict_profiles_and_native_stages() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = Scratch(
        std::env::temp_dir().join(format!("seseragi-cli-inherited-eq-{}", std::process::id())),
    );
    fs::create_dir(&scratch.0).unwrap();
    let engines = ["bun", "node"].map(|name| {
        let found = Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(name)
            .output()
            .unwrap();
        assert!(found.status.success());
        PathBuf::from(
            String::from_utf8(found.stdout)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
    });
    let fixtures = root.join("examples/spec/fixtures/rust-backend");
    let original_defect =
        fs::read_to_string(fixtures.join("ord-inherited-eq-defect.ssrg")).unwrap();
    let two_source = scratch.0.join("two-source-parameters.ssrg");
    fs::write(
        &two_source,
        original_defect
            .replace(
                "fn eq left: Mark -> (Mark -> Bool) = {",
                "fn eq left: Mark -> right: Mark -> Bool = {",
            )
            .replace("\\right -> True", "True"),
    )
    .unwrap();
    for (name, source, expected, code) in [
        (
            "values",
            fixtures.join("ord-inherited-eq.ssrg"),
            fs::read(fixtures.join("ord-inherited-eq.stdout")).unwrap(),
            0,
        ),
        (
            "effects",
            fixtures.join("ord-inherited-eq-effects.ssrg"),
            fs::read(fixtures.join("ord-inherited-eq-effects.stdout")).unwrap(),
            0,
        ),
        (
            "defect",
            fixtures.join("ord-inherited-eq-defect.ssrg"),
            b"acquired\nreleased\n".to_vec(),
            70,
        ),
        (
            "two-source",
            two_source,
            b"acquired\nunreachable\nreleased\n".to_vec(),
            0,
        ),
    ] {
        let native_out = scratch.0.join(format!("{name}-native"));
        let built = Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native_out)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let run = Command::new(native_out.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .current_dir(&scratch.0)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        assert_eq!(
            run.status.code(),
            Some(code),
            "{name}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(run.stdout, expected, "{name}");
        assert!(String::from_utf8_lossy(&run.stderr).contains("managed-live=0"));
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let destination = scratch.0.join(format!("{name}-{profile}"));
            let built = Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--profile", profile, "--out-dir"])
                .arg(&destination)
                .output()
                .unwrap();
            assert!(
                built.status.success(),
                "{name}/{profile}: {}",
                String::from_utf8_lossy(&built.stderr)
            );
            let text = fs::read_to_string(&source).unwrap();
            let raw = seseragi_driver::compile_module(
                seseragi_driver::CompileInput::new(
                    "inherited-eq.ssrg",
                    "proof/inherited-eq",
                    &text,
                )
                .with_profile(value),
            )
            .unwrap();
            fs::write(destination.join("raw.ts"), raw.generated.typescript).unwrap();
            let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root.join("node_modules/@types")],"baseUrl":root,"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":["raw.ts"]});
            fs::write(
                destination.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            let strict = Command::new(&engines[1])
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(destination.join("tsconfig.json"))
                .output()
                .unwrap();
            assert!(
                strict.status.success(),
                "{name}/{profile}: {}{}",
                String::from_utf8_lossy(&strict.stdout),
                String::from_utf8_lossy(&strict.stderr)
            );
            let manifest: serde_json::Value = serde_json::from_slice(
                &fs::read(destination.join("artifact-manifest.json")).unwrap(),
            )
            .unwrap();
            for engine in &engines {
                let run = Command::new(engine)
                    .arg(destination.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(&scratch.0)
                    .env("PATH", "")
                    .output()
                    .unwrap();
                assert_eq!(
                    run.status.code(),
                    Some(code),
                    "{name}/{profile}: {}",
                    String::from_utf8_lossy(&run.stderr)
                );
                assert_eq!(run.stdout, expected, "{name}/{profile}");
                if code == 0 {
                    assert!(run.stderr.is_empty());
                }
            }
        }
    }
}
