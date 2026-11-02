use std::{fs, process::Command};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
}

fn compare(source: &str, expected: &[u8]) {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let input = root.join("main.ssrg");
    fs::write(&input, source).unwrap();
    for profile in ["development", "release"] {
        let output = root.join(profile);
        successful(bounded(
            cli()
                .arg("build")
                .arg(&input)
                .args([
                    "--target",
                    "process",
                    "--source-map",
                    "omit",
                    "--profile",
                    profile,
                    "--out-dir",
                ])
                .arg(&output),
            root,
        ));
        for engine in ["node", "bun"] {
            let result = successful(bounded(
                Command::new(engine).arg(output.join("entry.js")),
                root,
            ));
            assert_eq!(result.stdout, expected, "{profile}-{engine}");
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
    let output = root.join("native");
    successful(bounded(
        cli()
            .args(["experimental", "build-native"])
            .arg(&input)
            .args(["--entry", "main", "--out-dir"])
            .arg(&output),
        root,
    ));
    let result = successful(bounded(
        &mut Command::new(output.join(format!("program{}", std::env::consts::EXE_SUFFIX))),
        root,
    ));
    assert_eq!(result.stdout, expected, "native");
    assert!(result.stderr.is_empty(), "{result:?}");
}

#[test]
fn global_unit_is_not_hidden_by_the_entry_parameter() {
    compare(
        "let unit = 1\npub effect fn main = println (show unit)\n",
        b"1\n",
    );
}

#[test]
fn implicit_parameters_preserve_global_and_captured_values_and_explicit_shadowing() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/implicit-unit-shadowing.ssrg"),
        include_bytes!(
            "../../../examples/spec/fixtures/rust-backend/implicit-unit-shadowing.stdout"
        ),
    );
}

#[test]
fn implicit_parameters_do_not_introduce_source_bindings_or_change_argument_types() {
    for source in [
        "pub effect fn main = println (show unit)\n",
        "let unit = 1\nfn explicit unit: Int -> Int = unit\npub effect fn main = println (show (explicit ()))\n",
    ] {
        let input = seseragi_driver::CompileInput::new("main.ssrg", "fixture/implicit-unit", source);
        assert_eq!(
            seseragi_driver::compile_core_module(input.clone()).unwrap_err(),
            seseragi_driver::compile_module(input).unwrap_err(),
        );
        let scratch = tempfile::tempdir().unwrap();
        let root = scratch.path();
        let input = root.join("main.ssrg");
        fs::write(&input, source).unwrap();
        let build = |target: &str| {
            bounded(
                cli()
                    .arg("build")
                    .arg(&input)
                    .args(["--target", target, "--out-dir"])
                    .arg(root.join(target)),
                root,
            )
        };
        let ts = build("process");
        assert_eq!(ts.status.code(), Some(2), "{ts:?}");
        assert!(ts.stdout.is_empty(), "{ts:?}");
        assert!(!ts.stderr.is_empty(), "{ts:?}");
        let native = build("native");
        assert_eq!(native.status.code(), ts.status.code(), "{native:?}");
        assert_eq!(native.stdout, ts.stdout);
        assert_eq!(
            native.stderr,
            format!("seseragi: {}\n", String::from_utf8(ts.stderr).unwrap()).as_bytes(),
        );
        assert!(!root.join("process").exists());
        assert!(!root.join("native").exists());
    }
}
