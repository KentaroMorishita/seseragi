//! A real pinned npm SDK goes through conversion, locking, build and host execution.
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn command(args: &[&str], cwd: &Path) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
#[test]
fn real_npm_base64_sdk_builds_and_runs_on_node_and_bun_without_wrappers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("apps/playground/tests/fixtures/interop-base64-js-1.5.1");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("package-provenance.json")).unwrap())
            .unwrap();
    assert_eq!(provenance["package"], "base64-js");
    assert_eq!(provenance["version"], "1.5.1");
    for (name, digest) in provenance["files"].as_object().unwrap() {
        assert_eq!(
            format!(
                "sha256:{:x}",
                Sha256::digest(fs::read(fixture.join("vendor/base64-js").join(name)).unwrap())
            ),
            digest.as_str().unwrap()
        );
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("seseragi-real-npm-{}-{unique}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let entries: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.join("workspace-files.json")).unwrap()).unwrap();
    for entry in entries.as_array().unwrap() {
        let relative = entry["path"].as_str().unwrap();
        // Browser workspaces are flat; the native package uses its default src layout.
        let target = directory.join(if relative == "main.ssrg" {
            "src/main.ssrg"
        } else {
            relative
        });
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(fixture.join(entry["fixture"].as_str().unwrap()), target).unwrap();
    }
    command(&["dts", "convert", "."], &directory);
    command(&["lock", "update", "."], &directory);
    for profile in ["development", "release"] {
        command(
            &[
                "build",
                ".",
                "--target",
                "process",
                "--profile",
                profile,
                "--out-dir",
                profile,
            ],
            &directory,
        );
        let output = directory.join(profile);
        let artifact: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        let entry = artifact["entry"].as_str().unwrap();
        for runtime in ["node", "bun"] {
            let executed = Command::new(runtime)
                .arg(entry)
                .current_dir(&output)
                .output()
                .unwrap();
            assert!(
                executed.status.success(),
                "{profile}/{runtime}: {}",
                String::from_utf8_lossy(&executed.stderr)
            );
            assert_eq!(
                String::from_utf8(executed.stdout).unwrap(),
                "SGVsbG8=:5.0\n",
                "{profile}/{runtime}"
            );
        }
    }
    let run = command(&["run", ".", "--target", "process"], &directory);
    assert_eq!(String::from_utf8(run.stdout).unwrap(), "SGVsbG8=:5.0\n");
    fs::remove_dir_all(directory).unwrap();
}
