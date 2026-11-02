#[path = "../../test-support/native_runtime.rs"]
mod link;
#[path = "support/generic_impl_process.rs"]
mod process;

use process::{bounded, runtime, successful};
use std::{fs, path::Path, process::Command};

fn assert_output(output: Vec<u8>, expected: &str, host: &str) {
    let output = String::from_utf8(output).unwrap();
    assert_eq!(output.lines().count(), expected.lines().count(), "{host}");
    for (actual, expected) in output.lines().zip(expected.lines()) {
        assert_eq!(actual, expected, "{host}");
    }
}

#[test]
fn file_close_errors_match_ts_and_native_for_text_bytes_modes_and_boundaries() {
    let scratch = tempfile::tempdir().unwrap();
    let scratch = scratch.path();
    let project = scratch.join("probe");
    fs::create_dir(&project).unwrap();
    fs::write(
        project.join("Cargo.toml"),
        r#"
[package]
name = "filesystem-write-close-probe"
version = "0.0.0"
edition = "2021"

[workspace]

[[bin]]
name = "probe"
path = "main.rs"
"#,
    )
    .unwrap();
    link::write_program(
        project.join("main.rs"),
        include_str!("fixtures/filesystem-write-close.rs"),
    )
    .unwrap();
    let mut cargo = Command::new("cargo");
    link::configure(&mut cargo);
    let native = successful(bounded(
        cargo
            .args(["run", "--offline", "--quiet", "--manifest-path"])
            .arg(project.join("Cargo.toml")),
        scratch,
    ));
    let mut expected = String::new();
    for api in ["text", "bytes", "read-text", "read-bytes"] {
        let reading = api.starts_with("read-");
        for mode in ["Replace", "Append", "CreateNew"] {
            for length in [0, "A\0雪🙂\n".len(), 65539] {
                for failure in [
                    "ok",
                    "open",
                    "read",
                    "write",
                    "flush",
                    "close",
                    "write+close",
                    "flush+close",
                ] {
                    let kind = if failure == "open"
                        || (reading && failure == "read")
                        || (!reading
                            && (failure.contains("flush")
                                || (length > 0 && failure.contains("write"))))
                    {
                        Some("FileAlreadyExists")
                    } else if failure.contains("close") {
                        Some("PermissionDenied")
                    } else {
                        None
                    };
                    let result = if let Some(kind) = kind {
                        format!("{}FileSystemError {{ operation: {}, path: \"output-雪.txt\", otherPath: Nothing, kind: {kind} }}",
                            if api == "read-text" { "FileAccessFailure " } else { "" },
                            if reading { "ReadFile" } else { "WriteFile" },
                        )
                    } else {
                        "success".into()
                    };
                    expected.push_str(&format!("{api}/{mode}/{length}/{failure}: {result}\n"));
                }
            }
        }
    }
    assert_output(native.stdout, &expected, "native");
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/filesystem-write-close.ts");
    let javascript = scratch.join("probe.mjs");
    successful(bounded(
        Command::new(runtime("bun", scratch))
            .arg("build")
            .arg(&source)
            .args(["--target", "node", "--outfile"])
            .arg(&javascript),
        scratch,
    ));
    for host in ["bun", "node"] {
        let output = successful(bounded(
            Command::new(runtime(host, scratch)).arg(&javascript),
            scratch,
        ));
        assert_output(output.stdout, &expected, host);
    }
}
