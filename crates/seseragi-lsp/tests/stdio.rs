use serde_json::{json, Value};
use seseragi_driver::format_module;
use seseragi_source::{LineIndex, PositionEncoding};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

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

fn file_uri(path: &Path) -> String {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    Url::from_file_path(path).unwrap().into()
}

fn published<'messages>(messages: &'messages [Value], uri: &str) -> &'messages Value {
    messages
        .iter()
        .rev()
        .find(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
        })
        .unwrap_or_else(|| panic!("diagnostics for {uri}: {messages:#?}"))
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
    assert!(version["commit"].as_str().is_some());
    assert!(version["channel"].as_str().is_some());
    assert!(version["target"].as_str().is_some());
    assert!(version["dirty"].is_boolean());
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
fn hover_uses_shared_types_for_destructured_let_and_match_bindings() {
    let uri = "file:///pattern-binding-hover.ssrg";
    let source = concat!(
        "let operations: (Int -> Int, Int -> Int) = ",
        "(\\value: Int -> value + 1, \\value: Int -> value * 2)\n",
        "let (increment, double) = operations\n",
        "let result: Int = increment 10 + double 10\n",
        "fn label pair: (Int, String) -> String =\n",
        "  match pair {\n",
        "    (number, text) -> text\n",
        "  }\n",
    );
    let locate = |offset| {
        let position = LineIndex::new(source)
            .try_locate_encoded(offset, PositionEncoding::Utf16)
            .unwrap();
        json!({"line": position.line, "character": position.character})
    };
    let increment = source.rfind("increment 10").unwrap();
    let text = source.rfind("text\n").unwrap();
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {
                "textDocument": {"hover": {"contentFormat": ["plaintext"]}}
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": locate(increment)}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": locate(text)}
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let let_hover = response(&messages, 2)["result"]["contents"]["value"]
        .as_str()
        .expect("let binding hover");
    assert!(
        let_hover.contains("increment\n  arg1: Int\n  -> Int"),
        "{let_hover}"
    );
    assert!(!let_hover.contains("unknown"), "{let_hover}");

    let match_hover = response(&messages, 3)["result"]["contents"]["value"]
        .as_str()
        .expect("match binding hover");
    assert!(match_hover.contains("text:\nString"), "{match_hover}");
    assert!(!match_hover.contains("unknown"), "{match_hover}");
}

#[test]
fn prelude_registry_methods_reach_hover_and_completion_with_canonical_signatures() {
    let uri = "file:///prelude-registry.ssrg";
    let source = "fn same left: Int -> right: Int -> Bool = eq left right\nlet probe = tra\n";
    let index = LineIndex::new(source);
    let hover_offset = source.find("eq left").unwrap() + 1;
    let completion_offset = source.find("tra").unwrap() + 3;
    let hover = index
        .try_locate_encoded(hover_offset, PositionEncoding::Utf16)
        .unwrap();
    let completion = index
        .try_locate_encoded(completion_offset, PositionEncoding::Utf16)
        .unwrap();
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"textDocument": {
                "hover": {"contentFormat": ["plaintext"]},
                "completion": {"completionItem": {"documentationFormat": ["plaintext"]}}
            }}}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": {
                "line": hover.line, "character": hover.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": uri}, "position": {
                "line": completion.line, "character": completion.character
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let hover = response(&messages, 2)["result"]["contents"]["value"]
        .as_str()
        .expect("Eq.eq hover");
    assert!(hover.contains("eq"), "{hover}");
    assert!(hover.contains("Eq<"), "{hover}");
    assert!(hover.contains("Bool"), "{hover}");

    let completions = response(&messages, 3)["result"].as_array().unwrap();
    let traverse = completions
        .iter()
        .find(|item| item["data"]["identity"] == "std/prelude::Traversable::traverse")
        .expect("Traversable.traverse completion");
    let detail = traverse["detail"].as_str().expect("traverse detail");
    assert!(detail.contains("Traversable<F"), "{detail}");
    assert!(detail.contains("Applicative<G"), "{detail}");
    for (identity, result) in [
        ("std/maybe::traverse", "Maybe<F<B>>"),
        ("std/either::traverse", "Either<E, F<B>>"),
    ] {
        let item = completions
            .iter()
            .find(|item| item["data"]["identity"] == identity)
            .expect("module-specific traverse completion");
        let detail = item["detail"].as_str().expect("traverse detail");
        assert!(detail.contains("Traversable<F"), "{detail}");
        assert!(detail.contains(result), "{detail}");
        assert!(!detail.contains("Applicative<G"), "{detail}");
    }
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
fn expected_record_completion_uses_analysis_and_recovers_an_unclosed_record() {
    let uri = "file:///expected-record-completion.ssrg";
    let source = r##"import * as dom from "std/web/dom"
import * as html from "std/web/html"

type Mode = | Ready
type Action = | Reset
let initial_mode: Mode = Ready
fn update action: Action -> mode: Mode -> Mode = mode

pub effect fn main =
  dom.app {
    target: "#app",
    initial: initial_mode,
    update
  }
"##;
    let incomplete = r#"import * as dom from "std/web/dom"

pub effect fn main =
  dom.app {
"#;
    let lsp_position = |text: &str, byte_offset: usize| {
        let position = LineIndex::new(text)
            .try_locate_encoded(byte_offset, PositionEncoding::Utf16)
            .unwrap();
        json!({"line": position.line, "character": position.character})
    };
    let source_cursor = source.rfind("  }").expect("app record close") + 2;
    let value_cursor = source.rfind("initial_mode,").expect("record field value") + 4;
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
                "position": lsp_position(source, source_cursor)
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 5, "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": uri},
                "position": lsp_position(source, value_cursor)
            }
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": incomplete}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {
                "textDocument": {"uri": uri},
                "position": lsp_position(incomplete, incomplete.len())
            }
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let complete = response(&messages, 2)["result"].as_array().unwrap();
    assert_eq!(complete.len(), 1, "{complete:?}");
    assert_eq!(complete[0]["label"], "view");
    assert_eq!(complete[0]["detail"], "Mode -> Html<Action>");
    assert_eq!(complete[0]["insertText"], "view: ");
    assert_eq!(complete[0]["kind"], 5);

    let value_completions = response(&messages, 5)["result"].as_array().unwrap();
    assert!(value_completions
        .iter()
        .any(|item| item["label"] == "initial_mode"));
    assert!(!value_completions
        .iter()
        .any(|item| item["data"]["kind"] == "expected-record-field"));

    let recovered = response(&messages, 3)["result"].as_array().unwrap();
    assert_eq!(
        recovered
            .iter()
            .map(|item| item["label"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["target", "initial", "update", "view"]
    );
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
fn binary_formats_the_latest_document_with_the_shared_cli_formatter() {
    let uri = "file:///formatting.ssrg";
    let source =
        include_str!("../../seseragi-formatter/tests/fixtures/canonical-layout.input.ssrg");
    let expected =
        include_str!("../../seseragi-formatter/tests/fixtures/canonical-layout.expected.ssrg");
    let formatted = format_module(uri, source).expect("valid source");
    assert!(formatted.changed);
    assert_eq!(formatted.text, expected);

    for (encoding_name, encoding) in [
        ("utf-8", PositionEncoding::Utf8),
        ("utf-16", PositionEncoding::Utf16),
    ] {
        let input = [
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {
                    "capabilities": {
                        "general": {"positionEncodings": [encoding_name]}
                    }
                }
            }),
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": {"textDocument": {
                    "uri": uri, "languageId": "seseragi", "version": 1,
                    "text": source
                }}
            }),
            json!({
                "jsonrpc": "2.0", "id": 2, "method": "textDocument/formatting",
                "params": {
                    "textDocument": {"uri": uri},
                    "options": {"tabSize": 4, "insertSpaces": false}
                }
            }),
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": formatted.text.clone()}]
                }
            }),
            json!({
                "jsonrpc": "2.0", "id": 3, "method": "textDocument/formatting",
                "params": {
                    "textDocument": {"uri": uri},
                    "options": {"tabSize": 2, "insertSpaces": true}
                }
            }),
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didChange",
                "params": {
                    "textDocument": {"uri": uri, "version": 3},
                    "contentChanges": [{"text": "pub let broken: Int =\n"}]
                }
            }),
            json!({
                "jsonrpc": "2.0", "id": 4, "method": "textDocument/formatting",
                "params": {
                    "textDocument": {"uri": uri},
                    "options": {"tabSize": 2, "insertSpaces": true}
                }
            }),
            json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown"}),
            json!({"jsonrpc": "2.0", "method": "exit"}),
        ];

        let messages = run_server(&input);
        assert_eq!(
            response(&messages, 1)["result"]["capabilities"]["documentFormattingProvider"],
            true
        );

        let edits = response(&messages, 2)["result"].as_array().unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0]["newText"], formatted.text);
        assert_eq!(
            edits[0]["range"]["start"],
            json!({"line": 0, "character": 0})
        );
        let end = LineIndex::new(source)
            .try_locate_encoded(source.len(), encoding)
            .unwrap();
        assert_eq!(
            edits[0]["range"]["end"],
            json!({"line": end.line, "character": end.character})
        );

        assert_eq!(response(&messages, 3)["result"], json!([]));
        assert_eq!(response(&messages, 4)["result"], json!([]));
        assert!(messages.iter().any(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["version"] == 3
                && message["params"]["diagnostics"]
                    .as_array()
                    .is_some_and(|diagnostics| !diagnostics.is_empty())
        }));
    }
}

#[test]
fn rename_honors_utf8_utf16_and_utf32_positions() {
    let uri = "file:///rename-position-encoding.ssrg";
    let source = concat!(
        "pub fn greet value: Int -> Int = value\n",
        "pub let result = length \"🙂\" |> greet\n",
    );
    let definition = source.find("greet").unwrap();
    let reference = source.rfind("greet").unwrap();

    for (encoding_name, encoding) in [
        ("utf-8", PositionEncoding::Utf8),
        ("utf-16", PositionEncoding::Utf16),
        ("utf-32", PositionEncoding::Utf32),
    ] {
        let line_index = LineIndex::new(source);
        let request = line_index.try_locate_encoded(reference, encoding).unwrap();
        let definition_start = line_index.try_locate_encoded(definition, encoding).unwrap();
        let reference_start = line_index.try_locate_encoded(reference, encoding).unwrap();
        let input = [
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"capabilities": {
                    "general": {"positionEncodings": [encoding_name]}
                }}
            }),
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": {"textDocument": {
                    "uri": uri, "languageId": "seseragi", "version": 3, "text": source
                }}
            }),
            json!({
                "jsonrpc": "2.0", "id": 2, "method": "textDocument/rename",
                "params": {
                    "textDocument": {"uri": uri},
                    "position": {"line": request.line, "character": request.character},
                    "newName": "salute"
                }
            }),
            json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
            json!({"jsonrpc": "2.0", "method": "exit"}),
        ];

        let messages = run_server(&input);
        assert_eq!(
            response(&messages, 1)["result"]["capabilities"]["positionEncoding"],
            encoding_name
        );
        let document_change = &response(&messages, 2)["result"]["documentChanges"][0];
        assert_eq!(document_change["textDocument"]["version"], 3);
        let edits = document_change["edits"].as_array().unwrap();
        assert_eq!(edits.len(), 2);
        assert!(edits.iter().any(|edit| {
            edit["range"]["start"]
                == json!({
                    "line": definition_start.line,
                    "character": definition_start.character
                })
        }));
        assert!(edits.iter().any(|edit| {
            edit["range"]["start"]
                == json!({
                    "line": reference_start.line,
                    "character": reference_start.character
                })
        }));
    }
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
            "params": {"capabilities": {
                "general": {"positionEncodings": ["utf-16"]},
                "textDocument": {
                    "hover": {"contentFormat": ["markdown", "plaintext"]},
                    "completion": {"completionItem": {
                        "documentationFormat": ["markdown", "plaintext"]
                    }},
                    "signatureHelp": {"signatureInformation": {
                        "documentationFormat": ["markdown", "plaintext"]
                    }}
                }
            }}
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
    assert_eq!(capabilities["referencesProvider"], true);
    assert_eq!(capabilities["renameProvider"]["prepareProvider"], true);
    assert_eq!(capabilities["workspaceSymbolProvider"], true);
    assert!(
        capabilities["semanticTokensProvider"]["legend"]["tokenTypes"]
            .as_array()
            .is_some_and(|types| types.iter().any(|item| item == "operator"))
    );

    let hover = &response(&messages, 2)["result"];
    assert_eq!(hover["contents"]["kind"], "markdown");
    assert!(hover["contents"]["value"].as_str().is_some_and(|value| {
        value.contains("```seseragi\nadd\n  left: Int\n  -> right: Int\n  -> Int\n```")
            && value.contains("features.ssrg")
    }));

    let completions = response(&messages, 3)["result"].as_array().unwrap();
    assert!(completions.iter().any(|item| item["label"] == "addOne"));
    assert!(completions.iter().any(|item| item["label"] == "join"));
    let add = completions
        .iter()
        .find(|item| item["label"] == "add")
        .expect("function completion");
    assert!(add["detail"].as_str().is_some_and(|detail| {
        detail.contains("left: Int") && detail.contains("right: Int") && detail.ends_with("Int")
    }));
    assert_eq!(add["documentation"]["kind"], "markdown");
    assert!(add["documentation"]["value"]
        .as_str()
        .is_some_and(|value| value.starts_with('`') && value.contains("right: Int")));
    let add_one = completions
        .iter()
        .find(|item| item["label"] == "addOne")
        .expect("partial function completion");
    assert!(add_one["detail"]
        .as_str()
        .is_some_and(|detail| { detail.contains("arg1: Int") && detail.ends_with("Int") }));

    let signature = &response(&messages, 4)["result"];
    assert_eq!(signature["signatures"][0]["label"], add["detail"]);
    assert!(signature["signatures"][0]["label"]
        .as_str()
        .is_some_and(|label| !label.contains('`') && !label.contains('\n')));
    assert_eq!(
        signature["signatures"][0]["documentation"]["kind"],
        "markdown"
    );
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

#[test]
fn type_presentation_honors_plaintext_fallback_without_losing_structure() {
    let uri = "file:///type-presentation.ssrg";
    let source = concat!(
        "fn inspect options: { callback: (Int -> String), values: Array<Maybe<Int>> } -> Int = 1\n",
        "let inspectAgain = inspect\n",
    );
    let reference = source.rfind("inspect").expect("inspect reference");
    let position = LineIndex::new(source)
        .try_locate_encoded(reference, PositionEncoding::Utf16)
        .unwrap();
    let position = json!({"line": position.line, "character": position.character});
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {
                "general": {"positionEncodings": ["utf-16"]},
                "textDocument": {
                    "hover": {"contentFormat": ["plaintext", "markdown"]},
                    "completion": {"completionItem": {
                        "documentationFormat": ["plaintext"]
                    }},
                    "signatureHelp": {"signatureInformation": {
                        "documentationFormat": ["plaintext"]
                    }}
                }
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": uri}, "position": position}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": uri}, "position": position}
        }),
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "textDocument/signatureHelp",
            "params": {"textDocument": {"uri": uri}, "position": position}
        }),
        json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let hover = &response(&messages, 2)["result"]["contents"];
    assert_eq!(hover["kind"], "plaintext");
    let hover_value = hover["value"].as_str().expect("plaintext hover");
    assert!(hover_value.contains("inspect\n  options: {"));
    assert!(hover_value.contains("callback: ("));
    assert!(hover_value.contains("values: Array<"));
    assert!(!hover_value.contains("```"));
    assert!(!hover_value.contains('`'));

    let completions = response(&messages, 3)["result"].as_array().unwrap();
    let inspect = completions
        .iter()
        .find(|item| item["label"] == "inspect")
        .expect("inspect completion");
    let compact = inspect["detail"].as_str().expect("compact detail");
    assert!(compact.contains("{ callback: (Int -> String), values: Array<Maybe<Int>> }"));
    assert!(!compact.contains('\n'));
    assert_eq!(inspect["documentation"]["kind"], "plaintext");
    assert!(inspect["documentation"]["value"]
        .as_str()
        .is_some_and(|value| value.starts_with(compact) && !value.contains('`')));

    let signature = &response(&messages, 4)["result"];
    assert_eq!(signature["signatures"][0]["label"], inspect["detail"]);
    assert_eq!(
        signature["signatures"][0]["documentation"]["kind"],
        "plaintext"
    );
}

#[test]
fn binary_resolves_reachable_workspace_imports_for_diagnostics_features_and_definitions() {
    let workspace = TempWorkspace::new();
    let main = concat!(
        "import { increment as next } from \"./domain\"\n",
        "pub fn run value: Int -> Int = next value\n",
    );
    let domain = "pub fn increment value: Int -> Int = value + 1\n";
    workspace.write("main.ssrg", main);
    workspace.write("domain.ssrg", domain);
    workspace.write("unrelated.ssrg", "pub let broken = missing\n");

    let root_uri = file_uri(workspace.path());
    let main_uri = file_uri(&workspace.path().join("main.ssrg"));
    let domain_uri = file_uri(&workspace.path().join("domain.ssrg"));
    let imported = main.rfind("next value").unwrap();
    let imported_position = LineIndex::new(main)
        .try_locate_encoded(imported, PositionEncoding::Utf16)
        .unwrap();
    let import_original_position = LineIndex::new(main)
        .try_locate_encoded(main.find("increment").unwrap(), PositionEncoding::Utf16)
        .unwrap();
    let completion_position = LineIndex::new(main)
        .try_locate_encoded(main.len(), PositionEncoding::Utf16)
        .unwrap();
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {
                "capabilities": {
                    "textDocument": {"hover": {"contentFormat": ["plaintext"]}}
                },
                "workspaceFolders": [{"uri": root_uri, "name": "fixture"}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": imported_position.line, "character": imported_position.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/definition",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": import_original_position.line,
                "character": import_original_position.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": completion_position.line, "character": completion_position.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 5, "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {
                    "line": import_original_position.line,
                    "character": import_original_position.character
                },
                "context": {"includeDeclaration": true}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 6, "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {
                    "line": import_original_position.line,
                    "character": import_original_position.character
                },
                "context": {"includeDeclaration": false}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "textDocument/prepareRename",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": import_original_position.line,
                "character": import_original_position.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 8, "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {
                    "line": imported_position.line, "character": imported_position.character
                },
                "newName": "advance"
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 9, "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {
                    "line": imported_position.line, "character": imported_position.character
                },
                "newName": "run"
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 10, "method": "workspace/symbol",
            "params": {"query": "INC"}
        }),
        json!({"jsonrpc": "2.0", "id": 11, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let diagnostics = published(&messages, &main_uri)["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert!(published(&messages, &domain_uri)["params"]["diagnostics"]
        .as_array()
        .is_some_and(Vec::is_empty));

    let hover = response(&messages, 2)["result"]["contents"]["value"]
        .as_str()
        .expect("hover for imported function");
    assert!(hover.contains("increment"), "{hover}");
    let definition = &response(&messages, 3)["result"];
    assert_eq!(definition["uri"], domain_uri);
    assert_eq!(
        definition["range"]["start"],
        json!({"line": 0, "character": 7})
    );
    let references = response(&messages, 5)["result"].as_array().unwrap();
    assert!(references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 0, "character": 7})
    }));
    assert!(references
        .iter()
        .any(|location| location["uri"] == main_uri));
    assert!(response(&messages, 6)["result"]
        .as_array()
        .is_some_and(|locations| locations
            .iter()
            .all(|location| location["uri"] != domain_uri)));
    assert!(response(&messages, 4)["result"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item["label"] == "next")));

    let prepared = &response(&messages, 7)["result"];
    assert_eq!(prepared["placeholder"], "increment");
    assert_eq!(
        prepared["range"]["start"],
        json!({"line": 0, "character": 9})
    );

    let changes = response(&messages, 8)["result"]["documentChanges"]
        .as_array()
        .expect("atomic rename document changes");
    let main_changes = changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == main_uri)
        .expect("consumer rename edits");
    assert_eq!(main_changes["textDocument"]["version"], 1);
    assert_eq!(
        main_changes["edits"].as_array().unwrap().len(),
        3,
        "{changes:#?}"
    );
    assert!(main_changes["edits"]
        .as_array()
        .unwrap()
        .iter()
        .all(|edit| edit["newText"] == "advance"));
    let domain_changes = changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == domain_uri)
        .expect("definition rename edit");
    assert!(domain_changes["textDocument"]["version"].is_null());
    assert_eq!(domain_changes["edits"].as_array().unwrap().len(), 1);

    assert_eq!(response(&messages, 9)["error"]["code"], -32803);
    assert!(response(&messages, 9)["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("conflicts with run")));

    let symbols = response(&messages, 10)["result"].as_array().unwrap();
    assert_eq!(symbols.len(), 1, "{symbols:#?}");
    assert_eq!(symbols[0]["name"], "increment");
    assert_eq!(symbols[0]["kind"], 12);
    assert_eq!(symbols[0]["location"]["uri"], domain_uri);
    assert!(symbols[0]["data"]["identity"]
        .as_str()
        .is_some_and(|identity| identity.ends_with("::increment")));
}

#[test]
fn workspace_operations_preserve_namespaces_reexports_scopes_traits_and_operators() {
    let workspace = TempWorkspace::new();
    let main = concat!(
        "import { Inspect, Ticket, score } from \"./domain\"\n",
        "import * as domain from \"./domain\"\n",
        "import { operator <+> } from \"./operators\"\n",
        "import * as evidence from \"./instances\"\n",
        "\n",
        "pub fn total ticket: Ticket -> Int =\n",
        "  domain.score ticket\n",
        "\n",
        "pub let combined: Int = 1 <+> 2\n",
    );
    let domain = concat!(
        "pub type Ticket =\n",
        "  | Ticket Int\n",
        "\n",
        "pub trait Inspect<A> {\n",
        "  fn inspect value: A -> String\n",
        "}\n",
        "\n",
        "pub fn score ticket: Ticket -> Int =\n",
        "  match ticket {\n",
        "    Ticket value -> value\n",
        "  }\n",
        "\n",
        "pub fn make -> Ticket = Ticket 1\n",
        "\n",
        "fn shadow score: Int -> Int = score\n",
    );
    let operators = concat!(
        "pub operator infixl 4 <+> left: Int -> right: Int -> Int =\n",
        "  left + right\n",
    );
    let instances = concat!(
        "import { Inspect, Ticket } from \"./domain\"\n",
        "\n",
        "instance Inspect<Ticket> {\n",
        "  fn inspect value: Ticket -> String = \"ticket\"\n",
        "}\n",
    );
    workspace.write("main.ssrg", main);
    workspace.write("domain.ssrg", domain);
    workspace.write("operators.ssrg", operators);
    workspace.write("instances.ssrg", instances);

    let position = |source: &str, offset: usize| {
        let position = LineIndex::new(source)
            .try_locate_encoded(offset, PositionEncoding::Utf16)
            .unwrap();
        json!({"line": position.line, "character": position.character})
    };
    let root_uri = file_uri(workspace.path());
    let main_uri = file_uri(&workspace.path().join("main.ssrg"));
    let domain_uri = file_uri(&workspace.path().join("domain.ssrg"));
    let operators_uri = file_uri(&workspace.path().join("operators.ssrg"));
    let instances_uri = file_uri(&workspace.path().join("instances.ssrg"));
    let type_position = position(domain, domain.find("-> Ticket").unwrap() + 3);
    let constructor_position = position(domain, domain.rfind("Ticket 1").unwrap());
    let score_position = position(main, main.find("domain.score").unwrap() + 7);
    let operator_position = position(main, main.rfind("<+>").unwrap());
    let trait_position = position(domain, domain.find("trait Inspect").unwrap() + 6);
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "workspaceFolders": [{
                "uri": root_uri, "name": "semantic-operations"
            }]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": domain_uri, "languageId": "seseragi", "version": 7,
                "text": domain
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/references",
            "params": {"textDocument": {"uri": domain_uri}, "position": type_position,
                "context": {"includeDeclaration": true}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/references",
            "params": {"textDocument": {"uri": domain_uri}, "position": constructor_position,
                "context": {"includeDeclaration": true}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 4, "method": "textDocument/references",
            "params": {"textDocument": {"uri": main_uri}, "position": score_position,
                "context": {"includeDeclaration": true}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 5, "method": "textDocument/references",
            "params": {"textDocument": {"uri": domain_uri}, "position": trait_position,
                "context": {"includeDeclaration": true}}
        }),
        json!({
            "jsonrpc": "2.0", "id": 6, "method": "textDocument/rename",
            "params": {"textDocument": {"uri": main_uri}, "position": operator_position,
                "newName": "<^>"}
        }),
        json!({
            "jsonrpc": "2.0", "id": 7, "method": "workspace/symbol",
            "params": {"query": "ticket"}
        }),
        json!({
            "jsonrpc": "2.0", "id": 8, "method": "textDocument/rename",
            "params": {"textDocument": {"uri": main_uri}, "position": score_position,
                "newName": "total"}
        }),
        json!({
            "jsonrpc": "2.0", "id": 9, "method": "workspace/symbol",
            "params": {"query": "shadow"}
        }),
        json!({
            "jsonrpc": "2.0", "id": 10, "method": "textDocument/rename",
            "params": {"textDocument": {"uri": main_uri}, "position": score_position,
                "newName": "points"}
        }),
        json!({"jsonrpc": "2.0", "id": 11, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    for uri in [&main_uri, &domain_uri, &operators_uri, &instances_uri] {
        let diagnostics = &published(&messages, uri)["params"]["diagnostics"];
        assert!(
            diagnostics.as_array().is_some_and(Vec::is_empty),
            "{uri}: {diagnostics:#?}"
        );
    }

    let type_references = response(&messages, 2)["result"].as_array().unwrap();
    assert!(type_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 12, "character": 15})
    }));
    let constructor_references = response(&messages, 3)["result"].as_array().unwrap();
    assert!(constructor_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 12, "character": 24})
    }));
    assert!(!type_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 12, "character": 24})
    }));
    assert!(!constructor_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 12, "character": 15})
    }));

    let score_references = response(&messages, 4)["result"].as_array().unwrap();
    assert!(score_references
        .iter()
        .any(|location| location["uri"] == main_uri));
    assert!(!score_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 14, "character": 10})
    }));

    let trait_references = response(&messages, 5)["result"].as_array().unwrap();
    assert!(trait_references.iter().any(|location| {
        location["uri"] == domain_uri
            && location["range"]["start"] == json!({"line": 3, "character": 10})
    }));
    assert!(trait_references.iter().any(|location| {
        location["uri"] == instances_uri
            && location["range"]["start"] == json!({"line": 2, "character": 9})
    }));

    let operator_changes = response(&messages, 6)["result"]["documentChanges"]
        .as_array()
        .unwrap_or_else(|| panic!("operator rename: {:#?}", response(&messages, 6)));
    assert_eq!(operator_changes.len(), 2, "{operator_changes:#?}");
    assert!(operator_changes
        .iter()
        .any(|change| change["textDocument"]["uri"] == operators_uri));
    assert!(operator_changes
        .iter()
        .any(|change| change["textDocument"]["uri"] == main_uri));

    let workspace_symbols = response(&messages, 7)["result"].as_array().unwrap();
    assert!(workspace_symbols
        .iter()
        .any(|symbol| { symbol["name"] == "Ticket" && symbol["data"]["namespace"] == "type" }));
    assert!(workspace_symbols
        .iter()
        .any(|symbol| { symbol["name"] == "Ticket" && symbol["data"]["namespace"] == "value" }));
    assert!(workspace_symbols.windows(2).all(|pair| {
        pair[0]["data"]["namespace"].as_str() <= pair[1]["data"]["namespace"].as_str()
    }));

    assert_eq!(response(&messages, 8)["error"]["code"], -32803);
    assert!(response(&messages, 8)["error"]["message"]
        .as_str()
        .is_some_and(|message| message.contains("conflicts with total")));

    let private_symbols = response(&messages, 9)["result"].as_array().unwrap();
    assert_eq!(private_symbols.len(), 1, "{private_symbols:#?}");
    assert_eq!(private_symbols[0]["name"], "shadow");
    assert_eq!(private_symbols[0]["location"]["uri"], domain_uri);

    let score_changes = response(&messages, 10)["result"]["documentChanges"]
        .as_array()
        .expect("namespace member rename edits");
    let main_score_edits = score_changes
        .iter()
        .find(|change| change["textDocument"]["uri"] == main_uri)
        .expect("consumer namespace member edit")["edits"]
        .as_array()
        .unwrap();
    assert!(
        main_score_edits.iter().any(|edit| {
            edit["range"]["start"] == json!({"line": 6, "character": 9})
                && edit["range"]["end"] == json!({"line": 6, "character": 14})
                && edit["newText"] == "points"
        }),
        "{main_score_edits:#?}"
    );
}

#[test]
fn binary_loads_the_canonical_playground_web_package() {
    let package = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/samples/project-flow-app")
        .canonicalize()
        .unwrap();
    let main_path = package.join("src/main.ssrg");
    let app_path = package.join("src/app.ssrg");
    let main = fs::read_to_string(&main_path).unwrap();
    let imported = main.rfind("create ()").unwrap();
    let position = LineIndex::new(&main)
        .try_locate_encoded(imported, PositionEncoding::Utf16)
        .unwrap();
    let root_uri = file_uri(&package);
    let main_uri = file_uri(&main_path);
    let app_uri = file_uri(&app_path);
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "workspaceFolders": [{
                "uri": root_uri, "name": "project-flow-app"
            }]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/definition",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": position.line, "character": position.character
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let diagnostics = published(&messages, &main_uri)["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(response(&messages, 2)["result"]["uri"], app_uri);
}

#[test]
fn workspace_references_and_rename_follow_reexported_symbols() {
    let workspace = TempWorkspace::new();
    workspace.write(
        "seseragi.toml",
        concat!(
            "[package]\n",
            "name = \"fixture/reexport\"\n",
            "version = \"0.0.0\"\n",
            "language = \">=0.1.0 <0.2.0\"\n\n",
            "[run]\n",
            "entry = \"main\"\n",
            "target = \"process\"\n",
        ),
    );
    let main = concat!(
        "import { userId } from \"./facade\"\n",
        "pub let current = userId 42\n",
    );
    workspace.write("src/main.ssrg", main);
    workspace.write(
        "src/facade.ssrg",
        "pub import { userId } from \"./model/user\"\n",
    );
    workspace.write(
        "src/model/user.ssrg",
        "pub fn userId value: Int -> Int = value\n",
    );
    let package = workspace.path();
    let main_path = package.join("src/main.ssrg");
    let facade_path = package.join("src/facade.ssrg");
    let model_path = package.join("src/model/user.ssrg");
    let offset = main.rfind("userId 42").unwrap();
    let position = LineIndex::new(&main)
        .try_locate_encoded(offset, PositionEncoding::Utf16)
        .unwrap();
    let root_uri = file_uri(&package);
    let main_uri = file_uri(&main_path);
    let facade_uri = file_uri(&facade_path);
    let model_uri = file_uri(&model_path);
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "workspaceFolders": [{
                "uri": root_uri, "name": "modules-reexport-run"
            }]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {"line": position.line, "character": position.character},
                "context": {"includeDeclaration": true}
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": main_uri},
                "position": {"line": position.line, "character": position.character},
                "newName": "createUserId"
            }
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let references = response(&messages, 2)["result"].as_array().unwrap();
    for uri in [&main_uri, &facade_uri, &model_uri] {
        assert!(
            references.iter().any(|location| location["uri"] == *uri),
            "missing {uri}: {references:#?}"
        );
    }
    let changes = response(&messages, 3)["result"]["documentChanges"]
        .as_array()
        .unwrap();
    for uri in [&main_uri, &facade_uri, &model_uri] {
        assert!(
            changes
                .iter()
                .any(|change| change["textDocument"]["uri"] == *uri),
            "missing {uri}: {changes:#?}"
        );
    }
}

#[test]
fn binary_exposes_portable_standard_metadata_from_the_parity_package() {
    let package = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("examples/spec/fixtures/projects/std-parity-portable")
        .canonicalize()
        .unwrap();
    let main_path = package.join("src/main.ssrg");
    let source = fs::read_to_string(&main_path).unwrap();
    let filter = source.find("arrays.filter").unwrap() + "arrays.".len();
    let filter_position = LineIndex::new(&source)
        .try_locate_encoded(filter, PositionEncoding::Utf16)
        .unwrap();
    let completion = source.rfind("lists.length").unwrap() + "lists.".len();
    let completion_position = LineIndex::new(&source)
        .try_locate_encoded(completion, PositionEncoding::Utf16)
        .unwrap();
    let root_uri = file_uri(&package);
    let main_uri = file_uri(&main_path);
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {"textDocument": {
                "hover": {"contentFormat": ["plaintext"]},
                "completion": {"completionItem": {"documentationFormat": ["plaintext"]}}
            }}, "workspaceFolders": [{"uri": root_uri, "name": "std-parity-portable"}]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": source
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": filter_position.line, "character": filter_position.character
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/completion",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": completion_position.line, "character": completion_position.character
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let diagnostics = published(&messages, &main_uri)["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let hover = response(&messages, 2)["result"]["contents"]["value"]
        .as_str()
        .unwrap();
    assert!(hover.contains("filter"), "{hover}");
    assert!(hover.contains("std/array::filter"), "{hover}");
    let completions = response(&messages, 3)["result"].as_array().unwrap();
    let length = completions
        .iter()
        .find(|item| item["label"] == "length")
        .expect("std/list length completion");
    assert!(length["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("List") && detail.contains("Int")));
}

#[test]
fn binary_reanalyzes_importers_when_an_open_dependency_changes_without_saving() {
    let workspace = TempWorkspace::new();
    let main = concat!(
        "import { increment } from \"./domain\"\n",
        "pub fn run value: Int -> Int = increment value\n",
    );
    let domain = "pub fn increment value: Int -> Int = value + 1\n";
    let changed_domain = "pub fn increment value: String -> String = value\n";
    workspace.write("main.ssrg", main);
    workspace.write("domain.ssrg", domain);

    let root_uri = file_uri(workspace.path());
    let main_uri = file_uri(&workspace.path().join("main.ssrg"));
    let domain_uri = file_uri(&workspace.path().join("domain.ssrg"));
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "workspaceFolders": [{"uri": root_uri, "name": "fixture"}]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": domain_uri, "languageId": "seseragi", "version": 1, "text": domain
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didChange",
            "params": {
                "textDocument": {"uri": domain_uri, "version": 2},
                "contentChanges": [{"text": changed_domain}]
            }
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "workspace/symbol",
            "params": {"query": "increment"}
        }),
        json!({
            "jsonrpc": "2.0", "id": 3, "method": "textDocument/rename",
            "params": {
                "textDocument": {"uri": domain_uri},
                "position": {"line": 0, "character": 7},
                "newName": "advance"
            }
        }),
        json!({"jsonrpc": "2.0", "id": 4, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    let diagnostics = published(&messages, &main_uri)["params"]["diagnostics"]
        .as_array()
        .unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "SES-T0101"),
        "{diagnostics:?}"
    );
    let symbols = response(&messages, 2)["result"].as_array().unwrap();
    assert_eq!(symbols.len(), 1, "{symbols:#?}");
    assert_eq!(symbols[0]["location"]["uri"], domain_uri);
    let changes = response(&messages, 3)["result"]["documentChanges"]
        .as_array()
        .unwrap();
    assert!(changes.iter().any(|change| {
        change["textDocument"]["uri"] == domain_uri && change["textDocument"]["version"] == 2
    }));
}

#[test]
fn binary_uses_the_local_package_graph_for_declared_package_imports() {
    let workspace = TempWorkspace::new();
    workspace.write(
        "seseragi.toml",
        concat!(
            "[package]\n",
            "name = \"fixture/editor-app\"\n",
            "version = \"1.0.0\"\n",
            "language = \">=0.1.0 <0.2.0\"\n\n",
            "[run]\n",
            "entry = \"main\"\n\n",
            "[dependencies]\n",
            "math = { package = \"fixture/editor-math\", path = \"vendor/math\" }\n",
        ),
    );
    workspace.write(
        "vendor/math/seseragi.toml",
        concat!(
            "[package]\n",
            "name = \"fixture/editor-math\"\n",
            "version = \"1.0.0\"\n",
            "language = \">=0.1.0 <0.2.0\"\n\n",
            "[exports]\n",
            "\".\" = \"lib\"\n",
        ),
    );
    let main = concat!(
        "import { increment } from \"math\"\n",
        "pub fn run value: Int -> Int = increment value\n",
    );
    workspace.write("src/main.ssrg", main);
    workspace.write(
        "vendor/math/src/lib.ssrg",
        "pub fn increment value: Int -> Int = value + 1\n",
    );

    let root_uri = file_uri(workspace.path());
    let main_uri = file_uri(&workspace.path().join("src/main.ssrg"));
    let library_uri = file_uri(&workspace.path().join("vendor/math/src/lib.ssrg"));
    let scratch_uri = "file:///outside-package.ssrg";
    let imported = main.rfind("increment value").unwrap();
    let position = LineIndex::new(main)
        .try_locate_encoded(imported, PositionEncoding::Utf16)
        .unwrap();
    let input = [
        json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"capabilities": {}, "workspaceFolders": [{"uri": root_uri, "name": "fixture"}]}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": scratch_uri, "languageId": "seseragi", "version": 1,
                "text": "pub let scratch: Int = 1\n"
            }}
        }),
        json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
            }}
        }),
        json!({
            "jsonrpc": "2.0", "id": 2, "method": "textDocument/definition",
            "params": {"textDocument": {"uri": main_uri}, "position": {
                "line": position.line, "character": position.character
            }}
        }),
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ];

    let messages = run_server(&input);
    assert!(published(&messages, &main_uri)["params"]["diagnostics"]
        .as_array()
        .is_some_and(Vec::is_empty));
    assert_eq!(response(&messages, 2)["result"]["uri"], library_uri);
}

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "seseragi-lsp-workspace-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, source).unwrap();
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
