use crate::{ContractVersion, ProviderContract, ProviderManifest, ServiceRequirement};
use semver::{Version, VersionReq};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredService {
    pub requirement: ServiceRequirement,
    pub contract_version: ContractVersion,
    pub traces: Vec<RequirementTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementTrace {
    pub package: String,
    pub module: String,
    pub source: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateVisibility {
    ToolchainBuiltin,
    RootDirectDependency,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCandidate {
    pub manifest: ProviderManifest,
    pub contract: ProviderContract,
    pub visibility: CandidateVisibility,
    pub package: ProviderPackageMetadata,
    pub artifact_digest: String,
    pub host_packages: Vec<ResolvedHostPackage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPackageMetadata {
    pub version: String,
    pub source_identity: String,
    pub content_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedHostPackage {
    pub name: String,
    pub version: String,
    pub source_identity: String,
    pub content_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderResolutionContext {
    pub target: String,
    pub backend_family: String,
    pub backend_abi_major: u64,
    pub runtime_features: BTreeSet<String>,
    pub explicit: BTreeMap<String, String>,
    pub defaults: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderErrorContext {
    pub service: String,
    pub required_contract_version: ContractVersion,
    pub traces: Vec<RequirementTrace>,
    pub target: String,
    pub backend_family: String,
    pub backend_abi_major: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderSelectionSource {
    Explicit,
    ToolchainDefault,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderResolution {
    pub selected: Vec<ProviderSelectionMetadata>,
    pub lock: ProviderLockMetadata,
    pub build: ProviderBuildMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSelectionMetadata {
    pub field: String,
    pub service: String,
    pub required_contract_version: ContractVersion,
    pub provider_contract_version: ContractVersion,
    pub provider: String,
    pub package_version: String,
    pub package_source: String,
    pub package_digest: String,
    pub artifact_digest: String,
    pub backend_family: String,
    pub backend_abi_major: u64,
    pub target: String,
    pub entry_module: String,
    pub entry_export: String,
    pub runtime_features: Vec<String>,
    pub host_packages: Vec<ResolvedHostPackage>,
    pub traces: Vec<RequirementTrace>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderLockMetadata {
    pub schema: u32,
    pub providers: Vec<ProviderSelectionMetadata>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBuildMetadata {
    pub schema: u32,
    pub target: String,
    pub backend_family: String,
    pub backend_abi_major: u64,
    pub runtime_features: Vec<String>,
    pub providers: Vec<ProviderSelectionMetadata>,
}

pub fn resolve_providers(
    requirements: &[RequiredService],
    candidates: &[ProviderCandidate],
    context: &ProviderResolutionContext,
) -> Result<ProviderResolution, ProviderResolutionError> {
    validate_catalog(candidates)?;
    let requirements = merge_requirements(requirements)?;
    let mut selected = Vec::with_capacity(requirements.len());
    for requirement in requirements {
        selected.push(resolve_requirement(&requirement, candidates, context)?);
    }
    selected.sort_by(|left, right| {
        left.service
            .cmp(&right.service)
            .then_with(|| left.field.cmp(&right.field))
    });
    let runtime_features = context.runtime_features.iter().cloned().collect::<Vec<_>>();
    Ok(ProviderResolution {
        lock: ProviderLockMetadata {
            schema: 1,
            providers: selected.clone(),
        },
        build: ProviderBuildMetadata {
            schema: 1,
            target: context.target.clone(),
            backend_family: context.backend_family.clone(),
            backend_abi_major: context.backend_abi_major,
            runtime_features,
            providers: selected.clone(),
        },
        selected,
    })
}

fn validate_catalog(candidates: &[ProviderCandidate]) -> Result<(), ProviderResolutionError> {
    let mut identities = BTreeSet::new();
    for candidate in candidates {
        if !identities.insert(&candidate.manifest.identity) {
            return Err(ProviderResolutionError::InvalidCatalog(format!(
                "duplicate visible provider identity {}",
                candidate.manifest.identity
            )));
        }
        if candidate.manifest.service != candidate.contract.identity
            || candidate.manifest.contract_version != candidate.contract.version
        {
            return Err(ProviderResolutionError::InvalidCatalog(format!(
                "provider {} manifest and Contract identity or version differ",
                candidate.manifest.identity
            )));
        }
        for value in [
            &candidate.package.version,
            &candidate.package.source_identity,
            &candidate.package.content_digest,
            &candidate.artifact_digest,
        ] {
            if value.trim().is_empty() {
                return Err(ProviderResolutionError::InvalidCatalog(format!(
                    "provider {} has incomplete package or artifact metadata",
                    candidate.manifest.identity
                )));
            }
        }
        if contains_absolute_path(&candidate.package.source_identity) {
            return Err(ProviderResolutionError::InvalidCatalog(format!(
                "provider {} package source metadata contains an absolute path",
                candidate.manifest.identity
            )));
        }
    }
    Ok(())
}

fn merge_requirements(
    requirements: &[RequiredService],
) -> Result<Vec<RequiredService>, ProviderResolutionError> {
    let mut merged: BTreeMap<(String, String), RequiredService> = BTreeMap::new();
    let mut majors: BTreeMap<String, u64> = BTreeMap::new();
    for requirement in requirements {
        let service = &requirement.requirement.service;
        if let Some(first) = majors.get(service).copied() {
            if first != requirement.contract_version.major {
                let mut traces = merged
                    .values()
                    .filter(|existing| existing.requirement.service == *service)
                    .flat_map(|existing| existing.traces.clone())
                    .chain(requirement.traces.clone())
                    .collect::<Vec<_>>();
                traces.sort_by(|left, right| {
                    left.source
                        .cmp(&right.source)
                        .then_with(|| left.start.cmp(&right.start))
                        .then_with(|| left.end.cmp(&right.end))
                });
                traces.dedup();
                return Err(ProviderResolutionError::RequirementConflict {
                    service: service.clone(),
                    majors: BTreeSet::from([first, requirement.contract_version.major])
                        .into_iter()
                        .collect(),
                    traces,
                });
            }
        } else {
            majors.insert(service.clone(), requirement.contract_version.major);
        }
        let key = (
            requirement.requirement.field.clone(),
            requirement.requirement.service.clone(),
        );
        match merged.get_mut(&key) {
            Some(existing) => {
                existing.contract_version.minor = existing
                    .contract_version
                    .minor
                    .max(requirement.contract_version.minor);
                existing.traces.extend(requirement.traces.clone());
            }
            None => {
                merged.insert(key, requirement.clone());
            }
        }
    }
    for requirement in merged.values_mut() {
        requirement.traces.sort_by(|left, right| {
            left.package
                .cmp(&right.package)
                .then_with(|| left.module.cmp(&right.module))
                .then_with(|| left.source.cmp(&right.source))
                .then_with(|| left.start.cmp(&right.start))
                .then_with(|| left.end.cmp(&right.end))
        });
        requirement.traces.dedup();
    }
    Ok(merged.into_values().collect())
}

fn resolve_requirement(
    requirement: &RequiredService,
    candidates: &[ProviderCandidate],
    context: &ProviderResolutionContext,
) -> Result<ProviderSelectionMetadata, ProviderResolutionError> {
    let service = &requirement.requirement.service;
    let mut service_candidates = candidates
        .iter()
        .filter(|candidate| candidate.manifest.service == *service)
        .collect::<Vec<_>>();
    service_candidates.sort_by(|left, right| left.manifest.identity.cmp(&right.manifest.identity));
    if service_candidates.is_empty() {
        return Err(ProviderResolutionError::Missing {
            context: error_context(requirement, context),
        });
    }

    let pinned = context
        .explicit
        .get(service)
        .map(|identity| (identity, true))
        .or_else(|| {
            context
                .defaults
                .get(service)
                .map(|identity| (identity, false))
        });
    if let Some((identity, explicit)) = pinned {
        let Some(candidate) = service_candidates
            .iter()
            .copied()
            .find(|candidate| candidate.manifest.identity == *identity)
        else {
            return Err(ProviderResolutionError::SelectionUnavailable {
                context: error_context(requirement, context),
                provider: identity.clone(),
                selection: if explicit {
                    ProviderSelectionSource::Explicit
                } else {
                    ProviderSelectionSource::ToolchainDefault
                },
            });
        };
        if let Some(rejection) = candidate_rejection(candidate, requirement, context) {
            return Err(ProviderResolutionError::Incompatible {
                context: error_context(requirement, context),
                provider: identity.clone(),
                rejection,
                selection: if explicit {
                    ProviderSelectionSource::Explicit
                } else {
                    ProviderSelectionSource::ToolchainDefault
                },
            });
        }
        return Ok(selection(candidate, requirement, context));
    }

    let mut compatible = Vec::new();
    let mut rejections = Vec::new();
    for candidate in service_candidates {
        match candidate_rejection(candidate, requirement, context) {
            Some(rejection) => rejections.push(rejection),
            None => compatible.push(candidate),
        }
    }
    compatible.sort_by(|left, right| left.manifest.identity.cmp(&right.manifest.identity));
    match compatible.as_slice() {
        [candidate] => Ok(selection(candidate, requirement, context)),
        [] => Err(ProviderResolutionError::NoCompatible {
            context: error_context(requirement, context),
            rejections,
        }),
        _ => Err(ProviderResolutionError::Ambiguous {
            context: error_context(requirement, context),
            providers: compatible
                .iter()
                .map(|candidate| candidate.manifest.identity.clone())
                .collect(),
        }),
    }
}

fn error_context(
    requirement: &RequiredService,
    context: &ProviderResolutionContext,
) -> ProviderErrorContext {
    ProviderErrorContext {
        service: requirement.requirement.service.clone(),
        required_contract_version: requirement.contract_version,
        traces: requirement.traces.clone(),
        target: context.target.clone(),
        backend_family: context.backend_family.clone(),
        backend_abi_major: context.backend_abi_major,
    }
}

fn candidate_rejection(
    candidate: &ProviderCandidate,
    requirement: &RequiredService,
    context: &ProviderResolutionContext,
) -> Option<CandidateRejection> {
    let mut reasons = Vec::new();
    if !candidate.manifest.targets.contains(&context.target) {
        reasons.push(CandidateRejectionReason::TargetMismatch);
    }
    if candidate.manifest.contract_version.major != requirement.contract_version.major
        || candidate.manifest.contract_version.minor < requirement.contract_version.minor
    {
        reasons.push(CandidateRejectionReason::ContractMismatch);
    }
    if candidate.manifest.backend.family != context.backend_family
        || candidate.manifest.backend.abi_major != context.backend_abi_major
    {
        reasons.push(CandidateRejectionReason::AbiMismatch);
    }
    let missing_runtime_features = candidate
        .manifest
        .requires
        .runtime_features
        .iter()
        .filter(|feature| !context.runtime_features.contains(*feature))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_runtime_features.is_empty() {
        reasons.push(CandidateRejectionReason::RuntimeFeatureMismatch {
            missing: missing_runtime_features,
        });
    }
    let host_package_errors = host_package_errors(candidate);
    if !host_package_errors.is_empty() {
        reasons.push(CandidateRejectionReason::HostPackageMismatch {
            packages: host_package_errors,
        });
    }
    let mut reasons = reasons.into_iter();
    reasons.next().map(|primary| CandidateRejection {
        provider: candidate.manifest.identity.clone(),
        primary,
        notes: reasons.collect(),
    })
}

fn host_package_errors(candidate: &ProviderCandidate) -> Vec<String> {
    let required_names = candidate
        .manifest
        .requires
        .host_packages
        .iter()
        .map(|required| required.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut errors = candidate
        .manifest
        .requires
        .host_packages
        .iter()
        .filter_map(|required| {
            let matches = candidate
                .host_packages
                .iter()
                .filter(|resolved| resolved.name == required.name)
                .collect::<Vec<_>>();
            let [resolved] = matches.as_slice() else {
                return Some(required.name.clone());
            };
            let valid_version = Version::parse(&resolved.version)
                .ok()
                .is_some_and(|version| {
                    VersionReq::parse(&required.version)
                        .ok()
                        .is_some_and(|range| range.matches(&version))
                });
            (!valid_version
                || resolved.source_identity.trim().is_empty()
                || contains_absolute_path(&resolved.source_identity)
                || resolved.content_digest.trim().is_empty())
            .then(|| required.name.clone())
        })
        .collect::<Vec<_>>();
    for resolved in &candidate.host_packages {
        if !required_names.contains(resolved.name.as_str())
            || candidate
                .host_packages
                .iter()
                .filter(|other| other.name == resolved.name)
                .count()
                != 1
        {
            errors.push(resolved.name.clone());
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

fn selection(
    candidate: &ProviderCandidate,
    requirement: &RequiredService,
    context: &ProviderResolutionContext,
) -> ProviderSelectionMetadata {
    let mut runtime_features = candidate.manifest.requires.runtime_features.clone();
    runtime_features.sort();
    let mut host_packages = candidate.host_packages.clone();
    host_packages.sort_by(|left, right| left.name.cmp(&right.name));
    ProviderSelectionMetadata {
        field: requirement.requirement.field.clone(),
        service: requirement.requirement.service.clone(),
        required_contract_version: requirement.contract_version,
        provider_contract_version: candidate.manifest.contract_version,
        provider: candidate.manifest.identity.clone(),
        package_version: candidate.package.version.clone(),
        package_source: candidate.package.source_identity.clone(),
        package_digest: candidate.package.content_digest.clone(),
        artifact_digest: candidate.artifact_digest.clone(),
        backend_family: candidate.manifest.backend.family.clone(),
        backend_abi_major: candidate.manifest.backend.abi_major,
        target: context.target.clone(),
        entry_module: candidate.manifest.entry.module.clone(),
        entry_export: candidate.manifest.entry.export_name.clone(),
        runtime_features,
        host_packages,
        traces: requirement.traces.clone(),
    }
}

fn contains_absolute_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("path:")
        || value.as_bytes().get(1) == Some(&b':')
        || value.contains("\\\\")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateRejection {
    pub provider: String,
    pub primary: CandidateRejectionReason,
    pub notes: Vec<CandidateRejectionReason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CandidateRejectionReason {
    TargetMismatch,
    ContractMismatch,
    AbiMismatch,
    RuntimeFeatureMismatch { missing: Vec<String> },
    HostPackageMismatch { packages: Vec<String> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderResolutionError {
    InvalidCatalog(String),
    Missing {
        context: ProviderErrorContext,
    },
    Ambiguous {
        context: ProviderErrorContext,
        providers: Vec<String>,
    },
    NoCompatible {
        context: ProviderErrorContext,
        rejections: Vec<CandidateRejection>,
    },
    Incompatible {
        context: ProviderErrorContext,
        provider: String,
        rejection: CandidateRejection,
        selection: ProviderSelectionSource,
    },
    RequirementConflict {
        service: String,
        majors: Vec<u64>,
        traces: Vec<RequirementTrace>,
    },
    SelectionUnavailable {
        context: ProviderErrorContext,
        provider: String,
        selection: ProviderSelectionSource,
    },
}

impl ProviderResolutionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidCatalog(_) => "SES-K0200",
            Self::Missing { .. } => "SES-K0201",
            Self::Ambiguous { .. } => "SES-K0202",
            Self::NoCompatible { rejections, .. } => rejections
                .first()
                .map(|rejection| rejection.primary.code())
                .unwrap_or("SES-K0206"),
            Self::Incompatible { rejection, .. } => rejection.primary.code(),
            Self::RequirementConflict { .. } => "SES-K0207",
            Self::SelectionUnavailable { .. } => "SES-K0208",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::InvalidCatalog(_) => "provider.invalid-catalog",
            Self::Missing { .. } => "provider.missing",
            Self::Ambiguous { .. } => "provider.ambiguous",
            Self::NoCompatible { rejections, .. } => match rejections.first() {
                Some(rejection) => rejection.primary.label(),
                None => "provider.runtime-feature-mismatch",
            },
            Self::Incompatible { rejection, .. } => rejection.primary.label(),
            Self::RequirementConflict { .. } => "provider.requirement-conflict",
            Self::SelectionUnavailable { .. } => "provider.selection-unavailable",
        }
    }

    pub fn primary_trace(&self) -> Option<&RequirementTrace> {
        match self {
            Self::Missing { context }
            | Self::Ambiguous { context, .. }
            | Self::NoCompatible { context, .. }
            | Self::Incompatible { context, .. }
            | Self::SelectionUnavailable { context, .. } => context.traces.first(),
            Self::RequirementConflict { traces, .. } => traces.first(),
            Self::InvalidCatalog(_) => None,
        }
    }
}

impl CandidateRejectionReason {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::TargetMismatch => "SES-K0203",
            Self::ContractMismatch => "SES-K0204",
            Self::AbiMismatch => "SES-K0205",
            Self::RuntimeFeatureMismatch { .. } | Self::HostPackageMismatch { .. } => "SES-K0206",
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::TargetMismatch => "provider.target-mismatch",
            Self::ContractMismatch => "provider.contract-mismatch",
            Self::AbiMismatch => "provider.abi-mismatch",
            Self::RuntimeFeatureMismatch { .. } | Self::HostPackageMismatch { .. } => {
                "provider.runtime-feature-mismatch"
            }
        }
    }
}

impl std::fmt::Display for ProviderResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} provider resolution failed: {self:?}",
            self.code()
        )
    }
}

impl std::error::Error for ProviderResolutionError {}

#[cfg(test)]
mod tests;
