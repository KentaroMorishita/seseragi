use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn directory(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "seseragi-persistent-foreign-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn copy(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy(&path, &target);
        } else {
            fs::copy(path, target).unwrap();
        }
    }
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .output()
        .unwrap()
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn fixture(name: &str, root: &Path) -> PathBuf {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/projects")
        .join(name);
    let package = root.join("package");
    copy(&source, &package);
    success(&cli(&["lock", "update", package.to_str().unwrap()]));
    package
}
fn execute_both(out: &Path, expected: &str) {
    let marker: Value =
        serde_json::from_slice(&fs::read(out.join(".seseragi-build.json")).unwrap()).unwrap();
    let artifact: Value =
        serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
    // The ownership marker identifies retained source; execute the public
    // artifact entry so Node compatibility does not require TS stripping.
    assert!(out.join(marker["entry"].as_str().unwrap()).is_file());
    let entry = artifact["entry"].as_str().unwrap();
    assert!(
        entry.ends_with(".js"),
        "public Node executable must be JavaScript"
    );
    for host in ["node", "bun"] {
        let output = Command::new(host).arg(out.join(entry)).output().unwrap();
        success(&output);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            expected,
            "{host} {entry}"
        );
    }
}

#[test]
fn persistent_node_bun_keep_lazy_foreign_paths_origin_and_composed_map_policy() {
    let root = directory("lazy");
    let package = fixture("foreign-task-load", &root);
    for profile in ["development", "release"] {
        for policy in ["emit", "omit"] {
            let out = root.join(format!("{profile}-{policy}"));
            success(&cli(&[
                "build",
                package.to_str().unwrap(),
                "--profile",
                profile,
                "--source-map",
                policy,
                "--out-dir",
                out.to_str().unwrap(),
            ]));
            execute_both(&out, "before\nhost:load\nhost:call:a\nhost:call:b\nA,B\n");
            let marker: Value =
                serde_json::from_slice(&fs::read(out.join(".seseragi-build.json")).unwrap())
                    .unwrap();
            let ts = marker["modules"][0]["typescript"].as_str().unwrap();
            let js = format!("{}.js", ts.strip_suffix(".ts").unwrap());
            let emitted = fs::read_to_string(out.join(&js)).unwrap();
            assert!(emitted.contains("../host/nested/lazy-module.mjs"));
            assert!(emitted.contains("seseragi://fixture/foreign-task-load/main"));
            let artifact: Value =
                serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap())
                    .unwrap();
            assert_eq!(artifact["sourceMap"]["policy"], policy);
            let map_path = format!("{js}.map");
            if policy == "emit" {
                assert!(
                    emitted.contains("sourceMappingURL="),
                    "no executable JS source-map linkage for {profile}: {js}"
                );
                let map: Value =
                    serde_json::from_slice(&fs::read(out.join(&map_path)).unwrap()).unwrap();
                assert!(
                    map["sources"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|s| s.as_str().unwrap()
                            == "seseragi://fixture/foreign-task-load@0.0.0::main"),
                    "JS map must reach Seseragi rather than stop at generated TS: {map}"
                );
                assert!(
                    map["sourcesContent"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|s| s.as_str().unwrap_or("").contains("foreign \"typescript\"")),
                    "{map}"
                );
                assert!(
                    artifact["sourceMap"]["files"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|s| s == &map_path),
                    "composed executable map missing from artifact policy: {artifact}"
                );
            } else {
                assert!(!out.join(&map_path).exists());
                assert!(!out.join(format!("{ts}.map")).exists());
                assert!(!emitted.contains("sourceMappingURL="));
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn persistent_node_bun_share_lazy_module_load_for_parallel_foreign_calls() {
    let root = directory("single-flight");
    let package = fixture("foreign-task-single-flight", &root);
    let out = root.join("build");
    success(&cli(&[
        "build",
        package.to_str().unwrap(),
        "--out-dir",
        out.to_str().unwrap(),
    ]));
    execute_both(&out, "host:load\nok\n");
    fs::remove_dir_all(root).unwrap();
}
