use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

#[test]
fn computed_callee_errors_reject_before_native_or_ts_artifacts() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-computed-callee-errors-")
        .tempdir()
        .unwrap();
    for (name, source, code) in [
        (
            "argument",
            "fn add left: Int -> right: Int -> Int = left + right\n\
             pub effect fn main = println (show ((match [add 41][0] { Just value -> value; Nothing -> add 0 }) \"wrong\"))\n",
            "SES-T0101",
        ),
        (
            "instance",
            "type Secret = | Secret Int\n\
             fn wrap value: Int -> Secret = Secret value\n\
             pub effect fn main = println (show ((if True then wrap else wrap) 42))\n",
            "SES-T0201",
        ),
    ] {
        let path = scratch.path().join(format!("{name}-λ.ssrg"));
        fs::write(&path, format!("// 雫🙂\n{source}")).unwrap();
        let rust = scratch.path().join(format!("{name}.rs"));
        let result = cli()
            .env("PATH", "")
            .args(["experimental", "emit-rust"])
            .arg(&path)
            .args(["--entry", "main", "--output"])
            .arg(&rust)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(2), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains(code));
        assert!(result.stdout.is_empty());
        assert!(!rust.exists() && !rust.with_extension("rust-backend.json").exists());
        for profile in ["development", "release"] {
            let output = scratch.path().join(format!("{name}-{profile}"));
            let result = cli()
                .env("PATH", "")
                .arg("build")
                .arg(&path)
                .args(["--profile", profile, "--out-dir"])
                .arg(&output)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(2), "{result:?}");
            assert!(String::from_utf8_lossy(&result.stderr).contains(code));
            assert!(result.stdout.is_empty() && !output.exists());
        }
    }
}

#[test]
fn computed_callees_preserve_results_cold_effects_and_defect_cleanup() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-computed-callees-")
        .tempdir()
        .unwrap();
    // Cargo supplies an absolute path; lexical parents avoid Windows verbatim
    // prefixes, which Bun does not accept as a tsconfig baseUrl.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let fixtures = root.join("examples/spec/fixtures/rust-backend");
    let engines = ["bun", "node"].map(|name| {
        let located = Command::new(if cfg!(windows) { "where" } else { "which" })
            .arg(name)
            .output()
            .unwrap();
        assert!(located.status.success());
        PathBuf::from(
            String::from_utf8(located.stdout)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        )
    });
    for (name, code) in [
        ("computed-callees", 0),
        ("computed-callees-effects", 0),
        ("computed-callees-trace", 0),
        ("computed-callees-defect", 70),
        ("computed-callees-returned-defect", 70),
    ] {
        let source = fixtures.join(format!("{name}.ssrg"));
        let expected = fs::read(fixtures.join(format!("{name}.stdout"))).unwrap();
        let destination = scratch.path().join(format!("{name}-native"));
        let built = cli()
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&destination)
            .output()
            .unwrap();
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let cwd = scratch.path().join(format!("{name}-native-run"));
        fs::create_dir(&cwd).unwrap();
        let native = Command::new(destination.join(if cfg!(windows) {
            "program.exe"
        } else {
            "program"
        }))
        .current_dir(&cwd)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
        assert_eq!(
            native.status.code(),
            Some(code),
            "{name}: {}",
            String::from_utf8_lossy(&native.stderr)
        );
        assert_eq!(native.stdout, expected, "{name}");
        assert!(String::from_utf8_lossy(&native.stderr).contains("managed-live=0"));
        if code == 70 {
            // The callee's addition Defect must precede the argument's division.
            assert!(!String::from_utf8_lossy(&native.stderr).contains("Int division by zero"));
            assert!(
                String::from_utf8_lossy(&native.stderr).contains("Seseragi Int overflow"),
                "{}",
                String::from_utf8_lossy(&native.stderr)
            );
        }
        if name == "computed-callees-trace" {
            native_computed_callee_trace(&source, &scratch.path().join("native-trace"));
        }
        for (profile, value) in [
            ("development", seseragi_driver::BuildProfile::Development),
            ("release", seseragi_driver::BuildProfile::Release),
        ] {
            let output = scratch.path().join(format!("{name}-{profile}"));
            let built = cli()
                .arg("build")
                .arg(&source)
                .args(["--profile", profile, "--out-dir"])
                .arg(&output)
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
                    "computed-callees.ssrg",
                    "proof/computed-callees",
                    &text,
                )
                .with_profile(value),
            )
            .unwrap();
            // Keep proof-only resolution settings away from the real CLI
            // artifact: Bun also discovers tsconfig files when running a bundle.
            let proof = scratch.path().join(format!("{name}-{profile}-proof"));
            fs::create_dir(&proof).unwrap();
            fs::write(proof.join("raw.ts"), raw.generated.typescript).unwrap();
            let mut files = vec![proof.join("raw.ts")];
            if profile == "development" {
                files.push(output.join("entry.ts"));
            }
            let config = serde_json::json!({"compilerOptions":{"strict":true,"noEmit":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","types":["bun","node"],"typeRoots":[root.join("node_modules/@types")],"baseUrl":root,"paths":{"@seseragi/runtime/*":["runtime/ts/src/*"]},"lib":["ES2022","DOM"],"allowImportingTsExtensions":true,"skipLibCheck":false},"files":files});
            fs::write(
                proof.join("tsconfig.json"),
                serde_json::to_vec(&config).unwrap(),
            )
            .unwrap();
            let strict = Command::new(&engines[1])
                .arg(root.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(proof.join("tsconfig.json"))
                .output()
                .unwrap();
            assert!(
                strict.status.success(),
                "{name}/{profile}: {}{}",
                String::from_utf8_lossy(&strict.stdout),
                String::from_utf8_lossy(&strict.stderr)
            );
            if name == "computed-callees-trace" {
                let observed = Command::new(&engines[1])
                    .arg(root.join("crates/seseragi-cli/tests/fixtures/computed-callee-trace.mjs"))
                    .arg(proof.join("raw.ts"))
                    .args(&engines)
                    .output()
                    .unwrap();
                assert!(
                    observed.status.success(),
                    "{profile}: {}{}",
                    String::from_utf8_lossy(&observed.stdout),
                    String::from_utf8_lossy(&observed.stderr)
                );
            }
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            for (index, engine) in engines.iter().enumerate() {
                let cwd = scratch.path().join(format!("{name}-{profile}-run-{index}"));
                fs::create_dir(&cwd).unwrap();
                let result = Command::new(engine)
                    .arg(output.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(&cwd)
                    .env("PATH", "")
                    .output()
                    .unwrap();
                assert_eq!(
                    result.status.code(),
                    Some(code),
                    "{name}/{profile}/{index}: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                assert_eq!(result.stdout, expected, "{name}/{profile}/{index}");
                if code == 70 {
                    assert!(String::from_utf8_lossy(&result.stderr).contains("runtime defect"));
                    assert!(
                        !String::from_utf8_lossy(&result.stderr).contains("Int division by zero")
                    );
                } else {
                    assert!(
                        result.stderr.is_empty(),
                        "{name}/{profile}/{index}: {}",
                        String::from_utf8_lossy(&result.stderr)
                    );
                }
            }
        }
    }
}

fn native_computed_callee_trace(source: &Path, folder: &Path) {
    let text = fs::read_to_string(source).unwrap();
    let core = seseragi_driver::compile_core_module(seseragi_driver::CompileInput::new(
        "trace.ssrg",
        "native/main",
        &text,
    ))
    .unwrap();
    let rust = seseragi_backend_rust::emit_rust_module(&core.core_ir, "probe").unwrap();
    let main = rust.source.rfind("fn main()").unwrap();
    let mut observed = rust.source[..main].to_owned();
    let id = |name: &str| {
        format!(
            "v_{}",
            name.bytes().map(|b| format!("{b:02x}")).collect::<String>()
        )
    };
    for name in ["condition", "calleeValue", "argument", "beforeReuse"] {
        let start = observed
            .find(&format!(
                "fn {}_direct(",
                id(&format!("native/main::{name}"))
            ))
            .unwrap();
        let body = start + observed[start..].find("-> V {").unwrap() + "-> V {".len();
        observed.insert_str(
            body,
            &format!(" TRACE.with(|trace| trace.borrow_mut().push({name:?})); "),
        );
    }
    observed.push_str(&format!(r#"
thread_local! {{ static TRACE: RefCell<Vec<&'static str>> = const {{ RefCell::new(Vec::new()) }}; }}
fn main() {{
 let value = {}(vec![]).apply(V::Unit);
 assert_eq!(value.render(), "[42, 43]");
 TRACE.with(|trace| assert_eq!(&*trace.borrow(), &["condition", "calleeValue", "argument", "beforeReuse"]));
 drop(value); clear_dictionaries(); assert_eq!(LIVE.with(Cell::get), 0);
}}
"#, id(&rust.metadata.entry)));
    fs::create_dir(folder).unwrap();
    let program = folder.join("program.rs");
    fs::write(&program, observed).unwrap();
    let binary = folder.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    let built = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&program)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let result = Command::new(&binary).env("PATH", "").output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stdout.is_empty() && result.stderr.is_empty());
}
