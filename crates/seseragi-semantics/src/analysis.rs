use crate::{diagnostics, resolve, typed, ResolvedModule, TypedModule, TypedModuleInterface};
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity, ModuleInterface};

/// A source occurrence whose nominal receiver owner was proved in the actual
/// typing environment. This editor fact is kept beside, rather than inside,
/// Typed HIR and never changes type or dictionary contracts.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisNominalReceiver {
    pub range: seseragi_syntax::ByteSpan,
    pub owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyzedModule {
    pub diagnostics: DiagnosticArtifact,
    pub resolved: ResolvedModule,
    pub typed_hir: TypedModule,
    pub typed_interface: TypedModuleInterface,
    pub nominal_receivers: Vec<AnalysisNominalReceiver>,
}

/// Resolves and types one syntax-checked module interface.
///
/// The caller owns physical source discovery and canonical module identity.
/// Passing the interface explicitly keeps those concerns out of semantic
/// analysis and lets the typed implementation and public interface share one
/// resolution and typing result.
pub fn analyze_module_interface(
    diagnostics: DiagnosticArtifact,
    interface: ModuleInterface,
    source: &str,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let shallow = interface.clone();
    let mut resolved = resolve::resolve_module_from_interface(interface, source);
    retain_deprecation_source(&mut resolved, &diagnostics.source);
    let diagnostics =
        diagnostics::semantic_diagnostics_from_resolved(diagnostics, &resolved, source);
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let (typed_hir, typed_interface) =
        typed::type_resolved_module_with_public_interface(shallow, resolved.clone());
    Ok(AnalyzedModule {
        diagnostics,
        resolved,
        typed_hir,
        typed_interface,
        nominal_receivers: Vec::new(),
    })
}

/// Resolves and types a module whose dependency identities and public
/// contracts were already fixed by the project linker.
pub fn analyze_linked_module(
    diagnostics: DiagnosticArtifact,
    linked: seseragi_project::LinkedModule,
    source: &str,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let shallow = linked.interface.clone();
    let mut resolved = resolve::resolve_linked_module(linked, source);
    retain_deprecation_source(&mut resolved, &diagnostics.source);
    let diagnostics =
        diagnostics::semantic_diagnostics_from_resolved(diagnostics, &resolved, source);
    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    let (typed_hir, typed_interface) =
        typed::type_resolved_module_with_public_interface(shallow, resolved.clone());
    Ok(AnalyzedModule {
        diagnostics,
        resolved,
        typed_hir,
        typed_interface,
        nominal_receivers: Vec::new(),
    })
}

/// Resolves and types a linked module while retaining recovery HIR when
/// syntax or semantic diagnostics are present. Recovery error expressions stay
/// invalid holes; all original diagnostics remain in the snapshot. Interactive
/// analysis needs that typed structure to answer expected-type queries at the invalid expression that
/// the user is actively completing. Compile paths continue to use
/// [`analyze_linked_module`] and stop before lowering on the same diagnostics.
pub fn analyze_linked_module_recovering(
    diagnostics: DiagnosticArtifact,
    linked: seseragi_project::LinkedModule,
    source: &str,
) -> Result<AnalyzedModule, DiagnosticArtifact> {
    let shallow = linked.interface.clone();
    let mut resolved = resolve::resolve_linked_module(linked, source);
    retain_deprecation_source(&mut resolved, &diagnostics.source);
    // A parser-invalid declaration may be absent or partially inferred in the
    // recovered interface. Its users can then produce unrelated-looking type
    // errors (for example Show<A> at a later call). Source-range filtering
    // cannot distinguish those cascades from valid semantic diagnostics.
    // Keep the parser artifact authoritative until the module parses, while
    // retaining best-effort resolved/typed facts for interactive queries.
    let diagnostics = if has_errors(&diagnostics) {
        diagnostics
    } else {
        let original_diagnostics = diagnostics.diagnostics.clone();
        let mut diagnostics =
            diagnostics::semantic_diagnostics_from_resolved(diagnostics, &resolved, source);
        if !original_diagnostics.is_empty() {
            diagnostics.diagnostics = original_diagnostics
                .into_iter()
                .chain(diagnostics.diagnostics)
                .enumerate()
                .map(|(index, mut diagnostic)| {
                    diagnostic.id = format!("d{}", index + 1);
                    diagnostic
                })
                .collect();
        }
        diagnostics
    };
    let (typed_hir, typed_interface, nominal_receivers) =
        typed::type_resolved_module_with_analysis(shallow, resolved.clone());
    Ok(AnalyzedModule {
        diagnostics,
        resolved,
        typed_hir,
        typed_interface,
        nominal_receivers,
    })
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

fn retain_deprecation_source(resolved: &mut ResolvedModule, source: &str) {
    for entry in &mut resolved.deprecations {
        entry.deprecation.source = source.to_owned();
    }
}
