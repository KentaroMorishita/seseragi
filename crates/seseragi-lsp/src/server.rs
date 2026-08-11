use crate::capabilities::{negotiate_position_encoding, position_encoding_name};
use crate::diagnostics;
use crate::features::{self, DocumentState, SEMANTIC_TOKEN_TYPES};
use crate::model::{
    CodeActionParams, DidChangeParams, DidChangeWatchedFilesParams,
    DidChangeWorkspaceFoldersParams, DidCloseParams, DidOpenParams, DocumentFormattingParams,
    InitializeParams, MarkupKind, ReferencesParams, SemanticTokensParams,
    TextDocumentPositionParams,
};
use crate::protocol::{self, ProtocolError};
use crate::workspace::{
    self, file_path, project_key_for, workspace_folder_paths, OpenDocument, ProjectKey,
    ProjectSnapshot,
};
use serde::Deserialize;
use serde_json::{json, Value};
use seseragi_source::{LineIndexError, PositionEncoding};
use std::collections::{BTreeMap, BTreeSet};
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
    workspace_folders: Vec<std::path::PathBuf>,
    open_documents: BTreeMap<String, OpenDocument>,
    scratch_documents: BTreeMap<String, DocumentState>,
    projects: BTreeMap<ProjectKey, ProjectSnapshot>,
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
                self.workspace_folders = workspace_folder_paths(
                    params
                        .workspace_folders
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .map(|folder| folder.uri.clone()),
                    params.root_uri.clone(),
                );
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
                            "workspace": {
                                "workspaceFolders": {
                                    "supported": true,
                                    "changeNotifications": true
                                }
                            },
                            "hoverProvider": true,
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": ["."]
                            },
                            "signatureHelpProvider": {
                                "triggerCharacters": [" ", "(", ","]
                            },
                            "definitionProvider": true,
                            "referencesProvider": true,
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
                self.open_documents.insert(
                    uri.clone(),
                    OpenDocument {
                        version: params.text_document.version,
                        source: params.text_document.text,
                        path: file_path(&uri),
                    },
                );
                self.reanalyze_document(&uri)
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
                let path = self
                    .open_documents
                    .get(&uri)
                    .and_then(|document| document.path.clone())
                    .or_else(|| file_path(&uri));
                self.open_documents.insert(
                    uri.clone(),
                    OpenDocument {
                        version: params.text_document.version,
                        source: change.text,
                        path,
                    },
                );
                self.reanalyze_document(&uri)
            }
            Some("textDocument/didClose") => {
                let params: DidCloseParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                self.close_document(&params.text_document.uri)
            }
            Some("workspace/didChangeWatchedFiles") => {
                let params: DidChangeWatchedFilesParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                self.reanalyze_watched_files(params)
            }
            Some("workspace/didChangeWorkspaceFolders") => {
                let params: DidChangeWorkspaceFoldersParams = match parse_params(&message) {
                    Some(params) => params,
                    None => return Ok(Vec::new()),
                };
                self.change_workspace_folders(params)
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
            Some("textDocument/definition") => Ok(vec![self.definition_response(id, &message)]),
            Some("textDocument/references") => Ok(vec![self.references_response(id, &message)]),
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

    fn reanalyze_document(&mut self, uri: &str) -> Result<Vec<Value>, ServerError> {
        self.refresh_open_document(uri);
        self.rebuild_documents()
    }

    fn refresh_open_document(&mut self, uri: &str) {
        let existing = self
            .projects
            .iter()
            .find_map(|(key, snapshot)| snapshot.documents.contains_key(uri).then(|| key.clone()));
        let inferred = self
            .open_documents
            .get(uri)
            .and_then(|document| document.path.as_deref())
            .and_then(|path| project_key_for(path, &self.workspace_folders));
        if let Some(key) = existing.or(inferred) {
            self.scratch_documents.remove(uri);
            self.refresh_project(key, uri);
            return;
        }
        self.refresh_scratch_document(uri);
    }

    fn refresh_project(&mut self, key: ProjectKey, preferred_uri: &str) {
        match workspace::analyze(&key, &self.open_documents) {
            Ok(snapshot) => {
                self.projects.insert(key, snapshot);
            }
            Err(_) => {
                let fallback_uri = self
                    .open_documents
                    .contains_key(preferred_uri)
                    .then(|| preferred_uri.to_owned())
                    .or_else(|| self.open_uri_for_project(&key));
                match fallback_uri {
                    Some(uri) => {
                        let mut documents = BTreeMap::new();
                        if let Some(document) = self.open_documents.get(&uri) {
                            documents.insert(
                                uri.clone(),
                                DocumentState::analyze(
                                    &uri,
                                    document.version,
                                    document.source.clone(),
                                ),
                            );
                        }
                        self.projects.insert(key, ProjectSnapshot { documents });
                    }
                    None => {
                        self.projects.remove(&key);
                    }
                }
            }
        }
    }

    fn refresh_scratch_document(&mut self, uri: &str) {
        let Some(document) = self.open_documents.get(uri) else {
            self.scratch_documents.remove(uri);
            return;
        };
        self.scratch_documents.insert(
            uri.to_owned(),
            DocumentState::analyze(uri, document.version, document.source.clone()),
        );
    }

    fn open_uri_for_project(&self, key: &ProjectKey) -> Option<String> {
        self.open_documents.iter().find_map(|(uri, document)| {
            document
                .path
                .as_deref()
                .filter(|path| key.contains_path(path))
                .map(|_| uri.clone())
        })
    }

    fn close_document(&mut self, uri: &str) -> Result<Vec<Value>, ServerError> {
        let closed = self.open_documents.remove(uri);
        self.scratch_documents.remove(uri);
        let closed_path = closed
            .as_ref()
            .and_then(|document| document.path.as_deref());
        let keys = self
            .projects
            .iter()
            .filter_map(|(key, snapshot)| {
                (snapshot.documents.contains_key(uri)
                    || closed_path.is_some_and(|path| key.contains_path(path)))
                .then(|| key.clone())
            })
            .collect::<Vec<_>>();
        for key in keys {
            if key.entry().is_some_and(|entry| closed_path == Some(entry)) {
                self.projects.remove(&key);
                continue;
            }
            if let Some(fallback_uri) = self.open_uri_for_project(&key) {
                self.refresh_project(key, &fallback_uri);
            } else {
                self.projects.remove(&key);
            }
        }
        let unrepresented = self
            .open_documents
            .keys()
            .filter(|uri| {
                !self
                    .projects
                    .values()
                    .any(|snapshot| snapshot.documents.contains_key(*uri))
            })
            .cloned()
            .collect::<Vec<_>>();
        for uri in unrepresented {
            self.refresh_open_document(&uri);
        }
        self.rebuild_documents()
    }

    fn reanalyze_watched_files(
        &mut self,
        params: DidChangeWatchedFilesParams,
    ) -> Result<Vec<Value>, ServerError> {
        let changed_paths = params
            .changes
            .into_iter()
            .filter_map(|event| {
                let _ = event.change_type;
                file_path(&event.uri)
            })
            .collect::<Vec<_>>();
        if changed_paths.is_empty() {
            return Ok(Vec::new());
        }
        let manifest_changed = |key: &ProjectKey| {
            changed_paths.iter().any(|path| {
                key.contains_path(path)
                    && path.file_name().is_some_and(|name| name == "seseragi.toml")
            })
        };
        let affected = self
            .projects
            .keys()
            .filter(|key| changed_paths.iter().any(|path| key.contains_path(path)))
            .cloned()
            .collect::<Vec<_>>();
        for key in affected {
            if manifest_changed(&key) {
                self.projects.remove(&key);
                continue;
            }
            if let Some(uri) = self.open_uri_for_project(&key) {
                self.refresh_project(key, &uri);
            }
        }
        let unrepresented = self
            .open_documents
            .iter()
            .filter_map(|(uri, document)| {
                let path = document.path.as_deref()?;
                changed_paths
                    .iter()
                    .any(|changed| path.starts_with(changed.parent().unwrap_or(changed)))
                    .then(|| uri.clone())
            })
            .filter(|uri| {
                !self
                    .projects
                    .values()
                    .any(|snapshot| snapshot.documents.contains_key(uri))
            })
            .collect::<Vec<_>>();
        for uri in unrepresented {
            self.refresh_open_document(&uri);
        }
        self.rebuild_documents()
    }

    fn change_workspace_folders(
        &mut self,
        params: DidChangeWorkspaceFoldersParams,
    ) -> Result<Vec<Value>, ServerError> {
        let removed = params
            .event
            .removed
            .into_iter()
            .filter_map(|folder| file_path(&folder.uri))
            .collect::<BTreeSet<_>>();
        self.workspace_folders
            .retain(|folder| !removed.contains(folder));
        self.workspace_folders.extend(
            params
                .event
                .added
                .into_iter()
                .filter_map(|folder| file_path(&folder.uri)),
        );
        self.workspace_folders.sort();
        self.workspace_folders.dedup();
        self.projects.clear();
        self.scratch_documents.clear();
        let open = self.open_documents.keys().cloned().collect::<Vec<_>>();
        for uri in open {
            self.refresh_open_document(&uri);
        }
        self.rebuild_documents()
    }

    fn rebuild_documents(&mut self) -> Result<Vec<Value>, ServerError> {
        let previous = std::mem::take(&mut self.documents);
        let mut documents = self.scratch_documents.clone();
        for snapshot in self.projects.values() {
            for (uri, document) in &snapshot.documents {
                let replace = documents.get(uri).is_none_or(|existing| {
                    existing.version.is_none() || document.version.is_some()
                });
                if replace {
                    documents.insert(uri.clone(), document.clone());
                }
            }
        }
        let uris = previous
            .keys()
            .chain(documents.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        self.documents = documents;
        let encoding = self.encoding.unwrap_or(PositionEncoding::Utf16);
        uris.into_iter()
            .map(|uri| match self.documents.get(&uri) {
                Some(document) => publish(&uri, document, encoding).map_err(ServerError::from),
                None => Ok(clear_diagnostics(&uri)),
            })
            .collect()
    }

    fn definition_response(&self, id: Option<Value>, message: &Value) -> Value {
        let result = parse_params::<TextDocumentPositionParams>(message)
            .and_then(|params| {
                let document = self.documents.get(&params.text_document.uri)?;
                let encoding = self.encoding.unwrap_or(PositionEncoding::Utf16);
                let identity = features::definition_identity(document, &params, encoding)?;
                self.definition_location(&identity)
                    .and_then(|(uri, document, start, end)| {
                        features::range_json(&document.source, start, end, encoding)
                            .map(|range| json!({"uri": uri, "range": range}))
                    })
                    .or_else(|| Some(features::definition(document, &params, encoding)))
            })
            .unwrap_or(Value::Null);
        response(id, result)
    }

    fn definition_location(&self, identity: &str) -> Option<(&str, &DocumentState, usize, usize)> {
        self.documents.iter().find_map(|(uri, document)| {
            let prefix = format!("{}::", document.analysis.module);
            document
                .analysis
                .symbols
                .iter()
                .find(|symbol| {
                    symbol.identity == identity
                        && symbol.identity.starts_with(&prefix)
                        && symbol.definition.end > symbol.definition.start
                })
                .map(|symbol| {
                    (
                        uri.as_str(),
                        document,
                        symbol.definition.start,
                        symbol.definition.end,
                    )
                })
        })
    }

    fn references_response(&self, id: Option<Value>, message: &Value) -> Value {
        let result = parse_params::<ReferencesParams>(message)
            .and_then(|params| {
                let document = self.documents.get(&params.text_document.uri)?;
                let encoding = self.encoding.unwrap_or(PositionEncoding::Utf16);
                let identity = features::definition_identity(
                    document,
                    &TextDocumentPositionParams {
                        text_document: params.text_document,
                        position: params.position,
                    },
                    encoding,
                )?;
                Some(self.reference_locations(
                    &identity,
                    params.context.include_declaration,
                    encoding,
                ))
            })
            .unwrap_or_else(|| json!([]));
        response(id, result)
    }

    fn reference_locations(
        &self,
        identity: &str,
        include_declaration: bool,
        encoding: PositionEncoding,
    ) -> Value {
        let mut locations = Vec::new();
        let mut seen = BTreeSet::new();
        for (uri, document) in &self.documents {
            let local_identity_prefix = format!("{}::", document.analysis.module);
            for occurrence in &document.analysis.symbol_occurrences {
                let Some(symbol) = document.analysis.symbols.get(occurrence.symbol as usize) else {
                    continue;
                };
                if symbol.identity != identity {
                    continue;
                }
                let is_declaration = symbol.identity.starts_with(&local_identity_prefix)
                    && symbol.definition.start == occurrence.range.start
                    && symbol.definition.end == occurrence.range.end;
                if !include_declaration && is_declaration {
                    continue;
                }
                let key = (uri, occurrence.range.start, occurrence.range.end);
                if !seen.insert(key) {
                    continue;
                }
                let Some(range) = features::range_json(
                    &document.source,
                    occurrence.range.start,
                    occurrence.range.end,
                    encoding,
                ) else {
                    continue;
                };
                locations.push(json!({"uri": uri, "range": range}));
            }
        }
        Value::Array(locations)
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
    let mut params = json!({"uri": uri, "diagnostics": diagnostics});
    if let Some(version) = document.version {
        params["version"] = json!(version);
    }
    Ok(json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": params
    }))
}

fn clear_diagnostics(uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {"uri": uri, "diagnostics": []}
    })
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
    use std::fs;
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use url::Url;

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
  html.div { clas: "hero", children: "Typo" }
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
            "class"
        );
    }

    #[test]
    fn watched_source_events_refresh_missing_and_restored_workspace_imports() {
        let workspace = TempWorkspace::new();
        let main = concat!(
            "import { increment } from \"./domain\"\n",
            "pub fn run value: Int -> Int = increment value\n",
        );
        let domain = "pub fn increment value: Int -> Int = value + 1\n";
        workspace.write("main.ssrg", main);
        workspace.write("domain.ssrg", domain);
        let root_uri = file_uri(workspace.path());
        let main_uri = file_uri(&workspace.path().join("main.ssrg"));
        let domain_uri = file_uri(&workspace.path().join("domain.ssrg"));
        let mut state = State::default();

        state
            .handle(json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"capabilities": {}, "workspaceFolders": [{"uri": root_uri, "name": "fixture"}]}
            }))
            .unwrap();
        state
            .handle(json!({
                "jsonrpc": "2.0", "method": "textDocument/didOpen",
                "params": {"textDocument": {
                    "uri": main_uri, "languageId": "seseragi", "version": 1, "text": main
                }}
            }))
            .unwrap();
        assert!(state.documents[&main_uri]
            .analysis
            .diagnostics
            .diagnostics
            .is_empty());

        fs::remove_file(workspace.path().join("domain.ssrg")).unwrap();
        state
            .handle(json!({
                "jsonrpc": "2.0", "method": "workspace/didChangeWatchedFiles",
                "params": {"changes": [{"uri": domain_uri, "type": 3}]}
            }))
            .unwrap();
        assert!(state.documents[&main_uri]
            .analysis
            .diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-N0104"));

        workspace.write("domain.ssrg", domain);
        state
            .handle(json!({
                "jsonrpc": "2.0", "method": "workspace/didChangeWatchedFiles",
                "params": {"changes": [{"uri": domain_uri, "type": 1}]}
            }))
            .unwrap();
        assert!(state.documents[&main_uri]
            .analysis
            .diagnostics
            .diagnostics
            .is_empty());
    }

    fn file_uri(path: &Path) -> String {
        Url::from_file_path(path.canonicalize().unwrap())
            .unwrap()
            .into()
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
                "seseragi-lsp-watched-workspace-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write(&self, relative: &str, source: &str) {
            fs::write(self.path.join(relative), source).unwrap();
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
