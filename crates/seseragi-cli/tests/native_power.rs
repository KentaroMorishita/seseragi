use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "seseragi-native-power-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn compare(source: &Path, expected: &[u8], exit: i32, defect: Option<&str>, trace: Option<&[u8]>) {
    let scratch = Scratch::new();
    let native = scratch.0.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        &scratch.0,
    ));
    let mut programs = vec![(
        "native".to_owned(),
        native.join(format!("program{}", std::env::consts::EXE_SUFFIX)),
        None,
        true,
    )];
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let output = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&output),
            &scratch.0,
        ));
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new(
                "native-power.ssrg",
                "proof/native-power",
                &fs::read_to_string(source).unwrap(),
            )
            .with_profile(build_profile),
        )
        .unwrap();
        let raw = output.join("raw-original.ts");
        fs::write(&raw, compiled.generated.typescript).unwrap();
        let mut files = vec![raw];
        if profile == "development" {
            files.push(output.join("entry.ts"));
        }
        let config = serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun","node"],
                "typeRoots": [root().join("node_modules/@types")],
                "baseUrl": root(),
                "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023","DOM","DOM.Iterable"],
            },
            "files": files,
        });
        fs::write(
            output.join("raw-tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        successful(bounded(
            Command::new(runtime("node", &scratch.0))
                .arg(root().join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(output.join("raw-tsconfig.json")),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = output.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            programs.push((
                format!("{profile}-{engine}"),
                runtime(engine, &scratch.0),
                Some(entry.clone()),
                false,
            ));
        }
    }
    assert_eq!(programs.len(), 5);
    for (label, executable, argument, native) in programs {
        let directory = scratch.0.join(format!("run-{label}"));
        fs::create_dir(&directory).unwrap();
        let mut command = Command::new(executable);
        command.current_dir(&directory).env("PATH", "");
        if let Some(argument) = argument {
            command.arg(argument);
        }
        if native {
            command.env("SESERAGI_NATIVE_METRICS", "1");
        }
        let output = bounded(&mut command, &directory);
        assert_eq!(output.status.code(), Some(exit), "{label}: {output:?}");
        assert_eq!(output.stdout, expected, "{label}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        if native {
            assert!(stderr.contains("managed-live=0"), "{label}: {stderr}");
            if let Some(defect) = defect {
                assert!(stderr.contains(defect), "{label}: {stderr}");
            }
        } else if defect.is_some() {
            assert_eq!(stderr, "seseragi: runtime defect\n", "{label}");
        } else {
            assert!(stderr.is_empty(), "{label}: {stderr}");
        }
        if let Some(trace) = trace {
            assert_eq!(
                fs::read(directory.join("trace.txt")).unwrap(),
                trace,
                "{label}"
            );
        }
    }
}

#[test]
fn powers_preserve_values_saved_methods_and_raw_strict_ts_on_bun_node_and_native() {
    let fixture = root().join("examples/spec/fixtures/rust-backend/native-power.ssrg");
    compare(
        &fixture,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/native-power.stdout"),
        0,
        None,
        None,
    );
}

#[test]
fn integer_power_defects_wait_for_scope_cleanup() {
    let scratch = Scratch::new();
    let original = fs::read_to_string(
        root().join("examples/spec/fixtures/rust-backend/int-remainder-defect.ssrg"),
    )
    .unwrap();
    assert_eq!(original.matches("(%)").count(), 1);
    assert_eq!(original.matches("selected 125").count(), 1);
    assert_eq!(original.matches("saved 0").count(), 1);
    for (name, exponent, reason) in [
        ("negative", "-1", "Int negative exponent"),
        ("overflow", "53", "Int overflow"),
    ] {
        let source = scratch.0.join(format!("{name}.ssrg"));
        fs::write(
            &source,
            original
                .replace("(%)", "(**)")
                .replace("selected 125", "selected 2")
                .replace("saved 0", &format!("saved ({exponent})")),
        )
        .unwrap();
        compare(
            &source,
            b"",
            70,
            Some(reason),
            Some(b"acquire\nsaved\nrelease\n"),
        );
    }
}
