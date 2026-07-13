use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn frame(message: &Value) -> Vec<u8> {
    let payload = serde_json::to_vec(message).unwrap();
    format!("Content-Length: {}\r\n\r\n", payload.len())
        .into_bytes()
        .into_iter()
        .chain(payload)
        .collect()
}

fn messages(bytes: &[u8]) -> Vec<Value> {
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

#[test]
fn binary_serves_open_document_diagnostics_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_seseragi-lsp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"general": {"positionEncodings": ["utf-16"]}}}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": "file:///human.ssrg", "languageId": "seseragi", "version": 1,
                "text": "// 🙂\npub let broken: Int =\n"
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in &input {
            stdin.write_all(&frame(message)).unwrap();
        }
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let messages = messages(&output.stdout);
    assert_eq!(
        messages[0]["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    assert_eq!(messages[1]["method"], "textDocument/publishDiagnostics");
    assert_eq!(messages[1]["params"]["uri"], "file:///human.ssrg");
    assert!(!messages[1]["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}
