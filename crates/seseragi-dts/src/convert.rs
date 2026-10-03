//! Deterministic declaration conversion without filesystem or host resolution.
use crate::config::{
    BindingsConfig, CallbackInvocation, CallbackLifetime, EntryConfig, Evaluation, SymbolConfig,
};
use crate::model::{Declaration, Function, Namespace, Scope, Span, TypeKind, TypeRef};
use crate::parser::parse_declarations;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Component, Path};

pub(crate) const METADATA_SCHEMA: u32 = 1;
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
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConvertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ConvertError {}

/// Validated binding policy shared by native and virtual-workspace callers.
#[derive(Clone, Debug)]
pub struct Converter {
    config: BindingsConfig,
    settings_digest: String,
}

/// An entry's complete conversion input. The caller resolves the host identity;
/// the converter neither loads that module nor reads declaration/metadata files.
#[derive(Clone, Debug)]
pub struct DeclarationInput<'a> {
    pub entry: &'a str,
    pub source: &'a str,
    pub host_module: HostModuleIdentity,
    pub previous_metadata: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedBinding {
    pub entry: String,
    pub output: String,
    pub source: String,
    /// The exact UTF-8 JSON text written by the native adapter, including LF.
    pub metadata: String,
    pub report: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub generated: Option<GeneratedBinding>,
    pub diagnostics: Vec<ConversionDiagnostic>,
}

impl ConversionResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|value| value.severity == DiagnosticSeverity::Error)
    }
}

impl Converter {
    pub fn new(settings_source: &str) -> Result<Self, ConvertError> {
        let config = parse_config(settings_source)?;
        validate_config(&config)?;
        Ok(Self {
            config,
            settings_digest: sha256(settings_source),
        })
    }

    /// Entries are returned in their stable configuration ID order.
    pub fn entries(&self) -> &BTreeMap<String, EntryConfig> {
        &self.config.entries
    }

    pub fn convert(&self, input: DeclarationInput<'_>) -> Result<ConversionResult, ConvertError> {
        self.prepare(input.entry, input.source, input.previous_metadata)?
            .finish(input.host_module)
    }

    pub(crate) fn prepare(
        &self,
        id: &str,
        declaration: &str,
        previous_metadata: Option<&str>,
    ) -> Result<PreparedConversion, ConvertError> {
        let entry = self
            .config
            .entries
            .get(id)
            .ok_or_else(|| ConvertError::new(format!("binding entry `{id}` does not exist")))?;
        let scope = match parse_declarations(declaration) {
            Ok(scope) => scope,
            Err(error) => {
                return Ok(PreparedConversion {
                    binding: None,
                    diagnostics: vec![diagnostic(
                        "SES-F0100",
                        DiagnosticSeverity::Error,
                        error.message,
                        &entry.declaration,
                        error.span,
                        None,
                    )],
                })
            }
        };
        let rendered = render_entry(id, entry, &scope, &entry.declaration);
        let diagnostics = rendered.diagnostics.clone();
        let binding = if diagnostics
            .iter()
            .any(|value| value.severity == DiagnosticSeverity::Error)
        {
            None
        } else {
            Some(PreparedBinding {
                id: id.to_owned(),
                entry: entry.clone(),
                input_digest: sha256(declaration),
                settings_digest: self.settings_digest.clone(),
                previous: previous_metadata.and_then(|source| serde_json::from_str(source).ok()),
                rendered,
            })
        };
        Ok(PreparedConversion {
            binding,
            diagnostics,
        })
    }
}

pub(crate) struct PreparedConversion {
    binding: Option<PreparedBinding>,
    pub(crate) diagnostics: Vec<ConversionDiagnostic>,
}

struct PreparedBinding {
    id: String,
    entry: EntryConfig,
    input_digest: String,
    settings_digest: String,
    previous: Option<BindingMetadata>,
    rendered: RenderedEntry,
}

impl PreparedConversion {
    #[cfg(feature = "filesystem")]
    pub(crate) fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|value| value.severity == DiagnosticSeverity::Error)
    }

    pub(crate) fn finish(
        self,
        host_module: HostModuleIdentity,
    ) -> Result<ConversionResult, ConvertError> {
        let Some(binding) = self.binding else {
            return Ok(ConversionResult {
                generated: None,
                diagnostics: self.diagnostics,
            });
        };
        if host_module.specifier != binding.entry.specifier || host_module.exact_identity.is_empty()
        {
            return Err(ConvertError::new(format!(
                "host identity for binding entry `{}` must resolve specifier `{}` to a nonempty exact identity",
                binding.id, binding.entry.specifier
            )));
        }
        let metadata = BindingMetadata {
            schema: METADATA_SCHEMA,
            kind: "seseragi-typescript-binding".to_owned(),
            generator: GeneratorIdentity {
                name: "seseragi-dts".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            entry: binding.id.clone(),
            declaration: binding.entry.declaration,
            output: binding.entry.output.clone(),
            specifier: binding.entry.specifier,
            host_module,
            evaluation: evaluation_name(binding.entry.evaluation).to_owned(),
            input_digest: binding.input_digest,
            settings_digest: binding.settings_digest,
            symbols: binding.rendered.symbols,
        };
        let report = build_report(
            &binding.id,
            binding.previous.as_ref(),
            &metadata,
            &self.diagnostics,
        );
        Ok(ConversionResult {
            generated: Some(GeneratedBinding {
                entry: binding.id,
                output: binding.entry.output,
                source: binding.rendered.source,
                metadata: json_string(&metadata)?,
                report: json_string(&report)?,
            }),
            diagnostics: self.diagnostics,
        })
    }
}

#[derive(Clone, Debug)]
struct RenderedEntry {
    source: String,
    symbols: Vec<GeneratedSymbol>,
    diagnostics: Vec<ConversionDiagnostic>,
}

fn render_entry(_id: &str, entry: &EntryConfig, scope: &Scope, file: &str) -> RenderedEntry {
    let mut context = RenderContext {
        entry,
        file,
        diagnostics: Vec::new(),
        symbols: Vec::new(),
        uses_big_int: false,
    };
    validate_symbol_settings(scope, &mut context);
    let body = render_scope(scope, &mut context, &[], 1);
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
    diagnostics: Vec<ConversionDiagnostic>,
    symbols: Vec<GeneratedSymbol>,
    uses_big_int: bool,
}

fn render_scope(
    scope: &Scope,
    context: &mut RenderContext<'_>,
    namespace: &[String],
    indent: usize,
) -> String {
    let mut declarations = scope.declarations.clone();
    declarations.sort_by_key(declaration_sort_key);
    let mut types = BTreeMap::<String, Declaration>::new();
    let mut functions = BTreeMap::<String, Vec<Function>>::new();
    let mut namespaces = BTreeMap::<String, Namespace>::new();
    for declaration in declarations {
        match declaration {
            Declaration::Function(function) => functions
                .entry(function.public_name.clone())
                .or_default()
                .push(function),
            Declaration::Namespace(value) => {
                namespaces.entry(value.public_name.clone()).or_insert(value);
            }
            other => {
                types
                    .entry(other.original_name().to_owned())
                    .or_insert(other);
            }
        }
    }
    let pad = "  ".repeat(indent);
    let mut sections = Vec::new();
    if !types.is_empty() {
        let mut lines = Vec::new();
        for declaration in types.into_values() {
            let (public, original, span, parameters) = match declaration {
                Declaration::Interface(value) => {
                    (value.public_name, value.name, value.name_span, Vec::new())
                }
                Declaration::Class(value) | Declaration::Enum(value) => (
                    value.public_name,
                    value.name,
                    value.name_span,
                    value.type_parameters,
                ),
                Declaration::TypeAlias(value) => (
                    value.public_name,
                    value.name,
                    value.name_span,
                    value.type_parameters,
                ),
                _ => continue,
            };
            let path = symbol_path(namespace, &public);
            let local = generated_name(
                &public,
                NameKind::Type,
                &context.entry.specifier,
                &symbol_path(namespace, &original),
                &path,
            );
            let generic = render_type_parameter_names(&parameters);
            lines.push(format!("{pad}opaque type {local}{generic}"));
            context.symbols.push(GeneratedSymbol {
                identity: symbol_identity(
                    &context.entry.specifier,
                    &symbol_path(namespace, &original),
                    "type",
                ),
                public_path: path,
                local_name: local,
                host_name: public,
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
        sections.push(lines.join("\n"));
    }
    if !functions.is_empty() {
        let mut lines = Vec::new();
        for (public_name, overloads) in functions {
            lines.extend(render_overloads(
                &public_name,
                &overloads,
                context,
                namespace,
                indent,
            ));
        }
        if !lines.is_empty() {
            sections.push(lines.join("\n"));
        }
    }
    if !namespaces.is_empty() {
        let mut blocks = Vec::new();
        for value in namespaces.into_values() {
            let public_path = symbol_path(namespace, &value.public_name);
            let local = generated_name(
                &value.public_name,
                NameKind::Value,
                &context.entry.specifier,
                &symbol_path(namespace, &value.original_name),
                &public_path,
            );
            let host = if local == value.public_name {
                String::new()
            } else {
                format!(" = {:?}", value.public_name)
            };
            let mut next_namespace = namespace.to_vec();
            next_namespace.push(value.public_name.clone());
            let nested = render_scope(&value.scope, context, &next_namespace, indent + 1);
            blocks.push(format!("{pad}namespace {local}{host} {{\n{nested}{pad}}}"));
            context.symbols.push(GeneratedSymbol {
                identity: symbol_identity(
                    &context.entry.specifier,
                    &symbol_path(namespace, &value.original_name),
                    "namespace",
                ),
                public_path,
                local_name: local,
                host_name: value.public_name,
                kind: "namespace".to_owned(),
                signature: "namespace".to_owned(),
                source: SourceSpan {
                    file: context.file.to_owned(),
                    start: value.name_span.start,
                    end: value.name_span.end,
                },
                rule: "merged-namespace".to_owned(),
            });
        }
        sections.push(blocks.join("\n\n"));
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
    let type_names = canonical_type_parameters(&function.type_parameters);
    let type_map = function
        .type_parameters
        .iter()
        .cloned()
        .zip(type_names.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    let returns_promise = is_promise(&function.result);
    let mut parameters = Vec::new();
    for parameter in &function.parameters {
        if let TypeKind::Function { result, .. } = &parameter.type_ref.kind {
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
                        parameter.name_span,
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
                        parameter.name_span,
                        Some(public_path.clone()),
                    ));
                    return None;
                }
                if callback.invocation == CallbackInvocation::Promise && !is_promise(result) {
                    context.diagnostics.push(diagnostic(
                        "SES-F0102",
                        DiagnosticSeverity::Error,
                        format!(
                            "promise callback `{}` must return PromiseLike",
                            parameter.name
                        ),
                        context.file,
                        parameter.name_span,
                        Some(public_path.clone()),
                    ));
                    return None;
                }
            }
        }
        let override_type = config.parameters.get(&parameter.name);
        let mut rendered = if let Some(override_type) = override_type {
            validate_override(&override_type.type_name, parameter.type_ref.span, context)?
        } else {
            render_type(
                &parameter.type_ref,
                &type_map,
                config,
                Some(&parameter.name),
                context,
                &public_path,
            )?
        };
        if parameter.optional {
            rendered = format!("Js.UndefinedOr<{rendered}>");
        }
        let rest = if parameter.rest { "..." } else { "" };
        parameters.push(format!("{rest}{}: {rendered}", parameter.name));
    }
    let result_ref = promise_inner(&function.result).unwrap_or(&function.result);
    let result = if let Some(override_type) = &config.result {
        validate_override(&override_type.type_name, result_ref.span, context)?
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
    type_ref: &TypeRef,
    type_parameters: &BTreeMap<String, String>,
    symbol: &SymbolConfig,
    callback_parameter: Option<&str>,
    context: &mut RenderContext<'_>,
    symbol_path: &str,
) -> Option<String> {
    match &type_ref.kind {
        TypeKind::Primitive(name) => match name.as_str() {
            "boolean" => Some("Bool".to_owned()),
            "string" => Some("String".to_owned()),
            "number" => Some("Float".to_owned()),
            "bigint" => {
                context.uses_big_int = true;
                Some("BigInt".to_owned())
            }
            "never" => Some("Never".to_owned()),
            "unknown" => Some("Js.Unknown".to_owned()),
            "object" => Some("Js.Object".to_owned()),
            "null" => Some("Js.Null".to_owned()),
            "undefined" => Some("Js.Undefined".to_owned()),
            "void" => Some("Unit".to_owned()),
            "any" if symbol.unsafe_any => {
                context.diagnostics.push(diagnostic(
                    "SES-F0101",
                    DiagnosticSeverity::Warning,
                    "explicit unsafe any fallback maps to Js.Unknown",
                    context.file,
                    type_ref.span,
                    Some(symbol_path.to_owned()),
                ));
                Some("Js.Unknown".to_owned())
            }
            "any" => unsupported_type(
                type_ref,
                "`any` requires an explicit unsafe fallback",
                context,
                symbol_path,
            ),
            other => unsupported_type(
                type_ref,
                &format!("unsupported TypeScript primitive `{other}`"),
                context,
                symbol_path,
            ),
        },
        TypeKind::Named(name) => match name.as_str() {
            "bigint" => {
                context.uses_big_int = true;
                Some("BigInt".to_owned())
            }
            "Uint8Array" => Some("Bytes".to_owned()),
            _ => Some(
                type_parameters
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| generated_name_simple(name, NameKind::Type)),
            ),
        },
        TypeKind::Generic { name, arguments } => {
            if matches!(name.as_str(), "Promise" | "PromiseLike") {
                let Some(argument) = arguments.first() else {
                    return unsupported_type(
                        type_ref,
                        "Promise requires one type argument",
                        context,
                        symbol_path,
                    );
                };
                return render_type(
                    argument,
                    type_parameters,
                    symbol,
                    callback_parameter,
                    context,
                    symbol_path,
                );
            }
            let rendered = arguments
                .iter()
                .map(|argument| {
                    render_type(
                        argument,
                        type_parameters,
                        symbol,
                        callback_parameter,
                        context,
                        symbol_path,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let name = match name.as_str() {
                "ReadonlyArray" => "Array".to_owned(),
                "Array" => "Js.MutableArray".to_owned(),
                other => generated_name_simple(other, NameKind::Type),
            };
            Some(format!("{name}<{}>", rendered.join(", ")))
        }
        TypeKind::ReadonlyArray(element) => render_type(
            element,
            type_parameters,
            symbol,
            callback_parameter,
            context,
            symbol_path,
        )
        .map(|element| format!("Array<{element}>")),
        TypeKind::MutableArray(element) => render_type(
            element,
            type_parameters,
            symbol,
            callback_parameter,
            context,
            symbol_path,
        )
        .map(|element| format!("Js.MutableArray<{element}>")),
        TypeKind::Tuple(members) => members
            .iter()
            .map(|member| {
                render_type(
                    member,
                    type_parameters,
                    symbol,
                    callback_parameter,
                    context,
                    symbol_path,
                )
            })
            .collect::<Option<Vec<_>>>()
            .map(|members| format!("({})", members.join(", "))),
        TypeKind::Union(members) => {
            let mut value = None;
            let mut has_null = false;
            let mut has_undefined = false;
            for member in members {
                match &member.kind {
                    TypeKind::Literal(literal) if literal == "null" => has_null = true,
                    TypeKind::Literal(literal) if literal == "undefined" => has_undefined = true,
                    TypeKind::Primitive(literal) if literal == "null" => has_null = true,
                    TypeKind::Primitive(literal) if literal == "undefined" => has_undefined = true,
                    _ if value.is_none() => value = Some(member),
                    _ => {
                        return unsupported_type(
                            type_ref,
                            "arbitrary unions require an explicit discriminated-union adapter",
                            context,
                            symbol_path,
                        );
                    }
                }
            }
            let Some(value) = value else {
                return unsupported_type(
                    type_ref,
                    "nullish union has no value type",
                    context,
                    symbol_path,
                );
            };
            let mut rendered = render_type(
                value,
                type_parameters,
                symbol,
                callback_parameter,
                context,
                symbol_path,
            )?;
            if has_null {
                rendered = format!("Js.NullOr<{rendered}>");
            }
            if has_undefined {
                rendered = format!("Js.UndefinedOr<{rendered}>");
            }
            Some(rendered)
        }
        TypeKind::Function { parameters, result } => {
            let callback = callback_parameter.and_then(|name| symbol.callbacks.get(name));
            let arguments = parameters
                .iter()
                .map(|parameter| {
                    render_type(
                        &parameter.type_ref,
                        type_parameters,
                        symbol,
                        None,
                        context,
                        symbol_path,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let result_ref = promise_inner(result).unwrap_or(result);
            let result = render_type(
                result_ref,
                type_parameters,
                symbol,
                None,
                context,
                symbol_path,
            )?;
            if callback.is_some() {
                let mut signature = result;
                for argument in arguments.into_iter().rev() {
                    signature = format!("{argument} -> {signature}");
                }
                Some(format!("({signature})"))
            } else {
                let args = match arguments.as_slice() {
                    [] => "Unit".to_owned(),
                    [single] => single.clone(),
                    _ => format!("({})", arguments.join(", ")),
                };
                Some(format!("Js.Callback<{args}, {result}>"))
            }
        }
        TypeKind::Literal(value) if value == "null" => Some("Js.Null".to_owned()),
        TypeKind::Literal(value) if value == "undefined" => Some("Js.Undefined".to_owned()),
        TypeKind::Literal(_) | TypeKind::Unsupported(_) => unsupported_type(
            type_ref,
            "unsupported TypeScript type requires an explicit adapter",
            context,
            symbol_path,
        ),
    }
}

fn unsupported_type(
    type_ref: &TypeRef,
    message: &str,
    context: &mut RenderContext<'_>,
    symbol: &str,
) -> Option<String> {
    context.diagnostics.push(diagnostic(
        "SES-F0101",
        DiagnosticSeverity::Error,
        message,
        context.file,
        type_ref.span,
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

fn validate_symbol_settings(scope: &Scope, context: &mut RenderContext<'_>) {
    let mut paths = BTreeSet::new();
    collect_symbol_paths(scope, &[], &mut paths);
    for key in context.entry.symbols.keys() {
        if !paths.contains(key) {
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

fn collect_symbol_paths(scope: &Scope, namespace: &[String], output: &mut BTreeSet<String>) {
    for declaration in &scope.declarations {
        let name = match declaration {
            Declaration::Function(value) => &value.public_name,
            Declaration::Namespace(value) => &value.public_name,
            Declaration::Interface(value) => &value.public_name,
            Declaration::Class(value) | Declaration::Enum(value) => &value.public_name,
            Declaration::TypeAlias(value) => &value.public_name,
        };
        output.insert(symbol_path(namespace, name));
        if let Declaration::Namespace(value) = declaration {
            let mut nested = namespace.to_vec();
            nested.push(name.clone());
            collect_symbol_paths(&value.scope, &nested, output);
        }
    }
}

fn is_promise(type_ref: &TypeRef) -> bool {
    matches!(
        &type_ref.kind,
        TypeKind::Generic { name, .. } if matches!(name.as_str(), "Promise" | "PromiseLike")
    )
}

fn promise_inner(type_ref: &TypeRef) -> Option<&TypeRef> {
    match &type_ref.kind {
        TypeKind::Generic { name, arguments }
            if matches!(name.as_str(), "Promise" | "PromiseLike") =>
        {
            arguments.first()
        }
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

fn declaration_sort_key(declaration: &Declaration) -> (u8, String) {
    let kind = match declaration {
        Declaration::Interface(_)
        | Declaration::Class(_)
        | Declaration::Enum(_)
        | Declaration::TypeAlias(_) => 0,
        Declaration::Function(_) => 1,
        Declaration::Namespace(_) => 2,
    };
    (kind, declaration.original_name().to_owned())
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

pub(crate) fn parse_config(source: &str) -> Result<BindingsConfig, ConvertError> {
    toml::from_str::<BindingsConfig>(source)
        .map_err(|error| ConvertError::new(format!("invalid binding settings: {error}")))
}

pub(crate) fn validate_config(config: &BindingsConfig) -> Result<(), ConvertError> {
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

pub(crate) fn evaluation_name(value: Evaluation) -> &'static str {
    match value {
        Evaluation::Pure => "pure",
        Evaluation::Task => "task",
    }
}

pub(crate) fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn json_string(value: &impl Serialize) -> Result<String, ConvertError> {
    let mut bytes = serde_json::to_string_pretty(value)
        .map_err(|error| ConvertError::new(format!("failed to encode generated JSON: {error}")))?;
    bytes.push('\n');
    Ok(bytes)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindingMetadata {
    pub(crate) schema: u32,
    kind: String,
    generator: GeneratorIdentity,
    pub(crate) entry: String,
    declaration: String,
    pub(crate) output: String,
    pub(crate) specifier: String,
    pub(crate) host_module: HostModuleIdentity,
    pub(crate) evaluation: String,
    pub(crate) input_digest: String,
    pub(crate) settings_digest: String,
    symbols: Vec<GeneratedSymbol>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostModuleIdentity {
    pub specifier: String,
    pub exact_identity: String,
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
