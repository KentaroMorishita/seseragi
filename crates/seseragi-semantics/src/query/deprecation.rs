use super::*;
use seseragi_syntax::{Deprecation, SurfaceDecl, SurfacePattern};

pub(super) mod members;
pub(super) mod namespaces;
mod source_calls;

pub(super) struct SymbolDeprecation {
    pub value: Deprecation,
    own_declaration: Option<ByteSpan>,
}

/// Metadata follows an already-resolved symbol, never a spelling or an enclosing
/// type/trait. In particular, constructors and methods do not inherit a parent's
/// deprecation notice.
pub(super) fn metadata(resolved: &ResolvedModule) -> BTreeMap<SymbolId, SymbolDeprecation> {
    let mut result = BTreeMap::new();
    if resolved.deprecations.is_empty()
        && resolved
            .imports
            .iter()
            .all(|import| import.export.deprecation.is_none())
    {
        return result;
    }
    let declarations = resolved
        .declarations
        .iter()
        .map(|declaration| {
            let span = declaration.span();
            ((span.start, span.end), declaration)
        })
        .collect::<BTreeMap<_, _>>();
    let methods = resolved
        .declarations
        .iter()
        .flat_map(|declaration| {
            if let SurfaceDecl::Trait { methods, .. } = declaration {
                methods
                    .iter()
                    .map(|method| ((method.span.start, method.span.end), method.name_span))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        })
        .collect::<BTreeMap<_, _>>();
    let definitions = resolved
        .symbols
        .iter()
        .filter(|symbol| symbol.scope == ScopeId(0) && symbol.kind != SymbolKind::Constructor)
        .map(|symbol| {
            (
                (symbol.namespace, symbol.origin.start, symbol.origin.end),
                symbol.id,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for entry in &resolved.deprecations {
        let key = (entry.declaration.start, entry.declaration.end);
        let target = declarations
            .get(&key)
            .and_then(|declaration| {
                Some(match declaration {
                    SurfaceDecl::Fn { name_span, .. } | SurfaceDecl::EffectFn { name_span, .. } => {
                        (SymbolNamespace::Value, *name_span)
                    }
                    SurfaceDecl::Let {
                        pattern: SurfacePattern::Name { name_span, .. },
                        ..
                    } => (SymbolNamespace::Value, *name_span),
                    SurfaceDecl::Alias { name_span, .. }
                    | SurfaceDecl::Struct { name_span, .. }
                    | SurfaceDecl::Type { name_span, .. }
                    | SurfaceDecl::Newtype { name_span, .. } => (SymbolNamespace::Type, *name_span),
                    SurfaceDecl::Trait { name_span, .. } => (SymbolNamespace::Trait, *name_span),
                    SurfaceDecl::Operator { spelling_span, .. } => {
                        (SymbolNamespace::Operator, *spelling_span)
                    }
                    _ => return None,
                })
            })
            .or_else(|| {
                methods
                    .get(&key)
                    .map(|span| (SymbolNamespace::Value, *span))
            });
        let Some((namespace, definition)) = target else {
            continue;
        };
        if let Some(symbol) = definitions.get(&(namespace, definition.start, definition.end)) {
            result.insert(
                *symbol,
                SymbolDeprecation {
                    value: entry.deprecation.clone(),
                    own_declaration: Some(entry.declaration),
                },
            );
        }
    }
    for import in &resolved.imports {
        if import.export.constructor_of.is_some() {
            continue;
        }
        let Some(value) = &import.export.deprecation else {
            continue;
        };
        let Some(symbol) = resolved.symbols.get(import.symbol.0 as usize) else {
            continue;
        };
        if symbol.canonical.as_deref() != Some(import.export.symbol.as_str())
            || !(super::namespace_name(symbol.namespace) == import.export.namespace
                || import.member_owner.is_some()
                    && import.export.namespace == "method"
                    && symbol.namespace == SymbolNamespace::Value)
        {
            continue;
        }
        result.insert(
            import.symbol,
            SymbolDeprecation {
                value: value.clone(),
                own_declaration: None,
            },
        );
    }
    result
}

pub(super) fn references(
    resolved: &ResolvedModule,
    members: &[members::Member],
    selected_members: &[(&members::Member, ByteSpan)],
) -> Vec<Diagnostic> {
    let metadata = metadata(resolved);
    if metadata.is_empty() && members.iter().all(|member| member.deprecation.is_none()) {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    let mut import_ranges = BTreeMap::new();
    for import in &resolved.imports {
        let end = import_ranges
            .entry(import.origin.start)
            .or_insert(import.origin.end);
        *end = (*end).max(import.origin.end);
    }
    let mut warn = |target: SymbolId, origin: ByteSpan| {
        let Some(entry) = metadata.get(&target) else {
            return;
        };
        if entry.own_declaration.is_some_and(|declaration| {
            declaration.start <= origin.start && origin.end <= declaration.end
        }) {
            return;
        }
        let Some(symbol) = resolved.symbols.get(target.0 as usize) else {
            return;
        };
        let Some(identity) = symbol.canonical.as_deref() else {
            return;
        };
        if !seen.insert((
            origin.start,
            origin.end,
            symbol.namespace,
            identity.to_owned(),
        )) {
            return;
        }
        let since = entry
            .value
            .since
            .as_ref()
            .map(|version| format!(" since {version}"))
            .unwrap_or_default();
        result.push(Diagnostic {
            id: String::new(),
            code: "SES-L0201".to_owned(),
            severity: DiagnosticSeverity::Warning,
            message_key: "lint.deprecated-symbol".to_owned(),
            primary: ByteRange {
                start: origin.start,
                end: origin.end,
            },
            related: vec![RelatedDiagnostic {
                source: Some(entry.value.source.clone()),
                message: format!(
                    "`{}` is deprecated{since}: {}",
                    symbol.spelling, entry.value.message
                ),
                primary: ByteRange {
                    start: entry.value.origin.start,
                    end: entry.value.origin.end,
                },
            }],
            fixes: Vec::new(),
            type_difference: None,
        });
    };
    for reference in &resolved.references {
        // Import declarations introduce bindings, rather than using their API.
        // Reexports are handled explicitly below, once per exported binding.
        if import_ranges
            .range(..=reference.origin.start)
            .next_back()
            .is_some_and(|(_, end)| reference.origin.end <= *end)
        {
            continue;
        }
        if let Some(target) = reference.target {
            warn(target, reference.origin);
        }
    }
    for import in &resolved.imports {
        if import.reexported_as.is_some() {
            warn(import.symbol, import.origin);
        }
    }
    for (member, origin) in selected_members.iter().copied() {
        let Some(value) = member.deprecation.as_ref() else {
            continue;
        };
        if member
            .own_declaration
            .is_some_and(|own| own.start <= origin.start && origin.end <= own.end)
            || !seen.insert((
                origin.start,
                origin.end,
                SymbolNamespace::Value,
                member.identity.clone(),
            ))
        {
            continue;
        }
        let since = value
            .since
            .as_ref()
            .map(|version| format!(" since {version}"))
            .unwrap_or_default();
        result.push(Diagnostic {
            id: String::new(),
            code: "SES-L0201".to_owned(),
            severity: DiagnosticSeverity::Warning,
            message_key: "lint.deprecated-symbol".to_owned(),
            primary: ByteRange {
                start: origin.start,
                end: origin.end,
            },
            related: vec![RelatedDiagnostic {
                source: Some(value.source.clone()),
                message: format!("`{}` is deprecated{since}: {}", member.name, value.message),
                primary: ByteRange {
                    start: value.origin.start,
                    end: value.origin.end,
                },
            }],
            fixes: Vec::new(),
            type_difference: None,
        });
    }
    result
}
