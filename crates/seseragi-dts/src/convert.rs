use crate::config::{
    BindingsConfig, CallbackInvocation, CallbackLifetime, EntryConfig, Evaluation, SymbolConfig,
};
use crate::semantic::{interpret_with_digest, SemanticError};
use serde::{Deserialize, Serialize};
use seseragi_interop::{
    PrimitiveDomain, SemanticGraph, SemanticSymbol, SemanticType, SymbolKind, TypeId,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path};

#[cfg(feature = "filesystem")]
pub(crate) mod filesystem;

const METADATA_SCHEMA: u32 = 1;
const REPORT_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConversionDiagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub file: String,
    pub start: usize,
    pub end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

#[derive(Debug)]
pub struct ConvertError(String);

impl ConvertError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConvertError {}

/// All host metadata needed to resolve an entry, supplied by its adapter.
/// `resolved_package` is the package.json text for a dependency when the host
/// manifest is not itself the imported package. Relative imports need neither.
#[derive(Clone, Copy, Debug)]
pub struct HostMetadata<'a> {
    pub manifest: &'a str,
    pub resolved_package: Option<&'a str>,
}

/// Convert one configured entry without reading or writing files.
/// Paths in the settings remain logical package-relative paths in diagnostics
/// and metadata. Previous metadata is optional; invalid JSON is treated as a
/// first conversion, matching the filesystem adapter.
#[derive(Clone, Copy, Debug)]
pub struct ConversionInput<'a> {
    pub settings: &'a str,
    pub entry: &'a str,
    pub declaration: &'a str,
    pub host: HostMetadata<'a>,
    pub previous_metadata: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConversionResult {
    pub generated: Option<GeneratedBinding>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub semantic: Option<SemanticGraph>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectionResult {
    pub source: Option<String>,
    pub diagnostics: Vec<ConversionDiagnostic>,
}

/// Project a validated common graph without parsing TypeScript declarations.
/// This bridge retains the existing opaque declaration policy; canonical
/// structural/nominal/ADT reconstruction is a separate projection contract.
pub fn project_graph(
    graph: &SemanticGraph,
    settings: &str,
    id: &str,
) -> Result<ProjectionResult, ConvertError> {
    graph
        .validate()
        .map_err(|error| ConvertError::new(error.to_string()))?;
    let config = parse_bindings(settings)?;
    let entry = config.entries.get(id).ok_or_else(|| {
        ConvertError::new(format!(
            "binding entry `{id}` does not exist in binding settings"
        ))
    })?;
    if graph
        .identity
        .as_ref()
        .is_some_and(|identity| identity.settings_digest != sha256(settings))
    {
        return Err(ConvertError::new(
            "semantic graph was interpreted with different binding settings",
        ));
    }
    let rendered = render_entry(id, entry, graph, &entry.declaration);
    let failed = rendered
        .diagnostics
        .iter()
        .any(|d| d.severity == DiagnosticSeverity::Error);
    Ok(ProjectionResult {
        source: (!failed).then_some(rendered.source),
        diagnostics: rendered.diagnostics,
    })
}

/// Text artifacts ready to publish under `output` in a generated workspace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GeneratedBinding {
    pub id: String,
    pub output: String,
    pub source: String,
    pub metadata: String,
    pub report: String,
}

/// Parse and validate settings with the same rules as native conversion.
pub fn parse_bindings(source: &str) -> Result<BindingsConfig, ConvertError> {
    let config = parse_config(source)?;
    validate_config(&config)?;
    Ok(config)
}

pub fn convert_entry(input: &ConversionInput<'_>) -> Result<ConversionResult, ConvertError> {
    let config = parse_config(input.settings)?;
    validate_config(&config)?;
    let entry = config.entries.get(input.entry).ok_or_else(|| {
        ConvertError::new(format!(
            "binding entry `{}` does not exist in binding settings",
            input.entry
        ))
    })?;
    convert_entry_with_host(
        input.entry,
        entry,
        input.declaration,
        &sha256(input.settings),
        input.previous_metadata,
        || resolve_host_metadata(&entry.specifier, input.host),
    )
}

fn convert_entry_with_host(
    id: &str,
    entry: &EntryConfig,
    declaration: &str,
    settings_digest: &str,
    previous: Option<&str>,
    host: impl FnOnce() -> Result<HostModuleIdentity, ConvertError>,
) -> Result<ConversionResult, ConvertError> {
    let graph = match interpret_with_digest(declaration, &entry.declaration, settings_digest) {
        Ok(graph) => graph,
        Err(SemanticError::Parse {
            message,
            start,
            end,
        }) => {
            return Ok(ConversionResult {
                generated: None,
                semantic: None,
                diagnostics: vec![diagnostic(
                    "SES-F0100",
                    DiagnosticSeverity::Error,
                    message,
                    &entry.declaration,
                    Span { start, end },
                    None,
                )],
            });
        }
        Err(error) => return Err(ConvertError::new(error.to_string())),
    };
    let rendered = render_entry(id, entry, &graph, &entry.declaration);
    if rendered
        .diagnostics
        .iter()
        .any(|d| d.severity == DiagnosticSeverity::Error)
    {
        return Ok(ConversionResult {
            generated: None,
            diagnostics: rendered.diagnostics,
            semantic: Some(graph),
        });
    }
    let previous = previous.and_then(|text| serde_json::from_str(text).ok());
    let metadata = BindingMetadata {
        schema: METADATA_SCHEMA,
        kind: "seseragi-typescript-binding".to_owned(),
        generator: GeneratorIdentity {
            name: "seseragi-dts".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        entry: id.to_owned(),
        declaration: entry.declaration.clone(),
        output: entry.output.clone(),
        specifier: entry.specifier.clone(),
        host_module: host()?,
        evaluation: evaluation_name(entry.evaluation).to_owned(),
        input_digest: sha256(declaration),
        settings_digest: settings_digest.to_owned(),
        symbols: rendered.symbols,
    };
    let report = build_report(id, previous.as_ref(), &metadata, &rendered.diagnostics);
    Ok(ConversionResult {
        generated: Some(GeneratedBinding {
            id: id.to_owned(),
            output: entry.output.clone(),
            source: rendered.source,
            metadata: json_text(&metadata)?,
            report: json_text(&report)?,
        }),
        diagnostics: rendered.diagnostics,
        semantic: Some(graph),
    })
}

fn resolve_host_metadata(
    specifier: &str,
    host: HostMetadata<'_>,
) -> Result<HostModuleIdentity, ConvertError> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return Ok(HostModuleIdentity {
            specifier: specifier.to_owned(),
            exact_identity: format!("workspace:{specifier}"),
        });
    }
    let package_name = if specifier.starts_with('@') {
        specifier.split('/').take(2).collect::<Vec<_>>().join("/")
    } else {
        specifier.split('/').next().unwrap_or(specifier).to_owned()
    };
    let subpath = specifier
        .strip_prefix(&package_name)
        .unwrap_or_default()
        .trim_start_matches('/');
    let root: serde_json::Value = serde_json::from_str(host.manifest)
        .map_err(|error| ConvertError::new(format!("invalid foreign host manifest: {error}")))?;
    let manifest =
        if root.get("name").and_then(|value| value.as_str()) == Some(package_name.as_str()) {
            root
        } else {
            let source = host.resolved_package.ok_or_else(|| {
                ConvertError::new(format!(
                    "missing resolved foreign package manifest for `{package_name}`"
                ))
            })?;
            serde_json::from_str(source).map_err(|error| {
                ConvertError::new(format!(
                    "invalid resolved foreign package manifest: {error}"
                ))
            })?
        };
    host_identity(specifier, &package_name, subpath, &manifest)
}

fn host_identity(
    specifier: &str,
    package_name: &str,
    subpath: &str,
    manifest: &serde_json::Value,
) -> Result<HostModuleIdentity, ConvertError> {
    let resolved_name = manifest
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| ConvertError::new("resolved foreign package manifest has no string name"))?;
    let version = manifest
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            ConvertError::new("resolved foreign package manifest has no string version")
        })?;
    if resolved_name != package_name {
        return Err(ConvertError::new(format!(
            "foreign specifier `{specifier}` resolved package `{resolved_name}` instead of `{package_name}`"
        )));
    }
    let exact_identity = if subpath.is_empty() {
        format!("{resolved_name}@{version}")
    } else {
        format!("{resolved_name}@{version}/{subpath}")
    };
    Ok(HostModuleIdentity {
        specifier: specifier.to_owned(),
        exact_identity,
    })
}

#[derive(Clone, Debug)]
struct RenderedEntry {
    source: String,
    symbols: Vec<GeneratedSymbol>,
    diagnostics: Vec<ConversionDiagnostic>,
}

fn render_entry(
    _id: &str,
    entry: &EntryConfig,
    graph: &SemanticGraph,
    file: &str,
) -> RenderedEntry {
    let mut context = RenderContext {
        entry,
        file,
        graph,
        diagnostics: Vec::new(),
        symbols: Vec::new(),
        uses_big_int: false,
        active_types: BTreeSet::new(),
        included: projection_subset(graph, entry),
    };
    validate_symbol_settings(graph, &mut context);
    let owned = graph
        .types
        .values()
        .filter_map(|node| match node {
            SemanticType::Namespace { members } => Some(members.values()),
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>();
    let roots = graph
        .symbols
        .iter()
        .filter(|(key, _)| !owned.contains(key) && !graph.private_symbols.contains(*key))
        .map(|(key, symbol)| (symbol.public_name.clone(), key.clone()))
        .collect();
    let body = render_scope(&roots, &mut context, &[], 1);
    let mut source = String::new();
    if context.uses_big_int {
        source.push_str("import { BigInt } from \"std/big-int\"\n\n");
    }
    source.push_str(&format!(
        "pub foreign \"typescript\" from {:?} {{\n",
        entry.specifier
    ));
    source.push_str(&body);
    source.push_str("}\n");
    RenderedEntry {
        source,
        symbols: context.symbols,
        diagnostics: context.diagnostics,
    }
}

struct RenderContext<'a> {
    entry: &'a EntryConfig,
    file: &'a str,
    graph: &'a SemanticGraph,
    diagnostics: Vec<ConversionDiagnostic>,
    symbols: Vec<GeneratedSymbol>,
    uses_big_int: bool,
    active_types: BTreeSet<TypeId>,
    included: Option<BTreeSet<String>>,
}

fn projection_subset(graph: &SemanticGraph, entry: &EntryConfig) -> Option<BTreeSet<String>> {
    let include = entry.include.as_ref()?;
    let mut selected = include.iter().cloned().collect::<BTreeSet<_>>();
    // Selecting a namespace explicitly includes its exported descendants.
    for key in graph.symbols.keys() {
        if include
            .iter()
            .any(|parent| key.starts_with(&format!("{parent}.")))
        {
            selected.insert(key.clone());
        }
    }
    let mut pending = selected
        .iter()
        .filter_map(|key| graph.symbols.get(key).map(|symbol| symbol.type_ref))
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(SemanticType::Reference { symbol }) = graph.types.get(&id) {
            if !graph.private_symbols.contains(symbol)
                && graph.symbols[symbol].kind == SymbolKind::Type
            {
                selected.insert(symbol.clone());
            }
        }
        pending.extend(graph.related_types(id));
    }
    Some(selected)
}

fn binding_type_name(context: &RenderContext<'_>, target: &str) -> String {
    let symbol = &context.graph.symbols[target];
    let mut parts = Vec::new();
    let segments = target.split('.').collect::<Vec<_>>();
    for count in 1..segments.len() {
        let prefix = segments[..count].join(".");
        if let Some(owner) = context.graph.symbols.get(&prefix) {
            let parent = &segments[..count - 1];
            parts.push(generated_name(
                &owner.public_name,
                NameKind::Value,
                &context.entry.specifier,
                &parent
                    .iter()
                    .copied()
                    .chain(std::iter::once(owner.declaration_name.as_str()))
                    .collect::<Vec<_>>()
                    .join("."),
                &prefix,
            ));
        }
    }
    let original = segments[..segments.len() - 1]
        .iter()
        .copied()
        .chain(std::iter::once(symbol.declaration_name.as_str()))
        .collect::<Vec<_>>()
        .join(".");
    parts.push(generated_name(
        &symbol.public_name,
        NameKind::Type,
        &context.entry.specifier,
        &original,
        target,
    ));
    parts.join(".")
}

#[derive(Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

fn symbol_span(symbol: &SemanticSymbol, rule: &str) -> Span {
    let origin = symbol
        .origins
        .iter()
        .find(|origin| origin.rule == rule)
        .or_else(|| symbol.origins.first());
    origin.map_or(Span { start: 0, end: 0 }, |origin| Span {
        start: origin.start,
        end: origin.end,
    })
}
fn type_span(graph: &SemanticGraph, id: TypeId) -> Span {
    graph
        .type_origins
        .get(&id)
        .and_then(|origins| origins.first())
        .map_or(Span { start: 0, end: 0 }, |origin| Span {
            start: origin.start,
            end: origin.end,
        })
}

struct Function {
    original_name: String,
    public_name: String,
    name_span: Span,
    span: Span,
    type_parameters: Vec<String>,
    parameters: Vec<seseragi_interop::CallParameter>,
    result: TypeId,
    generic_binder: Option<TypeId>,
    receiver: Option<TypeId>,
}
fn function_view(graph: &SemanticGraph, symbol: &SemanticSymbol, id: TypeId) -> Option<Function> {
    let (type_parameters, body) = match &graph.types[&id] {
        SemanticType::Generic { parameters, body } => {
            (parameters.iter().map(|p| p.name.clone()).collect(), *body)
        }
        _ => (Vec::new(), id),
    };
    let SemanticType::Callable {
        parameters,
        result,
        receiver,
    } = &graph.types[&body]
    else {
        return None;
    };
    let span = type_span(graph, body);
    // Each overload's callable origin owns its exact declaration range.
    let name_span = symbol
        .origins
        .iter()
        .filter(|origin| origin.rule == "public-name")
        .find(|origin| origin.start >= span.start && origin.end <= span.end)
        .map_or(symbol_span(symbol, "public-name"), |origin| Span {
            start: origin.start,
            end: origin.end,
        });
    Some(Function {
        original_name: symbol.declaration_name.clone(),
        public_name: symbol.public_name.clone(),
        name_span,
        span,
        type_parameters,
        parameters: parameters.clone(),
        result: *result,
        generic_binder: matches!(graph.types[&id], SemanticType::Generic { .. }).then_some(id),
        receiver: *receiver,
    })
}

fn render_scope(
    members: &BTreeMap<String, String>,
    context: &mut RenderContext<'_>,
    namespace: &[String],
    indent: usize,
) -> String {
    let pad = "  ".repeat(indent);
    let mut declarations = members
        .values()
        .filter(|key| {
            context.included.as_ref().is_none_or(|included| {
                included.contains(*key)
                    || included
                        .iter()
                        .any(|selected| selected.starts_with(&format!("{key}.")))
            })
        })
        .map(|key| context.graph.symbols[key].clone())
        .collect::<Vec<_>>();
    declarations.sort_by_key(|symbol| {
        (
            match symbol.kind {
                SymbolKind::Type => 0,
                SymbolKind::Callable => 1,
                SymbolKind::Namespace => 2,
            },
            if symbol.kind == SymbolKind::Type {
                symbol.declaration_name.clone()
            } else {
                symbol.public_name.clone()
            },
        )
    });
    let mut sections = Vec::new();
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut namespaces = Vec::new();
    for symbol in declarations {
        let public_path = symbol_path(namespace, &symbol.public_name);
        let original_path = symbol_path(namespace, &symbol.declaration_name);
        match symbol.kind {
            SymbolKind::Type => {
                let local = generated_name(
                    &symbol.public_name,
                    NameKind::Type,
                    &context.entry.specifier,
                    &original_path,
                    &public_path,
                );
                let parameters = match &context.graph.types[&symbol.type_ref] {
                    SemanticType::Generic { parameters, .. } => parameters
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                };
                let generic = render_type_parameter_names(&parameters);
                types.push(format!("{pad}opaque type {local}{generic}"));
                let span = symbol_span(&symbol, "public-name");
                context.symbols.push(GeneratedSymbol {
                    identity: symbol_identity(&context.entry.specifier, &original_path, "type"),
                    public_path,
                    local_name: local,
                    host_name: symbol.public_name,
                    kind: "opaque-type".to_owned(),
                    signature: format!("opaque{generic}"),
                    source: SourceSpan {
                        file: context.file.to_owned(),
                        start: span.start,
                        end: span.end,
                    },
                    rule: "opaque-foreign-type".to_owned(),
                });
            }
            SymbolKind::Callable => {
                let signatures = match &context.graph.types[&symbol.type_ref] {
                    SemanticType::Overload { signatures } => signatures.clone(),
                    _ => vec![symbol.type_ref],
                };
                let overloads = signatures
                    .iter()
                    .map(|id| function_view(context.graph, &symbol, *id))
                    .collect::<Option<Vec<_>>>();
                let Some(overloads) = overloads else {
                    context.diagnostics.push(diagnostic(
                        "SES-F0101",
                        DiagnosticSeverity::Error,
                        "nested generic callable requires an explicit semantic projection policy",
                        context.file,
                        symbol_span(&symbol, "declaration"),
                        Some(public_path),
                    ));
                    continue;
                };
                functions.extend(render_overloads(
                    &symbol.public_name,
                    &overloads,
                    context,
                    namespace,
                    indent,
                ));
            }
            SymbolKind::Namespace => {
                let SemanticType::Namespace { members } = &context.graph.types[&symbol.type_ref]
                else {
                    continue;
                };
                let members = members.clone();
                let local = generated_name(
                    &symbol.public_name,
                    NameKind::Value,
                    &context.entry.specifier,
                    &original_path,
                    &public_path,
                );
                let host = if local == symbol.public_name {
                    String::new()
                } else {
                    format!(" = {:?}", symbol.public_name)
                };
                let mut nested = namespace.to_vec();
                nested.push(symbol.public_name.clone());
                let body = render_scope(&members, context, &nested, indent + 1);
                namespaces.push(format!("{pad}namespace {local}{host} {{\n{body}{pad}}}"));
                let span = symbol_span(&symbol, "public-name");
                context.symbols.push(GeneratedSymbol {
                    identity: symbol_identity(
                        &context.entry.specifier,
                        &original_path,
                        "namespace",
                    ),
                    public_path,
                    local_name: local,
                    host_name: symbol.public_name,
                    kind: "namespace".to_owned(),
                    signature: "namespace".to_owned(),
                    source: SourceSpan {
                        file: context.file.to_owned(),
                        start: span.start,
                        end: span.end,
                    },
                    rule: "merged-namespace".to_owned(),
                });
            }
        }
    }
    for lines in [types, functions] {
        if !lines.is_empty() {
            sections.push(lines.join("\n"));
        }
    }
    if !namespaces.is_empty() {
        sections.push(namespaces.join("\n\n"));
    }
    if sections.is_empty() {
        String::new()
    } else {
        format!("{}\n", sections.join("\n\n"))
    }
}

fn render_overloads(
    public_name: &str,
    overloads: &[Function],
    context: &mut RenderContext<'_>,
    namespace: &[String],
    indent: usize,
) -> Vec<String> {
    let config = context
        .entry
        .symbols
        .get(&symbol_path(namespace, public_name));
    if overloads.len() > 1 {
        let Some(config) = config else {
            context.diagnostics.push(diagnostic(
                "SES-F0102",
                DiagnosticSeverity::Error,
                format!(
                    "overloaded export `{}` requires explicit signature selection",
                    symbol_path(namespace, public_name)
                ),
                context.file,
                overloads[0].name_span,
                Some(symbol_path(namespace, public_name)),
            ));
            return Vec::new();
        };
        if config.overloads.is_empty() {
            context.diagnostics.push(diagnostic(
                "SES-F0102",
                DiagnosticSeverity::Error,
                format!("overloaded export `{public_name}` has no configured local bindings"),
                context.file,
                overloads[0].name_span,
                Some(symbol_path(namespace, public_name)),
            ));
            return Vec::new();
        }
        return config
            .overloads
            .iter()
            .filter_map(|(local, selection)| {
                let Some(function) = overloads.get(selection.signature) else {
                    context.diagnostics.push(diagnostic(
                        "SES-F0102",
                        DiagnosticSeverity::Error,
                        format!(
                            "overload selection `{local}` uses missing signature {}",
                            selection.signature
                        ),
                        context.file,
                        overloads[0].name_span,
                        Some(symbol_path(namespace, public_name)),
                    ));
                    return None;
                };
                render_function(function, Some(local), config, context, namespace, indent)
            })
            .collect();
    }
    let function = &overloads[0];
    render_function(
        function,
        config.and_then(|config| config.local.as_deref()),
        config.unwrap_or(&SymbolConfig::default()),
        context,
        namespace,
        indent,
    )
    .into_iter()
    .collect()
}

fn render_function(
    function: &Function,
    configured_local: Option<&str>,
    config: &SymbolConfig,
    context: &mut RenderContext<'_>,
    namespace: &[String],
    indent: usize,
) -> Option<String> {
    let public_path = symbol_path(namespace, &function.public_name);
    if function.receiver.is_some() {
        context.diagnostics.push(diagnostic(
            "SES-F0102",
            DiagnosticSeverity::Error,
            "contextual receiver requires an explicit bound-method ABI adapter",
            context.file,
            function.span,
            Some(public_path),
        ));
        return None;
    }
    let specialized = explicit_specialization(function, config, context.graph);
    if !specialized && function.generic_binder.is_some_and(|binder| matches!(&context.graph.types[&binder], SemanticType::Generic { parameters, .. } if parameters.iter().any(|parameter| parameter.constraint.is_some() || parameter.default.is_some()))) {
        context.diagnostics.push(diagnostic("SES-F0102", DiagnosticSeverity::Error,
            "generic constraints and defaults require a semantic projection policy; emitting unconstrained parameters would erase their meaning",
            context.file, function.span, Some(public_path)));
        return None;
    }
    let local = configured_local.map(str::to_owned).unwrap_or_else(|| {
        generated_name(
            &function.public_name,
            NameKind::Value,
            &context.entry.specifier,
            &symbol_path(namespace, &function.original_name),
            &public_path,
        )
    });
    if !valid_identifier(&local, NameKind::Value) || reserved(&local) {
        context.diagnostics.push(diagnostic(
            "SES-F0104",
            DiagnosticSeverity::Error,
            format!("configured local name `{local}` is not a valid value identifier"),
            context.file,
            function.name_span,
            Some(public_path),
        ));
        return None;
    }
    let mode = config.mode.unwrap_or(context.entry.evaluation);
    if mode == Evaluation::Pure && context.entry.evaluation != Evaluation::Pure {
        context.diagnostics.push(diagnostic(
            "SES-F0104",
            DiagnosticSeverity::Error,
            "a task-load module cannot approve a pure function",
            context.file,
            function.name_span,
            Some(public_path),
        ));
        return None;
    }
    let type_names = if specialized {
        vec![]
    } else {
        canonical_type_parameters(&function.type_parameters)
    };
    let type_map = function
        .type_parameters
        .iter()
        .cloned()
        .zip(type_names.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    let returns_promise = is_promise(context.graph, function.result);
    let mut parameters = Vec::new();
    for parameter in &function.parameters {
        if let SemanticType::Callable { result, .. } = &context.graph.types[&parameter.type_ref] {
            if let Some(callback) = config.callbacks.get(&parameter.name) {
                if callback.lifetime == CallbackLifetime::Retained && callback.release.is_none() {
                    context.diagnostics.push(diagnostic(
                        "SES-F0102",
                        DiagnosticSeverity::Error,
                        format!(
                            "retained callback `{}` requires an explicit release contract",
                            parameter.name
                        ),
                        context.file,
                        parameter
                            .origins
                            .first()
                            .map_or(Span { start: 0, end: 0 }, |o| Span {
                                start: o.start,
                                end: o.end,
                            }),
                        Some(public_path.clone()),
                    ));
                    return None;
                }
                if callback.lifetime == CallbackLifetime::UntilSettled && !returns_promise {
                    context.diagnostics.push(diagnostic(
                        "SES-F0102",
                        DiagnosticSeverity::Error,
                        format!(
                            "until-settled callback `{}` requires a Promise result",
                            parameter.name
                        ),
                        context.file,
                        parameter
                            .origins
                            .first()
                            .map_or(Span { start: 0, end: 0 }, |o| Span {
                                start: o.start,
                                end: o.end,
                            }),
                        Some(public_path.clone()),
                    ));
                    return None;
                }
                if callback.invocation == CallbackInvocation::Promise
                    && !is_promise(context.graph, *result)
                {
                    context.diagnostics.push(diagnostic(
                        "SES-F0102",
                        DiagnosticSeverity::Error,
                        format!(
                            "promise callback `{}` must return PromiseLike",
                            parameter.name
                        ),
                        context.file,
                        parameter
                            .origins
                            .first()
                            .map_or(Span { start: 0, end: 0 }, |o| Span {
                                start: o.start,
                                end: o.end,
                            }),
                        Some(public_path.clone()),
                    ));
                    return None;
                }
            }
        }
        let override_type = config.parameters.get(&parameter.name);
        let mut rendered = if let Some(override_type) = override_type {
            validate_override(
                &override_type.type_name,
                type_span(context.graph, parameter.type_ref),
                context,
            )?
        } else {
            render_type(
                parameter.type_ref,
                &type_map,
                config,
                Some(&parameter.name),
                context,
                &public_path,
            )?
        };
        if parameter.optional && !override_type.is_some_and(|policy| policy.required) {
            rendered = format!("Js.UndefinedOr<{rendered}>");
        }
        let rest = if parameter.rest { "..." } else { "" };
        parameters.push(format!("{rest}{}: {rendered}", parameter.name));
    }
    let result_ref = promise_inner(context.graph, function.result).unwrap_or(function.result);
    let result = if let Some(override_type) = &config.result {
        if override_type.required {
            context.diagnostics.push(diagnostic(
                "SES-F0104",
                DiagnosticSeverity::Error,
                "required is only a parameter input policy",
                context.file,
                function.span,
                Some(public_path),
            ));
            return None;
        }
        validate_override(
            &override_type.type_name,
            type_span(context.graph, result_ref),
            context,
        )?
    } else {
        render_type(result_ref, &type_map, config, None, context, &public_path)?
    };
    let mode = if returns_promise {
        Evaluation::Task
    } else {
        mode
    };
    let generics = if type_names.is_empty() {
        String::new()
    } else {
        format!("<{}>", type_names.join(", "))
    };
    let mut signature = format!("{} fn {local}{generics}", evaluation_name(mode));
    if !parameters.is_empty() {
        signature.push(' ');
        signature.push_str(&parameters.join(" -> "));
        signature.push_str(" -> ");
    } else {
        signature.push_str(" -> ");
    }
    signature.push_str(&result);
    if local != function.public_name {
        signature.push_str(&format!(" = {:?}", function.public_name));
    }
    let pad = "  ".repeat(indent);
    let line = wrap_signature(&pad, &signature);
    context.symbols.push(GeneratedSymbol {
        identity: symbol_identity(
            &context.entry.specifier,
            &symbol_path(namespace, &function.original_name),
            "value",
        ),
        public_path,
        local_name: local,
        host_name: function.public_name.clone(),
        kind: "function".to_owned(),
        signature: signature.clone(),
        source: SourceSpan {
            file: context.file.to_owned(),
            start: function.span.start,
            end: function.span.end,
        },
        rule: if returns_promise {
            "promise-task-function".to_owned()
        } else {
            format!("{}-function", evaluation_name(mode))
        },
    });
    Some(line)
}

fn render_type(
    id: TypeId,
    type_parameters: &BTreeMap<String, String>,
    symbol: &SymbolConfig,
    callback_parameter: Option<&str>,
    context: &mut RenderContext<'_>,
    symbol_path: &str,
) -> Option<String> {
    if context.active_types.len() >= 256 || !context.active_types.insert(id) {
        return unsupported_type(
            id,
            "recursive or excessively deep inline type requires an explicit opaque boundary",
            context,
            symbol_path,
        );
    }
    let result = render_type_node(
        id,
        type_parameters,
        symbol,
        callback_parameter,
        context,
        symbol_path,
    );
    context.active_types.remove(&id);
    result
}

fn explicit_specialization(
    function: &Function,
    config: &SymbolConfig,
    graph: &SemanticGraph,
) -> bool {
    let Some(binder) = function.generic_binder else {
        return false;
    };
    let Some(result) = &config.result else {
        return false;
    };
    if function
        .parameters
        .iter()
        .any(|parameter| !config.parameters.contains_key(&parameter.name))
    {
        return false;
    }
    let mut bindings = BTreeMap::new();
    for (id, name) in function
        .parameters
        .iter()
        .map(|parameter| {
            (
                parameter.type_ref,
                config.parameters[&parameter.name].type_name.as_str(),
            )
        })
        .chain(std::iter::once((
            promise_inner(graph, function.result).unwrap_or(function.result),
            result.type_name.as_str(),
        )))
    {
        let node = &graph.types[&id];
        let expected = match node {
            SemanticType::Parameter {
                binder: owner,
                index,
            } if *owner == binder => {
                if let Some(previous) = bindings.insert(*index, name) {
                    if previous != name {
                        return false;
                    }
                }
                let SemanticType::Generic { parameters, .. } = &graph.types[&binder] else {
                    return false;
                };
                match parameters[*index as usize].constraint {
                    Some(id) => &graph.types[&id],
                    None => continue,
                }
            }
            other => other,
        };
        let SemanticType::Primitive { domain } = expected else {
            return false;
        };
        if !matches!(
            (domain, name),
            (PrimitiveDomain::String, "String" | "Char")
                | (PrimitiveDomain::Boolean, "Bool")
                | (PrimitiveDomain::Float64, "Float" | "Int")
                | (PrimitiveDomain::Integer, "Int")
                | (PrimitiveDomain::Binary, "Bytes")
        ) {
            return false;
        }
    }
    true
}

fn render_type_node(
    id: TypeId,
    type_parameters: &BTreeMap<String, String>,
    symbol: &SymbolConfig,
    callback_parameter: Option<&str>,
    context: &mut RenderContext<'_>,
    symbol_path: &str,
) -> Option<String> {
    // A bridge projection of the existing supported subset. Every input node
    // is interpreted by the frontend first; no TypeScript AST is visible here.
    let node = context.graph.types[&id].clone();
    match node {
        SemanticType::Primitive { domain } => match domain {
            PrimitiveDomain::Unit => Some("Unit".to_owned()),
            PrimitiveDomain::Never => Some("Never".to_owned()),
            PrimitiveDomain::Boolean => Some("Bool".to_owned()),
            PrimitiveDomain::String => Some("String".to_owned()),
            PrimitiveDomain::Float64 => Some("Float".to_owned()),
            PrimitiveDomain::Integer => Some("Int".to_owned()),
            PrimitiveDomain::BigInteger => {
                context.uses_big_int = true;
                Some("BigInt".to_owned())
            }
            PrimitiveDomain::Unknown => Some("Js.Unknown".to_owned()),
            PrimitiveDomain::Object => Some("Js.Object".to_owned()),
            PrimitiveDomain::Null => Some("Js.Null".to_owned()),
            PrimitiveDomain::Undefined => Some("Js.Undefined".to_owned()),
            PrimitiveDomain::Binary => Some("Bytes".to_owned()),
            PrimitiveDomain::Dynamic if symbol.unsafe_any => {
                context.diagnostics.push(diagnostic(
                    "SES-F0101",
                    DiagnosticSeverity::Warning,
                    "explicit unsafe any fallback maps to Js.Unknown",
                    context.file,
                    type_span(context.graph, id),
                    Some(symbol_path.to_owned()),
                ));
                Some("Js.Unknown".to_owned())
            }
            PrimitiveDomain::Dynamic => unsupported_type(
                id,
                "`any` requires an explicit unsafe fallback",
                context,
                symbol_path,
            ),
        },
        SemanticType::Parameter { binder, index } => {
            let SemanticType::Generic { parameters, .. } = &context.graph.types[&binder] else {
                return None;
            };
            type_parameters
                .get(&parameters[index as usize].name)
                .cloned()
        }
        SemanticType::Reference { symbol: target } => {
            if context.graph.private_symbols.contains(&target) {
                render_type(
                    context.graph.symbols[&target].type_ref,
                    type_parameters,
                    symbol,
                    callback_parameter,
                    context,
                    symbol_path,
                )
            } else {
                Some(binding_type_name(context, &target))
            }
        }
        SemanticType::Opaque {
            identity,
            arguments,
        } => {
            if !context.graph.symbols.iter().any(|(key, declaration)| {
                !context.graph.private_symbols.contains(key)
                    && declaration.kind == SymbolKind::Type
                    && (key == &identity || declaration.declaration_name == identity)
            }) {
                context.diagnostics.push(diagnostic("SES-F0110", DiagnosticSeverity::Error,
                    format!("opaque external identity `{identity}` has no declared binding or resolved runtime ABI"), context.file, type_span(context.graph, id), Some(symbol_path.to_owned())));
                return None;
            }
            let name = generated_name_simple(&identity, NameKind::Type);
            if arguments.is_empty() {
                Some(name)
            } else {
                let args = arguments
                    .into_iter()
                    .map(|arg| {
                        render_type(
                            arg,
                            type_parameters,
                            symbol,
                            callback_parameter,
                            context,
                            symbol_path,
                        )
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(format!("{name}<{}>", args.join(", ")))
            }
        }
        SemanticType::Apply {
            constructor,
            arguments,
        } => {
            let name = render_type(
                constructor,
                type_parameters,
                symbol,
                callback_parameter,
                context,
                symbol_path,
            )?;
            let args = arguments
                .into_iter()
                .map(|arg| {
                    render_type(
                        arg,
                        type_parameters,
                        symbol,
                        callback_parameter,
                        context,
                        symbol_path,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            Some(format!("{name}<{}>", args.join(", ")))
        }
        SemanticType::Async { value } => render_type(
            value,
            type_parameters,
            symbol,
            callback_parameter,
            context,
            symbol_path,
        ),
        SemanticType::Collection { element, mutable } => render_type(
            element,
            type_parameters,
            symbol,
            callback_parameter,
            context,
            symbol_path,
        )
        .map(|value| {
            format!(
                "{}<{value}>",
                if mutable { "Js.MutableArray" } else { "Array" }
            )
        }),
        SemanticType::Tuple { elements } => elements
            .into_iter()
            .map(|id| {
                render_type(
                    id,
                    type_parameters,
                    symbol,
                    callback_parameter,
                    context,
                    symbol_path,
                )
            })
            .collect::<Option<Vec<_>>>()
            .map(|parts| format!("({})", parts.join(", "))),
        SemanticType::Union { members } => {
            let mut value = None;
            let mut null = false;
            let mut undefined = false;
            for member in members {
                match context.graph.types[&member] {
                    SemanticType::Primitive {
                        domain: PrimitiveDomain::Null,
                    } => null = true,
                    SemanticType::Primitive {
                        domain: PrimitiveDomain::Undefined,
                    } => undefined = true,
                    _ if value.is_none() => value = Some(member),
                    _ => {
                        return unsupported_type(
                            id,
                            "arbitrary unions require an explicit discriminated-union adapter",
                            context,
                            symbol_path,
                        )
                    }
                }
            }
            let Some(value) = value else {
                return unsupported_type(
                    id,
                    "nullish union has no value type",
                    context,
                    symbol_path,
                );
            };
            let mut result = render_type(
                value,
                type_parameters,
                symbol,
                callback_parameter,
                context,
                symbol_path,
            )?;
            if null {
                result = format!("Js.NullOr<{result}>");
            }
            if undefined {
                result = format!("Js.UndefinedOr<{result}>");
            }
            Some(result)
        }
        SemanticType::Callable {
            parameters,
            result,
            receiver,
        } => {
            if receiver.is_some() {
                return unsupported_type(
                    id,
                    "contextual receiver requires an explicit bound-method ABI adapter",
                    context,
                    symbol_path,
                );
            }
            let callback = callback_parameter.and_then(|name| symbol.callbacks.get(name));
            let args = parameters
                .into_iter()
                .map(|p| {
                    render_type(
                        p.type_ref,
                        type_parameters,
                        symbol,
                        None,
                        context,
                        symbol_path,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let result = promise_inner(context.graph, result).unwrap_or(result);
            let result = render_type(result, type_parameters, symbol, None, context, symbol_path)?;
            if callback.is_some() {
                let mut signature = result;
                for argument in args.into_iter().rev() {
                    signature = format!("{argument} -> {signature}");
                }
                Some(format!("({signature})"))
            } else {
                let args = match args.as_slice() {
                    [] => "Unit".to_owned(),
                    [one] => one.clone(),
                    _ => format!("({})", args.join(", ")),
                };
                Some(format!("Js.Callback<{args}, {result}>"))
            }
        }
        SemanticType::Nullable { value } | SemanticType::Optional { value } => {
            let wrapper = if matches!(context.graph.types[&id], SemanticType::Nullable { .. }) {
                "Js.NullOr"
            } else {
                "Js.UndefinedOr"
            };
            render_type(value, type_parameters, symbol, None, context, symbol_path)
                .map(|inner| format!("{wrapper}<{inner}>"))
        }
        _ => unsupported_type(
            id,
            "unsupported TypeScript type requires an explicit adapter",
            context,
            symbol_path,
        ),
    }
}
fn unsupported_type(
    id: TypeId,
    message: &str,
    context: &mut RenderContext<'_>,
    symbol: &str,
) -> Option<String> {
    context.diagnostics.push(diagnostic(
        "SES-F0101",
        DiagnosticSeverity::Error,
        message,
        context.file,
        type_span(context.graph, id),
        Some(symbol.to_owned()),
    ));
    None
}

fn validate_override(value: &str, span: Span, context: &mut RenderContext<'_>) -> Option<String> {
    if matches!(
        value,
        "Int" | "Char" | "Float" | "String" | "Bool" | "Bytes"
    ) {
        Some(value.to_owned())
    } else {
        context.diagnostics.push(diagnostic(
            "SES-F0104",
            DiagnosticSeverity::Error,
            format!("unsupported explicit boundary type override `{value}`"),
            context.file,
            span,
            None,
        ));
        None
    }
}

fn validate_symbol_settings(graph: &SemanticGraph, context: &mut RenderContext<'_>) {
    if let Some(include) = &context.entry.include {
        if include.is_empty() || include.iter().collect::<BTreeSet<_>>().len() != include.len() {
            context.diagnostics.push(diagnostic(
                "SES-F0104",
                DiagnosticSeverity::Error,
                "projection include must be a nonempty set of distinct exported symbols",
                context.file,
                Span { start: 0, end: 0 },
                None,
            ));
        }
        for key in include {
            if !graph.symbols.contains_key(key) || graph.private_symbols.contains(key) {
                context.diagnostics.push(diagnostic(
                    "SES-F0104",
                    DiagnosticSeverity::Error,
                    format!("projection include refers to missing or private symbol `{key}`"),
                    context.file,
                    Span { start: 0, end: 0 },
                    Some(key.clone()),
                ));
            }
        }
    }
    for key in context.entry.symbols.keys() {
        if !graph.symbols.contains_key(key) {
            context.diagnostics.push(diagnostic(
                "SES-F0104",
                DiagnosticSeverity::Error,
                format!("binding settings refer to unknown symbol `{key}`"),
                context.file,
                Span { start: 0, end: 0 },
                Some(key.clone()),
            ));
        }
    }
}
fn is_promise(graph: &SemanticGraph, id: TypeId) -> bool {
    matches!(graph.types[&id], SemanticType::Async { .. })
}
fn promise_inner(graph: &SemanticGraph, id: TypeId) -> Option<TypeId> {
    match graph.types[&id] {
        SemanticType::Async { value } => Some(value),
        _ => None,
    }
}

fn wrap_signature(pad: &str, signature: &str) -> String {
    let full = format!("{pad}{signature}");
    if full.len() <= 88 {
        return full;
    }
    let Some(split) = signature.rfind(" -> ") else {
        return full;
    };
    format!(
        "{pad}{}\n{pad}  -> {}",
        &signature[..split],
        &signature[split + 4..]
    )
}

fn render_type_parameter_names(parameters: &[String]) -> String {
    let names = canonical_type_parameters(parameters);
    if names.is_empty() {
        String::new()
    } else {
        format!("<{}>", names.join(", "))
    }
}

fn canonical_type_parameters(parameters: &[String]) -> Vec<String> {
    parameters
        .iter()
        .enumerate()
        .map(|(index, _)| {
            if index < 26 {
                ((b'A' + index as u8) as char).to_string()
            } else {
                format!("A{}", index + 1)
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
enum NameKind {
    Type,
    Value,
}

fn generated_name(
    spelling: &str,
    kind: NameKind,
    specifier: &str,
    original_path: &str,
    public_path: &str,
) -> String {
    let candidate = generated_name_simple(spelling, kind);
    if valid_identifier(&candidate, kind) && !reserved(&candidate) {
        return candidate;
    }
    let identity = symbol_identity(
        specifier,
        original_path,
        match kind {
            NameKind::Type => "type",
            NameKind::Value => "value",
        },
    );
    let mut hasher = Sha256::new();
    hasher.update(identity.as_bytes());
    hasher.update([0]);
    hasher.update(public_path.as_bytes());
    hasher.update([0]);
    hasher.update(spelling.as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    let stem = if candidate.is_empty() {
        match kind {
            NameKind::Type => "TsType",
            NameKind::Value => "tsValue",
        }
    } else {
        &candidate
    };
    format!("{stem}__{}", &hash[..8])
}

fn generated_name_simple(spelling: &str, kind: NameKind) -> String {
    let words = split_words(spelling);
    if words.is_empty() {
        return String::new();
    }
    match kind {
        NameKind::Type => words
            .iter()
            .map(|word| upper_first(word))
            .collect::<String>(),
        NameKind::Value => {
            let mut output = words[0].to_ascii_lowercase();
            for word in &words[1..] {
                output.push_str(&upper_first(word));
            }
            output
        }
    }
}

fn split_words(value: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower = false;
    for character in value.chars() {
        if matches!(character, '-' | '_' | ' ') {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            previous_lower = false;
            continue;
        }
        if !character.is_ascii_alphanumeric() {
            continue;
        }
        if character.is_ascii_uppercase() && previous_lower && !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
        previous_lower = character.is_ascii_lowercase();
        current.push(character);
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn upper_first(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
        None => String::new(),
    }
}

fn valid_identifier(value: &str, kind: NameKind) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    let first_valid = match kind {
        NameKind::Type => first.is_ascii_uppercase(),
        NameKind::Value => first.is_ascii_lowercase(),
    };
    first_valid && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn reserved(value: &str) -> bool {
    matches!(
        value,
        "as" | "case"
            | "do"
            | "effect"
            | "else"
            | "false"
            | "fn"
            | "foreign"
            | "if"
            | "import"
            | "in"
            | "let"
            | "match"
            | "module"
            | "namespace"
            | "opaque"
            | "pub"
            | "rec"
            | "struct"
            | "then"
            | "trait"
            | "true"
            | "type"
            | "where"
            | "with"
    )
}

fn symbol_identity(specifier: &str, original_path: &str, namespace: &str) -> String {
    format!("{specifier}::{original_path}::{namespace}")
}

fn symbol_path(namespace: &[String], name: &str) -> String {
    if namespace.is_empty() {
        name.to_owned()
    } else {
        format!("{}.{}", namespace.join("."), name)
    }
}

fn diagnostic(
    code: &str,
    severity: DiagnosticSeverity,
    message: impl Into<String>,
    file: &str,
    span: Span,
    symbol: Option<String>,
) -> ConversionDiagnostic {
    ConversionDiagnostic {
        code: code.to_owned(),
        severity,
        message: message.into(),
        file: file.to_owned(),
        start: span.start,
        end: span.end,
        symbol,
    }
}

fn parse_config(source: &str) -> Result<BindingsConfig, ConvertError> {
    toml::from_str::<BindingsConfig>(source)
        .map_err(|error| ConvertError::new(format!("invalid binding settings: {error}")))
}

fn validate_config(config: &BindingsConfig) -> Result<(), ConvertError> {
    if config.schema != 1 {
        return Err(ConvertError::new(format!(
            "unsupported binding settings schema {}; expected 1",
            config.schema
        )));
    }
    if config.entries.is_empty() {
        return Err(ConvertError::new(
            "binding settings must define at least one entry",
        ));
    }
    let mut declarations = BTreeSet::new();
    let mut specifiers = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    for (id, entry) in &config.entries {
        if !valid_entry_id(id) {
            return Err(ConvertError::new(format!(
                "binding entry ID `{id}` must be ASCII lower kebab-case"
            )));
        }
        validate_relative_path("declaration", &entry.declaration)?;
        validate_relative_path("output", &entry.output)?;
        if Path::new(&entry.output).extension().is_some() {
            return Err(ConvertError::new(format!(
                "binding output `{}` must be a module path without an extension",
                entry.output
            )));
        }
        if entry.specifier.is_empty() {
            return Err(ConvertError::new(format!(
                "binding entry `{id}` has an empty specifier"
            )));
        }
        if !declarations.insert(&entry.declaration) {
            return Err(ConvertError::new(format!(
                "binding declaration `{}` is used by more than one entry",
                entry.declaration
            )));
        }
        if !specifiers.insert(&entry.specifier) {
            return Err(ConvertError::new(format!(
                "binding specifier `{}` is used by more than one entry",
                entry.specifier
            )));
        }
        if !outputs.insert(&entry.output) {
            return Err(ConvertError::new(format!(
                "binding output `{}` is used by more than one entry",
                entry.output
            )));
        }
    }
    Ok(())
}

fn valid_entry_id(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|value| value.is_ascii_lowercase())
        && chars.all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
        && !value.ends_with('-')
        && !value.contains("--")
}

fn validate_relative_path(field: &str, value: &str) -> Result<(), ConvertError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::CurDir
                    | Component::ParentDir
                    | Component::RootDir
                    | Component::Prefix(_)
            )
        })
    {
        return Err(ConvertError::new(format!(
            "binding {field} `{value}` must be a package-relative normalized path"
        )));
    }
    Ok(())
}

fn evaluation_name(value: Evaluation) -> &'static str {
    match value {
        Evaluation::Pure => "pure",
        Evaluation::Task => "task",
    }
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn json_text(value: &impl Serialize) -> Result<String, ConvertError> {
    let mut bytes = serde_json::to_string_pretty(value)
        .map_err(|error| ConvertError::new(format!("failed to encode generated JSON: {error}")))?;
    bytes.push('\n');
    Ok(bytes)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BindingMetadata {
    schema: u32,
    kind: String,
    generator: GeneratorIdentity,
    entry: String,
    declaration: String,
    output: String,
    specifier: String,
    host_module: HostModuleIdentity,
    evaluation: String,
    input_digest: String,
    settings_digest: String,
    symbols: Vec<GeneratedSymbol>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostModuleIdentity {
    specifier: String,
    exact_identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GeneratorIdentity {
    name: String,
    version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedSymbol {
    identity: String,
    public_path: String,
    local_name: String,
    host_name: String,
    kind: String,
    signature: String,
    source: SourceSpan,
    rule: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SourceSpan {
    file: String,
    start: usize,
    end: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConversionReport {
    schema: u32,
    kind: &'static str,
    entry: String,
    added: Vec<String>,
    changed: Vec<String>,
    removed: Vec<String>,
    unsupported: Vec<String>,
    warnings: Vec<ConversionDiagnostic>,
}

fn build_report(
    entry: &str,
    previous: Option<&BindingMetadata>,
    current: &BindingMetadata,
    diagnostics: &[ConversionDiagnostic],
) -> ConversionReport {
    let before = previous
        .map(|metadata| {
            metadata
                .symbols
                .iter()
                .map(|symbol| (symbol.identity.clone(), symbol.signature.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let after = current
        .symbols
        .iter()
        .map(|symbol| (symbol.identity.clone(), symbol.signature.clone()))
        .collect::<BTreeMap<_, _>>();
    let added = after
        .keys()
        .filter(|identity| !before.contains_key(*identity))
        .cloned()
        .collect();
    let changed = after
        .iter()
        .filter(|(identity, signature)| before.get(*identity).is_some_and(|old| old != *signature))
        .map(|(identity, _)| identity.clone())
        .collect();
    let removed = before
        .keys()
        .filter(|identity| !after.contains_key(*identity))
        .cloned()
        .collect();
    ConversionReport {
        schema: REPORT_SCHEMA,
        kind: "seseragi-typescript-conversion-report",
        entry: entry.to_owned(),
        added,
        changed,
        removed,
        unsupported: Vec::new(),
        warnings: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
            .cloned()
            .collect(),
    }
}
