use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("seseragi-lint-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        Self { root }
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, source).unwrap();
        path
    }

    fn run(&self, arguments: &[&str], path: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_seseragi"))
            .arg("lint")
            .args(arguments)
            .arg(path)
            .output()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn frame(message: &serde_json::Value) -> Vec<u8> {
    let payload = serde_json::to_vec(message).unwrap();
    format!("Content-Length: {}\r\n\r\n", payload.len())
        .into_bytes()
        .into_iter()
        .chain(payload)
        .collect()
}

fn messages(bytes: &[u8]) -> Vec<serde_json::Value> {
    let mut remaining = bytes;
    let mut result = Vec::new();
    while !remaining.is_empty() {
        let boundary = remaining
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let headers = std::str::from_utf8(&remaining[..boundary]).unwrap();
        let length = headers
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let payload_start = boundary + 4;
        result.push(
            serde_json::from_slice(&remaining[payload_start..payload_start + length]).unwrap(),
        );
        remaining = &remaining[payload_start + length..];
    }
    result
}

fn utf8_position(source: &str, offset: usize) -> serde_json::Value {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let character = prefix
        .rsplit_once('\n')
        .map_or(prefix.len(), |(_, line)| line.len());
    serde_json::json!({"line": line, "character": character})
}

#[test]
fn single_file_lint_is_warn_only_unless_denied() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "main.ssrg",
        "fn value -> Int = {\n  let unused = 1\n  let used = 2\n  used\n}\n",
    );
    let default = fixture.run(&[], &path);
    assert_eq!(default.status.code(), Some(0));
    assert!(default.stdout.is_empty());
    let text = String::from_utf8_lossy(&default.stderr);
    assert!(text.contains("SES-L0301"), "{text}");
    assert!(text.contains("unused"), "{text}");
    assert!(
        !text.contains("`used` has no resolved references"),
        "{text}"
    );

    let denied = fixture.run(&["--deny-warnings", "--diagnostic-format", "json"], &path);
    assert_eq!(denied.status.code(), Some(1));
    assert!(denied.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&denied.stderr).unwrap();
    let diagnostics = value["diagnostics"][0]["diagnostics"]["diagnostics"]
        .as_array()
        .unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0]["code"], "SES-L0301");
    assert_eq!(diagnostics[0]["severity"], "Warning");
    assert_eq!(diagnostics[0]["messageKey"], "lint.unused-local-binding");
    assert_eq!(diagnostics[0]["fixes"], serde_json::json!([]));
    let range = &diagnostics[0]["primary"];
    let source = fs::read_to_string(path).unwrap();
    assert_eq!(
        &source[range["start"].as_u64().unwrap() as usize..range["end"].as_u64().unwrap() as usize],
        "unused"
    );
}

#[test]
fn cli_json_and_lsp_publish_the_same_utf8_lint_fact() {
    let fixture = Fixture::new();
    let source = "fn value -> Int = {\n  // 🙂\n  let unused = 1\n  1\n}\n";
    let path = fixture.write("unicode.ssrg", source);
    let output = fixture.run(&["--diagnostic-format", "json"], &path);
    assert_eq!(output.status.code(), Some(0));
    let cli: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let cli_diagnostic = &cli["diagnostics"][0]["diagnostics"]["diagnostics"][0];

    let uri = url::Url::from_file_path(&path).unwrap().to_string();
    let input = [
        serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"general": {"positionEncodings": ["utf-8"]}}}
        }),
        serde_json::json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
        serde_json::json!({"jsonrpc": "2.0", "method": "exit"}),
    ];
    let framed = input.iter().flat_map(frame).collect::<Vec<_>>();
    let mut output = Vec::new();
    seseragi_lsp::run(Cursor::new(framed), &mut output).unwrap();
    let lsp = messages(&output);
    let lsp_diagnostic = &lsp
        .iter()
        .find(|message| message["method"] == "textDocument/publishDiagnostics")
        .unwrap()["params"]["diagnostics"][0];

    assert_eq!(lsp_diagnostic["code"], cli_diagnostic["code"]);
    assert_eq!(lsp_diagnostic["severity"], 2);
    assert_eq!(cli_diagnostic["severity"], "Warning");
    assert_eq!(
        lsp_diagnostic["data"]["messageKey"],
        cli_diagnostic["messageKey"]
    );
    let start = cli_diagnostic["primary"]["start"].as_u64().unwrap() as usize;
    let end = cli_diagnostic["primary"]["end"].as_u64().unwrap() as usize;
    assert_eq!(
        lsp_diagnostic["range"]["start"],
        utf8_position(source, start)
    );
    assert_eq!(lsp_diagnostic["range"]["end"], utf8_position(source, end));
}

#[test]
fn missing_source_inside_a_package_is_rejected() {
    let fixture = Fixture::new();
    fixture.write(
        "seseragi.toml",
        "[package]\nname = \"fixture/lint-missing\"\nversion = \"0.1.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"main\"\n",
    );
    fixture.write("src/main.ssrg", "pub let value: Int = 1\n");
    let missing = fixture.root.join("src/missing.ssrg");
    let output = fixture.run(&[], &missing);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("lint path does not exist"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn compiler_errors_are_not_downgraded_or_followed_by_speculative_lints() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "broken.ssrg",
        "fn value -> Int = {\n  let unused = missing\n  1\n}\n",
    );
    let output = fixture.run(&["--diagnostic-format", "json"], &path);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let diagnostics = value["diagnostics"][0]["diagnostics"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(diagnostics.iter().any(|entry| entry["severity"] == "Error"));
    assert!(diagnostics.iter().all(|entry| entry["code"] != "SES-L0301"));
}

#[test]
fn package_lint_analyzes_linked_modules_in_source_order() {
    let fixture = Fixture::new();
    fixture.write(
        "seseragi.toml",
        "[package]\nname = \"fixture/lint\"\nversion = \"0.1.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"main\"\n",
    );
    fixture.write(
        "src/main.ssrg",
        "import { answer } from \"./util\"\n\npub fn main -> Int = {\n  let unusedMain = 1\n  answer\n}\n",
    );
    fixture.write(
        "src/util.ssrg",
        "pub let answer: Int = {\n  let unusedUtil = 2\n  42\n}\n",
    );
    let output = fixture.run(&["--diagnostic-format", "json"], &fixture.root);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let documents = value["diagnostics"].as_array().unwrap();
    assert_eq!(documents.len(), 2);
    assert!(documents[0]["path"]
        .as_str()
        .unwrap()
        .ends_with("src/main.ssrg"));
    assert!(documents[1]["path"]
        .as_str()
        .unwrap()
        .ends_with("src/util.ssrg"));
    assert_eq!(
        documents[0]["diagnostics"]["diagnostics"][0]["code"],
        "SES-L0301"
    );
    assert_eq!(
        documents[1]["diagnostics"]["diagnostics"][0]["code"],
        "SES-L0301"
    );

    fixture.write("src/main.ssrg", "pub let result: Int =\n");
    let broken = fixture.run(&["--diagnostic-format", "json"], &fixture.root);
    assert_eq!(broken.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&broken.stderr).unwrap();
    assert!(value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|document| {
            document["diagnostics"]["diagnostics"]
                .as_array()
                .is_some_and(|entries| entries.iter().any(|entry| entry["severity"] == "Error"))
        }));
}

#[test]
fn dependency_is_used_for_analysis_but_not_linted() {
    let fixture = Fixture::new();
    fixture.write(
        "seseragi.toml",
        "[package]\nname = \"fixture/lint-root\"\nversion = \"0.1.0\"\nlanguage = \"^0.1.0\"\n\n[dependencies]\ndomain = { package = \"fixture/lint-domain\", path = \"vendor/domain\" }\n",
    );
    fixture.write(
        "src/main.ssrg",
        "import { answer } from \"domain\"\n\npub let result: Int = answer\n",
    );
    fixture.write(
        "vendor/domain/seseragi.toml",
        "[package]\nname = \"fixture/lint-domain\"\nversion = \"0.1.0\"\nlanguage = \"^0.1.0\"\n\n[exports]\n\".\" = \"lib\"\n",
    );
    let dependency = "pub let answer: Int = {\n  let unusedDependency = 1\n  42\n}\n";
    fixture.write("vendor/domain/src/lib.ssrg", dependency);
    let output = fixture.run(&["--diagnostic-format", "json"], &fixture.root);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let documents = value["diagnostics"].as_array().unwrap();
    assert_eq!(documents.len(), 1);
    assert!(documents[0]["path"]
        .as_str()
        .unwrap()
        .ends_with("src/main.ssrg"));
    assert_eq!(
        documents[0]["diagnostics"]["diagnostics"],
        serde_json::json!([])
    );

    fixture.write(
        "vendor/domain/src/lib.ssrg",
        "pub let answer: Int = missing\n",
    );
    let broken = fixture.run(&["--diagnostic-format", "json"], &fixture.root);
    assert_eq!(broken.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&broken.stderr).unwrap();
    let documents = value["diagnostics"].as_array().unwrap();
    assert!(documents.iter().any(|document| {
        document["path"]
            .as_str()
            .is_some_and(|path| path.ends_with("vendor/domain/src/lib.ssrg"))
            && document["diagnostics"]["diagnostics"]
                .as_array()
                .is_some_and(|entries| entries.iter().any(|entry| entry["severity"] == "Error"))
    }));
}
