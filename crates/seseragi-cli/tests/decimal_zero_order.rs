#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn decimal_zero_order_keeps_original_numeric_results_on_bun_node_and_both_profiles() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let source =
        root.join("examples/spec/fixtures/rust-backend-regressions/decimal-zero-order.ssrg");
    let original = fs::read(&source).unwrap();
    let expected = include_bytes!(
        "../../../examples/spec/fixtures/rust-backend-regressions/decimal-zero-order.stdout"
    );
    let path = std::env::temp_dir().join(format!(
        "seseragi-decimal-zero-order-{}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    let scratch = Scratch(path);
    for profile in ["development", "release"] {
        let out = scratch.0.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&out),
            &scratch.0,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime(engine, &scratch.0))
                    .arg(out.join(manifest["entry"].as_str().unwrap()))
                    .env("PATH", ""),
                &out,
            ));
            assert_eq!(result.stdout, expected, "{profile}/{engine}");
            assert!(
                result.stderr.is_empty(),
                "{profile}/{engine}: {:?}",
                result.stderr
            );
        }
    }
    assert_eq!(fs::read(source).unwrap(), original);
}
