use crate::capabilities::{negotiate_position_encoding, position_encoding_name};
use crate::diagnostics;
use crate::features::{self, DocumentState, SEMANTIC_TOKEN_TYPES};
use crate::model::{
    CodeActionParams, DidChangeParams, DidCloseParams, DidOpenParams, DocumentFormattingParams,
    InitializeParams, MarkupKind, SemanticTokensParams, TextDocumentPositionParams,
};
use crate::protocol::{self, ProtocolError};
use serde::Deserialize;
use serde_json::{json, Value};
use seseragi_source::{LineIndexError, PositionEncoding};
use std::collections::BTreeMap;
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
    hover_markup: MarkupKind,
    completion_markup: MarkupKind,
    signature_markup: MarkupKind,
    documents: BTreeMap<String, DocumentState>,
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
                self.hover_markup =
                    MarkupKind::negotiate(&params.capabilities.text_document.hover.content_format);
                self.completion_markup = MarkupKind::negotiate(
                    &params
                        .capabilities
                        .text_document
                        .completion
                        .completion_item
                        .documentation_format,
                );
                self.signature_markup = MarkupKind::negotiate(
                    &params
                        .capabilities
                        .text_document
                        .signature_help
                        .signature_information
                        .documentation_format,
                );
                Ok(vec![response(
                    id,
                    json!({
                        "capabilities": {
                            "positionEncoding": position_encoding_name(encoding),
                            "textDocumentSync": {"openClose": true, "change": 1},
                            "hoverProvider": true,
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": ["."]
                            },
                            "signatureHelpProvider": {
                                "triggerCharacters": [" ", "(", ","]
                            },
                            "definitionProvider": true,
                            "codeActionProvider": true,
                            "documentFormattingProvider": true,
                            "semanticTokensProvider": {
                                "legend": {
                                    "tokenTypes": SEMANTIC_TOKEN_TYPES,
                                    "tokenModifiers": []
                                },
                                "full": true,
                                "range": false
                            }
                        },
                        "serverInfo": {
                            "name": crate::SERVER_NAME,
                            "version": crate::SERVER_VERSION
                        },
                        "experimental": {
                            "seseragi": {
                                "protocolVersion": crate::PROTOCOL_VERSION,
                                "analysisSchemaVersion": crate::ANALYSIS_SCHEMA_VERSION,
                                "build": seseragi_release::build_metadata(crate::SERVER_NAME)
                            }
                        }
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
                let uri = params.text_document.uri;
                let document = DocumentState::analyze(
                    &uri,
                    params.text_document.version,
                    params.text_document.text,
                );
                let published = publish(
                    &uri,
                    &document,
                    self.encoding.unwrap_or(PositionEncoding::Utf16),
                )?;
                self.documents.insert(uri, document);
                Ok(vec![published])
            }
            Some("textDocument/didChange") => {
                let params: DidChangeParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                let Some(change) = params.content_changes.into_iter().last() else {
                    return Ok(Vec::new());
                };
                let uri = params.text_document.uri;
                let document =
                    DocumentState::analyze(&uri, params.text_document.version, change.text);
                let published = publish(
                    &uri,
                    &document,
                    self.encoding.unwrap_or(PositionEncoding::Utf16),
                )?;
                self.documents.insert(uri, document);
                Ok(vec![published])
            }
            Some("textDocument/didClose") => {
                let params: DidCloseParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                self.documents.remove(&params.text_document.uri);
                Ok(vec![json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {"uri": params.text_document.uri, "diagnostics": []}
                })])
            }
            Some("textDocument/hover") => {
                let markup = self.hover_markup;
                Ok(vec![self.position_response(
                    id,
                    &message,
                    |document, params, encoding| {
                        features::hover(document, params, encoding, markup)
                    },
                    Value::Null,
                )])
            }
            Some("textDocument/completion") => {
                let markup = self.completion_markup;
                Ok(vec![self.position_response(
                    id,
                    &message,
                    |document, params, encoding| {
                        features::completion(document, params, encoding, markup)
                    },
                    json!([]),
                )])
            }
            Some("textDocument/signatureHelp") => {
                let markup = self.signature_markup;
                Ok(vec![self.position_response(
                    id,
                    &message,
                    |document, params, encoding| {
                        features::signature_help(document, params, encoding, markup)
                    },
                    Value::Null,
                )])
            }
            Some("textDocument/definition") => Ok(vec![self.position_response(
                id,
                &message,
                features::definition,
                Value::Null,
            )]),
            Some("textDocument/codeAction") => {
                let result = parse_params::<CodeActionParams>(&message)
                    .and_then(|params| {
                        self.documents
                            .get(&params.text_document.uri)
                            .map(|document| {
                                features::code_actions(
                                    document,
                                    &params,
                                    self.encoding.unwrap_or(PositionEncoding::Utf16),
                                )
                            })
                    })
                    .unwrap_or_else(|| json!([]));
                Ok(vec![response(id, result)])
            }
            Some("textDocument/formatting") => {
                let result = parse_params::<DocumentFormattingParams>(&message)
                    .and_then(|params| self.documents.get(&params.text_document.uri))
                    .map(|document| {
                        features::document_formatting(
                            document,
                            self.encoding.unwrap_or(PositionEncoding::Utf16),
                        )
                    })
                    .unwrap_or_else(|| json!([]));
                Ok(vec![response(id, result)])
            }
            Some("textDocument/semanticTokens/full") => {
                let result = parse_params::<SemanticTokensParams>(&message)
                    .and_then(|params| self.documents.get(&params.text_document.uri))
                    .map(|document| {
                        features::semantic_tokens(
                            document,
                            self.encoding.unwrap_or(PositionEncoding::Utf16),
                        )
                    })
                    .unwrap_or_else(|| json!({"data": []}));
                Ok(vec![response(id, result)])
            }
            Some(_) if id.is_some() => Ok(vec![json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": "Method not found"}
            })]),
            _ => Ok(Vec::new()),
        }
    }

    fn position_response<F>(
        &self,
        id: Option<Value>,
        message: &Value,
        feature: F,
        fallback: Value,
    ) -> Value
    where
        F: FnOnce(&DocumentState, &TextDocumentPositionParams, PositionEncoding) -> Value,
    {
        let result = parse_params::<TextDocumentPositionParams>(message)
            .and_then(|params| {
                self.documents
                    .get(&params.text_document.uri)
                    .map(|document| {
                        feature(
                            document,
                            &params,
                            self.encoding.unwrap_or(PositionEncoding::Utf16),
                        )
                    })
            })
            .unwrap_or(fallback);
        response(id, result)
    }
}

fn publish(
    uri: &str,
    document: &DocumentState,
    encoding: PositionEncoding,
) -> Result<Value, LineIndexError> {
    let diagnostics =
        diagnostics::convert(&document.analysis.diagnostics, &document.source, encoding)?;
    Ok(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "version": document.version, "diagnostics": diagnostics}
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
            &DocumentState::analyze(
                "file:///unresolved.ssrg",
                1,
                "pub fn useMissing value: Int -> Int = missing\n".to_owned(),
            ),
            PositionEncoding::Utf16,
        )
        .unwrap();
        let mismatch = publish(
            "file:///mismatch.ssrg",
            &DocumentState::analyze(
                "file:///mismatch.ssrg",
                1,
                "pub fn bad value: Int -> String = value\n".to_owned(),
            ),
            PositionEncoding::Utf16,
        )
        .unwrap();
        let structured = publish(
            "file:///structured-mismatch.ssrg",
            &DocumentState::analyze(
                "file:///structured-mismatch.ssrg",
                1,
                "pub fn bad value: (String -> Int) -> (Int -> String) = value\n".to_owned(),
            ),
            PositionEncoding::Utf16,
        )
        .unwrap();

        assert_eq!(unresolved["params"]["diagnostics"][0]["code"], "SES-N0001");
        assert_eq!(mismatch["params"]["diagnostics"][0]["code"], "SES-T0101");
        assert_eq!(
            unresolved["params"]["diagnostics"][0]["message"],
            "Name could not be resolved"
        );
        assert_ne!(
            unresolved["params"]["diagnostics"][0]["message"],
            unresolved["params"]["diagnostics"][0]["data"]["messageKey"]
        );
        assert!(unresolved["params"]["diagnostics"][0]["relatedInformation"]
            .as_array()
            .is_some_and(|items| !items.is_empty()));
        assert_eq!(
            mismatch["params"]["diagnostics"][0]["data"]["expectedType"],
            "String"
        );
        assert_eq!(
            mismatch["params"]["diagnostics"][0]["data"]["actualType"],
            "Int"
        );
        assert_eq!(
            structured["params"]["diagnostics"][0]["data"]["typeDifference"]["entries"][0]
                ["message"],
            "parameter 1: expected Int, actual String"
        );
        assert_eq!(
            structured["params"]["diagnostics"][0]["data"]["typeDifference"]["entries"][1]
                ["message"],
            "return type: expected String, actual Int"
        );
    }

    #[test]
    fn publishes_shared_standard_html_prop_diagnostics() {
        let source = r#"import * as html from "std/web/html"

fn view -> html.Html<Never> =
  html.div { clasName: "hero", children: "Typo" }
"#;
        let published = publish(
            "file:///html-props.ssrg",
            &DocumentState::analyze("file:///html-props.ssrg", 1, source.to_owned()),
            PositionEncoding::Utf16,
        )
        .unwrap();
        let diagnostics = published["params"]["diagnostics"].as_array().unwrap();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0]["code"], "SES-L0101");
        assert_eq!(
            diagnostics[0]["data"]["messageKey"],
            "web.html.unknown-prop"
        );
        assert_eq!(
            diagnostics[0]["data"]["fixes"][0]["edits"][0]["replacement"],
            "className"
        );
    }
}
