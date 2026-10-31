use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn signal_fixtures_and_official_samples_match_bun_node_and_native() {
    let scratch = Scratch(
        std::env::temp_dir().join(format!("seseragi-native-signal-{}", std::process::id())),
    );
    fs::create_dir(&scratch.0).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    for (name, sample) in [
        ("signal-transactions", false),
        ("signal-observer-queue", false),
        ("signal-switch-distinct", false),
        ("signal-many-subscribers", false),
        ("signal-scopes", false),
        ("signal-state", true),
        ("signal-composition", true),
        ("type-aliases", true),
    ] {
        let input = if sample {
            root.join(format!("examples/samples/{name}/main.ssrg"))
        } else {
            root.join(format!("examples/spec/fixtures/rust-backend/{name}.ssrg"))
        };
        let directory = scratch.0.join(name);
        fs::create_dir(&directory).unwrap();
        let expected = if sample {
            successful(bounded(
                Command::new(env!("CARGO_BIN_EXE_seseragi"))
                    .arg("run")
                    .arg(&input),
                &directory,
            ))
            .stdout
        } else {
            fs::read(input.with_extension("stdout")).unwrap()
        };
        let native = directory.join("native");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&input)
                .args(["--entry", "main", "--out-dir"])
                .arg(&native),
            &directory,
        ));
        let output = successful(bounded(
            Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
                .env("PATH", "")
                .env("SESERAGI_NATIVE_METRICS", "1"),
            &directory,
        ));
        assert_eq!(output.stdout, expected, "{name}/native");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("managed-live=0"),
            "{name}: {output:?}"
        );
        let ts = directory.join("ts");
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&input)
                .args(["--target", "process", "--out-dir"])
                .arg(&ts),
            &directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(ts.join("artifact-manifest.json")).unwrap()).unwrap();
        let entry = ts.join(manifest["entry"].as_str().unwrap());
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, &directory))
                    .arg(&entry)
                    .env("PATH", ""),
                &directory,
            ));
            assert_eq!(output.stdout, expected, "{name}/{engine}");
            assert!(output.stderr.is_empty(), "{name}/{engine}: {output:?}");
        }
    }
}
