use crate::{ProviderResolution, RequirementTrace};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProviderCompatibilityContext {
    pub target_extensions: Vec<TargetExtensionRequirement>,
    pub runtime_packages: Vec<RuntimePackageCompatibility>,
    pub compiler_features: Vec<CompilerFeatureRequirement>,
    pub conformance: Vec<ProviderConformanceRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetExtensionRequirement {
    pub extension: String,
    pub trace: RequirementTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimePackageCompatibility {
    pub provider: String,
    pub required_identity: String,
    pub required_digest: String,
    pub actual_identity: String,
    pub actual_digest: String,
    pub trace: RequirementTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompilerFeatureRequirement {
    pub provider: String,
    pub required: BTreeSet<String>,
    pub supported: BTreeSet<String>,
    pub trace: RequirementTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderConformanceRequirement {
    pub provider: String,
    pub required_profile: String,
    pub required_digest: String,
    pub actual_profile: Option<String>,
    pub actual_digest: Option<String>,
    pub trace: RequirementTrace,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderCompatibilityError {
    ExtensionMismatch {
        target: String,
        extension: String,
        trace: RequirementTrace,
    },
    RuntimeMismatch {
        provider: String,
        required_identity: String,
        required_digest: String,
        actual_identity: String,
        actual_digest: String,
        trace: RequirementTrace,
    },
    CompilerMismatch {
        provider: String,
        missing: Vec<String>,
        trace: RequirementTrace,
    },
    ConformanceMismatch {
        provider: String,
        required_profile: String,
        required_digest: String,
        actual_profile: Option<String>,
        actual_digest: Option<String>,
        trace: RequirementTrace,
    },
}

impl ProviderCompatibilityError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ExtensionMismatch { .. } => "SES-K0209",
            Self::RuntimeMismatch { .. } => "SES-K0210",
            Self::CompilerMismatch { .. } => "SES-K0211",
            Self::ConformanceMismatch { .. } => "SES-K0212",
        }
    }

    pub const fn label(&self) -> &'static str {
        match self {
            Self::ExtensionMismatch { .. } => "provider.extension-mismatch",
            Self::RuntimeMismatch { .. } => "provider.runtime-mismatch",
            Self::CompilerMismatch { .. } => "provider.compiler-mismatch",
            Self::ConformanceMismatch { .. } => "provider.conformance-mismatch",
        }
    }

    pub const fn trace(&self) -> &RequirementTrace {
        match self {
            Self::ExtensionMismatch { trace, .. }
            | Self::RuntimeMismatch { trace, .. }
            | Self::CompilerMismatch { trace, .. }
            | Self::ConformanceMismatch { trace, .. } => trace,
        }
    }
}

impl std::fmt::Display for ProviderCompatibilityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {} compatibility check failed: {self:?}",
            self.code(),
            self.label()
        )
    }
}

impl std::error::Error for ProviderCompatibilityError {}

pub fn validate_target_extensions(
    target: &str,
    context: &ProviderCompatibilityContext,
) -> Result<(), ProviderCompatibilityError> {
    let mut requirements = context.target_extensions.iter().collect::<Vec<_>>();
    requirements.sort_by(|left, right| {
        left.extension
            .cmp(&right.extension)
            .then_with(|| trace_key(&left.trace).cmp(&trace_key(&right.trace)))
    });
    for requirement in requirements {
        if !target_supports_extension(target, &requirement.extension) {
            return Err(ProviderCompatibilityError::ExtensionMismatch {
                target: target.to_owned(),
                extension: requirement.extension.clone(),
                trace: requirement.trace.clone(),
            });
        }
    }
    Ok(())
}

pub fn validate_selected_provider_compatibility(
    resolution: &ProviderResolution,
    context: &ProviderCompatibilityContext,
) -> Result<(), ProviderCompatibilityError> {
    let selected = resolution
        .selected
        .iter()
        .map(|provider| provider.provider.as_str())
        .collect::<BTreeSet<_>>();

    let runtime = unique_by_provider(&context.runtime_packages, |requirement| {
        requirement.provider.as_str()
    });
    for (provider, requirement) in runtime {
        if !selected.contains(provider)
            || (requirement.required_identity == requirement.actual_identity
                && requirement.required_digest == requirement.actual_digest)
        {
            continue;
        }
        return Err(ProviderCompatibilityError::RuntimeMismatch {
            provider: requirement.provider.clone(),
            required_identity: requirement.required_identity.clone(),
            required_digest: requirement.required_digest.clone(),
            actual_identity: requirement.actual_identity.clone(),
            actual_digest: requirement.actual_digest.clone(),
            trace: requirement.trace.clone(),
        });
    }

    let compiler = unique_by_provider(&context.compiler_features, |requirement| {
        requirement.provider.as_str()
    });
    for (provider, requirement) in compiler {
        if !selected.contains(provider) {
            continue;
        }
        let missing = requirement
            .required
            .difference(&requirement.supported)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(ProviderCompatibilityError::CompilerMismatch {
                provider: requirement.provider.clone(),
                missing,
                trace: requirement.trace.clone(),
            });
        }
    }

    let conformance = unique_by_provider(&context.conformance, |requirement| {
        requirement.provider.as_str()
    });
    for (provider, requirement) in conformance {
        if !selected.contains(provider)
            || (requirement.actual_profile.as_deref()
                == Some(requirement.required_profile.as_str())
                && requirement.actual_digest.as_deref()
                    == Some(requirement.required_digest.as_str()))
        {
            continue;
        }
        return Err(ProviderCompatibilityError::ConformanceMismatch {
            provider: requirement.provider.clone(),
            required_profile: requirement.required_profile.clone(),
            required_digest: requirement.required_digest.clone(),
            actual_profile: requirement.actual_profile.clone(),
            actual_digest: requirement.actual_digest.clone(),
            trace: requirement.trace.clone(),
        });
    }
    Ok(())
}

fn unique_by_provider<'a, T>(
    values: &'a [T],
    provider: impl Fn(&'a T) -> &'a str,
) -> BTreeMap<&'a str, &'a T> {
    values
        .iter()
        .map(|value| (provider(value), value))
        .collect()
}

fn target_supports_extension(target: &str, extension: &str) -> bool {
    target == extension || target.strip_prefix(extension) == Some("-process")
}

fn trace_key(trace: &RequirementTrace) -> (&str, &str, u32, u32) {
    (&trace.source, &trace.module, trace.start, trace.end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContractVersion, ProviderBuildMetadata, ProviderLockMetadata, ProviderResolution,
        ProviderSelectionMetadata,
    };

    fn trace() -> RequirementTrace {
        RequirementTrace {
            package: "fixture/app".to_owned(),
            module: "fixture/app::main".to_owned(),
            source: "src/main.ssrg".to_owned(),
            start: 20,
            end: 25,
        }
    }

    fn resolution(provider: &str) -> ProviderResolution {
        let selected = ProviderSelectionMetadata {
            field: "clock".to_owned(),
            service: "std/clock::Clock".to_owned(),
            required_contract_version: ContractVersion { major: 1, minor: 0 },
            provider_contract_version: ContractVersion { major: 1, minor: 0 },
            provider: provider.to_owned(),
            package_version: "1.0.0".to_owned(),
            package_source: "registry:fixture/provider@1.0.0".to_owned(),
            package_digest: "sha256:package".to_owned(),
            artifact_digest: "sha256:artifact".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            target: "bun-process".to_owned(),
            entry_module: "fixture/provider/clock".to_owned(),
            entry_export: "provider".to_owned(),
            runtime_features: Vec::new(),
            host_packages: Vec::new(),
            traces: vec![trace()],
        };
        ProviderResolution {
            selected: vec![selected.clone()],
            lock: ProviderLockMetadata {
                schema: 1,
                providers: vec![selected.clone()],
            },
            build: ProviderBuildMetadata {
                schema: 1,
                target: "bun-process".to_owned(),
                backend_family: "typescript".to_owned(),
                backend_abi_major: 1,
                runtime_features: Vec::new(),
                providers: vec![selected],
            },
        }
    }

    #[test]
    fn validates_the_four_post_resolution_handshake_diagnostics() {
        let provider = "fixture/runtime-bun#clock";
        let mut context = ProviderCompatibilityContext {
            target_extensions: vec![TargetExtensionRequirement {
                extension: "browser".to_owned(),
                trace: trace(),
            }],
            ..ProviderCompatibilityContext::default()
        };
        assert_eq!(
            validate_target_extensions("bun-process", &context)
                .unwrap_err()
                .code(),
            "SES-K0209"
        );

        context.target_extensions.clear();
        context.runtime_packages.push(RuntimePackageCompatibility {
            provider: provider.to_owned(),
            required_identity: "@seseragi/runtime@1.0.0".to_owned(),
            required_digest: "sha256:locked".to_owned(),
            actual_identity: "@seseragi/runtime@1.0.1".to_owned(),
            actual_digest: "sha256:actual".to_owned(),
            trace: trace(),
        });
        assert_eq!(
            validate_selected_provider_compatibility(&resolution(provider), &context)
                .unwrap_err()
                .code(),
            "SES-K0210"
        );

        context.runtime_packages.clear();
        context.compiler_features.push(CompilerFeatureRequirement {
            provider: provider.to_owned(),
            required: BTreeSet::from(["provider-resource-v1".to_owned()]),
            supported: BTreeSet::new(),
            trace: trace(),
        });
        assert_eq!(
            validate_selected_provider_compatibility(&resolution(provider), &context)
                .unwrap_err()
                .code(),
            "SES-K0211"
        );

        context.compiler_features.clear();
        context.conformance.push(ProviderConformanceRequirement {
            provider: provider.to_owned(),
            required_profile: "clock-v1".to_owned(),
            required_digest: "sha256:required".to_owned(),
            actual_profile: None,
            actual_digest: None,
            trace: trace(),
        });
        assert_eq!(
            validate_selected_provider_compatibility(&resolution(provider), &context)
                .unwrap_err()
                .code(),
            "SES-K0212"
        );
    }

    #[test]
    fn accepts_a_matching_bun_extension_and_selected_evidence() {
        let provider = "fixture/runtime-bun#clock";
        let context = ProviderCompatibilityContext {
            target_extensions: vec![TargetExtensionRequirement {
                extension: "bun".to_owned(),
                trace: trace(),
            }],
            runtime_packages: vec![RuntimePackageCompatibility {
                provider: provider.to_owned(),
                required_identity: "@seseragi/runtime@1.0.0".to_owned(),
                required_digest: "sha256:runtime".to_owned(),
                actual_identity: "@seseragi/runtime@1.0.0".to_owned(),
                actual_digest: "sha256:runtime".to_owned(),
                trace: trace(),
            }],
            compiler_features: vec![CompilerFeatureRequirement {
                provider: provider.to_owned(),
                required: BTreeSet::from(["provider-value-v1".to_owned()]),
                supported: BTreeSet::from(["provider-value-v1".to_owned()]),
                trace: trace(),
            }],
            conformance: vec![ProviderConformanceRequirement {
                provider: provider.to_owned(),
                required_profile: "clock-v1".to_owned(),
                required_digest: "sha256:clock".to_owned(),
                actual_profile: Some("clock-v1".to_owned()),
                actual_digest: Some("sha256:clock".to_owned()),
                trace: trace(),
            }],
        };
        validate_target_extensions("bun-process", &context).unwrap();
        validate_selected_provider_compatibility(&resolution(provider), &context).unwrap();
    }
}
