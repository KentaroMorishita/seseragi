use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(default)]
    pub capabilities: ClientCapabilities,
    #[serde(default)]
    pub root_uri: Option<String>,
    #[serde(default)]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    #[serde(default, rename = "name")]
    pub _name: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub general: GeneralCapabilities,
    #[serde(default)]
    pub text_document: TextDocumentClientCapabilities,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralCapabilities {
    #[serde(default)]
    pub position_encodings: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    #[serde(default)]
    pub hover: HoverClientCapabilities,
    #[serde(default)]
    pub completion: CompletionClientCapabilities,
    #[serde(default)]
    pub signature_help: SignatureHelpClientCapabilities,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverClientCapabilities {
    #[serde(default)]
    pub content_format: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionClientCapabilities {
    #[serde(default)]
    pub completion_item: CompletionItemClientCapabilities,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemClientCapabilities {
    #[serde(default)]
    pub documentation_format: Vec<String>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpClientCapabilities {
    #[serde(default)]
    pub signature_information: SignatureInformationClientCapabilities,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureInformationClientCapabilities {
    #[serde(default)]
    pub documentation_format: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MarkupKind {
    Markdown,
    #[default]
    PlainText,
}

impl MarkupKind {
    pub fn negotiate(formats: &[String]) -> Self {
        formats
            .iter()
            .find_map(|format| match format.as_str() {
                "markdown" => Some(Self::Markdown),
                "plaintext" => Some(Self::PlainText),
                _ => None,
            })
            .unwrap_or_default()
    }

    pub fn lsp_name(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::PlainText => "plaintext",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenParams {
    pub text_document: TextDocumentItem,
}

#[derive(Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    pub version: i64,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<ContentChange>,
}

#[derive(Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i64,
}

#[derive(Deserialize)]
pub struct ContentChange {
    pub text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCloseParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeWatchedFilesParams {
    pub changes: Vec<FileEvent>,
}

#[derive(Deserialize)]
pub struct FileEvent {
    pub uri: String,
    #[serde(rename = "type")]
    pub change_type: u8,
}

#[derive(Deserialize)]
pub struct DidChangeWorkspaceFoldersParams {
    pub event: WorkspaceFoldersChangeEvent,
}

#[derive(Deserialize)]
pub struct WorkspaceFoldersChangeEvent {
    #[serde(default)]
    pub added: Vec<WorkspaceFolder>,
    #[serde(default)]
    pub removed: Vec<WorkspaceFolder>,
}

#[derive(Deserialize)]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentPositionParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferencesParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    #[serde(default)]
    pub context: ReferenceContext,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameParams {
    pub text_document: TextDocumentIdentifier,
    pub position: Position,
    pub new_name: String,
}

#[derive(Deserialize)]
pub struct WorkspaceSymbolParams {
    #[serde(default)]
    pub query: String,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceContext {
    #[serde(default)]
    pub include_declaration: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionParams {
    pub text_document: TextDocumentIdentifier,
    pub range: Range,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFormattingParams {
    pub text_document: TextDocumentIdentifier,
}
