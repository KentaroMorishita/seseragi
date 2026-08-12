use crate::{main_provider_requirements, ProjectModuleInput};
use seseragi_provider::{
    is_builtin_service, resolve_providers, validate_provider_target,
    validate_selected_provider_compatibility, validate_target_extensions, ContractVersion,
    ProviderCandidate, ProviderCompatibilityContext, ProviderContract, ProviderResolution,
    ProviderResolutionContext, ProviderResolutionError, RequiredService, RequirementTrace,
};
use seseragi_semantics::{AnalyzedModule, TypedDecl};
use seseragi_syntax::{parse_surface_ast, ByteSpan, SurfaceDecl, SurfaceRequirement, Visibility};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectProviderConfiguration {
    pub entry_module: String,
    pub contracts: Vec<ProviderContract>,
    pub candidates: Vec<ProviderCandidate>,
    pub context: ProviderResolutionContext,
    pub transitive_requirements: Vec<RequiredService>,
    pub compatibility: ProviderCompatibilityContext,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderDiagnosticDetails {
    pub service: Option<String>,
    pub target: Option<String>,
    pub backend_family: Option<String>,
    pub backend_abi_major: Option<u64>,
    pub provider: Option<String>,
    pub candidates: Vec<String>,
    pub compatible_targets: Vec<String>,
    pub reasons: Vec<String>,
    pub required: Vec<String>,
    pub actual: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectProviderDiagnostic {
    pub code: String,
    pub label: String,
    pub message: String,
    pub trace: Option<RequirementTrace>,
    pub details: ProviderDiagnosticDetails,
}

pub(crate) fn plan_project_providers(
    analyzed: &BTreeMap<String, AnalyzedModule>,
    inputs: &BTreeMap<String, ProjectModuleInput>,
    configuration: &ProjectProviderConfiguration,
) -> Result<ProviderResolution, ProjectProviderDiagnostic> {
    let Some(entry) = analyzed.get(&configuration.entry_module) else {
        return Err(invalid_configuration(
            configuration,
            format!(
                "provider resolution entry module `{}` was not analyzed",
                configuration.entry_module
            ),
            None,
        ));
    };
    let Some(input) = inputs.get(&configuration.entry_module) else {
        return Err(invalid_configuration(
            configuration,
            format!(
                "provider resolution entry module `{}` has no source input",
                configuration.entry_module
            ),
            None,
        ));
    };
    let requirements = main_provider_requirements(&entry.typed_hir).map_err(|error| {
        invalid_configuration(
            configuration,
            format!("cannot read provider requirements from main: {error}"),
            main_origin(&entry.typed_hir).map(|span| trace(input, span)),
        )
    })?;
    let traced = trace_main_requirements(input, &requirements);

    if let Err(mismatch) = validate_provider_target(&requirements, &configuration.context.target) {
        let missing = mismatch.missing.first().cloned();
        let trace = missing.as_ref().and_then(|service| {
            traced
                .iter()
                .find(|requirement| requirement.requirement.service == *service)
                .and_then(|requirement| requirement.traces.first())
                .cloned()
        });
        return Err(ProjectProviderDiagnostic {
            code: mismatch.code().to_owned(),
            label: mismatch.label().to_owned(),
            message: format!(
                "target `{}` cannot provide required service{} {}",
                mismatch.target,
                plural(mismatch.missing.len()),
                display_values(&mismatch.missing)
            ),
            trace,
            details: ProviderDiagnosticDetails {
                target: Some(mismatch.target),
                required: mismatch.required,
                actual: mismatch.available,
                compatible_targets: mismatch.compatible_targets,
                ..ProviderDiagnosticDetails::default()
            },
        });
    }

    if let Err(error) =
        validate_target_extensions(&configuration.context.target, &configuration.compatibility)
    {
        return Err(compatibility_diagnostic(error));
    }

    let contracts = index_contracts(configuration, input)?;
    let mut required = Vec::new();
    for mut requirement in traced {
        if is_builtin_service(&requirement.requirement.service) {
            continue;
        }
        let Some(contract) = contracts.get(requirement.requirement.service.as_str()) else {
            return Err(ProjectProviderDiagnostic {
                code: "SES-K0201".to_owned(),
                label: "provider.missing".to_owned(),
                message: format!(
                    "required service `{}` has no visible Provider Contract artifact",
                    requirement.requirement.service
                ),
                trace: requirement.traces.first().cloned(),
                details: base_details(configuration, Some(&requirement.requirement.service)),
            });
        };
        requirement.contract_version = contract.version;
        required.push(requirement);
    }
    required.extend(configuration.transitive_requirements.clone());

    let resolution =
        resolve_providers(&required, &configuration.candidates, &configuration.context)
            .map_err(|error| resolution_diagnostic(configuration, error))?;
    validate_selected_provider_compatibility(&resolution, &configuration.compatibility)
        .map_err(compatibility_diagnostic)?;
    Ok(resolution)
}

fn index_contracts<'a>(
    configuration: &'a ProjectProviderConfiguration,
    input: &ProjectModuleInput,
) -> Result<BTreeMap<&'a str, &'a ProviderContract>, ProjectProviderDiagnostic> {
    let mut contracts = BTreeMap::new();
    for contract in &configuration.contracts {
        if contracts
            .insert(contract.identity.as_str(), contract)
            .is_some()
        {
            return Err(invalid_configuration(
                configuration,
                format!(
                    "duplicate Provider Contract identity `{}`",
                    contract.identity
                ),
                main_span(input).map(|span| trace(input, span)),
            ));
        }
    }
    Ok(contracts)
}

fn trace_main_requirements(
    input: &ProjectModuleInput,
    requirements: &[seseragi_provider::ServiceRequirement],
) -> Vec<RequiredService> {
    let spans = main_requirement_spans(input);
    let fallback = main_span(input).unwrap_or(ByteSpan { start: 0, end: 0 });
    requirements
        .iter()
        .map(|requirement| RequiredService {
            requirement: requirement.clone(),
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![trace(
                input,
                spans.get(&requirement.field).copied().unwrap_or(fallback),
            )],
        })
        .collect()
}

fn main_requirement_spans(input: &ProjectModuleInput) -> BTreeMap<String, ByteSpan> {
    let surface = parse_surface_ast(&input.source_name, &input.source);
    let Some(SurfaceDecl::EffectFn { requirements, .. }) =
        surface.declarations.iter().find(|declaration| {
            matches!(
                declaration,
                SurfaceDecl::EffectFn {
                    visibility: Visibility::Public,
                    name,
                    ..
                } if name == "main"
            )
        })
    else {
        return BTreeMap::new();
    };
    requirements
        .iter()
        .map(|requirement| match requirement {
            SurfaceRequirement::Shorthand { name, span } => (lower_camel(name), *span),
            SurfaceRequirement::Field { name, span, .. } => (name.clone(), *span),
        })
        .collect()
}

fn main_span(input: &ProjectModuleInput) -> Option<ByteSpan> {
    parse_surface_ast(&input.source_name, &input.source)
        .declarations
        .into_iter()
        .find_map(|declaration| match declaration {
            SurfaceDecl::EffectFn {
                visibility: Visibility::Public,
                name,
                span,
                ..
            } if name == "main" => Some(span),
            _ => None,
        })
}

fn main_origin(module: &seseragi_semantics::TypedModule) -> Option<ByteSpan> {
    module
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TypedDecl::EffectFn {
                symbol,
                visibility: Visibility::Public,
                origin,
                ..
            } if symbol.ends_with("::main") => Some(*origin),
            _ => None,
        })
}

fn trace(input: &ProjectModuleInput, span: ByteSpan) -> RequirementTrace {
    RequirementTrace {
        package: input
            .package_scope
            .clone()
            .unwrap_or_else(|| "workspace".to_owned()),
        module: input.module_id.clone(),
        source: input.source_name.clone(),
        start: u32::try_from(span.start).unwrap_or(u32::MAX),
        end: u32::try_from(span.end).unwrap_or(u32::MAX),
    }
}

fn lower_camel(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn invalid_configuration(
    configuration: &ProjectProviderConfiguration,
    message: String,
    trace: Option<RequirementTrace>,
) -> ProjectProviderDiagnostic {
    ProjectProviderDiagnostic {
        code: "SES-K0200".to_owned(),
        label: "provider.invalid-catalog".to_owned(),
        message,
        trace,
        details: base_details(configuration, None),
    }
}

fn resolution_diagnostic(
    configuration: &ProjectProviderConfiguration,
    error: ProviderResolutionError,
) -> ProjectProviderDiagnostic {
    let mut details = base_details(configuration, resolution_service(&error));
    match &error {
        ProviderResolutionError::Ambiguous { providers, .. } => {
            details.candidates = providers.clone();
        }
        ProviderResolutionError::NoCompatible { rejections, .. } => {
            details.candidates = rejections
                .iter()
                .map(|rejection| rejection.provider.clone())
                .collect();
            details.reasons = rejections
                .iter()
                .map(|rejection| rejection.primary.label().to_owned())
                .collect();
        }
        ProviderResolutionError::Incompatible {
            provider,
            rejection,
            ..
        } => {
            details.provider = Some(provider.clone());
            details.reasons = std::iter::once(&rejection.primary)
                .chain(&rejection.notes)
                .map(|reason| reason.label().to_owned())
                .collect();
        }
        ProviderResolutionError::RequirementConflict { majors, .. } => {
            details.required = majors.iter().map(u64::to_string).collect();
        }
        ProviderResolutionError::SelectionUnavailable { provider, .. } => {
            details.provider = Some(provider.clone());
        }
        ProviderResolutionError::InvalidCatalog(message) => {
            details.reasons = vec![message.clone()];
        }
        ProviderResolutionError::Missing { .. } => {}
    }
    ProjectProviderDiagnostic {
        code: error.code().to_owned(),
        label: error.label().to_owned(),
        message: error.to_string(),
        trace: error.primary_trace().cloned(),
        details,
    }
}

fn compatibility_diagnostic(
    error: seseragi_provider::ProviderCompatibilityError,
) -> ProjectProviderDiagnostic {
    use seseragi_provider::ProviderCompatibilityError;

    let mut details = ProviderDiagnosticDetails::default();
    match &error {
        ProviderCompatibilityError::ExtensionMismatch {
            target, extension, ..
        } => {
            details.target = Some(target.clone());
            details.required = vec![extension.clone()];
            details.actual = vec![target.clone()];
        }
        ProviderCompatibilityError::RuntimeMismatch {
            provider,
            required_identity,
            required_digest,
            actual_identity,
            actual_digest,
            ..
        } => {
            details.provider = Some(provider.clone());
            details.required = vec![required_identity.clone(), required_digest.clone()];
            details.actual = vec![actual_identity.clone(), actual_digest.clone()];
        }
        ProviderCompatibilityError::CompilerMismatch {
            provider, missing, ..
        } => {
            details.provider = Some(provider.clone());
            details.required = missing.clone();
        }
        ProviderCompatibilityError::ConformanceMismatch {
            provider,
            required_profile,
            required_digest,
            actual_profile,
            actual_digest,
            ..
        } => {
            details.provider = Some(provider.clone());
            details.required = vec![required_profile.clone(), required_digest.clone()];
            details.actual = actual_profile
                .iter()
                .chain(actual_digest.iter())
                .cloned()
                .collect();
        }
    }
    ProjectProviderDiagnostic {
        code: error.code().to_owned(),
        label: error.label().to_owned(),
        message: error.to_string(),
        trace: Some(error.trace().clone()),
        details,
    }
}

fn base_details(
    configuration: &ProjectProviderConfiguration,
    service: Option<&str>,
) -> ProviderDiagnosticDetails {
    ProviderDiagnosticDetails {
        service: service.map(str::to_owned),
        target: Some(configuration.context.target.clone()),
        backend_family: Some(configuration.context.backend_family.clone()),
        backend_abi_major: Some(configuration.context.backend_abi_major),
        ..ProviderDiagnosticDetails::default()
    }
}

fn resolution_service(error: &ProviderResolutionError) -> Option<&str> {
    match error {
        ProviderResolutionError::Missing { context }
        | ProviderResolutionError::Ambiguous { context, .. }
        | ProviderResolutionError::NoCompatible { context, .. }
        | ProviderResolutionError::Incompatible { context, .. }
        | ProviderResolutionError::SelectionUnavailable { context, .. } => Some(&context.service),
        ProviderResolutionError::RequirementConflict { service, .. } => Some(service),
        ProviderResolutionError::InvalidCatalog(_) => None,
    }
}

fn display_values(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}
