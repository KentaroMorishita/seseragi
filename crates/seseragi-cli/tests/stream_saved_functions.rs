use std::{fs, path::Path, process::Command};

#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn typescript(source: &Path, directory: &Path, expected: &[u8]) {
    for profile in ["development", "release"] {
        let out = directory.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(source)
                .args(["--target", "process", "--profile", profile, "--out-dir"])
                .arg(&out),
            directory,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["bun", "node"] {
            let output = successful(bounded(
                Command::new(runtime(engine, directory))
                    .arg(out.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(directory)
                    .env("PATH", ""),
                directory,
            ));
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert!(output.stderr.is_empty(), "{profile}/{engine}: {output:?}");
        }
    }
}

fn compare(name: &str, expected: &[u8]) {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-stream-saved-")
        .tempdir()
        .unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/rust-backend-regressions")
        .join(name);
    let native = scratch.path().join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&source)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        scratch.path(),
    ));
    let output = successful(bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        scratch.path(),
    ));
    assert_eq!(output.stdout, expected, "native");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
    typescript(&source, scratch.path(), &output.stdout);
}

#[test]
fn stream_saved_collect_and_partial_map_match_native() {
    compare("stream-saved-collect.ssrg", b"[1, 2, 3]\n");
    compare("stream-partial-map.ssrg", b"[2, 3, 4]\n");
}

#[test]
fn stream_saved_functions_keep_generic_returned_stages_and_cold_effects() {
    compare(
        "stream-saved-functions.ssrg",
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend-regressions/stream-saved-functions.stdout"
        ),
    );
}

#[test]
fn stream_saved_iterable_sources_keep_the_selected_dictionary() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-stream-iterable-")
        .tempdir()
        .unwrap();
    let source = scratch.path().join("main.ssrg");
    fs::write(
        &source,
        r#"
import * as streams from "std/stream"
fn materialize<C> values: C -> Effect<{}, Never, Array<Int>> where Iterable<C, Int> = {
  let fromIterable: C -> streams.Stream<{}, Never, Int> = streams.fromIterable
  streams.runCollect (fromIterable values)
}
pub effect fn main = do {
  let fromArray: Array<Int> -> streams.Stream<{}, Never, Int> = streams.fromIterable
  let fromList: List<Int> -> streams.Stream<{}, Never, Int> = streams.fromIterable
  array <- streams.runCollect (fromArray [1, 2, 3])
  println (show array)
  list <- streams.runCollect (fromList `[4, 5])
  println (show list)
  empty <- streams.runCollect (fromArray [])
  println (show empty)
  genericArray <- materialize [6, 7]
  println (show genericArray)
  genericList <- materialize `[8, 9]
  println (show genericList)
}
"#,
    )
    .unwrap();
    typescript(
        &source,
        scratch.path(),
        b"[1, 2, 3]\n[4, 5]\n[]\n[6, 7]\n[8, 9]\n",
    );
}
