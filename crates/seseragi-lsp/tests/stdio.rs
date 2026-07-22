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

fn run_server(input: &[Value]) -> Vec<Value> {
    run_server_with_args(input, &[])
}

fn run_server_with_args(input: &[Value], args: &[&str]) -> Vec<Value> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_seseragi-lsp"));
    command.args(args);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        for message in input {
            stdin.write_all(&frame(message)).unwrap();
        }
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    messages(&output.stdout)
}

fn response(messages: &[Value], id: i64) -> &Value {
    messages.iter().find(|message| message["id"] == id).unwrap()
}

#[test]
fn binary_reports_its_distribution_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_seseragi-lsp"))
        .arg("--version-json")
        .output()
        .unwrap();
    assert!(output.status.success());
    let version: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(version["name"], "seseragi-lsp");
    assert_eq!(version["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(version["protocolVersion"], 1);
    assert_eq!(version["analysisSchemaVersion"], 1);
}

#[test]
fn binary_accepts_explicit_stdio_transport() {
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}}
        }),
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server_with_args(&input, &["--stdio"]);
    assert_eq!(messages[0]["result"]["serverInfo"]["name"], "seseragi-lsp");
}

#[test]
fn namespace_completion_stays_scoped_inside_an_incomplete_nested_expression() {
    let uri = "file:///nested-completion.ssrg";
    let source = concat!(
        "// 🙂\n",
        "import * as html from \"std/web/html\"\n",
        "fn view -> html.Html<Never> =\n",
        "  html.main {\n",
        "    children: [\n",
        "      html.  \n",
    );
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"general": {"positionEncodings": ["utf-16"]}}}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 5, "character": 13}
            }
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": "import * as html from \"std/web/html\"\nfn view = unknown.\n"}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": 1, "character": 18}
            }
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let nested = response(&messages, 2)["result"].as_array().unwrap();
    assert!(nested.iter().any(|item| item["label"] == "div"));
    assert!(nested.iter().any(|item| item["label"] == "span"));
    assert!(!nested.iter().any(|item| item["label"] == "Maybe"));
    assert!(!nested.iter().any(|item| item["label"] == "Monoid"));

    let unresolved = response(&messages, 3)["result"].as_array().unwrap();
    assert!(unresolved.is_empty(), "{unresolved:?}");
}

#[test]
fn binary_serves_open_document_diagnostics_over_stdio() {
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
    let messages = run_server(&input);
    assert_eq!(
        messages[0]["result"]["capabilities"]["positionEncoding"],
        "utf-16"
    );
    assert_eq!(
        messages[0]["result"]["experimental"]["seseragi"]["protocolVersion"],
        1
    );
    assert_eq!(messages[1]["method"], "textDocument/publishDiagnostics");
    assert_eq!(messages[1]["params"]["uri"], "file:///human.ssrg");
    assert!(!messages[1]["params"]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn binary_serves_analysis_features_and_quick_fixes_over_stdio() {
    let uri = "file:///features.ssrg";
    let source = concat!(
        "// 🙂\n",
        "import * as html from \"std/web/html\"\n",
        "fn add left: Int -> right: Int -> Int = left + right\n",
        "let addOne: Int -> Int = add 1\n",
        "let page = html.\n",
    );
    let fixed_source = concat!(
        "pub struct User {\n",
        "  name: String,\n",
        "  score: Int,\n",
        "}\n",
        "\n",
        "let user: User = User { name: \"Aki\", score: 42 }\n",
        "let label: String = user.nmae\n",
    );
    let position = |line, character| json!({"line": line, "character": character});
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"general": {"positionEncodings": ["utf-16"]}}}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": position(3, 26)}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": uri}, "position": position(3, 30)}
        }),
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "textDocument/signatureHelp",
            "params": {"textDocument": {"uri": uri}, "position": position(3, 30)}
        }),
        json!({
            "jsonrpc": "2.0", "id": 5, "method": "textDocument/definition",
            "params": {"textDocument": {"uri": uri}, "position": position(3, 26)}
        }),
        json!({
            "jsonrpc": "2.0", "id": 6, "method": "textDocument/semanticTokens/full",
            "params": {"textDocument": {"uri": uri}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 10, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": uri}, "position": position(4, 16)}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": fixed_source}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "textDocument/codeAction",
            "params": {
                "textDocument": {"uri": uri},
                "range": {"start": position(0, 0), "end": position(7, 0)},
                "context": {"diagnostics": []}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 8, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": position(99, 0)}
        }),
        json!({"jsonrpc": "2.0", "id": 9, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let capabilities = &response(&messages, 1)["result"]["capabilities"];
    assert_eq!(capabilities["hoverProvider"], true);
    assert_eq!(capabilities["definitionProvider"], true);
    assert!(
        capabilities["semanticTokensProvider"]["legend"]["tokenTypes"]
            .as_array()
            .is_some_and(|types| types.iter().any(|item| item == "operator"))
    );

    let hover = &response(&messages, 2)["result"];
    assert!(hover["contents"]["value"]
        .as_str()
        .is_some_and(|value| value.contains("add left: Int") && value.contains("features.ssrg")));

    let completions = response(&messages, 3)["result"].as_array().unwrap();
    assert!(completions.iter().any(|item| item["label"] == "addOne"));
    assert!(completions.iter().any(|item| item["label"] == "join"));

    let signature = &response(&messages, 4)["result"];
    assert!(signature["signatures"][0]["label"]
        .as_str()
        .is_some_and(|label| label.contains("right: Int")));
    assert_eq!(signature["activeParameter"], 1);

    let definition = &response(&messages, 5)["result"];
    assert_eq!(definition["uri"], uri);
    assert_eq!(definition["range"]["start"], position(2, 3));

    assert!(!response(&messages, 6)["result"]["data"]
        .as_array()
        .unwrap()
        .is_empty());

    let namespace_completion = response(&messages, 10)["result"].as_array().unwrap();
    assert!(namespace_completion
        .iter()
        .any(|item| item["label"] == "div"));
    assert!(!namespace_completion
        .iter()
        .any(|item| item["label"] == "addOne"));

    let actions = response(&messages, 7)["result"].as_array().unwrap();
    assert!(actions
        .iter()
        .any(|action| { action["edit"]["changes"][uri][0]["newText"] == "name" }));
    assert!(response(&messages, 8)["result"].is_null());
}
