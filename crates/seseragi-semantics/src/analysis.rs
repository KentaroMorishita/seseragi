use crate::{diagnostics, resolve, typed, ResolvedModule, TypedModule, TypedModuleInterface};
use seseragi_syntax::{DiagnosticArtifact, DiagnosticSeverity, ModuleInterface};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyzedModule {
    pub diagnostics: DiagnosticArtifact,
    pub resolved: ResolvedModule,
    pub typed_hir: TypedModule,
    pub typed_interface: TypedModuleInterface,
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
    let resolved = resolve::resolve_module_from_interface(interface, source);
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
    let resolved = resolve::resolve_linked_module(linked, source);
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
    })
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}
