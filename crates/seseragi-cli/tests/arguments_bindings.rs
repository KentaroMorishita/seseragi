use std::{fs, path::Path, process::Command};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, runtime, successful};

fn compare(source: &str, expected: &[u8], failure: Option<&str>) {
    let scratch = tempfile::tempdir().unwrap();
    let root = scratch.path();
    let file = root.join("main.ssrg");
    fs::write(&file, source).unwrap();
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap();
    let code = if failure.is_some() { 1 } else { 0 };
    for (profile, build_profile) in [
        ("development", seseragi_driver::BuildProfile::Development),
        ("release", seseragi_driver::BuildProfile::Release),
    ] {
        let out = root.join(profile);
        successful(bounded(
            Command::new(env!("CARGO_BIN_EXE_seseragi"))
                .arg("build")
                .arg(&file)
                .args([
                    "--target",
                    "process",
                    "--source-map",
                    "omit",
                    "--profile",
                    profile,
                    "--out-dir",
                ])
                .arg(&out),
            root,
        ));
        let compiled = seseragi_driver::compile_module(
            seseragi_driver::CompileInput::new("main.ssrg", "proof/arguments", source)
                .with_profile(build_profile),
        )
        .unwrap();
        let raw = out.join("raw.ts");
        fs::write(&raw, compiled.generated.typescript).unwrap();
        let mut files = vec![raw];
        if profile == "development" {
            files.push(out.join("entry.ts"));
        }
        let config = serde_json::json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": false,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun", "node"],
                "typeRoots": [repository.join("node_modules/@types")],
                "baseUrl": repository,
                "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]},
                "lib": ["ES2023", "DOM", "DOM.Iterable"]
            },
            "files": files
        });
        fs::write(out.join("tsconfig.json"), config.to_string()).unwrap();
        successful(bounded(
            Command::new(runtime("node", root))
                .arg(repository.join("node_modules/typescript/bin/tsc"))
                .arg("--project")
                .arg(out.join("tsconfig.json")),
            root,
        ));
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for engine in ["node", "bun"] {
            let output = bounded(
                Command::new(runtime(engine, root))
                    .arg(out.join(manifest["entry"].as_str().unwrap()))
                    .current_dir(root)
                    .env("PATH", ""),
                root,
            );
            assert_eq!(
                output.status.code(),
                Some(code),
                "{profile}/{engine}: {output:?}"
            );
            assert_eq!(output.stdout, expected, "{profile}/{engine}");
            assert_eq!(
                output.stderr,
                failure
                    .map(|message| format!("{message}\n"))
                    .unwrap_or_default()
                    .as_bytes(),
                "{profile}/{engine}"
            );
        }
    }
    let native = root.join("native");
    successful(bounded(
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .args(["experimental", "build-native"])
            .arg(&file)
            .args(["--entry", "main", "--out-dir"])
            .arg(&native),
        root,
    ));
    let output = bounded(
        Command::new(native.join(format!("program{}", std::env::consts::EXE_SUFFIX)))
            .current_dir(root)
            .env("PATH", ""),
        root,
    );
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert_eq!(output.stdout, expected);
    if let Some(message) = failure {
        let stderr = String::from_utf8(output.stderr).unwrap();
        let location = format!("{}:2:", file.display());
        let (column, actual) = stderr
            .strip_prefix(&location)
            .unwrap()
            .split_once(": ")
            .unwrap();
        assert!(column.parse::<usize>().is_ok(), "{stderr}");
        assert_eq!(actual, format!("{message}\n"));
    } else {
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn arguments_reproduction_builds_strict_ts_and_runs_on_node_bun_and_native() {
    compare(
        "let arguments = 1\npub effect fn main = println (show arguments)\n",
        b"1\n",
        None,
    );
}

#[test]
fn arguments_bindings_keep_scopes_collisions_and_int_boundaries() {
    compare(
        include_str!("../../../examples/spec/fixtures/rust-backend/arguments-bindings.ssrg"),
        include_bytes!("../../../examples/spec/fixtures/rust-backend/arguments-bindings.stdout"),
        None,
    );
}

#[test]
fn arguments_parameter_preserves_typed_failure_message_and_exit_status() {
    compare(
        "import * as effects from \"std/effect\"\neffect fn failed arguments: String -> Unit fails String = effects.fail arguments\npub effect fn main -> Unit fails String = failed \"binding failed\"\n",
        b"",
        Some("binding failed"),
    );
}
