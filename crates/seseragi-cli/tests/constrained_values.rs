use std::{fs, path::Path, process::Command};

fn success(command: &mut Command) -> std::process::Output {
    let output = command.output().unwrap();
    assert!(output.status.success(), "{command:?}\n{output:?}");
    output
}

#[test]
fn constrained_values_match_ts_in_every_first_class_position() {
    let scratch = tempfile::Builder::new()
        .prefix("seseragi-constrained-values-")
        .tempdir()
        .unwrap();
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/spec/fixtures/rust-backend");
    for name in [
        "constrained-values",
        "constrained-hkt-partials",
        "equality-partials",
        "method-references",
        "polymorphic-globals",
    ] {
        let source = fixtures.join(format!("{name}.ssrg"));
        let expected = fs::read(source.with_extension("stdout")).unwrap();
        let ts = success(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("run")
                .arg(&source),
        );
        assert_eq!(ts.stdout, expected, "TS {name}");
        assert!(ts.stderr.is_empty(), "TS {name}: {ts:?}");
        let directory = scratch.path().join(name);
        success(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .args(["experimental", "build-native"])
                .arg(&source)
                .args(["--entry", "main", "--out-dir"])
                .arg(&directory),
        );
        let native = success(
            Command::new(directory.join(if cfg!(windows) {
                "program.exe"
            } else {
                "program"
            }))
            .env("PATH", "")
            .env("SESERAGI_NATIVE_METRICS", "1"),
        );
        assert_eq!(native.stdout, expected, "native {name}");
        assert_eq!(native.stdout, ts.stdout, "native / TS {name}");
        assert!(
            String::from_utf8_lossy(&native.stderr).contains("managed-live=0"),
            "native {name}: {native:?}"
        );
    }
}
