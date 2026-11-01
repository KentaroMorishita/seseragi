use std::{fs, path::Path, process::Command};

#[allow(dead_code)]
#[path = "support/generic_impl_process.rs"]
mod process;
use process::{bounded, successful};

fn check_generated_typescript(source: &Path, profile: &str, output: &Path, scratch: &Path) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source_text = fs::read_to_string(source).unwrap();
    let profile = match profile {
        "development" => seseragi_driver::BuildProfile::Development,
        "release" => seseragi_driver::BuildProfile::Release,
        _ => unreachable!(),
    };
    let compiled = seseragi_driver::compile_module(
        seseragi_driver::CompileInput::new(
            source.to_str().unwrap(),
            "fixture/html-saved",
            &source_text,
        )
        .with_profile(profile),
    )
    .unwrap();
    let raw = output.join("original.ts");
    fs::write(&raw, compiled.generated.typescript).unwrap();
    let config = output.join("tsconfig.json");
    fs::write(
        &config,
        serde_json::json!({
            "compilerOptions": {
                "strict": true, "skipLibCheck": false, "noEmit": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "Bundler",
                "allowImportingTsExtensions": true, "types": ["bun", "node"],
                "typeRoots": [root.join("node_modules/@types")],
                "lib": ["ES2023", "DOM", "DOM.Iterable"],
                "baseUrl": root, "paths": {"@seseragi/runtime/*": ["runtime/ts/src/*"]}
            }, "files": [raw]
        })
        .to_string(),
    )
    .unwrap();
    successful(bounded(
        Command::new("node")
            .arg(root.join("node_modules/typescript/bin/tsc"))
            .arg("--project")
            .arg(config),
        scratch,
    ));
}

#[test]
fn inferred_html_helpers_build_and_match_direct_calls_in_both_profiles() {
    let scratch = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/spec/fixtures/compile/html-saved-helpers.ssrg");
    let expected = fs::read(source.with_extension("stdout")).unwrap();
    let cli = env!("CARGO_BIN_EXE_seseragi");
    let checked = successful(bounded(
        Command::new(cli).arg("lint").arg(&source),
        scratch.path(),
    ));
    assert!(checked.stderr.is_empty(), "{checked:?}");
    for profile in ["development", "release"] {
        let output = scratch.path().join(profile);
        let built = successful(bounded(
            Command::new(cli)
                .arg("build")
                .arg(&source)
                .args(["--profile", profile, "--out-dir"])
                .arg(&output),
            scratch.path(),
        ));
        assert!(built.stderr.is_empty(), "{built:?}");
        check_generated_typescript(&source, profile, &output, scratch.path());
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(output.join("artifact-manifest.json")).unwrap())
                .unwrap();
        for runtime in ["bun", "node"] {
            let result = successful(bounded(
                Command::new(runtime).arg(output.join(manifest["entry"].as_str().unwrap())),
                scratch.path(),
            ));
            assert_eq!(result.stdout, expected, "{profile} / {runtime}");
            assert!(result.stderr.is_empty(), "{result:?}");
        }
    }
}

#[test]
fn invalid_stored_html_helper_calls_report_diagnostics_without_building() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("invalid.ssrg");
    let output = scratch.path().join("output");
    for (helper, argument) in [
        ("style", "{ color: 42 }"),
        ("style", "{ variables: { accentColor: 42 } }"),
        ("div", "{ children: 42 }"),
        ("fragment", "42"),
    ] {
        fs::write(
            &source,
            format!(
                "import * as html from \"std/web/html\"\nlet saved = html.{helper}\nlet invalid = saved {argument}\npub effect fn main = println \"unexpected\"\n"
            ),
        )
        .unwrap();
        for command in ["lint", "build"] {
            let mut process = Command::new(env!("CARGO_BIN_EXE_seseragi"));
            process.arg(command).arg(&source);
            if command == "build" {
                process.arg("--out-dir").arg(&output);
            }
            let result = bounded(&mut process, scratch.path());
            assert_eq!(result.status.code(), Some(2), "{result:?}");
            assert!(result.stdout.is_empty(), "{result:?}");
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("SES-T0201"),
                "{result:?}"
            );
            assert!(!output.exists());
        }
    }
}
