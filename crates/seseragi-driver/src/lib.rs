//! Pure compiler orchestration for one already-identified Seseragi module.
//!
//! Filesystem discovery, package identity, and import linking belong to the
//! project layer. This crate owns the ordered frontend-to-backend pipeline and
//! never derives logical identity from a physical path.

mod analyze;
mod compile;
mod dependencies;
mod format;
mod input;
mod local_package;
mod local_project;
mod output;
mod output_plan;
mod project_compile;
mod provider_plan;
mod provider_requirements;
mod reporting;

pub use analyze::analyze_module;
pub use compile::{
    compile_linked_module, compile_linked_module_with_output_paths, compile_module,
    LinkedCompileError,
};
pub use format::{format_module, format_module_with_options};
pub use input::CompileInput;
pub use local_package::{compile_local_package, CompiledLocalPackage, LocalPackageCompileError};
pub use local_project::{
    compile_local_project, compile_local_project_with_providers, CompiledLocalProject,
    LocalProjectCompileError,
};
pub use output::CompiledModule;
pub use output_plan::{
    generated_output_paths, plan_typescript_outputs, TypeScriptInstanceOutput,
    TypeScriptModuleOutput, TypeScriptOutputPlanError,
};
pub use project_compile::{
    analyze_project, analyze_project_with_providers, compile_project,
    compile_project_with_providers, AnalyzedProject, CompiledProject, ProjectCompileError,
    ProjectModuleDiagnostics, ProjectModuleInput,
};
pub use provider_plan::{
    ProjectProviderConfiguration, ProjectProviderDiagnostic, ProviderDiagnosticDetails,
};
pub use provider_requirements::{main_provider_requirements, ProviderRequirementError};
pub use reporting::render_terminal_diagnostics;
pub use seseragi_formatter::{FormatOptions, FormattedSource, DEFAULT_LINE_WIDTH};
pub use seseragi_provider::{
    is_builtin_service, validate_provider_target, CompilerFeatureRequirement,
    ProviderCompatibilityContext, ProviderCompatibilityError, ProviderConformanceRequirement,
    ProviderTargetMismatch, RuntimePackageCompatibility, TargetExtensionRequirement,
};
pub use seseragi_provider::{
    resolve_providers, CandidateRejection, CandidateRejectionReason, CandidateVisibility,
    ContractRequirement, ContractVersion, HostPackageRequirement, LogicalRecordField, LogicalType,
    OperationKind, Portability, PrimitiveType, ProviderBackend, ProviderBuildMetadata,
    ProviderCandidate, ProviderContract, ProviderContractError, ProviderContractKind,
    ProviderEntry, ProviderErrorContext, ProviderLockMetadata, ProviderManifest,
    ProviderManifestError, ProviderManifestKind, ProviderOperation, ProviderPackageMetadata,
    ProviderRequirements, ProviderResolution, ProviderResolutionContext, ProviderResolutionError,
    ProviderSelectionMetadata, ProviderSelectionSource, RequiredService, RequirementTrace,
    ResolvedHostPackage, ServiceRequirement,
};
pub use seseragi_semantics::{
    AnalysisCallable, AnalysisCallableOccurrence, AnalysisCompletionContext,
    AnalysisCompletionField, AnalysisDocument, AnalysisParameter, AnalysisReferenceItem,
    AnalysisSymbol, AnalysisSymbolOccurrence, AnalysisTypeOccurrence, TypeCallableDocument,
    TypeCallableParameterDocument, TypeConstraintDocument, TypeDocument, TypeDocumentField,
    TypeParameterDocument, TypeRenderLayout, TypeRenderMarkup, TypeRenderOptions,
    TypeSchemeDocument,
};
