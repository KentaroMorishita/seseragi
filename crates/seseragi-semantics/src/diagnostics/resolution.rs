use crate::{ResolveIssue, ResolvedModule, SymbolNamespace};
use seseragi_syntax::{
    ByteRange, ByteSpan, Diagnostic, DiagnosticEdit, DiagnosticFix, DiagnosticSeverity,
    RelatedDiagnostic, SurfaceDecl,
};

pub(super) fn collect_resolution_diagnostics(
    resolved: &ResolvedModule,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for issue in &resolved.issues {
        let primary = byte_range(issue.primary);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.primary == primary)
        {
            continue;
        }
        diagnostics.push(Diagnostic {
            type_difference: None,
            id: String::new(),
            code: issue.code.clone(),
            severity: DiagnosticSeverity::Error,
            message_key: issue.message_key.clone(),
            primary,
            related: issue_context(resolved, issue),
            fixes: issue
                .suggestion
                .iter()
                .map(|suggestion| DiagnosticFix {
                    title: format!("Replace with `{suggestion}`"),
                    edits: vec![DiagnosticEdit {
                        range: name_range(resolved, issue),
                        replacement: suggestion.clone(),
                    }],
                })
                .collect(),
        });
    }
}

/// The written name an issue reports, without the type arguments that may
/// follow it, so that a replacement keeps them.
fn name_range(resolved: &ResolvedModule, issue: &ResolveIssue) -> ByteRange {
    let primary = byte_range(issue.primary);
    resolved
        .references
        .iter()
        .find(|reference| reference.origin == issue.primary)
        .map(|reference| ByteRange {
            start: primary.start,
            end: (primary.start + reference.spelling.len()).min(primary.end),
        })
        .unwrap_or(primary)
}

/// A name that an imported namespace exports says how to reach it. Another
/// unresolved type name points at itself, not at the declaration around it.
fn issue_context(resolved: &ResolvedModule, issue: &ResolveIssue) -> Vec<RelatedDiagnostic> {
    let at_issue = |message: &str| {
        vec![RelatedDiagnostic {
            source: None,
            message: message.to_owned(),
            primary: byte_range(issue.primary),
        }]
    };
    if let Some(hint) = &issue.hint {
        return at_issue(hint);
    }
    if unresolved_type_reference(resolved, issue.primary).is_some() {
        return at_issue("type name");
    }
    if let Some(name) = unresolved_value_reference(resolved, issue) {
        return at_issue(&format!("`{name}` is not defined or imported here"));
    }
    containing_declaration(&resolved.declarations, issue.primary)
        .map(|declaration| {
            vec![RelatedDiagnostic {
                source: None,
                message: declaration_context(declaration).to_owned(),
                primary: byte_range(declaration.span()),
            }]
        })
        .unwrap_or_default()
}

fn unresolved_type_reference(resolved: &ResolvedModule, primary: ByteSpan) -> Option<&str> {
    resolved
        .references
        .iter()
        .find(|reference| {
            reference.origin == primary
                && reference.namespace == SymbolNamespace::Type
                && reference.target.is_none()
        })
        .map(|reference| reference.spelling.as_str())
}

fn unresolved_value_reference<'a>(
    resolved: &'a ResolvedModule,
    issue: &ResolveIssue,
) -> Option<&'a str> {
    (issue.message_key == "name.unresolved")
        .then(|| {
            resolved.references.iter().find(|reference| {
                reference.origin == issue.primary
                    && reference.namespace == SymbolNamespace::Value
                    && reference.target.is_none()
            })
        })
        .flatten()
        .map(|reference| reference.spelling.as_str())
}

/// The spellings of the type names that did not resolve.
pub(super) fn unresolved_type_names(resolved: &ResolvedModule) -> Vec<&str> {
    resolved
        .issues
        .iter()
        .filter(|issue| issue.code == "SES-N0001")
        .filter_map(|issue| unresolved_type_reference(resolved, issue.primary))
        .collect()
}

/// A type mismatch that involves an unresolved type name follows from that
/// name; the unresolved name is the error to fix. A declaration where the same
/// spelling resolves, such as its own type parameter `A`, keeps its errors.
pub(super) fn follows_from_unresolved_type(
    resolved: &ResolvedModule,
    diagnostic: &Diagnostic,
    names: &[&str],
) -> bool {
    let (expected, actual) = diagnostic.expected_actual_types();
    let scope = containing_declaration(
        &resolved.declarations,
        ByteSpan {
            start: diagnostic.primary.start,
            end: diagnostic.primary.end,
        },
    )
    .map(SurfaceDecl::span);
    let resolves_here = |name: &str| {
        scope.is_some_and(|scope| {
            resolved.references.iter().any(|reference| {
                reference.namespace == SymbolNamespace::Type
                    && reference.spelling == name
                    && reference.target.is_some()
                    && scope.start <= reference.origin.start
                    && reference.origin.end <= scope.end
            })
        })
    };
    [expected, actual].iter().flatten().any(|text| {
        names
            .iter()
            .any(|name| mentions_type_name(text, name) && !resolves_here(name))
    })
}

fn mentions_type_name(text: &str, name: &str) -> bool {
    let is_name = |character: char| character.is_alphanumeric() || character == '_';
    text.match_indices(name).any(|(start, _)| {
        let end = start + name.len();
        !text[..start]
            .chars()
            .next_back()
            .is_some_and(|before| is_name(before) || before == '.')
            && !text[end..].chars().next().is_some_and(is_name)
    })
}

fn containing_declaration(declarations: &[SurfaceDecl], primary: ByteSpan) -> Option<&SurfaceDecl> {
    declarations
        .iter()
        .filter(|declaration| {
            let span = declaration.span();
            span.start <= primary.start && primary.end <= span.end
        })
        .min_by_key(|declaration| {
            let span = declaration.span();
            span.end.saturating_sub(span.start)
        })
}

fn declaration_context(declaration: &SurfaceDecl) -> &'static str {
    match declaration {
        SurfaceDecl::Fn { .. } => "pure function body",
        SurfaceDecl::EffectFn { .. } => "effect function body",
        SurfaceDecl::Let { .. } => "let declaration",
        SurfaceDecl::Type { .. } => "type declaration",
        SurfaceDecl::Newtype { .. } => "newtype declaration",
        SurfaceDecl::Alias { .. } => "alias declaration",
        SurfaceDecl::Struct { .. } => "struct declaration",
        SurfaceDecl::Trait { .. } => "trait declaration",
        SurfaceDecl::Operator { .. } => "operator declaration",
        SurfaceDecl::Impl { .. } => "impl declaration",
        SurfaceDecl::Instance { .. } => "instance declaration",
    }
}

fn byte_range(span: ByteSpan) -> ByteRange {
    ByteRange {
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_unresolved_adt_payload_from_resolver_issues() {
        let artifact = crate::semantic_diagnostics(
            "artifact/unresolved-payload/main.ssrg",
            "type Label = | Present Strng\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-N0001");
        assert_eq!(artifact.diagnostics[0].message_key, "name.unresolved");
        assert_eq!(artifact.diagnostics[0].related[0].message, "type name");
    }

    #[test]
    fn reports_duplicate_constructor_from_resolver_issues() {
        let artifact = crate::semantic_diagnostics(
            "artifact/duplicate-constructor/main.ssrg",
            "type First = | Same\ntype Second = | Same\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-N0002");
        assert_eq!(
            artifact.diagnostics[0].message_key,
            "name.duplicate-definition"
        );
        assert_eq!(
            artifact.diagnostics[0].related[0].message,
            "type declaration"
        );
    }

    #[test]
    fn reports_an_unknown_instance_trait_at_the_trait_name() {
        let artifact = crate::semantic_diagnostics(
            "artifact/unknown-instance-trait/main.ssrg",
            "newtype Score = Int\ninstance Missing<Score> {}\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-N0001");
        assert_eq!(artifact.diagnostics[0].message_key, "name.unresolved");
        assert_eq!(
            artifact.diagnostics[0].primary,
            seseragi_syntax::ByteRange { start: 29, end: 36 }
        );
        assert_eq!(
            artifact.diagnostics[0].related[0].message,
            "instance declaration"
        );
    }

    #[test]
    fn keeps_later_local_functions_out_of_an_earlier_function_scope() {
        let artifact = crate::semantic_diagnostics(
            "artifact/local-forward-reference/main.ssrg",
            "fn broken value: Int -> Int = {\n\
               fn even current: Int -> Int = odd current\n\
               fn odd current: Int -> Int = current\n\
               even value\n\
             }\n",
        );

        assert_eq!(artifact.diagnostics.len(), 1);
        assert_eq!(artifact.diagnostics[0].code, "SES-N0001");
        assert_eq!(artifact.diagnostics[0].message_key, "name.unresolved");
    }
}
