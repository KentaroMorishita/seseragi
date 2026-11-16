use crate::model::{CodeActionParams, MarkupKind, Position, TextDocumentPositionParams};
use serde_json::{json, Value};
use seseragi_driver::{
    analyze_module, format_module, AnalysisCompletionContext, AnalysisDocument,
    AnalysisReferenceItem, AnalysisSymbol, CompileInput, TypeDocument,
};
use seseragi_source::{EncodedPosition, LineIndex, PositionEncoding};
use seseragi_syntax::{is_custom_operator_candidate, lex, parse_surface_ast, ByteSpan, TokenKind};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SEMANTIC_TOKEN_TYPES: [&str; 22] = [
    "namespace",
    "type",
    "class",
    "enum",
    "interface",
    "struct",
    "typeParameter",
    "parameter",
    "variable",
    "property",
    "enumMember",
    "event",
    "function",
    "method",
    "macro",
    "keyword",
    "modifier",
    "comment",
    "string",
    "number",
    "regexp",
    "operator",
];

#[derive(Clone)]
pub(crate) struct DocumentState {
    pub(crate) version: Option<i64>,
    pub(crate) source: String,
    pub(crate) analysis: AnalysisDocument,
    pub(crate) lint_enabled: bool,
    pub(crate) target_diagnostics: Vec<crate::target_diagnostics::TargetDiagnostic>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SymbolKey {
    pub(crate) identity: String,
    pub(crate) namespace: String,
}

impl DocumentState {
    pub(crate) fn analyze(uri: &str, version: i64, source: String) -> Self {
        Self::analyze_with_version(uri, Some(version), source)
    }

    pub(crate) fn analyze_with_version(uri: &str, version: Option<i64>, source: String) -> Self {
        let analysis = analyze_module(CompileInput::new(uri, uri, &source));
        Self {
            version,
            source,
            analysis,
            lint_enabled: true,
            target_diagnostics: Vec::new(),
        }
    }

    pub(crate) fn from_analysis(
        version: Option<i64>,
        source: String,
        analysis: AnalysisDocument,
        lint_enabled: bool,
    ) -> Self {
        Self {
            version,
            source,
            analysis,
            lint_enabled,
            target_diagnostics: Vec::new(),
        }
    }

    pub(crate) fn byte_position(
        &self,
        position: Position,
        encoding: PositionEncoding,
    ) -> Option<usize> {
        LineIndex::new(&self.source)
            .try_offset_encoded(
                EncodedPosition {
                    line: position.line,
                    character: position.character,
                },
                encoding,
            )
            .ok()
    }

    /// Like [`Self::byte_position`], but a position past the end of its line
    /// or of the document is clamped instead of rejected. Clients send range
    /// bounds such as "line 40" for a whole-document request (LSP 3.17 lets
    /// a range end past the content).
    pub(crate) fn clamped_byte_position(
        &self,
        position: Position,
        encoding: PositionEncoding,
    ) -> usize {
        if let Some(offset) = self.byte_position(position, encoding) {
            return offset;
        }
        let index = LineIndex::new(&self.source);
        let line = usize::try_from(position.line).unwrap_or(usize::MAX);
        match index.line_starts().get(line.saturating_add(1)) {
            // Past the end of a line: stop before that line's terminator.
            Some(&next) => {
                let line_text = &self.source[index.line_starts()[line]..next];
                next - (line_text.len() - line_text.trim_end_matches(['\n', '\r']).len())
            }
            None => self.source.len(),
        }
    }

    pub(crate) fn query_position(
        &self,
        position: Position,
        encoding: PositionEncoding,
    ) -> Option<usize> {
        let byte = self.byte_position(position, encoding)?;
        if self.analysis.symbol_at(byte).is_some()
            || self.analysis.type_at(byte).is_some()
            || self.analysis.callable_at(byte).is_some()
        {
            return Some(byte);
        }
        byte.checked_sub(1)
    }
}

pub(crate) fn definition_key(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Option<SymbolKey> {
    symbol_at_position(document, params, encoding).map(symbol_key)
}

pub(crate) fn symbol_key(symbol: &AnalysisSymbol) -> SymbolKey {
    SymbolKey {
        identity: symbol.identity.clone(),
        namespace: symbol.namespace.clone(),
    }
}

pub(crate) fn symbol_at_position<'document>(
    document: &'document DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Option<&'document AnalysisSymbol> {
    let byte = document.byte_position(params.position, encoding)?;
    document
        .analysis
        .symbol_at(byte)
        .or_else(|| {
            byte.checked_sub(1)
                .and_then(|position| document.analysis.symbol_at(position))
        })
        .or_else(|| imported_symbol_at_byte(document, byte))
        .or_else(|| {
            byte.checked_sub(1)
                .and_then(|position| imported_symbol_at_byte(document, position))
        })
}

pub(crate) fn symbol_range_at_position(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Option<ByteSpan> {
    let byte = document.byte_position(params.position, encoding)?;
    symbol_occurrence_at_byte(document, byte)
        .or_else(|| {
            byte.checked_sub(1)
                .and_then(|position| symbol_occurrence_at_byte(document, position))
        })
        .or_else(|| imported_name_range_at_byte(document, byte))
        .or_else(|| {
            byte.checked_sub(1)
                .and_then(|position| imported_name_range_at_byte(document, position))
        })
}

fn symbol_occurrence_at_byte(document: &DocumentState, position: usize) -> Option<ByteSpan> {
    document
        .analysis
        .symbol_occurrences
        .iter()
        .filter(|occurrence| occurrence.range.start <= position && position < occurrence.range.end)
        .min_by_key(|occurrence| occurrence.range.end.saturating_sub(occurrence.range.start))
        .map(|occurrence| occurrence.range)
}

fn imported_symbol_at_byte(document: &DocumentState, position: usize) -> Option<&AnalysisSymbol> {
    let import = parse_surface_ast(&document.analysis.module, &document.source)
        .imports
        .into_iter()
        .flat_map(|import| import.items)
        .find(|item| item.name_span.start <= position && position < item.name_span.end)?;
    let local = import.alias_span.unwrap_or(import.name_span);
    let expected_namespace = match import.namespace.as_str() {
        "operator" => Some("operator"),
        "namespace" => Some("module"),
        _ => None,
    };
    let candidates = document
        .analysis
        .symbols
        .iter()
        .filter(|symbol| {
            symbol.definition == local
                && expected_namespace.is_none_or(|namespace| symbol.namespace == namespace)
        })
        .collect::<Vec<_>>();
    let first = *candidates.first()?;
    candidates
        .iter()
        .all(|candidate| symbol_key(candidate) == symbol_key(first))
        .then_some(first)
}

fn imported_name_range_at_byte(document: &DocumentState, position: usize) -> Option<ByteSpan> {
    parse_surface_ast(&document.analysis.module, &document.source)
        .imports
        .into_iter()
        .flat_map(|import| import.items)
        .find(|item| item.name_span.start <= position && position < item.name_span.end)
        .map(|item| item.name_span)
}

pub(crate) fn valid_rename_name(symbol: &AnalysisSymbol, new_name: &str) -> bool {
    if symbol.namespace == "operator" {
        return is_custom_operator_candidate(new_name);
    }
    let tokens = lex("<rename>", new_name)
        .tokens
        .into_iter()
        .filter(|token| {
            !matches!(
                token.kind,
                TokenKind::TriviaComment
                    | TokenKind::TriviaNewline
                    | TokenKind::TriviaSpace
                    | TokenKind::Eof
            )
        })
        .collect::<Vec<_>>();
    let [token] = tokens.as_slice() else {
        return false;
    };
    if token.start != 0 || token.end != new_name.len() {
        return false;
    }
    match (symbol.namespace.as_str(), symbol.kind.as_str()) {
        ("type" | "trait", _) | (_, "constructor" | "type-parameter") => {
            token.kind == TokenKind::IdentifierUpper
        }
        ("module", _) => matches!(
            token.kind,
            TokenKind::IdentifierLower | TokenKind::IdentifierUpper
        ),
        _ => token.kind == TokenKind::IdentifierLower,
    }
}

pub(crate) fn imported_name_ranges(document: &DocumentState, key: &SymbolKey) -> Vec<ByteSpan> {
    parse_surface_ast(&document.analysis.module, &document.source)
        .imports
        .into_iter()
        .flat_map(|import| import.items)
        .flat_map(|item| {
            let local = item.alias_span.unwrap_or(item.name_span);
            let matches = document.analysis.symbols.iter().any(|symbol| {
                symbol.identity == key.identity
                    && symbol.namespace == key.namespace
                    && symbol.definition == local
            });
            if !matches {
                return Vec::new();
            }
            let mut ranges = vec![item.name_span];
            if local != item.name_span {
                ranges.push(local);
            }
            ranges
        })
        .collect()
}

pub(crate) fn precise_symbol_range(
    document: &DocumentState,
    range: ByteSpan,
    symbol: &AnalysisSymbol,
) -> ByteSpan {
    let Some(source) = document.source.get(range.start..range.end) else {
        return range;
    };
    let Some(prefix) = source.strip_suffix(&symbol.name) else {
        return range;
    };
    if !prefix.ends_with('.') {
        return range;
    }
    ByteSpan {
        start: range.end - symbol.name.len(),
        end: range.end,
    }
}

pub(crate) fn workspace_symbol_kind(symbol: &AnalysisSymbol) -> u8 {
    match (symbol.namespace.as_str(), symbol.kind.as_str()) {
        ("module", _) => 3,
        (_, "function" | "effect-function" | "generic-foreign-function") => 12,
        (_, "trait-method" | "inherent-method") => 6,
        (_, "constructor") => 9,
        ("type", _) => 23,
        ("trait", _) => 11,
        ("field", _) => 8,
        ("operator", _) => 25,
        _ => 13,
    }
}

pub(crate) fn hover(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
    markup: MarkupKind,
) -> Value {
    let Some(position) = document.query_position(params.position, encoding) else {
        return Value::Null;
    };
    let symbol = document.analysis.symbol_at(position);
    let callable = document.analysis.callable_at(position);
    let inferred = document.analysis.type_at(position);
    if symbol.is_none() && callable.is_none() && inferred.is_none() {
        return Value::Null;
    }

    let reference = symbol.and_then(|symbol| reference_for(&document.analysis, &symbol.identity));
    let mut lines = Vec::new();
    let compact_signature = callable
        .map(|callable| callable.signature.clone())
        .or_else(|| reference.and_then(|item| item.signature.clone()));
    let signature = callable
        .map(|callable| callable.multiline_signature.clone())
        .or_else(|| reference.and_then(|item| item.multiline_signature.clone()))
        .or_else(|| {
            symbol.and_then(|symbol| {
                symbol
                    .multiline_type_name
                    .as_ref()
                    .map(|type_name| format!("{}:\n{type_name}", symbol.name))
            })
        });
    if let Some(signature) = &signature {
        lines.push(type_block(signature, markup));
    }
    if let Some(symbol) = symbol {
        lines.push(match markup {
            MarkupKind::Markdown => format!("`{}` · `{}`", symbol.module, symbol.identity),
            MarkupKind::PlainText => format!("{} · {}", symbol.module, symbol.identity),
        });
    }
    if let Some(type_occurrence) = inferred {
        let already_in_signature = compact_signature
            .as_ref()
            .is_some_and(|value| value.contains(&type_occurrence.type_name));
        if !already_in_signature {
            lines.push(format!(
                "Inferred type:\n{}",
                type_block(&type_occurrence.multiline_type_name, markup)
            ));
        }
    }
    if let Some(callable) = callable {
        if !callable.remaining_parameters.is_empty() {
            lines.push(format!(
                "Remaining: {}",
                callable
                    .remaining_parameters
                    .iter()
                    .map(|parameter| {
                        let label = match &parameter.name {
                            Some(name) => format!("{name}: {}", parameter.type_name),
                            None => parameter.type_name.clone(),
                        };
                        type_inline(&label, markup)
                    })
                    .collect::<Vec<_>>()
                    .join(" → ")
            ));
        }
    }
    let description = symbol
        .and_then(|symbol| symbol.description.as_deref())
        .or_else(|| reference.map(|item| item.description.as_str()));
    if let Some(description) = description {
        lines.push(description.to_owned());
    }
    if let Some(deprecation) = symbol.and_then(|symbol| symbol.deprecation.as_ref()) {
        lines.push(deprecation_notice(deprecation));
    }

    let range = hover_range(&document.analysis, position)
        .and_then(|range| range_json(&document.source, range.start, range.end, encoding));
    json!({
        "contents": markup_content(lines.join("\n\n"), markup),
        "range": range,
    })
}

pub(crate) fn completion(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
    markup: MarkupKind,
) -> Value {
    let Some(position) = document.byte_position(params.position, encoding) else {
        return json!([]);
    };
    // A trailing namespace access is more specific than a recovered enclosing
    // record expectation (for example html. inside an unfinished children list).
    let access = namespace_prefix(&document.source, position);
    let namespace = access.map(|(prefix, _)| prefix);
    if let Some(context) = namespace
        .is_none()
        .then(|| completion_context(document, position))
        .flatten()
    {
        if !context.record_fields.is_empty() {
            return Value::Array(
                context
                    .record_fields
                    .iter()
                    .map(|field| {
                        json!({
                            "label": field.name,
                            "kind": 5,
                            "detail": field.type_name,
                            "documentation": completion_documentation(
                                Some(&field.type_name),
                                Some(&format!(
                                    "Expected field of {}.",
                                    type_inline(&context.type_name, markup)
                                )),
                                markup,
                            ),
                            "insertText": format!("{}: ", field.name),
                            "filterText": field.name,
                            "sortText": format!("0-{}", field.name),
                            "data": {"kind": "expected-record-field"},
                        })
                    })
                    .collect(),
            );
        }
    }
    let namespace_module = namespace.and_then(|prefix| {
        document
            .analysis
            .visible_symbols(position)
            .into_iter()
            .find(|symbol| symbol.namespace == "module" && symbol.name == prefix)
            .map(|symbol| symbol.module.clone())
            .or_else(|| imported_namespace_module(&document.source, prefix))
    });

    let mut items = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(module) = namespace_module {
        for item in document
            .analysis
            .standard_library_catalog()
            .iter()
            .filter(|item| item.module == module)
        {
            push_reference_completion(&mut items, &mut seen, item, "0", markup);
        }
    } else if let Some((name, range)) = access {
        // This bounded path completes a lexical value receiver. Do not treat
        // a chained field token as an unrelated same-named local binding.
        let simple_receiver = !document.source[..range.start].trim_end().ends_with('.');
        let methods = if simple_receiver {
            document.analysis.members_of_receiver(name, range.start)
        } else {
            Vec::new()
        };
        for symbol in methods {
            let key = format!("{}:{}", symbol.namespace, symbol.identity);
            if seen.insert(key) {
                items.push(symbol_completion(symbol, markup));
            }
        }
    } else {
        for symbol in document.analysis.visible_symbols(position) {
            let key = format!("{}:{}", symbol.namespace, symbol.identity);
            if !seen.insert(key) {
                continue;
            }
            items.push(symbol_completion(symbol, markup));
        }
        for item in document.analysis.standard_library_catalog() {
            push_reference_completion(&mut items, &mut seen, item, "1", markup);
        }
    }
    Value::Array(items)
}

fn completion_context(
    document: &DocumentState,
    position: usize,
) -> Option<AnalysisCompletionContext> {
    document
        .analysis
        .completion_at(position)
        .cloned()
        .or_else(|| {
            let recovered = completion_recovery_source(&document.source, position)?;
            analyze_module(CompileInput::new(
                &document.analysis.module,
                &document.analysis.module,
                &recovered,
            ))
            .completion_at(position)
            .cloned()
        })
}

fn completion_recovery_source(source: &str, position: usize) -> Option<String> {
    let before = source.get(..position)?;
    let after = source.get(position..)?;
    let tokens = lex("<completion>", before);
    let last = tokens
        .tokens
        .iter()
        .rev()
        .find(|token| {
            !matches!(
                token.kind,
                TokenKind::TriviaComment
                    | TokenKind::TriviaNewline
                    | TokenKind::TriviaSpace
                    | TokenKind::Eof
            )
        })
        .map(|token| token.kind);
    let placeholder = match last {
        Some(TokenKind::PunctuationColon) => "()",
        Some(TokenKind::IdentifierLower) => ": ()",
        _ => "",
    };
    let mut recovered = format!("{before}{placeholder}{after}");
    let mut delimiters = Vec::new();
    for token in lex("<completion>", &recovered).tokens {
        match token.kind {
            TokenKind::PunctuationBraceLeft => delimiters.push(TokenKind::PunctuationBraceRight),
            TokenKind::PunctuationParenLeft => delimiters.push(TokenKind::PunctuationParenRight),
            TokenKind::PunctuationListLeft | TokenKind::PunctuationSquareLeft => {
                delimiters.push(TokenKind::PunctuationSquareRight)
            }
            TokenKind::PunctuationBraceRight
            | TokenKind::PunctuationParenRight
            | TokenKind::PunctuationSquareRight => {
                if delimiters.last() == Some(&token.kind) {
                    delimiters.pop();
                }
            }
            _ => {}
        }
    }
    for delimiter in delimiters.into_iter().rev() {
        recovered.push(match delimiter {
            TokenKind::PunctuationBraceRight => '}',
            TokenKind::PunctuationParenRight => ')',
            TokenKind::PunctuationSquareRight => ']',
            _ => unreachable!("completion recovery stores only closing delimiters"),
        });
    }
    (recovered != source).then_some(recovered)
}

pub(crate) fn signature_help(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
    markup: MarkupKind,
) -> Value {
    let Some(position) = document.query_position(params.position, encoding) else {
        return Value::Null;
    };
    let Some(callable) = document.analysis.callable_at(position) else {
        return Value::Null;
    };
    let applied = callable
        .parameters
        .len()
        .saturating_sub(callable.remaining_parameters.len());
    let active_parameter = applied.min(callable.parameters.len().saturating_sub(1));
    let documentation = reference_for(&document.analysis, &callable.identity)
        .map(|item| item.description.clone())
        .unwrap_or_default();
    let parameters = callable
        .parameters
        .iter()
        .map(|parameter| {
            json!({
                "label": match &parameter.name {
                    Some(name) => format!("{name}: {}", parameter.type_name),
                    None => parameter.type_name.clone(),
                }
            })
        })
        .collect::<Vec<_>>();
    json!({
        "signatures": [{
            "label": callable.signature,
            "documentation": markup_content(documentation, markup),
            "parameters": parameters,
            "activeParameter": active_parameter,
        }],
        "activeSignature": 0,
        "activeParameter": active_parameter,
    })
}

pub(crate) fn definition(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Value {
    let Some(position) = document.query_position(params.position, encoding) else {
        return Value::Null;
    };
    let Some(definition) = document.analysis.definition_of(position) else {
        return Value::Null;
    };
    let Some(range) = range_json(&document.source, definition.start, definition.end, encoding)
    else {
        return Value::Null;
    };
    json!({"uri": params.text_document.uri, "range": range})
}

pub(crate) fn code_actions(
    document: &DocumentState,
    params: &CodeActionParams,
    encoding: PositionEncoding,
) -> Value {
    let Some(request_start) = document.byte_position(params.range.start, encoding) else {
        return json!([]);
    };
    let Some(request_end) = document.byte_position(params.range.end, encoding) else {
        return json!([]);
    };
    let mut actions = Vec::new();
    for diagnostic in &document.analysis.diagnostics.diagnostics {
        if !intersects(
            diagnostic.primary.start,
            diagnostic.primary.end,
            request_start,
            request_end,
        ) {
            continue;
        }
        for fix in &diagnostic.fixes {
            let edits = fix
                .edits
                .iter()
                .filter_map(|edit| {
                    Some(json!({
                        "range": range_json(
                            &document.source,
                            edit.range.start,
                            edit.range.end,
                            encoding,
                        )?,
                        "newText": edit.replacement,
                    }))
                })
                .collect::<Vec<_>>();
            if edits.len() != fix.edits.len() {
                continue;
            }
            let changes = BTreeMap::from([(params.text_document.uri.clone(), edits)]);
            actions.push(json!({
                "title": fix.title,
                "kind": "quickfix",
                "isPreferred": true,
                "edit": {"changes": changes},
            }));
        }
    }
    Value::Array(actions)
}

pub(crate) fn document_formatting(document: &DocumentState, encoding: PositionEncoding) -> Value {
    let Ok(formatted) = format_module(&document.analysis.source, &document.source) else {
        // didOpen/didChange already publish the shared parser diagnostics. Returning
        // no edits keeps an invalid buffer untouched without producing a second,
        // editor-specific error surface.
        return json!([]);
    };
    if !formatted.changed {
        return json!([]);
    }
    let Some(range) = range_json(&document.source, 0, document.source.len(), encoding) else {
        return json!([]);
    };
    json!([{
        "range": range,
        "newText": formatted.text,
    }])
}

pub(crate) fn semantic_tokens(document: &DocumentState, encoding: PositionEncoding) -> Value {
    let mut tokens = BTreeMap::new();
    for occurrence in &document.analysis.symbol_occurrences {
        let Some(symbol) = document.analysis.symbols.get(occurrence.symbol as usize) else {
            continue;
        };
        let (start, end) = spelling_range(&document.source, occurrence.range, &symbol.name)
            .unwrap_or((occurrence.range.start, occurrence.range.end));
        let Some(range) = encoded_range(&document.source, start, end, encoding) else {
            continue;
        };
        if range.start.line != range.end.line || range.start.character == range.end.character {
            continue;
        }
        let token = tokens
            .entry((range.start.line, range.start.character, range.end.character))
            .or_insert((semantic_token_type(symbol), 0usize));
        token.0 = semantic_token_type(symbol);
        // A newtype's declaration name also declares its constructor. A
        // constructor occurrence elsewhere must not inherit this type notice.
        token.1 |= usize::from(symbol.deprecation.is_some());
    }

    let mut data = Vec::new();
    let mut previous_line = 0;
    let mut previous_character = 0;
    let mut previous_end = None;
    for ((line, character, end), (token_type, modifiers)) in tokens {
        if previous_end.is_some_and(|(previous_line, previous_end)| {
            previous_line == line && character < previous_end
        }) {
            continue;
        }
        let delta_line = line - previous_line;
        let delta_start = if delta_line == 0 {
            character - previous_character
        } else {
            character
        };
        data.extend([
            delta_line,
            delta_start,
            end - character,
            token_type,
            modifiers,
        ]);
        previous_line = line;
        previous_character = character;
        previous_end = Some((line, end));
    }
    json!({"data": data})
}

fn symbol_completion(symbol: &AnalysisSymbol, markup: MarkupKind) -> Value {
    let detail = symbol
        .callable
        .as_ref()
        .map(|callable| callable.signature.clone())
        .or_else(|| symbol.type_name.clone())
        .unwrap_or_else(|| symbol.kind.clone());
    let description = match (&symbol.description, &symbol.deprecation) {
        (Some(description), Some(deprecation)) => Some(format!(
            "{description}\n\n{}",
            deprecation_notice(deprecation)
        )),
        (None, Some(deprecation)) => Some(deprecation_notice(deprecation)),
        (description, None) => description.clone(),
    };
    let mut item = json!({
        "label": symbol.name,
        "kind": completion_kind(&symbol.kind, &symbol.namespace, symbol.callable.is_some()),
        "detail": detail,
        "documentation": completion_documentation(
            symbol.callable.as_ref().map(|callable| callable.signature.as_str())
                .or(symbol.type_name.as_deref()),
            description.as_deref(),
            markup,
        ),
        "sortText": format!("0-{}", symbol.name),
        "data": {"identity": symbol.identity, "namespace": symbol.namespace},
    });
    if symbol.deprecation.is_some() {
        item["tags"] = json!([1]);
    }
    item
}

fn deprecation_notice(deprecation: &seseragi_syntax::Deprecation) -> String {
    let since = deprecation
        .since
        .as_ref()
        .map(|version| format!(" since {version}"))
        .unwrap_or_default();
    format!("Deprecated{since}: {}", deprecation.message)
}

pub(crate) fn document_symbols(
    document: &DocumentState,
    uri: &str,
    encoding: PositionEncoding,
) -> Value {
    let symbols = document
        .analysis
        .workspace_symbols()
        .into_iter()
        .filter_map(|symbol| {
            let range = range_json(
                &document.source,
                symbol.definition.start,
                symbol.definition.end,
                encoding,
            )?;
            let mut item = json!({
                "name": symbol.name,
                "kind": workspace_symbol_kind(symbol),
                "location": {"uri": uri, "range": range},
                "containerName": symbol.module,
            });
            if symbol.deprecation.is_some() {
                item["tags"] = json!([1]);
            }
            Some(item)
        })
        .collect::<Vec<_>>();
    json!(symbols)
}

fn push_reference_completion(
    items: &mut Vec<Value>,
    seen: &mut BTreeSet<String>,
    item: &AnalysisReferenceItem,
    priority: &str,
    markup: MarkupKind,
) {
    let key = format!("{}:{}", item.kind, item.identity);
    if !seen.insert(key) {
        return;
    }
    items.push(json!({
        "label": item.name,
        "kind": completion_kind(&item.kind, &item.kind, item.signature.is_some()),
        "detail": item.signature,
        "documentation": completion_documentation(
            item.signature.as_deref(),
            Some(&item.description),
            markup,
        ),
        "sortText": format!("{priority}-{}", item.name),
        "data": {"identity": item.identity, "module": item.module},
    }));
}

fn completion_documentation(
    compact_type: Option<&str>,
    description: Option<&str>,
    markup: MarkupKind,
) -> Value {
    let value = compact_type
        .map(|type_name| type_inline(type_name, markup))
        .into_iter()
        .chain(
            description
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        )
        .collect::<Vec<_>>()
        .join("\n\n");
    if value.is_empty() {
        Value::Null
    } else {
        markup_content(value, markup)
    }
}

fn markup_content(value: String, markup: MarkupKind) -> Value {
    json!({"kind": markup.lsp_name(), "value": value})
}

fn type_block(value: &str, markup: MarkupKind) -> String {
    match markup {
        MarkupKind::Markdown => format!("```seseragi\n{value}\n```"),
        MarkupKind::PlainText => value.to_owned(),
    }
}

fn type_inline(value: &str, markup: MarkupKind) -> String {
    match markup {
        MarkupKind::Markdown => format!("`{value}`"),
        MarkupKind::PlainText => value.to_owned(),
    }
}

fn completion_kind(kind: &str, namespace: &str, callable: bool) -> u8 {
    if callable {
        return if matches!(kind, "trait-method" | "inherent-method") {
            2
        } else {
            3
        };
    }
    match (namespace, kind) {
        ("module", _) => 9,
        ("type", "type-parameter") => 25,
        ("type", _) | (_, "opaque-type") => 7,
        ("trait", _) | (_, "trait") => 8,
        (_, "constructor") => 4,
        ("field", _) => 5,
        ("operator", _) | (_, "operator") => 24,
        _ => 6,
    }
}

fn semantic_token_type(symbol: &AnalysisSymbol) -> usize {
    match (symbol.namespace.as_str(), symbol.kind.as_str()) {
        ("module", _) => 0,
        ("type", "type-parameter") => 6,
        ("type", _) => 1,
        ("trait", _) => 4,
        ("field", _) => 9,
        ("operator", _) => 21,
        (_, "parameter" | "pattern-binding") => 7,
        (_, "constructor") => 10,
        (_, "trait-method" | "inherent-method") => 13,
        (_, "function" | "effect-function" | "generic-foreign-function" | "prelude") => 12,
        _ if symbol.callable.is_some() => 12,
        _ => 8,
    }
}

fn reference_for<'analysis>(
    analysis: &'analysis AnalysisDocument,
    identity: &str,
) -> Option<&'analysis AnalysisReferenceItem> {
    analysis
        .standard_library_catalog()
        .iter()
        .find(|item| item.identity == identity)
}

fn namespace_prefix(source: &str, position: usize) -> Option<(&str, ByteSpan)> {
    let before = source.get(..position)?.trim_end();
    let prefix = before.strip_suffix('.')?;
    let start = prefix
        .char_indices()
        .rev()
        .find_map(|(offset, scalar)| {
            (!(scalar == '_'
                || scalar == '\''
                || seseragi_syntax::unicode::is_xid_continue(scalar)))
            .then_some(offset + scalar.len_utf8())
        })
        .unwrap_or(0);
    let name = &prefix[start..];
    (!name.is_empty()).then_some((
        name,
        ByteSpan {
            start,
            end: prefix.len(),
        },
    ))
}

fn imported_namespace_module(source: &str, prefix: &str) -> Option<String> {
    parse_surface_ast("<completion>", source)
        .imports
        .into_iter()
        .find_map(|import| {
            import
                .items
                .iter()
                .any(|item| item.namespace == "namespace" && item.alias.as_deref() == Some(prefix))
                .then_some(import.specifier)
        })
}

fn hover_range(analysis: &AnalysisDocument, position: usize) -> Option<ByteSpan> {
    analysis
        .symbol_occurrences
        .iter()
        .filter(|occurrence| contains(occurrence.range, position))
        .map(|occurrence| occurrence.range)
        .chain(
            analysis
                .type_occurrences
                .iter()
                .filter(|occurrence| contains(occurrence.range, position))
                .map(|occurrence| occurrence.range),
        )
        .chain(
            analysis
                .callable_occurrences
                .iter()
                .filter(|occurrence| contains(occurrence.range, position))
                .map(|occurrence| occurrence.range),
        )
        .min_by_key(|range| range.end.saturating_sub(range.start))
}

fn spelling_range(source: &str, range: ByteSpan, spelling: &str) -> Option<(usize, usize)> {
    let text = source.get(range.start..range.end)?;
    let relative = text.rfind(spelling)?;
    Some((
        range.start + relative,
        range.start + relative + spelling.len(),
    ))
}

fn contains(range: ByteSpan, position: usize) -> bool {
    range.start <= position && position < range.end
}

fn intersects(start: usize, end: usize, request_start: usize, request_end: usize) -> bool {
    if request_start == request_end {
        return start <= request_start && request_start <= end;
    }
    start < request_end && request_start < end
}

pub(crate) fn range_json(
    source: &str,
    start: usize,
    end: usize,
    encoding: PositionEncoding,
) -> Option<Value> {
    let range = encoded_range(source, start, end, encoding)?;
    Some(json!({
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character},
    }))
}

#[derive(Clone, Copy)]
struct EncodedRange {
    start: EncodedPosition,
    end: EncodedPosition,
}

fn encoded_range(
    source: &str,
    start: usize,
    end: usize,
    encoding: PositionEncoding,
) -> Option<EncodedRange> {
    let index = LineIndex::new(source);
    Some(EncodedRange {
        start: index.try_locate_encoded(start, encoding).ok()?,
        end: index.try_locate_encoded(end, encoding).ok()?,
    })
}

pub(crate) fn range_formatting(
    document: &DocumentState,
    range: crate::model::Range,
    encoding: PositionEncoding,
) -> Value {
    let (Some(start), Some(end)) = (
        document.byte_position(range.start, encoding),
        document.byte_position(range.end, encoding),
    ) else {
        return json!([]);
    };
    let edits = seseragi_driver::format_module_range(
        &document.analysis.source,
        &document.source,
        start..end,
        Default::default(),
    );
    Value::Array(
        edits
            .into_iter()
            .filter_map(|edit| {
                range_json(&document.source, edit.range.start, edit.range.end, encoding)
                    .map(|range| json!({"range": range, "newText": edit.text}))
            })
            .collect(),
    )
}

/// Highlights every occurrence, in this document, of the name under the
/// cursor: its definition as a write, its uses as reads, and the name in an
/// import list as text.
pub(crate) fn document_highlights(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Value {
    const TEXT: u8 = 1;
    const READ: u8 = 2;
    const WRITE: u8 = 3;
    let Some(symbol) = symbol_at_position(document, params, encoding) else {
        return json!([]);
    };
    let key = symbol_key(symbol);
    let local_prefix = format!("{}::", document.analysis.module);
    let mut highlights = BTreeMap::new();
    for occurrence in &document.analysis.symbol_occurrences {
        let Some(candidate) = document.analysis.symbols.get(occurrence.symbol as usize) else {
            continue;
        };
        if symbol_key(candidate) != key {
            continue;
        }
        let range = precise_symbol_range(document, occurrence.range, candidate);
        let kind = if candidate.identity.starts_with(&local_prefix)
            && candidate.definition == occurrence.range
        {
            WRITE
        } else {
            READ
        };
        highlights.entry((range.start, range.end)).or_insert(kind);
    }
    for range in imported_name_ranges(document, &key) {
        highlights.entry((range.start, range.end)).or_insert(TEXT);
    }
    Value::Array(
        highlights
            .into_iter()
            .filter_map(|((start, end), kind)| {
                let range = range_json(&document.source, start, end, encoding)?;
                Some(json!({"range": range, "kind": kind}))
            })
            .collect(),
    )
}

/// A nominal type in a value's type. `canonical` is absent for a type
/// declared in the same module, which is then found by `name`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NominalType {
    pub(crate) canonical: Option<String>,
    pub(crate) name: String,
}

/// The nominal types in the type of the value under the cursor, outermost
/// first, for `textDocument/typeDefinition`.
pub(crate) fn nominal_types_at(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Vec<NominalType> {
    let Some(position) = document.query_position(params.position, encoding) else {
        return Vec::new();
    };
    let symbol = document.analysis.symbol_at(position);
    if symbol.is_some_and(|symbol| symbol.namespace == "type") {
        // A type name is its own definition; typeDefinition adds nothing.
        return Vec::new();
    }
    let type_document = symbol
        .and_then(|symbol| symbol.type_document.as_ref())
        .or_else(|| {
            document
                .analysis
                .type_at(position)
                .map(|occurrence| &occurrence.type_document)
        });
    let mut identities = Vec::new();
    if let Some(type_document) = type_document {
        collect_nominal_types(type_document, &mut identities);
    }
    identities
}

fn collect_nominal_types(document: &TypeDocument, identities: &mut Vec<NominalType>) {
    use TypeDocument::*;
    match document {
        Named {
            name,
            canonical,
            arguments,
        } => {
            let nominal = NominalType {
                canonical: canonical.clone(),
                name: name.clone(),
            };
            if !identities.contains(&nominal) {
                identities.push(nominal);
            }
            for argument in arguments {
                collect_nominal_types(argument, identities);
            }
        }
        TypeFunction { body, .. } => collect_nominal_types(body, identities),
        Variable { arguments, .. } => {
            for argument in arguments {
                collect_nominal_types(argument, identities);
            }
        }
        Function { parameters, result } => {
            collect_nominal_types(result, identities);
            for parameter in parameters {
                collect_nominal_types(parameter, identities);
            }
        }
        Tuple { elements: operands } | RequirementMerge { operands } => {
            for operand in operands {
                collect_nominal_types(operand, identities);
            }
        }
        Record { fields, .. } => {
            for field in fields {
                collect_nominal_types(&field.type_ref, identities);
            }
        }
        TypeFunctionParameter { .. } | TypeConstructor { .. } | Unknown => {}
    }
}

/// Which inlay hint kinds the editor asked for. Both are on by default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InlayHintSettings {
    pub(crate) binding_types: bool,
    pub(crate) effect_rows: bool,
}

impl Default for InlayHintSettings {
    fn default() -> Self {
        Self {
            binding_types: true,
            effect_rows: true,
        }
    }
}

impl InlayHintSettings {
    /// Reads `{ "bindingTypes": bool, "effectRows": bool }`; a missing key
    /// keeps its default.
    pub(crate) fn from_json(value: &Value) -> Option<Self> {
        let object = value.as_object()?;
        let flag = |name: &str| object.get(name).and_then(Value::as_bool).unwrap_or(true);
        Some(Self {
            binding_types: flag("bindingTypes"),
            effect_rows: flag("effectRows"),
        })
    }
}

pub(crate) fn inlay_hints(
    document: &DocumentState,
    start: usize,
    end: usize,
    settings: InlayHintSettings,
    encoding: PositionEncoding,
) -> Value {
    let mut hints = Vec::new();
    if settings.binding_types {
        binding_type_hints(document, start, end, &mut hints);
    }
    if settings.effect_rows {
        effect_row_hints(document, start, end, &mut hints);
    }
    hints.sort_by_key(|hint| hint.0);
    Value::Array(
        hints
            .into_iter()
            .filter_map(|(offset, label, padding_left)| {
                let position = encoded_range(&document.source, offset, offset, encoding)?.start;
                Some(json!({
                    "position": {"line": position.line, "character": position.character},
                    "label": label,
                    "kind": 1,
                    "paddingLeft": padding_left,
                }))
            })
            .collect(),
    )
}

/// `: Type` after a `let`, lambda parameter, `do` bind or pattern binding
/// that has no annotation. A binding whose type was not inferred gets none.
fn binding_type_hints(
    document: &DocumentState,
    start: usize,
    end: usize,
    hints: &mut Vec<(usize, String, bool)>,
) {
    let source = &document.source;
    for symbol in &document.analysis.symbols {
        if !matches!(
            symbol.kind.as_str(),
            "let" | "parameter" | "pattern-binding"
        ) || symbol.name.starts_with('_')
        {
            continue;
        }
        let definition = symbol.definition;
        if definition.end < start
            || definition.end > end
            || source.get(definition.start..definition.end) != Some(symbol.name.as_str())
        {
            continue;
        }
        let (Some(type_name), Some(type_document)) = (&symbol.type_name, &symbol.type_document)
        else {
            continue;
        };
        if contains_unknown(type_document) {
            continue;
        }
        let annotated = source[definition.end..]
            .trim_start_matches([' ', '\t'])
            .starts_with(':');
        if !annotated {
            hints.push((definition.end, format!(": {type_name}"), false));
        }
    }
}

/// ` with … fails …` after the header of an `effect fn` that leaves its
/// requirements or failure type to inference.
fn effect_row_hints(
    document: &DocumentState,
    start: usize,
    end: usize,
    hints: &mut Vec<(usize, String, bool)>,
) {
    let source = &document.source;
    let tokens = lex("", source).tokens;
    for declaration in parse_surface_ast(&document.analysis.module, source).declarations {
        let seseragi_syntax::SurfaceDecl::EffectFn {
            name_span,
            requirements,
            failure,
            ..
        } = declaration
        else {
            continue;
        };
        if !requirements.is_empty() && failure.is_some() {
            continue;
        }
        let Some(equals) = tokens
            .iter()
            .find(|token| token.start >= name_span.end && token.kind == TokenKind::OperatorEquals)
        else {
            continue;
        };
        let header_end = source[..equals.start].trim_end().len();
        if header_end < start || header_end > end {
            continue;
        }
        let Some(symbol) = document
            .analysis
            .symbols
            .iter()
            .find(|symbol| symbol.definition == name_span)
        else {
            continue;
        };
        let Some((row, error)) = symbol.type_document.as_ref().and_then(effect_row) else {
            continue;
        };
        let mut label = String::new();
        if requirements.is_empty() {
            if let TypeDocument::Record { fields, .. } = row {
                if !fields.is_empty() {
                    let items = fields
                        .iter()
                        .map(|field| {
                            let rendered = field.type_ref.render(Default::default());
                            let shorthand = matches!(&field.type_ref, TypeDocument::Named { name, arguments, .. }
                                if arguments.is_empty()
                                    && seseragi_syntax::unicode::lowercase_first(name) == field.name);
                            if shorthand {
                                rendered
                            } else {
                                format!("{}: {rendered}", field.name)
                            }
                        })
                        .collect::<Vec<_>>();
                    label.push_str(&format!("with {}", items.join(", ")));
                }
            }
        }
        if failure.is_none()
            && !matches!(error, TypeDocument::Named { name, .. } if name == "Never")
        {
            if !label.is_empty() {
                label.push(' ');
            }
            label.push_str(&format!("fails {}", error.render(Default::default())));
        }
        if !label.is_empty() {
            hints.push((header_end, label, true));
        }
    }
}

/// The requirement row and failure type of `Effect<R, E, A>`, after any
/// parameters.
fn effect_row(document: &TypeDocument) -> Option<(&TypeDocument, &TypeDocument)> {
    match document {
        TypeDocument::Function { result, .. } => effect_row(result),
        TypeDocument::TypeFunction { body, .. } => effect_row(body),
        TypeDocument::Named {
            name, arguments, ..
        } if name == "Effect" && arguments.len() == 3 => (!contains_unknown(&arguments[0])
            && !contains_unknown(&arguments[1]))
        .then(|| (&arguments[0], &arguments[1])),
        _ => None,
    }
}

fn contains_unknown(document: &TypeDocument) -> bool {
    use TypeDocument::*;
    match document {
        Unknown => true,
        Named { arguments, .. } | Variable { arguments, .. } => {
            arguments.iter().any(contains_unknown)
        }
        TypeFunction { body, .. } => contains_unknown(body),
        Function { parameters, result } => {
            contains_unknown(result) || parameters.iter().any(contains_unknown)
        }
        Tuple { elements: operands } | RequirementMerge { operands } => {
            operands.iter().any(contains_unknown)
        }
        Record { fields, .. } => fields.iter().any(|field| contains_unknown(&field.type_ref)),
        TypeFunctionParameter { .. } | TypeConstructor { .. } => false,
    }
}

/// What `textDocument/implementation` looks for.
pub(crate) enum ImplementationTarget {
    /// Every instance of this trait.
    Trait(SymbolKey),
    /// The instance methods for a trait method. `argument` is the head name
    /// of the first argument's type at a call, used to pick its instance.
    Method {
        method: SymbolKey,
        name: String,
        argument: Option<String>,
    },
}

pub(crate) fn implementation_target(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Option<ImplementationTarget> {
    let symbol = symbol_at_position(document, params, encoding)?;
    match symbol.kind.as_str() {
        "trait" => Some(ImplementationTarget::Trait(symbol_key(symbol))),
        "trait-method" => {
            let byte = document.byte_position(params.position, encoding)?;
            let occurrence = document
                .analysis
                .symbol_occurrences
                .iter()
                .filter(|occurrence| {
                    occurrence.range.start <= byte + 1 && byte <= occurrence.range.end
                })
                .min_by_key(|occurrence| occurrence.range.end - occurrence.range.start)?;
            Some(ImplementationTarget::Method {
                method: symbol_key(symbol),
                name: symbol.name.clone(),
                argument: call_argument_type(document, occurrence.range.end),
            })
        }
        _ => None,
    }
}

/// The head name of the type of the argument that follows a callee ending
/// at `callee_end`, as in `describe (Team { size: 2 })`.
fn call_argument_type(document: &DocumentState, callee_end: usize) -> Option<String> {
    let rest = document.source.get(callee_end..)?;
    let trimmed = rest.trim_start_matches([' ', '\t']);
    let mut start = callee_end + (rest.len() - trimmed.len());
    if trimmed.starts_with('(') {
        let inner = &trimmed[1..];
        start += 1 + (inner.len() - inner.trim_start().len());
    }
    let occurrence = document
        .analysis
        .type_occurrences
        .iter()
        .filter(|occurrence| occurrence.range.start == start)
        .max_by_key(|occurrence| occurrence.range.end)?;
    match &occurrence.type_document {
        TypeDocument::Named { name, .. } => Some(name.clone()),
        _ => None,
    }
}

/// The trait whose declaration in `document` contains the method defined at
/// `method_start`.
pub(crate) fn trait_declaring(document: &DocumentState, method_start: usize) -> Option<SymbolKey> {
    parse_surface_ast(&document.analysis.module, &document.source)
        .declarations
        .into_iter()
        .find_map(|declaration| match declaration {
            seseragi_syntax::SurfaceDecl::Trait {
                name_span, span, ..
            } if span.start <= method_start && method_start < span.end => {
                trait_key_at(document, name_span)
            }
            _ => None,
        })
}

fn trait_key_at(document: &DocumentState, name_span: ByteSpan) -> Option<SymbolKey> {
    document
        .analysis
        .symbol_occurrences
        .iter()
        .find(|occurrence| occurrence.range == name_span)
        .and_then(|occurrence| document.analysis.symbols.get(occurrence.symbol as usize))
        .filter(|symbol| symbol.kind == "trait")
        .map(symbol_key)
}

/// Instances of `trait_key` in this document, or their `method` when one is
/// named, with whether the instance's first type argument has the head
/// name `argument`.
pub(crate) fn implementations_in(
    document: &DocumentState,
    trait_key: &SymbolKey,
    method: Option<&str>,
    argument: Option<&str>,
) -> Vec<(ByteSpan, bool)> {
    let mut found = Vec::new();
    for declaration in parse_surface_ast(&document.analysis.module, &document.source).declarations {
        let seseragi_syntax::SurfaceDecl::Instance {
            trait_name_span,
            arguments,
            methods,
            span,
            ..
        } = declaration
        else {
            continue;
        };
        if trait_key_at(document, trait_name_span).as_ref() != Some(trait_key) {
            continue;
        }
        let matches = match (argument, arguments.first()) {
            (Some(argument), Some(seseragi_syntax::TypeRef::Named { name, .. })) => {
                name.rsplit('.').next() == Some(argument)
            }
            _ => false,
        };
        match method {
            None => found.push((span, matches)),
            Some(method) => {
                if let Some(definition) = methods.iter().find(|candidate| candidate.name == method)
                {
                    found.push((definition.name_span, matches));
                }
            }
        }
    }
    found
}

/// The definition of a type declared in this document under `name`.
pub(crate) fn local_type_definition(document: &DocumentState, name: &str) -> Option<ByteSpan> {
    let prefix = format!("{}::", document.analysis.module);
    document
        .analysis
        .symbols
        .iter()
        .find(|symbol| {
            symbol.namespace == "type"
                && symbol.kind == "type"
                && symbol.name == name
                && symbol.identity.starts_with(&prefix)
                && symbol.definition.end > symbol.definition.start
        })
        .map(|symbol| symbol.definition)
}
