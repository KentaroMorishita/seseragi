use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

const WARNING_IMPORT: &str = "import * as html from \"std/web/html\"\n";
const WARNING_DECLARATION: &str =
    "\npub fn typo -> html.Html<Never> =\n  html.div { clas: \"hero\", children: \"Typo\" }\n";

const REQUIRES_PROVIDER: &str = "import * as http from \"std/http\"\n\npub effect fn main -> Unit\nwith httpClient: http.HttpClient =\n  succeed ()\n";

struct OwnedChild(Option<Child>);

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn cli(root: &Path, arguments: &[&str], empty_path: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi"));
    command
        .args(arguments)
        .current_dir(root)
        .env("TMPDIR", root.join("runtime"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if empty_path {
        command.env("PATH", "");
    }
    let mut owned = OwnedChild(Some(command.spawn().unwrap()));
    let pid = owned.0.as_ref().unwrap().id();
    let started = Instant::now();
    while owned.0.as_mut().unwrap().try_wait().unwrap().is_none() {
        assert!(
            started.elapsed() < Duration::from_secs(15),
            "CLI {pid} timed out"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let output = owned.0.take().unwrap().wait_with_output().unwrap();
    eprintln!("owned CLI {pid} reaped: {:?}", output.status.code());
    output
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                collect(root, &entry.path(), files);
            } else {
                files.insert(
                    entry.path().strip_prefix(root).unwrap().to_owned(),
                    fs::read(entry.path()).unwrap(),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    collect(root, root, &mut files);
    files
}

#[test]
fn standalone_provider_requirement_is_a_text_or_json_diagnostic_without_a_host() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    fs::create_dir(root.join("runtime")).unwrap();
    for with_warning in [false, true] {
        let source = if with_warning {
            format!("{WARNING_IMPORT}{REQUIRES_PROVIDER}{WARNING_DECLARATION}")
        } else {
            REQUIRES_PROVIDER.to_owned()
        };
        fs::write(root.join("main.ssrg"), source).unwrap();
        for format in ["text", "json"] {
            let output = cli(
                root,
                &["run", "main.ssrg", "--diagnostic-format", format],
                true,
            );
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("requires a resolved provider"), "{stderr}");
            assert!(stderr.contains("std/http::HttpClient"), "{stderr}");
            assert!(!stderr.contains("panicked"), "{stderr}");
            if format == "json" {
                // from_str rejects a second concatenated JSON document.
                let value: serde_json::Value = serde_json::from_str(&stderr).unwrap();
                assert_eq!(value["schema"], 1);
                let diagnostics = value["diagnostics"][0]["diagnostics"]["diagnostics"]
                    .as_array()
                    .unwrap();
                let diagnostic = diagnostics
                    .iter()
                    .find(|value| value["messageKey"] == "entry.invalid")
                    .unwrap();
                assert!(diagnostic["labels"][0]["message"]
                    .as_str()
                    .unwrap()
                    .contains("httpClient"));
                assert_eq!(
                    diagnostics
                        .iter()
                        .filter(|value| value["severity"] == "Warning")
                        .count(),
                    usize::from(with_warning)
                );
                assert_eq!(diagnostics.len(), 1 + usize::from(with_warning));
            }
            assert_eq!(fs::read_dir(root.join("runtime")).unwrap().count(), 0);
        }
    }
    let output = cli(
        root,
        &[
            "build",
            "main.ssrg",
            "--target",
            "process",
            "--out-dir",
            "out",
        ],
        true,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires a resolved provider"));
    assert!(!root.join("out").exists());
    assert_eq!(fs::read_dir(root.join("runtime")).unwrap().count(), 0);
}

#[test]
fn console_standalone_still_runs_and_provider_rejection_preserves_completed_build() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    fs::create_dir(root.join("runtime")).unwrap();
    fs::write(
        root.join("main.ssrg"),
        "pub effect fn main -> Unit with Console fails ConsoleError =\n  println \"standalone works\"\n",
    )
    .unwrap();
    for format in ["text", "json"] {
        let output = cli(
            root,
            &["run", "main.ssrg", "--diagnostic-format", format],
            false,
        );
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(output.stdout, b"standalone works\n");
        assert!(output.stderr.is_empty(), "{:?}", output);
        assert_eq!(fs::read_dir(root.join("runtime")).unwrap().count(), 0);
    }
    let arguments = [
        "build",
        "main.ssrg",
        "--target",
        "process",
        "--out-dir",
        "out",
    ];
    let output = cli(root, &arguments, false);
    assert!(output.status.success(), "{:?}", output);
    let previous = snapshot(&root.join("out"));
    assert!(!previous.is_empty());
    fs::write(root.join("main.ssrg"), REQUIRES_PROVIDER).unwrap();
    let output = cli(root, &arguments, true);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires a resolved provider"));
    assert_eq!(snapshot(&root.join("out")), previous);
    assert_eq!(fs::read_dir(root.join("runtime")).unwrap().count(), 0);
    assert_eq!(fs::read_dir(root).unwrap().count(), 3);
}

#[test]
fn successful_json_run_emits_compile_warnings_before_application_initialization() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    fs::create_dir(root.join("runtime")).unwrap();
    let body = r#"foreign "typescript" from "./host.mjs" {
  pure fn identity value: String -> String
}
pub effect fn main = println (identity "standalone works")
"#;
    fs::write(
        root.join("main.ssrg"),
        format!("{WARNING_IMPORT}{body}{WARNING_DECLARATION}"),
    )
    .unwrap();
    fs::write(root.join("host.mjs"), "import {writeSync} from 'node:fs';\nwriteSync(2, 'program-started\\n');\nexport function identity(value) { return value; }\n").unwrap();
    let output = cli(
        root,
        &["run", "main.ssrg", "--diagnostic-format", "json"],
        false,
    );
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(output.stdout, b"standalone works\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let (diagnostic, marker) = stderr.split_once('\n').unwrap();
    let value: serde_json::Value = serde_json::from_str(diagnostic).unwrap();
    assert_eq!(
        value["diagnostics"][0]["diagnostics"]["diagnostics"][0]["severity"],
        "Warning"
    );
    assert_eq!(marker, "program-started\n");
    assert_eq!(fs::read_dir(root.join("runtime")).unwrap().count(), 0);
}
