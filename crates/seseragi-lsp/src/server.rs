use crate::capabilities::{negotiate_position_encoding, position_encoding_name};
use crate::diagnostics;
use crate::model::{DidChangeParams, DidCloseParams, DidOpenParams, InitializeParams};
use crate::protocol::{self, ProtocolError};
use serde::Deserialize;
use serde_json::{json, Value};
use seseragi_driver::{compile_module, CompileInput};
use seseragi_source::{LineIndexError, PositionEncoding};
use std::fmt;
use std::io::{BufRead, Write};

#[derive(Debug)]
pub enum ServerError {
    Protocol(ProtocolError),
    InvalidCompilerRange(LineIndexError),
}

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(error) => error.fmt(formatter),
            Self::InvalidCompilerRange(error) => {
                write!(
                    formatter,
                    "compiler emitted an invalid diagnostic range: {error}"
                )
            }
        }
    }
}

impl std::error::Error for ServerError {}

impl From<ProtocolError> for ServerError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

impl From<LineIndexError> for ServerError {
    fn from(error: LineIndexError) -> Self {
        Self::InvalidCompilerRange(error)
    }
}

#[derive(Default)]
struct State {
    encoding: Option<PositionEncoding>,
}

pub fn run(mut reader: impl BufRead, mut writer: impl Write) -> Result<(), ServerError> {
    let mut state = State::default();
    while let Some(message) = protocol::read_message(&mut reader)? {
        let method = message.get("method").and_then(Value::as_str);
        if method == Some("exit") {
            break;
        }
        for outgoing in state.handle(message)? {
            protocol::write_message(&mut writer, &outgoing)?;
        }
    }
    Ok(())
}

impl State {
    fn handle(&mut self, message: Value) -> Result<Vec<Value>, ServerError> {
        let method = message.get("method").and_then(Value::as_str);
        let id = message.get("id").cloned();
        match method {
            Some("initialize") => {
                let params: InitializeParams = parse_params(&message).unwrap_or_default();
                let encoding =
                    negotiate_position_encoding(&params.capabilities.general.position_encodings);
                self.encoding = Some(encoding);
                Ok(vec![response(
                    id,
                    json!({
                        "capabilities": {
                            "positionEncoding": position_encoding_name(encoding),
                            "textDocumentSync": {"openClose": true, "change": 1}
                        },
                        "serverInfo": {"name": "seseragi-lsp", "version": env!("CARGO_PKG_VERSION")}
                    }),
                )])
            }
            Some("initialized") => Ok(Vec::new()),
            Some("shutdown") => Ok(vec![response(id, Value::Null)]),
            Some("textDocument/didOpen") => {
                let params: DidOpenParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                Ok(vec![publish(
                    &params.text_document.uri,
                    Some(params.text_document.version),
                    &params.text_document.text,
                    self.encoding.unwrap_or(PositionEncoding::Utf16),
                )?])
            }
            Some("textDocument/didChange") => {
                let params: DidChangeParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                let Some(change) = params.content_changes.last() else {
                    return Ok(Vec::new());
                };
                Ok(vec![publish(
                    &params.text_document.uri,
                    Some(params.text_document.version),
                    &change.text,
                    self.encoding.unwrap_or(PositionEncoding::Utf16),
                )?])
            }
            Some("textDocument/didClose") => {
                let params: DidCloseParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                Ok(vec![json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {"uri": params.text_document.uri, "diagnostics": []}
                })])
            }
            Some(_) if id.is_some() => Ok(vec![json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": "Method not found"}
            })]),
            _ => Ok(Vec::new()),
        }
    }
}

fn publish(
    uri: &str,
    version: Option<i64>,
    source: &str,
    encoding: PositionEncoding,
) -> Result<Value, LineIndexError> {
    let input = CompileInput::new(uri, uri, source);
    let artifact = match compile_module(input) {
        Ok(compiled) => compiled.diagnostics,
        Err(diagnostics) => diagnostics,
    };
    let diagnostics = diagnostics::convert(&artifact, source, encoding)?;
    Ok(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "version": version, "diagnostics": diagnostics}
    }))
}

fn response(id: Option<Value>, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id.unwrap_or(Value::Null), "result": result})
}

fn parse_params<T: for<'de> Deserialize<'de>>(message: &Value) -> Option<T> {
    serde_json::from_value(message.get("params")?.clone()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol;
    use std::io::Cursor;

    fn framed(messages: &[Value]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for message in messages {
            protocol::write_message(&mut bytes, message).unwrap();
        }
        bytes
    }

    #[test]
    fn open_document_publishes_shared_driver_diagnostics() {
        let input = framed(&[
            json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"capabilities": {"general": {"positionEncodings": ["utf-8"]}}}
            }),
            json!({
                "jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": {"textDocument": {
                    "uri": "file:///app.ssrg", "languageId": "seseragi", "version": 7,
                    "text": "pub let broken: Int =\n"
                }}
            }),
            json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
            json!({"jsonrpc": "2.0", "method": "exit"}),
        ]);
        let mut output = Vec::new();
        run(Cursor::new(input), &mut output).unwrap();

        let mut reader = Cursor::new(output);
        let initialize = protocol::read_message(&mut reader).unwrap().unwrap();
        let published = protocol::read_message(&mut reader).unwrap().unwrap();
        let shutdown = protocol::read_message(&mut reader).unwrap().unwrap();
        assert_eq!(
            initialize["result"]["capabilities"]["positionEncoding"],
            "utf-8"
        );
        assert_eq!(published["method"], "textDocument/publishDiagnostics");
        assert_eq!(published["params"]["uri"], "file:///app.ssrg");
        assert_eq!(published["params"]["version"], 7);
        assert!(!published["params"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(shutdown["id"], 2);
    }

    #[test]
    fn publishes_resolver_and_type_diagnostics_from_the_driver() {
        let unresolved = publish(
            "file:///unresolved.ssrg",
            Some(1),
            "pub fn useMissing value: Int -> Int = missing\n",
            PositionEncoding::Utf16,
        )
        .unwrap();
        let mismatch = publish(
            "file:///mismatch.ssrg",
            Some(1),
            "pub fn bad value: Int -> String = value\n",
            PositionEncoding::Utf16,
        )
        .unwrap();

        assert_eq!(unresolved["params"]["diagnostics"][0]["code"], "SES-N0001");
        assert_eq!(mismatch["params"]["diagnostics"][0]["code"], "SES-T0101");
    }
}
