use super::*;
use crate::ResolvedNamespaceDeprecation;
use seseragi_syntax::{lex, Token, TokenKind};

pub(in crate::query) struct NamespaceUse<'a> {
    entry: &'a ResolvedNamespaceDeprecation,
    range: ByteSpan,
}

pub(in crate::query) fn uses<'a>(
    resolved: &'a ResolvedModule,
    source: Option<&str>,
) -> Vec<NamespaceUse<'a>> {
    if resolved.namespace_deprecations.is_empty() {
        return Vec::new();
    }
    let tokens = source.map(|source| {
        lex(&resolved.source, source)
            .tokens
            .into_iter()
            .filter(|token| {
                !matches!(
                    token.kind,
                    TokenKind::TriviaSpace
                        | TokenKind::TriviaNewline
                        | TokenKind::TriviaComment
                        | TokenKind::Eof
                )
            })
            .collect::<Vec<_>>()
    });
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in &resolved.namespace_deprecations {
        if let Some(range) = entry.reexport_origin {
            if seen.insert((range.start, range.end, entry.metadata.symbol.as_str())) {
                result.push(NamespaceUse { entry, range });
            }
        }
    }
    let Some(tokens) = tokens else { return result };
    for reference in &resolved.references {
        let Some(identity) = reference
            .target
            .and_then(|id| resolved.symbols.get(id.0 as usize))
            .and_then(|symbol| symbol.canonical.as_deref())
        else {
            continue;
        };
        if resolved.dependencies.iter().any(|dependency| {
            dependency.origin.start <= reference.origin.start
                && reference.origin.end <= dependency.origin.end
        }) {
            continue;
        }
        for entry in &resolved.namespace_deprecations {
            if !entry
                .metadata
                .members
                .iter()
                .any(|member| member == identity)
                || entry.own_declaration.is_some_and(|own| {
                    own.start <= reference.origin.start && reference.origin.end <= own.end
                })
            {
                continue;
            }
            let Some(range) = owner_token(&tokens, reference, &entry.path) else {
                continue;
            };
            if seen.insert((range.start, range.end, entry.metadata.symbol.as_str())) {
                result.push(NamespaceUse { entry, range });
            }
        }
    }
    result
}

fn owner_token(
    tokens: &[Token],
    reference: &crate::ResolvedReference,
    path: &str,
) -> Option<ByteSpan> {
    let owner = path.split('.').collect::<Vec<_>>();
    let spelling = reference.spelling.split('.').collect::<Vec<_>>();
    if spelling.len() <= owner.len() || !spelling.starts_with(&owner) {
        return None;
    }
    let start = tokens.partition_point(|token| token.end <= reference.origin.start);
    let reference_tokens = &tokens[start..];
    for (index, name) in spelling.iter().enumerate() {
        let token = reference_tokens.get(index * 2)?;
        if token.start < reference.origin.start
            || token.end > reference.origin.end
            || !matches!(
                token.kind,
                TokenKind::IdentifierLower | TokenKind::IdentifierUpper
            )
            || token.raw != *name
        {
            return None;
        }
        if index + 1 < spelling.len()
            && reference_tokens.get(index * 2 + 1)?.kind != TokenKind::PunctuationDot
        {
            return None;
        }
    }
    let token = reference_tokens.get((owner.len() - 1) * 2)?;
    Some(ByteSpan {
        start: token.start,
        end: token.end,
    })
}

pub(in crate::query) fn project(
    resolved: &ResolvedModule,
    uses: &[NamespaceUse<'_>],
    symbols: &mut Vec<AnalysisSymbol>,
    occurrences: &mut Vec<AnalysisSymbolOccurrence>,
) {
    for entry in &resolved.namespace_deprecations {
        let existing = symbols.iter().position(|symbol| {
            symbol.identity == entry.metadata.symbol
                && symbol.namespace == "module"
                && symbol.name == entry.path
        });
        let id = if let Some(index) = existing {
            symbols[index].deprecation = Some(entry.metadata.deprecation.clone());
            symbols[index].kind = "foreign-namespace".to_owned();
            symbols[index].id
        } else {
            let id = symbols.len() as u32;
            symbols.push(AnalysisSymbol {
                id,
                identity: entry.metadata.symbol.clone(),
                name: entry.path.clone(),
                module: super::super::module_from_identity(
                    &entry.metadata.symbol,
                    &resolved.module,
                ),
                namespace: "module".to_owned(),
                kind: "foreign-namespace".to_owned(),
                definition: if entry.own_declaration.is_some() {
                    entry.metadata.name_span
                } else {
                    ByteSpan { start: 0, end: 0 }
                },
                type_name: None,
                multiline_type_name: None,
                type_document: None,
                callable: None,
                description: None,
                deprecation: Some(entry.metadata.deprecation.clone()),
                member_owner: None,
                scope: ScopeId(0),
            });
            if entry.own_declaration.is_some() {
                occurrences.push(AnalysisSymbolOccurrence {
                    range: entry.metadata.name_span,
                    symbol: id,
                });
            }
            id
        };
        for usage in uses.iter().filter(|usage| std::ptr::eq(usage.entry, entry)) {
            occurrences.push(AnalysisSymbolOccurrence {
                range: usage.range,
                symbol: id,
            });
        }
    }
}

pub(in crate::query) fn warnings(uses: &[NamespaceUse<'_>]) -> Vec<Diagnostic> {
    uses.iter()
        .map(|usage| {
            let notice = &usage.entry.metadata.deprecation;
            let since = notice
                .since
                .as_ref()
                .map(|version| format!(" since {version}"))
                .unwrap_or_default();
            Diagnostic {
                id: String::new(),
                code: "SES-L0201".to_owned(),
                severity: DiagnosticSeverity::Warning,
                message_key: "lint.deprecated-symbol".to_owned(),
                primary: ByteRange {
                    start: usage.range.start,
                    end: usage.range.end,
                },
                related: vec![RelatedDiagnostic {
                    source: Some(notice.source.clone()),
                    message: format!(
                        "`{}` is deprecated{since}: {}",
                        usage.entry.path, notice.message
                    ),
                    primary: ByteRange {
                        start: notice.origin.start,
                        end: notice.origin.end,
                    },
                }],
                fixes: Vec::new(),
                type_difference: None,
            }
        })
        .collect()
}
