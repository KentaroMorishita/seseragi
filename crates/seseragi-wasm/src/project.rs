use serde::{Deserialize, Serialize};
use seseragi_driver::{
    analyze_project as analyze_driver_project, analyze_project_with_providers,
    compile_project as compile_driver_project, compile_project_with_providers, format_module,
    CandidateVisibility, CompilerFeatureRequirement, ContractVersion, LinkedCompileError,
    ProjectCompileError, ProjectModuleInput, ProjectProviderConfiguration, ProviderCandidate,
    ProviderCompatibilityContext, ProviderConformanceRequirement, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ResolvedHostPackage, RuntimePackageCompatibility, ServiceRequirement,
    TargetExtensionRequirement, TypeScriptOutputPlanError,
};
use seseragi_lowering::{GeneratedBundle, TypeScriptLoweringError};
use seseragi_project::{
    classify_specifier, resolve_relative_specifier, ImportSpecifier, LinkError, LinkTargetError,
    ModuleGraph, ModuleGraphError, ModulePath,
};
use seseragi_runtime::{project_main_contract, MainContract};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, ByteSpan, DiagnosticArtifact,
    DiagnosticSeverity,
};
use std::collections::{BTreeMap, BTreeSet};
use wasm_bindgen::prelude::*;

const PROJECT_SCHEMA: u32 = 1;
const PACKAGE_SCOPE: &str = "playground";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRequest {
    schema: u32,
    entry: String,
    files: Vec<ProjectSource>,
    #[serde(default)]
    provider: Option<ProjectProviderRequest>,
}

#[derive(Clone, Deserialize)]
struct ProjectSource {
    path: String,
    source: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectProviderRequest {
    target: String,
    backend_family: String,
    backend_abi_major: u64,
    #[serde(default)]
    runtime_features: Vec<String>,
    #[serde(default)]
    explicit: BTreeMap<String, String>,
    #[serde(default)]
    defaults: BTreeMap<String, String>,
    #[serde(default)]
    contracts: Vec<serde_json::Value>,
    #[serde(default)]
    candidates: Vec<ProjectProviderCandidate>,
    #[serde(default)]
    transitive_requirements: Vec<ProjectRequiredService>,
    #[serde(default)]
    compatibility: ProjectProviderCompatibility,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectProviderCandidate {
    manifest: serde_json::Value,
    contract: serde_json::Value,
    visibility: ProjectProviderVisibility,
    package: ProjectProviderPackage,
    artifact_digest: String,
    #[serde(default)]
    host_packages: Vec<ProjectResolvedHostPackage>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ProjectProviderVisibility {
    ToolchainBuiltin,
    RootDirectDependency,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectProviderPackage {
    version: String,
    source_identity: String,
    content_digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectResolvedHostPackage {
    name: String,
    version: String,
    source_identity: String,
    content_digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectRequiredService {
    field: String,
    service: String,
    contract_version: ContractVersion,
    traces: Vec<ProjectRequirementTrace>,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectRequirementTrace {
    package: String,
    module: String,
    source: String,
    start: u32,
    end: u32,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectProviderCompatibility {
    #[serde(default)]
    target_extensions: Vec<ProjectTargetExtension>,
    #[serde(default)]
    runtime_packages: Vec<ProjectRuntimePackage>,
    #[serde(default)]
    compiler_features: Vec<ProjectCompilerFeatures>,
    #[serde(default)]
    conformance: Vec<ProjectProviderConformance>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectTargetExtension {
    extension: String,
    trace: ProjectRequirementTrace,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectRuntimePackage {
    provider: String,
    required_identity: String,
    required_digest: String,
    actual_identity: String,
    actual_digest: String,
    trace: ProjectRequirementTrace,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectCompilerFeatures {
    provider: String,
    required: Vec<String>,
    supported: Vec<String>,
    trace: ProjectRequirementTrace,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProjectProviderConformance {
    provider: String,
    required_profile: String,
    required_digest: String,
    actual_profile: Option<String>,
    actual_digest: Option<String>,
    trace: ProjectRequirementTrace,
}

struct BrowserProject {
    graph: ModuleGraph<String>,
    inputs: Vec<ProjectModuleInput>,
    entry_module: String,
    paths: BTreeMap<String, String>,
    provider: Option<ProjectProviderConfiguration>,
}

#[derive(Clone, Copy, Serialize)]
struct SourceRange {
    start: usize,
    end: usize,
}

impl From<ByteSpan> for SourceRange {
    fn from(value: ByteSpan) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProblem {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary: Option<SourceRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<ProjectProviderProblemDetails>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectProviderProblemDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend_abi_major: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    candidates: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    compatible_targets: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    reasons: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    required: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    actual: Vec<String>,
}

impl ProjectProblem {
    fn workspace(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            path: None,
            primary: None,
            label: None,
            details: None,
        }
    }

    fn file(
        code: &str,
        message: impl Into<String>,
        path: impl Into<String>,
        primary: Option<ByteSpan>,
    ) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            path: Some(path.into()),
            primary: primary.map(SourceRange::from),
            label: None,
            details: None,
        }
    }
}

#[derive(Serialize)]
struct ProjectFileDiagnostics {
    path: String,
    diagnostics: DiagnosticArtifact,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectGeneratedModule {
    path: String,
    module: String,
    generated: GeneratedBundle,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectEntry {
    path: String,
    module: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    contract: Option<MainContract>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectCompileResponse {
    Success {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        modules: Vec<ProjectGeneratedModule>,
        entry: ProjectEntry,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectAnalysisDocument {
    path: String,
    module: String,
    document: seseragi_driver::AnalysisDocument,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectAnalysisResponse {
    Success {
        schema: u32,
        documents: Vec<ProjectAnalysisDocument>,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
enum ProjectFormatResponse {
    Success {
        schema: u32,
        path: String,
        source: String,
        changed: bool,
    },
    Failure {
        schema: u32,
        diagnostics: Vec<ProjectFileDiagnostics>,
        problems: Vec<ProjectProblem>,
    },
}

struct ProjectFailure {
    diagnostics: Vec<ProjectFileDiagnostics>,
    problems: Vec<ProjectProblem>,
}

impl ProjectFailure {
    fn problem(problem: ProjectProblem) -> Self {
        Self {
            diagnostics: Vec::new(),
            problems: vec![problem],
        }
    }

    fn diagnostics(path: impl Into<String>, diagnostics: DiagnosticArtifact) -> Self {
        Self {
            diagnostics: vec![ProjectFileDiagnostics {
                path: path.into(),
                diagnostics,
            }],
            problems: Vec::new(),
        }
    }

    fn diagnostics_many(diagnostics: Vec<ProjectFileDiagnostics>) -> Self {
        Self {
            diagnostics,
            problems: Vec::new(),
        }
    }
}

/// Compiles an in-memory browser workspace through the shared project driver.
#[wasm_bindgen]
pub fn compile_project(request: &str) -> String {
    let response = match prepare_project(request) {
        Ok(project) => compile_prepared_project(project),
        Err(failure) => ProjectCompileResponse::Failure {
            schema: PROJECT_SCHEMA,
            diagnostics: failure.diagnostics,
            problems: failure.problems,
        },
    };
    serde_json::to_string(&response).expect("project compile response must serialize")
}

/// Analyzes every source in an in-memory browser workspace after linking its
/// local imports to the same typed public interfaces used by compilation.
#[wasm_bindgen]
pub fn analyze_project(request: &str) -> String {
    let response = match prepare_project(request) {
        Ok(project) => analyze_prepared_project(project),
        Err(failure) => ProjectAnalysisResponse::Failure {
            schema: PROJECT_SCHEMA,
            diagnostics: failure.diagnostics,
            problems: failure.problems,
        },
    };
    serde_json::to_string(&response).expect("project analysis response must serialize")
}

/// Formats one path selected from the versioned workspace request.
#[wasm_bindgen]
pub fn format_project_file(request: &str, path: &str) -> String {
    let response =
        match parse_request(request).and_then(|request| select_format_source(request, path)) {
            Ok(source) => match format_module(path, &source) {
                Ok(formatted) => ProjectFormatResponse::Success {
                    schema: PROJECT_SCHEMA,
                    path: path.to_owned(),
                    source: formatted.text,
                    changed: formatted.changed,
                },
                Err(diagnostics) => ProjectFormatResponse::Failure {
                    schema: PROJECT_SCHEMA,
                    diagnostics: vec![ProjectFileDiagnostics {
                        path: path.to_owned(),
                        diagnostics,
                    }],
                    problems: Vec::new(),
                },
            },
            Err(failure) => ProjectFormatResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            },
        };
    serde_json::to_string(&response).expect("project format response must serialize")
}

fn compile_prepared_project(project: BrowserProject) -> ProjectCompileResponse {
    let BrowserProject {
        graph,
        inputs,
        entry_module,
        paths,
        provider,
    } = project;
    let compiled = match provider {
        Some(configuration) => compile_project_with_providers(graph, inputs, configuration),
        None => compile_driver_project(graph, inputs),
    };
    match compiled {
        Ok(compiled) => {
            let diagnostics = compiled
                .order
                .iter()
                .filter_map(|module| {
                    let compiled_module = compiled.modules.get(module)?;
                    (!compiled_module.diagnostics.diagnostics.is_empty()).then(|| {
                        ProjectFileDiagnostics {
                            path: path_for_module(&paths, module),
                            diagnostics: compiled_module.diagnostics.clone(),
                        }
                    })
                })
                .collect();
            let modules = compiled
                .order
                .iter()
                .map(|module| ProjectGeneratedModule {
                    path: path_for_module(&paths, module),
                    module: module.clone(),
                    generated: compiled
                        .modules
                        .get(module)
                        .expect("compiled order names a module")
                        .generated
                        .clone(),
                })
                .collect();
            let (contract, error) = match project_main_contract(&compiled, &entry_module) {
                Ok(contract) => (Some(contract), None),
                Err(error) => (None, Some(error)),
            };
            ProjectCompileResponse::Success {
                schema: PROJECT_SCHEMA,
                diagnostics,
                modules,
                entry: ProjectEntry {
                    path: path_for_module(&paths, &entry_module),
                    module: entry_module,
                    contract,
                    error,
                },
            }
        }
        Err(error) => {
            let failure = driver_failure(error, &paths);
            ProjectCompileResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            }
        }
    }
}

fn analyze_prepared_project(project: BrowserProject) -> ProjectAnalysisResponse {
    let BrowserProject {
        graph,
        inputs,
        paths,
        provider,
        ..
    } = project;
    let analyzed = match provider {
        Some(configuration) => analyze_project_with_providers(graph, inputs, configuration),
        None => analyze_driver_project(graph, inputs),
    };
    match analyzed {
        Ok(analyzed) => ProjectAnalysisResponse::Success {
            schema: PROJECT_SCHEMA,
            documents: analyzed
                .order
                .iter()
                .map(|module| ProjectAnalysisDocument {
                    path: path_for_module(&paths, module),
                    module: module.clone(),
                    document: analyzed
                        .documents
                        .get(module)
                        .expect("analysis order names a document")
                        .clone(),
                })
                .collect(),
        },
        Err(error) => {
            let failure = driver_failure(error, &paths);
            ProjectAnalysisResponse::Failure {
                schema: PROJECT_SCHEMA,
                diagnostics: failure.diagnostics,
                problems: failure.problems,
            }
        }
    }
}

fn prepare_project(request: &str) -> Result<BrowserProject, ProjectFailure> {
    let request = parse_request(request)?;
    if request.files.is_empty() {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            "workspace must contain at least one source file",
        )));
    }

    let ProjectRequest {
        files,
        entry,
        provider,
        ..
    } = request;
    let mut sources = BTreeMap::<String, ProjectSource>::new();
    let mut modules_by_path = BTreeMap::<String, String>::new();
    let mut paths = BTreeMap::<String, String>::new();
    for file in files {
        let (canonical_path, module_path, module) = source_identity(&file.path)?;
        if canonical_path != file.path {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("source path must be normalized as `{canonical_path}`"),
                file.path,
                None,
            )));
        }
        if sources.contains_key(&canonical_path) || paths.contains_key(&module) {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                "workspace contains a duplicate source path or module identity",
                canonical_path,
                None,
            )));
        }
        modules_by_path.insert(module_path, module.clone());
        paths.insert(module, canonical_path.clone());
        sources.insert(canonical_path, file);
    }

    let (entry_path, _, entry_module) = source_identity(&entry)?;
    if entry_path != entry || !sources.contains_key(&entry_path) {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("entry source `{}` is not present in the workspace", entry),
        )));
    }

    let mut graph = ModuleGraph::new();
    let mut inputs = Vec::with_capacity(sources.len());
    for (path, file) in &sources {
        let (_, current_path, module) = source_identity(path)?;
        let diagnostics = parse_diagnostics(path, &file.source);
        let has_parse_errors = diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error);
        if has_parse_errors {
            graph
                .add_module(module.clone(), [])
                .map_err(|error| graph_failure(error, &paths))?;
            inputs.push(
                ProjectModuleInput::new(
                    path,
                    module.clone(),
                    &file.source,
                    format!("{current_path}.js"),
                )
                .with_package_scope(PACKAGE_SCOPE),
            );
            continue;
        }
        let unlinked = parse_unlinked_module_interface(path, &module, &file.source);
        let mut dependencies = BTreeMap::new();
        for import in unlinked.imports {
            let target_path = match classify_specifier(&import.specifier) {
                Ok(ImportSpecifier::Standard(_)) => continue,
                Ok(ImportSpecifier::Relative(specifier)) => resolve_relative_specifier(
                    &ModulePath::parse(&current_path).expect("validated path"),
                    &specifier,
                )
                .map_err(|error| {
                    ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!("cannot resolve import `{}`: {error}", import.specifier),
                        path,
                        Some(import.span),
                    ))
                })?
                .as_str()
                .to_owned(),
                Ok(ImportSpecifier::SelfPackage(target)) => ModulePath::parse(&target)
                    .map_err(|error| {
                        ProjectFailure::problem(ProjectProblem::file(
                            "SES-N0104",
                            format!("cannot resolve import `{}`: {error}", import.specifier),
                            path,
                            Some(import.span),
                        ))
                    })?
                    .as_str()
                    .to_owned(),
                Ok(ImportSpecifier::Generated(_)) | Ok(ImportSpecifier::Package(_)) => {
                    return Err(ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!(
                            "browser workspace does not resolve import `{}`",
                            import.specifier
                        ),
                        path,
                        Some(import.span),
                    )));
                }
                Err(error) => {
                    return Err(ProjectFailure::problem(ProjectProblem::file(
                        "SES-N0104",
                        format!("cannot resolve import `{}`: {error}", import.specifier),
                        path,
                        Some(import.span),
                    )));
                }
            };
            let Some(dependency) = modules_by_path.get(&target_path) else {
                return Err(ProjectFailure::problem(ProjectProblem::file(
                    "SES-N0104",
                    format!(
                        "import `{}` resolves to missing module `{target_path}`",
                        import.specifier
                    ),
                    path,
                    Some(import.span),
                )));
            };
            if dependencies
                .insert(import.specifier.clone(), dependency.clone())
                .is_some()
            {
                return Err(ProjectFailure::problem(ProjectProblem::file(
                    "SES-N0101",
                    format!("duplicate import specifier `{}`", import.specifier),
                    path,
                    Some(import.span),
                )));
            }
        }
        graph
            .add_module(module.clone(), dependencies)
            .map_err(|error| graph_failure(error, &paths))?;
        inputs.push(
            ProjectModuleInput::new(path, &module, &file.source, format!("{current_path}.js"))
                .with_package_scope(PACKAGE_SCOPE),
        );
    }
    graph
        .topological_order()
        .map_err(|error| graph_failure(error, &paths))?;
    let provider = provider
        .map(|provider| prepare_provider_configuration(provider, &entry_module, &entry_path))
        .transpose()?;

    Ok(BrowserProject {
        graph,
        inputs,
        entry_module,
        paths,
        provider,
    })
}

fn prepare_provider_configuration(
    request: ProjectProviderRequest,
    entry_module: &str,
    entry_path: &str,
) -> Result<ProjectProviderConfiguration, ProjectFailure> {
    if request.target.trim().is_empty()
        || request.backend_family.trim().is_empty()
        || request.backend_abi_major == 0
    {
        return Err(provider_request_failure(
            entry_path,
            "provider target, backend family and non-zero ABI major are required",
        ));
    }
    let contracts = request
        .contracts
        .into_iter()
        .map(|contract| {
            ProviderContract::from_json(&contract.to_string()).map_err(|error| {
                provider_request_failure(
                    entry_path,
                    format!("invalid Provider Contract artifact: {error}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let candidates = request
        .candidates
        .into_iter()
        .map(|candidate| {
            let manifest =
                ProviderManifest::from_json(&candidate.manifest.to_string()).map_err(|error| {
                    provider_request_failure(
                        entry_path,
                        format!("invalid provider manifest artifact: {error}"),
                    )
                })?;
            let contract =
                ProviderContract::from_json(&candidate.contract.to_string()).map_err(|error| {
                    provider_request_failure(
                        entry_path,
                        format!("invalid provider Contract artifact: {error}"),
                    )
                })?;
            Ok(ProviderCandidate {
                manifest,
                contract,
                visibility: match candidate.visibility {
                    ProjectProviderVisibility::ToolchainBuiltin => {
                        CandidateVisibility::ToolchainBuiltin
                    }
                    ProjectProviderVisibility::RootDirectDependency => {
                        CandidateVisibility::RootDirectDependency
                    }
                },
                package: ProviderPackageMetadata {
                    version: candidate.package.version,
                    source_identity: candidate.package.source_identity,
                    content_digest: candidate.package.content_digest,
                },
                artifact_digest: candidate.artifact_digest,
                host_packages: candidate
                    .host_packages
                    .into_iter()
                    .map(|package| ResolvedHostPackage {
                        name: package.name,
                        version: package.version,
                        source_identity: package.source_identity,
                        content_digest: package.content_digest,
                    })
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, ProjectFailure>>()?;
    let transitive_requirements = request
        .transitive_requirements
        .into_iter()
        .map(|requirement| RequiredService {
            requirement: ServiceRequirement {
                field: requirement.field,
                service: requirement.service,
            },
            contract_version: requirement.contract_version,
            traces: requirement
                .traces
                .into_iter()
                .map(requirement_trace)
                .collect(),
        })
        .collect();
    let compatibility = ProviderCompatibilityContext {
        target_extensions: request
            .compatibility
            .target_extensions
            .into_iter()
            .map(|requirement| TargetExtensionRequirement {
                extension: requirement.extension,
                trace: requirement_trace(requirement.trace),
            })
            .collect(),
        runtime_packages: request
            .compatibility
            .runtime_packages
            .into_iter()
            .map(|requirement| RuntimePackageCompatibility {
                provider: requirement.provider,
                required_identity: requirement.required_identity,
                required_digest: requirement.required_digest,
                actual_identity: requirement.actual_identity,
                actual_digest: requirement.actual_digest,
                trace: requirement_trace(requirement.trace),
            })
            .collect(),
        compiler_features: request
            .compatibility
            .compiler_features
            .into_iter()
            .map(|requirement| CompilerFeatureRequirement {
                provider: requirement.provider,
                required: requirement.required.into_iter().collect(),
                supported: requirement.supported.into_iter().collect(),
                trace: requirement_trace(requirement.trace),
            })
            .collect(),
        conformance: request
            .compatibility
            .conformance
            .into_iter()
            .map(|requirement| ProviderConformanceRequirement {
                provider: requirement.provider,
                required_profile: requirement.required_profile,
                required_digest: requirement.required_digest,
                actual_profile: requirement.actual_profile,
                actual_digest: requirement.actual_digest,
                trace: requirement_trace(requirement.trace),
            })
            .collect(),
    };
    Ok(ProjectProviderConfiguration {
        entry_module: entry_module.to_owned(),
        contracts,
        candidates,
        context: ProviderResolutionContext {
            target: request.target,
            backend_family: request.backend_family,
            backend_abi_major: request.backend_abi_major,
            runtime_features: request.runtime_features.into_iter().collect(),
            explicit: request.explicit,
            defaults: request.defaults,
        },
        transitive_requirements,
        compatibility,
    })
}

fn requirement_trace(trace: ProjectRequirementTrace) -> RequirementTrace {
    RequirementTrace {
        package: trace.package,
        module: trace.module,
        source: trace.source,
        start: trace.start,
        end: trace.end,
    }
}

fn provider_request_failure(path: &str, message: impl Into<String>) -> ProjectFailure {
    let mut problem = ProjectProblem::file("SES-K0200", message, path, None);
    problem.label = Some("provider.invalid-catalog".to_owned());
    ProjectFailure::problem(problem)
}

fn parse_request(request: &str) -> Result<ProjectRequest, ProjectFailure> {
    let request: ProjectRequest = serde_json::from_str(request).map_err(|error| {
        ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("invalid project request JSON: {error}"),
        ))
    })?;
    if request.schema != PROJECT_SCHEMA {
        return Err(ProjectFailure::problem(ProjectProblem::workspace(
            "SES-K0001",
            format!("project request schema must be {PROJECT_SCHEMA}"),
        )));
    }
    Ok(request)
}

fn select_format_source(request: ProjectRequest, path: &str) -> Result<String, ProjectFailure> {
    let (canonical, _, _) = source_identity(path)?;
    if canonical != path {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!("source path must be normalized as `{canonical}`"),
            path,
            None,
        )));
    }
    let mut seen = BTreeSet::new();
    let mut selected = None;
    for file in request.files {
        let (file_path, _, _) = source_identity(&file.path)?;
        if file_path != file.path || !seen.insert(file_path.clone()) {
            return Err(ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                "workspace contains a non-normalized or duplicate source path",
                file.path,
                None,
            )));
        }
        if file_path == path {
            selected = Some(file.source);
        }
    }
    selected.ok_or_else(|| {
        ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "format target is not present in the workspace",
            path,
            None,
        ))
    })
}

fn source_identity(path: &str) -> Result<(String, String, String), ProjectFailure> {
    if path.contains('\0') {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "source path must not contain NUL",
            path,
            None,
        )));
    }
    let Some(module_path) = path.strip_suffix(".ssrg") else {
        return Err(ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            "source path must end in `.ssrg`",
            path,
            None,
        )));
    };
    let module_path = ModulePath::parse(module_path).map_err(|error| {
        ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!("invalid source path `{path}`: {error}"),
            path,
            None,
        ))
    })?;
    let module_path = module_path.as_str().to_owned();
    let canonical_path = format!("{module_path}.ssrg");
    let module = format!("{PACKAGE_SCOPE}/{module_path}");
    Ok((canonical_path, module_path, module))
}

fn graph_failure(
    error: ModuleGraphError<String>,
    paths: &BTreeMap<String, String>,
) -> ProjectFailure {
    let (code, message, module) = match error {
        ModuleGraphError::Cycle { modules } => (
            "SES-N0103",
            format!(
                "module import graph contains a cycle: {}",
                modules.join(" -> ")
            ),
            modules.first().cloned(),
        ),
        ModuleGraphError::MissingModule { module, dependency } => (
            "SES-N0104",
            format!("module `{module}` depends on missing module `{dependency}`"),
            Some(module),
        ),
        ModuleGraphError::DuplicateModule { module } => (
            "SES-K0001",
            format!("duplicate module `{module}`"),
            Some(module),
        ),
        ModuleGraphError::DuplicateSpecifier { module, specifier } => (
            "SES-N0101",
            format!("module `{module}` contains duplicate specifier `{specifier}`"),
            Some(module),
        ),
    };
    ProjectFailure::problem(ProjectProblem {
        code: code.to_owned(),
        message,
        path: module.map(|module| path_for_module(paths, &module)),
        primary: None,
        label: None,
        details: None,
    })
}

fn driver_failure(error: ProjectCompileError, paths: &BTreeMap<String, String>) -> ProjectFailure {
    match error {
        ProjectCompileError::Diagnostics { modules } => ProjectFailure::diagnostics_many(
            modules
                .into_iter()
                .map(|module| ProjectFileDiagnostics {
                    path: path_for_module(paths, &module.module),
                    diagnostics: module.diagnostics,
                })
                .collect(),
        ),
        ProjectCompileError::Graph(error) => graph_failure(error, paths),
        ProjectCompileError::Link { module, errors } => ProjectFailure {
            diagnostics: Vec::new(),
            problems: errors
                .into_iter()
                .map(|error| link_problem(error, path_for_module(paths, &module)))
                .collect(),
        },
        ProjectCompileError::DuplicateInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("duplicate project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        ProjectCompileError::UnexpectedInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("unexpected project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        ProjectCompileError::MissingInput { module } => {
            ProjectFailure::problem(ProjectProblem::file(
                "SES-K0001",
                format!("missing project input `{module}`"),
                path_for_module(paths, &module),
                None,
            ))
        }
        ProjectCompileError::DuplicateOutputPath {
            path,
            first_module,
            second_module,
        } => ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!("modules `{first_module}` and `{second_module}` both generate output `{path}`"),
            path_for_module(paths, &first_module),
            None,
        )),
        ProjectCompileError::GraphImportMismatch {
            module,
            graph_specifiers,
            source_specifiers,
        } => ProjectFailure::problem(ProjectProblem::file(
            "SES-K0001",
            format!(
                "project graph imports do not match source imports (graph: {}; source: {})",
                format_specifiers(&graph_specifiers),
                format_specifiers(&source_specifiers)
            ),
            path_for_module(paths, &module),
            None,
        )),
        ProjectCompileError::LinkTarget { module, error } => ProjectFailure::problem(
            link_target_problem(error, paths, path_for_module(paths, &module)),
        ),
        ProjectCompileError::OutputPlan { module, error } => {
            ProjectFailure::problem(output_plan_problem(error, path_for_module(paths, &module)))
        }
        ProjectCompileError::Compile {
            module,
            error: LinkedCompileError::Diagnostics(diagnostics),
        } => ProjectFailure::diagnostics(path_for_module(paths, &module), diagnostics),
        ProjectCompileError::Compile {
            module,
            error: LinkedCompileError::TypeScriptPlan(error),
        } => ProjectFailure::problem(typescript_plan_problem(
            error,
            path_for_module(paths, &module),
        )),
        ProjectCompileError::Provider { diagnostic } => {
            let trace = diagnostic.trace.as_ref();
            ProjectFailure::problem(ProjectProblem {
                code: diagnostic.code,
                message: diagnostic.message,
                path: trace.map(|trace| trace.source.clone()),
                primary: trace.map(|trace| SourceRange {
                    start: trace.start as usize,
                    end: trace.end as usize,
                }),
                label: Some(diagnostic.label),
                details: Some(ProjectProviderProblemDetails {
                    service: diagnostic.details.service,
                    target: diagnostic.details.target,
                    backend_family: diagnostic.details.backend_family,
                    backend_abi_major: diagnostic.details.backend_abi_major,
                    provider: diagnostic.details.provider,
                    candidates: diagnostic.details.candidates,
                    compatible_targets: diagnostic.details.compatible_targets,
                    reasons: diagnostic.details.reasons,
                    required: diagnostic.details.required,
                    actual: diagnostic.details.actual,
                }),
            })
        }
    }
}

fn format_specifiers(specifiers: &[String]) -> String {
    if specifiers.is_empty() {
        "none".to_owned()
    } else {
        specifiers
            .iter()
            .map(|specifier| format!("`{specifier}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn link_target_problem(
    error: LinkTargetError,
    paths: &BTreeMap<String, String>,
    fallback_path: String,
) -> ProjectProblem {
    let (code, message, path, primary) = match error {
        LinkTargetError::ModuleMismatch { header, interface } => {
            let path = path_for_module(paths, &header);
            (
                "SES-K0001",
                format!(
                    "linked module identity differs between source header `{header}` and interface `{interface}`"
                ),
                path,
                None,
            )
        }
        LinkTargetError::SourceMismatch { header, interface } => (
            "SES-K0001",
            format!(
                "linked source identity differs between source header `{header}` and interface `{interface}`"
            ),
            fallback_path,
            None,
        ),
        LinkTargetError::MissingPublicExport {
            module,
            namespace,
            name,
            declaration,
        } => {
            let path = path_for_module(paths, &module);
            (
                "SES-N0104",
                format!("module `{module}` does not publicly export `{namespace}.{name}`"),
                path,
                Some(declaration),
            )
        }
    };
    ProjectProblem::file(code, message, path, primary)
}

fn output_plan_problem(error: TypeScriptOutputPlanError, path: String) -> ProjectProblem {
    let message = match error {
        TypeScriptOutputPlanError::InvalidImporterPath { path } => {
            format!("generated importer path `{path}` is invalid")
        }
        TypeScriptOutputPlanError::InvalidDependencyPath { module, path } => {
            format!("generated dependency path `{path}` for module `{module}` is invalid")
        }
        TypeScriptOutputPlanError::InvalidGeneratedOutputPath { path } => {
            format!("generated output path `{path}` is invalid")
        }
        TypeScriptOutputPlanError::DuplicateModule { module } => {
            format!("module `{module}` appears more than once in the TypeScript output plan")
        }
        TypeScriptOutputPlanError::DuplicateOutputPath { path } => {
            format!("TypeScript output path `{path}` is assigned more than once")
        }
        TypeScriptOutputPlanError::DuplicateInstanceIdentity { module, identity } => format!(
            "instance `{identity}` from module `{module}` appears more than once in the TypeScript output plan"
        ),
        TypeScriptOutputPlanError::DuplicateInstanceExport {
            module,
            dictionary_export,
        } => format!(
            "instance export `{dictionary_export}` from module `{module}` appears more than once"
        ),
    };
    ProjectProblem::file("SES-K0001", message, path, None)
}

fn typescript_plan_problem(error: TypeScriptLoweringError, path: String) -> ProjectProblem {
    let message = match error {
        TypeScriptLoweringError::MissingOutputSpecifier {
            module,
            source_specifier,
        } => format!(
            "TypeScript output for module `{module}` has no mapping for source import `{source_specifier}`"
        ),
        TypeScriptLoweringError::MissingInstanceOutput { module, identity } => format!(
            "TypeScript output for module `{module}` has no instance output for `{identity}`"
        ),
        TypeScriptLoweringError::MissingInstanceOutputSpecifier { module, identity } => format!(
            "TypeScript output for module `{module}` has no output specifier for instance `{identity}`"
        ),
        TypeScriptLoweringError::MissingExternalTypeBinding { canonical } => {
            format!("TypeScript output has no external type binding for `{canonical}`")
        }
        TypeScriptLoweringError::MissingSourceTypeProvider { canonical } => {
            format!("TypeScript output has no source type provider for `{canonical}`")
        }
        TypeScriptLoweringError::AmbiguousSourceTypeProvider { canonical } => {
            format!("TypeScript output has multiple source type providers for `{canonical}`")
        }
        TypeScriptLoweringError::MissingTypeOutputSpecifier { module, canonical } => format!(
            "TypeScript output for module `{module}` has no output specifier for type `{canonical}`"
        ),
        TypeScriptLoweringError::ImportNameCollision { local } => {
            format!("TypeScript import name `{local}` is assigned more than once")
        }
    };
    ProjectProblem::file("SES-K0001", message, path, None)
}

fn link_problem(error: LinkError, path: String) -> ProjectProblem {
    let (code, message) = match &error {
        LinkError::UnresolvedSpecifier { specifier, .. } => (
            "SES-N0104",
            format!("module specifier `{specifier}` could not be resolved"),
        ),
        LinkError::MissingExport { module, name, .. } => (
            "SES-N0104",
            format!("module `{module}` does not export `{name}`"),
        ),
        LinkError::PrivateExport { module, name, .. } => (
            "SES-N0102",
            format!("module `{module}` keeps `{name}` private"),
        ),
        LinkError::DuplicateImport { local_name, .. } => (
            "SES-N0101",
            format!("import name `{local_name}` is ambiguous"),
        ),
        LinkError::MissingNamespaceAlias { .. } => {
            ("SES-N0104", "namespace import requires an alias".to_owned())
        }
        LinkError::UnsupportedImportNamespace { namespace, .. } => (
            "SES-N0104",
            format!("import namespace `{namespace}` is unsupported"),
        ),
    };
    ProjectProblem::file(code, message, path, Some(error.origin()))
}

fn path_for_module(paths: &BTreeMap<String, String>, module: &str) -> String {
    paths
        .get(module)
        .cloned()
        .unwrap_or_else(|| format!("{module}.ssrg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn request(files: Value, entry: &str) -> String {
        json!({ "schema": 1, "entry": entry, "files": files }).to_string()
    }

    fn request_with_provider(files: Value, entry: &str, provider: Value) -> String {
        json!({
            "schema": 1,
            "entry": entry,
            "files": files,
            "provider": provider
        })
        .to_string()
    }

    fn provider_request(service: &str) -> Value {
        json!({
            "target": "bun-process",
            "backendFamily": "typescript",
            "backendAbiMajor": 1,
            "runtimeFeatures": ["foreign.task-load"],
            "contracts": [{
                "schema": 1,
                "kind": "provider-contract",
                "identity": service,
                "version": { "major": 1, "minor": 0 },
                "requirement": { "field": "clock", "type": service },
                "operations": [{
                    "identity": format!("{service}#now"),
                    "kind": "one-shot",
                    "input": { "kind": "unit" },
                    "success": { "kind": "primitive", "name": "int" },
                    "failure": { "kind": "never" },
                    "portability": { "kind": "portable" },
                    "summary": "Read the clock."
                }]
            }]
        })
    }

    #[test]
    fn returns_the_same_structured_provider_problem_from_analyze_and_compile() {
        let source = concat!(
            "pub type Clock = | Clock\n\n",
            "pub effect fn main -> Unit with clock: Clock =\n",
            "  succeed ()\n"
        );
        let request = request_with_provider(
            json!([{ "path": "main.ssrg", "source": source }]),
            "main.ssrg",
            provider_request("playground/main::Clock"),
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "failure");
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["problems"], analyzed["problems"]);
        assert_eq!(compiled["problems"].as_array().unwrap().len(), 1);
        let problem = &compiled["problems"][0];
        assert_eq!(problem["code"], "SES-K0201");
        assert_eq!(problem["label"], "provider.missing");
        assert_eq!(problem["path"], "main.ssrg");
        assert!(
            problem["primary"]["end"].as_u64().unwrap()
                > problem["primary"]["start"].as_u64().unwrap()
        );
        assert_eq!(problem["details"]["service"], "playground/main::Clock");
        assert_eq!(problem["details"]["target"], "bun-process");
        assert_eq!(problem["details"]["backendFamily"], "typescript");
        assert_eq!(problem["details"]["backendAbiMajor"], 1);
        assert!(compiled.get("modules").is_none());
    }

    #[test]
    fn target_prefilter_reports_one_problem_before_provider_resolution() {
        let source = "pub effect fn main -> Unit with Dom =\n  succeed ()\n";
        let request = request_with_provider(
            json!([{ "path": "main.ssrg", "source": source }]),
            "main.ssrg",
            json!({
                "target": "bun-process",
                "backendFamily": "typescript",
                "backendAbiMajor": 1
            }),
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["problems"], analyzed["problems"]);
        assert_eq!(compiled["problems"].as_array().unwrap().len(), 1);
        assert_eq!(compiled["problems"][0]["code"], "SES-K0203");
        assert_eq!(
            compiled["problems"][0]["details"]["compatibleTargets"],
            json!(["browser"])
        );
        assert!(compiled.get("modules").is_none());
    }

    #[test]
    fn compiles_and_analyzes_a_multi_module_browser_workspace() {
        let request = request(
            json!([
                {
                    "path": "domain.ssrg",
                    "source": "pub fn double value: Int -> Int = value + value\n"
                },
                {
                    "path": "main.ssrg",
                    "source": "import { double } from \"./domain\"\n\npub effect fn main -> Unit with Console fails ConsoleError =\n  double 21 |> debug |> println\n"
                }
            ]),
            "main.ssrg",
        );

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "success");
        assert_eq!(compiled["modules"].as_array().unwrap().len(), 2);
        assert_eq!(compiled["entry"]["path"], "main.ssrg");
        assert_eq!(
            compiled["entry"]["contract"]["environment"][0]["service"],
            "console"
        );
        assert!(compiled["modules"][1]["generated"]["typescript"]
            .as_str()
            .unwrap()
            .contains("from \"./domain.js\""));

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        assert_eq!(analyzed["documents"].as_array().unwrap().len(), 2);
        let main = analyzed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|document| document["path"] == "main.ssrg")
            .unwrap();
        assert!(main["document"]["symbols"]
            .as_array()
            .unwrap()
            .iter()
            .any(|symbol| symbol["name"] == "double" && symbol["module"] == "playground/domain"));
    }

    #[test]
    fn returns_nested_never_evidence_for_a_project_dom_runtime_failure() {
        let request = request(
            json!([{
                "path": "main.ssrg",
                "source": "import * as dom from \"std/web/dom\"\n\npub effect fn main -> Unit\nfails dom.DomRuntimeError<Never> =\n  succeed ()\n"
            }]),
            "main.ssrg",
        );

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();

        assert_eq!(compiled["status"], "success");
        assert_eq!(
            compiled["entry"]["contract"]["failureRenderer"],
            json!({
                "kind": "show",
                "module": "@seseragi/runtime/show",
                "export": "domRuntimeErrorShow",
                "arguments": [{
                    "module": "@seseragi/runtime/show",
                    "export": "neverShow"
                }]
            })
        );
    }

    #[test]
    fn selects_a_failure_dictionary_from_a_generated_provider_module() {
        let request = request(
            json!([
                {
                    "path": "domain/provider.ssrg",
                    "source": "pub type InputError deriving Show =\n  | InvalidInput String\n"
                },
                {
                    "path": "effects/facade.ssrg",
                    "source": "import { InputError, InvalidInput } from \"../domain/provider\"\n\npub effect fn reject input: String =\n  fail (InvalidInput input)\n"
                },
                {
                    "path": "app/main.ssrg",
                    "source": "import { reject } from \"../effects/facade\"\n\npub effect fn main =\n  do {\n    reject \"lizard\"\n    succeed ()\n  }\n"
                }
            ]),
            "app/main.ssrg",
        );

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "success");
        assert_eq!(
            compiled["entry"]["contract"]["failureRenderer"]["module"], "./domain/provider.ts",
            "{compiled}"
        );
        assert_eq!(
            compiled["entry"]["contract"]["failureRenderer"]["export"],
            "__ssrg$instance$Show$0"
        );
    }

    #[test]
    fn reports_missing_modules_cycles_duplicates_and_file_diagnostics() {
        let missing = request(
            json!([{ "path": "main.ssrg", "source": "import { value } from \"./missing\"\n" }]),
            "main.ssrg",
        );
        let missing: Value = serde_json::from_str(&compile_project(&missing)).unwrap();
        assert_eq!(missing["status"], "failure");
        assert_eq!(missing["problems"][0]["code"], "SES-N0104");
        assert_eq!(missing["problems"][0]["path"], "main.ssrg");
        assert!(missing["problems"][0]["primary"]["end"].as_u64().unwrap() > 0);

        let cycle = request(
            json!([
                { "path": "a.ssrg", "source": "import { b } from \"./b\"\npub let a = b\n" },
                { "path": "b.ssrg", "source": "import { a } from \"./a\"\npub let b = a\n" }
            ]),
            "a.ssrg",
        );
        let cycle: Value = serde_json::from_str(&analyze_project(&cycle)).unwrap();
        assert_eq!(cycle["problems"][0]["code"], "SES-N0103");

        let duplicate = request(
            json!([
                { "path": "main.ssrg", "source": "pub let first = 1\n" },
                { "path": "main.ssrg", "source": "pub let second = 2\n" }
            ]),
            "main.ssrg",
        );
        let duplicate: Value = serde_json::from_str(&compile_project(&duplicate)).unwrap();
        assert_eq!(duplicate["problems"][0]["code"], "SES-K0001");

        let invalid = request(
            json!([{ "path": "main.ssrg", "source": "pub let broken: Int =\n" }]),
            "main.ssrg",
        );
        let invalid: Value = serde_json::from_str(&analyze_project(&invalid)).unwrap();
        assert_eq!(invalid["diagnostics"][0]["path"], "main.ssrg");
        assert!(
            invalid["diagnostics"][0]["diagnostics"]["diagnostics"][0]["primary"]["end"]
                .as_u64()
                .is_some()
        );
    }

    #[test]
    fn rejects_noncanonical_and_empty_browser_source_module_paths() {
        let decomposed = request(
            json!([{ "path": "cafe\u{301}.ssrg", "source": "pub let answer = 42\n" }]),
            "cafe\u{301}.ssrg",
        );
        let decomposed: Value = serde_json::from_str(&compile_project(&decomposed)).unwrap();
        assert_eq!(decomposed["status"], "failure");
        assert_eq!(decomposed["problems"][0]["code"], "SES-K0001");
        assert_eq!(decomposed["problems"][0]["path"], "cafe\u{301}.ssrg");
        assert_eq!(
            decomposed["problems"][0]["message"],
            "source path must be normalized as `café.ssrg`"
        );

        let empty_module = request(
            json!([{ "path": ".ssrg", "source": "pub let answer = 42\n" }]),
            ".ssrg",
        );
        let empty_module: Value = serde_json::from_str(&compile_project(&empty_module)).unwrap();
        assert_eq!(empty_module["status"], "failure");
        assert_eq!(empty_module["problems"][0]["code"], "SES-K0001");
        assert_eq!(
            empty_module["problems"][0]["message"],
            "invalid source path `.ssrg`: module path must not be empty"
        );
    }

    #[test]
    fn preserves_single_file_semantic_diagnostics_between_analysis_and_compile() {
        let request = request(
            json!([{
                "path": "main.ssrg",
                "source": "pub fn wrong unit: Unit -> Int = \"wrong\"\n"
            }]),
            "main.ssrg",
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        let analysis_diagnostics = analyzed["documents"][0]["document"]["diagnostics"].clone();
        assert_eq!(analysis_diagnostics["diagnostics"][0]["code"], "SES-T0101");

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["diagnostics"][0]["path"], "main.ssrg");
        assert_eq!(
            compiled["diagnostics"][0]["diagnostics"],
            analysis_diagnostics
        );
        assert_eq!(compiled["problems"], json!([]));
    }

    #[test]
    fn preserves_explicit_failure_contract_diagnostics_between_analysis_and_compile() {
        let request = request(
            json!([{
                "path": "main.ssrg",
                "source": "pub effect fn main -> Unit\nwith Stdin, Console\nfails String =\n  do { readLine (); println \"done\" }\n"
            }]),
            "main.ssrg",
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        let analysis_diagnostics = analyzed["documents"][0]["document"]["diagnostics"].clone();
        let diagnostic = &analysis_diagnostics["diagnostics"][0];
        assert_eq!(diagnostic["messageKey"], "effect.explicit-failure-mismatch");
        assert_eq!(diagnostic["primary"], json!({ "start": 53, "end": 59 }));
        assert_eq!(diagnostic["related"].as_array().unwrap().len(), 3);
        assert_eq!(diagnostic["typeDifference"]["expectedType"], "String");
        assert_eq!(diagnostic["typeDifference"]["actualType"], "StdinError");

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["diagnostics"][0]["path"], "main.ssrg");
        assert_eq!(
            compiled["diagnostics"][0]["diagnostics"],
            analysis_diagnostics
        );
        assert_eq!(compiled["problems"], json!([]));
    }

    #[test]
    fn preserves_dependency_semantic_diagnostics_between_analysis_and_compile() {
        let request = request(
            json!([
                {
                    "path": "domain.ssrg",
                    "source": "pub fn wrong unit: Unit -> Int = \"wrong\"\n"
                },
                {
                    "path": "main.ssrg",
                    "source": "import { wrong } from \"./domain\"\n\npub let value = wrong ()\n"
                }
            ]),
            "main.ssrg",
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        let domain = analyzed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|document| document["path"] == "domain.ssrg")
            .unwrap();
        let analysis_diagnostics = domain["document"]["diagnostics"].clone();
        assert_eq!(analysis_diagnostics["diagnostics"][0]["code"], "SES-T0101");

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["diagnostics"][0]["path"], "domain.ssrg");
        assert_eq!(
            compiled["diagnostics"][0]["diagnostics"],
            analysis_diagnostics
        );
        assert_eq!(compiled["problems"], json!([]));
    }

    #[test]
    fn aggregates_parser_and_semantic_diagnostics_for_analysis_and_compile() {
        let request = request(
            json!([
                {
                    "path": "z-semantic.ssrg",
                    "source": "pub fn wrong unit: Unit -> Int = \"wrong\"\n"
                },
                {
                    "path": "a-parse.ssrg",
                    "source": "pub let broken: Int =\n"
                }
            ]),
            "z-semantic.ssrg",
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "failure");
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["diagnostics"], analyzed["diagnostics"]);
        assert_eq!(
            compiled["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostics| diagnostics["path"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["a-parse.ssrg", "z-semantic.ssrg"]
        );
        assert!(compiled["diagnostics"][0]["diagnostics"]["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic["messageKey"]
                .as_str()
                .unwrap()
                .starts_with("parser.")));
        assert_eq!(
            compiled["diagnostics"][1]["diagnostics"]["diagnostics"][0]["code"],
            "SES-T0101"
        );
        assert_eq!(compiled["problems"], json!([]));
    }

    #[test]
    fn keeps_multiple_semantic_diagnostics_identical_between_analysis_and_compile() {
        let request = request(
            json!([
                {
                    "path": "z-last.ssrg",
                    "source": "pub fn wrong unit: Unit -> Int = \"last\"\n"
                },
                {
                    "path": "a-first.ssrg",
                    "source": "pub fn wrong unit: Unit -> Bool = 1\n"
                }
            ]),
            "a-first.ssrg",
        );

        let analyzed: Value = serde_json::from_str(&analyze_project(&request)).unwrap();
        assert_eq!(analyzed["status"], "success");
        let analysis_diagnostics = analyzed["documents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|document| {
                json!({
                    "path": document["path"].clone(),
                    "diagnostics": document["document"]["diagnostics"].clone()
                })
            })
            .collect::<Vec<_>>();

        let compiled: Value = serde_json::from_str(&compile_project(&request)).unwrap();
        assert_eq!(compiled["status"], "failure");
        assert_eq!(compiled["diagnostics"], json!(analysis_diagnostics));
        assert_eq!(compiled["problems"], json!([]));
    }

    #[test]
    fn reports_compile_only_failures_without_debug_formatting() {
        let paths = BTreeMap::from([("playground/main".to_owned(), "main.ssrg".to_owned())]);
        let failure = driver_failure(
            ProjectCompileError::Compile {
                module: "playground/main".to_owned(),
                error: LinkedCompileError::TypeScriptPlan(
                    TypeScriptLoweringError::MissingOutputSpecifier {
                        module: "playground/domain".to_owned(),
                        source_specifier: "./domain".to_owned(),
                    },
                ),
            },
            &paths,
        );
        assert!(failure.diagnostics.is_empty());
        let problem = &failure.problems[0];

        assert_eq!(problem.code, "SES-K0001");
        assert_eq!(problem.path.as_deref(), Some("main.ssrg"));
        assert_eq!(
            problem.message,
            "TypeScript output for module `playground/domain` has no mapping for source import `./domain`"
        );
    }

    #[test]
    fn formats_one_file_through_the_workspace_request_shape() {
        let request = request(
            json!([{ "path": "main.ssrg", "source": "pub let value: Int = 1   \r\n" }]),
            "main.ssrg",
        );
        let formatted: Value =
            serde_json::from_str(&format_project_file(&request, "main.ssrg")).unwrap();

        assert_eq!(formatted["status"], "success");
        assert_eq!(formatted["path"], "main.ssrg");
        assert_eq!(formatted["source"], "pub let value: Int = 1\n");
        assert_eq!(formatted["changed"], true);
    }
}
