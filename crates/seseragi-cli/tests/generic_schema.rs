//! Common-IR generic declarations retain relations and checked runtime codecs.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[path = "support/temporary.rs"]
mod temporary_support;

fn cli(args: &[&str], package: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seseragi"))
        .args(args)
        .current_dir(package)
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
fn copy(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let to = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), to).unwrap();
        }
    }
}
fn fixture() -> PathBuf {
    let package = temporary_support::temporary_directory("seseragi-generic-schema");
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/fixtures/projects/dts-generic-schema"),
        &package,
    );
    success(&cli(&["dts", "convert", "."], &package));
    success(&cli(&["lock", "update", "."], &package));
    package
}

#[test]
fn generated_generic_schema_imports_run_first_class_and_partial_on_node_and_bun() {
    let package = fixture();
    let expected = "linked\nlinked\nnested\nTrue\n";
    let generated =
        fs::read_to_string(package.join(".seseragi/generated/schema-api.ssrg")).unwrap();
    assert!(generated.contains("opaque type Schema<A>"), "{generated}");
    assert!(generated.contains("schema: Schema<A> -> A"), "{generated}");
    let run = cli(&["run", ".", "--target", "process"], &package);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    for profile in ["development", "release"] {
        success(&cli(
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
            &package,
        ));
        let out = package.join(profile);
        let artifact: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for host in ["node", "bun"] {
            let run = Command::new(host)
                .arg(out.join(artifact["entry"].as_str().unwrap()))
                .output()
                .unwrap();
            success(&run);
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                expected,
                "{profile}/{host}"
            );
        }
    }
    fs::remove_dir_all(package).unwrap();
}

#[test]
fn generic_schema_relations_reject_mismatched_calls_and_host_returns() {
    let package = fixture();
    let main = package.join("src/main.ssrg");
    let source = fs::read_to_string(&main).unwrap();
    fs::write(
        &main,
        source.replace("read<Bool> boolSchema", "read<Bool> textSchema"),
    )
    .unwrap();
    let lint = cli(&["lint", ".", "--diagnostic-format", "json"], &package);
    assert!(
        !lint.status.success(),
        "{}",
        String::from_utf8_lossy(&lint.stdout)
    );
    assert!(
        String::from_utf8_lossy(&lint.stdout).contains("type")
            || String::from_utf8_lossy(&lint.stderr).contains("type"),
        "{lint:?}"
    );
    fs::write(&main, source).unwrap();
    let host = package.join("host/node_modules/schema-api/index.mjs");
    let js = fs::read_to_string(&host).unwrap();
    fs::write(&host, js.replace("return schema.value;", "return 7;")).unwrap();
    // Deliberately refresh the artifact identity after changing the host implementation.
    // This reaches the selected codec boundary rather than testing stale bytes.
    success(&cli(&["dts", "convert", "."], &package));
    success(&cli(&["lock", "update", "."], &package));
    let run = cli(&["run", ".", "--target", "process"], &package);
    assert!(
        !run.status.success(),
        "incorrect host return was accepted: {run:?}"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        diagnostic.contains("ForeignFailure") && diagnostic.contains("string"),
        "{diagnostic}"
    );
    fs::remove_dir_all(package).unwrap();
}

#[test]
fn pure_generated_generic_schema_namespace_reexports_keep_selected_codecs() {
    let package = fixture();
    let settings = package.join("seseragi.bindings.toml");
    let value = fs::read_to_string(&settings)
        .unwrap()
        .replace("evaluation = \"task\"", "evaluation = \"pure\"");
    fs::write(&settings, value).unwrap();
    fs::write(
        package.join("src/main.ssrg"),
        r#"import { make, read } from "./api"
import * as facade from "./api"
pub effect fn main = do {
  println (read<String> (make<String> "pure"))
  println (show (read<Bool> (make<Bool> True)))
  println (facade.raw.nested.echo<String> "namespace")
}
"#,
    )
    .unwrap();
    success(&cli(&["dts", "convert", "."], &package));
    success(&cli(&["lock", "update", "."], &package));
    let expected = "pure\nTrue\nnamespace\n";
    let run = cli(&["run", ".", "--target", "process"], &package);
    success(&run);
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    for profile in ["development", "release"] {
        success(&cli(
            &["build", ".", "--profile", profile, "--out-dir", profile],
            &package,
        ));
        let out = package.join(profile);
        let artifact: serde_json::Value =
            serde_json::from_slice(&fs::read(out.join("artifact-manifest.json")).unwrap()).unwrap();
        for host in ["node", "bun"] {
            let run = Command::new(host)
                .arg(out.join(artifact["entry"].as_str().unwrap()))
                .output()
                .unwrap();
            success(&run);
            assert_eq!(
                String::from_utf8_lossy(&run.stdout),
                expected,
                "{profile}/{host}"
            );
        }
    }
    fs::remove_dir_all(package).unwrap();
}
