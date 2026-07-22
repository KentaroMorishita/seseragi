use crate::model::{CodeActionParams, Position, TextDocumentPositionParams};
use serde_json::{json, Value};
use seseragi_driver::{
    analyze_module, AnalysisDocument, AnalysisReferenceItem, AnalysisSymbol, CompileInput,
};
use seseragi_source::{EncodedPosition, LineIndex, PositionEncoding};
use seseragi_syntax::ByteSpan;
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
    pub(crate) version: i64,
    pub(crate) source: String,
    pub(crate) analysis: AnalysisDocument,
}

impl DocumentState {
    pub(crate) fn analyze(uri: &str, version: i64, source: String) -> Self {
        let analysis = analyze_module(CompileInput::new(uri, uri, &source));
        Self {
            version,
            source,
            analysis,
        }
    }

    fn byte_position(&self, position: Position, encoding: PositionEncoding) -> Option<usize> {
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

    fn query_position(&self, position: Position, encoding: PositionEncoding) -> Option<usize> {
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

pub(crate) fn hover(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
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
    let signature = callable
        .map(|callable| callable.signature.clone())
        .or_else(|| reference.and_then(|item| item.signature.clone()))
        .or_else(|| {
            symbol.and_then(|symbol| {
                symbol
                    .type_name
                    .as_ref()
                    .map(|type_name| format!("{}: {type_name}", symbol.name))
            })
        });
    if let Some(signature) = &signature {
        lines.push(format!("```seseragi\n{signature}\n```"));
    }
    if let Some(symbol) = symbol {
        lines.push(format!("`{}` · `{}`", symbol.module, symbol.identity));
    }
    if let Some(type_occurrence) = inferred {
        let already_in_signature = signature
            .as_ref()
            .is_some_and(|value| value.contains(&type_occurrence.type_name));
        if !already_in_signature {
            lines.push(format!("Inferred type: `{}`", type_occurrence.type_name));
        }
    }
    if let Some(callable) = callable {
        if !callable.constraints.is_empty() {
            lines.push(format!(
                "Constraints: `{}`",
                callable.constraints.join(", ")
            ));
        }
        if !callable.remaining_parameters.is_empty() {
            lines.push(format!(
                "Remaining: {}",
                callable
                    .remaining_parameters
                    .iter()
                    .map(|parameter| match &parameter.name {
                        Some(name) => format!("`{name}: {}`", parameter.type_name),
                        None => format!("`{}`", parameter.type_name),
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

    let range = hover_range(&document.analysis, position)
        .and_then(|range| range_json(&document.source, range.start, range.end, encoding));
    json!({
        "contents": {"kind": "markdown", "value": lines.join("\n\n")},
        "range": range,
    })
}

pub(crate) fn completion(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
) -> Value {
    let Some(position) = document.byte_position(params.position, encoding) else {
        return json!([]);
    };
    let namespace_module = namespace_prefix(&document.source, position).and_then(|prefix| {
        document
            .analysis
            .visible_symbols(position)
            .into_iter()
            .find(|symbol| symbol.namespace == "module" && symbol.name == prefix)
            .map(|symbol| symbol.module.clone())
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
            push_reference_completion(&mut items, &mut seen, item, "0");
        }
    } else {
        for symbol in document.analysis.visible_symbols(position) {
            let key = format!("{}:{}", symbol.namespace, symbol.identity);
            if !seen.insert(key) {
                continue;
            }
            items.push(symbol_completion(symbol));
        }
        for item in document.analysis.standard_library_catalog() {
            push_reference_completion(&mut items, &mut seen, item, "1");
        }
    }
    Value::Array(items)
}

pub(crate) fn signature_help(
    document: &DocumentState,
    params: &TextDocumentPositionParams,
    encoding: PositionEncoding,
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
            "documentation": {"kind": "markdown", "value": documentation},
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
        tokens.insert(
            (range.start.line, range.start.character, range.end.character),
            semantic_token_type(symbol),
        );
    }

    let mut data = Vec::new();
    let mut previous_line = 0;
    let mut previous_character = 0;
    let mut previous_end = None;
    for ((line, character, end), token_type) in tokens {
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
        data.extend([delta_line, delta_start, end - character, token_type, 0]);
        previous_line = line;
        previous_character = character;
        previous_end = Some((line, end));
    }
    json!({"data": data})
}

fn symbol_completion(symbol: &AnalysisSymbol) -> Value {
    let detail = symbol
        .callable
        .as_ref()
        .map(|callable| callable.signature.clone())
        .or_else(|| symbol.type_name.clone())
        .unwrap_or_else(|| symbol.kind.clone());
    json!({
        "label": symbol.name,
        "kind": completion_kind(&symbol.kind, &symbol.namespace, symbol.callable.is_some()),
        "detail": detail,
        "documentation": symbol.description.as_ref().map(|description| {
            json!({"kind": "markdown", "value": description})
        }),
        "sortText": format!("0-{}", symbol.name),
        "data": {"identity": symbol.identity, "namespace": symbol.namespace},
    })
}

fn push_reference_completion(
    items: &mut Vec<Value>,
    seen: &mut BTreeSet<String>,
    item: &AnalysisReferenceItem,
    priority: &str,
) {
    let key = format!("{}:{}", item.kind, item.identity);
    if !seen.insert(key) {
        return;
    }
    items.push(json!({
        "label": item.name,
        "kind": completion_kind(&item.kind, &item.kind, item.signature.is_some()),
        "detail": item.signature,
        "documentation": {"kind": "markdown", "value": item.description},
        "sortText": format!("{priority}-{}", item.name),
        "data": {"identity": item.identity, "module": item.module},
    }));
}

fn completion_kind(kind: &str, namespace: &str, callable: bool) -> u8 {
    if callable {
        return if kind == "trait-method" { 2 } else { 3 };
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
        (_, "trait-method") => 13,
        (_, "function" | "effect-function" | "prelude") => 12,
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

fn namespace_prefix(source: &str, position: usize) -> Option<&str> {
    let before = source.get(..position)?.trim_end();
    let prefix = before.strip_suffix('.')?;
    let start = prefix
        .char_indices()
        .rev()
        .find_map(|(offset, scalar)| {
            (!(scalar == '_' || scalar == '\'' || scalar.is_alphanumeric()))
                .then_some(offset + scalar.len_utf8())
        })
        .unwrap_or(0);
    let name = &prefix[start..];
    (!name.is_empty()).then_some(name)
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

fn range_json(source: &str, start: usize, end: usize, encoding: PositionEncoding) -> Option<Value> {
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
