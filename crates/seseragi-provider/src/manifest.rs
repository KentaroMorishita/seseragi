use crate::{valid_kebab_identifier, valid_type_identity, ContractVersion};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderManifest {
    pub schema: u32,
    pub kind: ProviderManifestKind,
    pub identity: String,
    pub service: String,
    pub contract_version: ContractVersion,
    pub backend: ProviderBackend,
    pub targets: Vec<String>,
    pub entry: ProviderEntry,
    pub requires: ProviderRequirements,
}

impl ProviderManifest {
    pub fn from_json(raw: &str) -> Result<Self, ProviderManifestError> {
        let manifest: Self = serde_json::from_str(raw)
            .map_err(|error| ProviderManifestError::new(format!("invalid JSON schema: {error}")))?;
        manifest.validate()?;
        Ok(manifest)
    }

    fn validate(&self) -> Result<(), ProviderManifestError> {
        if self.schema != 1 {
            return Err(ProviderManifestError::new(
                "provider manifest must use schema 1",
            ));
        }
        validate_provider_identity(&self.identity)?;
        valid_type_identity(&self.service, "provider service identity")
            .map_err(ProviderManifestError::new)?;
        if self.contract_version.major == 0 {
            return Err(ProviderManifestError::new(
                "provider contract version major must be greater than zero",
            ));
        }
        valid_kebab_identifier(&self.backend.family, "provider backend family")
            .map_err(ProviderManifestError::new)?;
        if self.backend.abi_major == 0 {
            return Err(ProviderManifestError::new(
                "provider backend ABI major must be greater than zero",
            ));
        }
        validate_unique_identifiers(&self.targets, "provider target")?;
        if self.targets.is_empty() {
            return Err(ProviderManifestError::new(
                "provider targets must not be empty",
            ));
        }
        validate_module_specifier(&self.entry.module)?;
        validate_typescript_export(&self.entry.export_name)?;
        validate_runtime_features(&self.requires.runtime_features)?;
        validate_host_packages(&self.requires.host_packages)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderManifestKind {
    RuntimeProvider,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderBackend {
    pub family: String,
    pub abi_major: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderEntry {
    pub module: String,
    #[serde(rename = "export")]
    pub export_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderRequirements {
    pub runtime_features: Vec<String>,
    pub host_packages: Vec<HostPackageRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HostPackageRequirement {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderManifestError(String);

impl ProviderManifestError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for ProviderManifestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ProviderManifestError {}

fn validate_provider_identity(identity: &str) -> Result<(), ProviderManifestError> {
    let Some((package, provider)) = identity.split_once('#') else {
        return Err(ProviderManifestError::new(
            "provider identity must contain one # separator",
        ));
    };
    if provider.contains('#') || package.is_empty() || provider.is_empty() {
        return Err(ProviderManifestError::new(
            "provider identity must contain one # separator",
        ));
    }
    for segment in package.split('/') {
        valid_kebab_identifier(segment, "provider identity package")
            .map_err(ProviderManifestError::new)?;
    }
    valid_kebab_identifier(provider, "provider identity name").map_err(ProviderManifestError::new)
}

fn validate_module_specifier(specifier: &str) -> Result<(), ProviderManifestError> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || specifier.ends_with('/')
        || specifier.contains("//")
        || specifier.contains('\0')
    {
        return Err(ProviderManifestError::new(
            "provider entry.module must be a canonical package specifier",
        ));
    }
    let segments = specifier.split('/').collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err(ProviderManifestError::new(
            "provider entry.module must include a package export",
        ));
    }
    for segment in segments {
        valid_kebab_identifier(segment, "provider entry.module")
            .map_err(ProviderManifestError::new)?;
    }
    Ok(())
}

fn validate_typescript_export(value: &str) -> Result<(), ProviderManifestError> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        || !chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(ProviderManifestError::new(
            "provider entry.export must be a TypeScript identifier",
        ));
    }
    Ok(())
}

fn validate_unique_identifiers(
    values: &[String],
    label: &str,
) -> Result<(), ProviderManifestError> {
    let mut seen = BTreeSet::new();
    for value in values {
        valid_kebab_identifier(value, label).map_err(ProviderManifestError::new)?;
        if !seen.insert(value) {
            return Err(ProviderManifestError::new(format!(
                "{label} is duplicated: {value}"
            )));
        }
    }
    Ok(())
}

fn validate_runtime_features(features: &[String]) -> Result<(), ProviderManifestError> {
    let mut seen = BTreeSet::new();
    for feature in features {
        if feature.is_empty() {
            return Err(ProviderManifestError::new(
                "provider runtime feature must not be empty",
            ));
        }
        for segment in feature.split('.') {
            valid_kebab_identifier(segment, "provider runtime feature")
                .map_err(ProviderManifestError::new)?;
        }
        if !seen.insert(feature) {
            return Err(ProviderManifestError::new(format!(
                "provider runtime feature is duplicated: {feature}"
            )));
        }
    }
    Ok(())
}

fn validate_host_packages(
    packages: &[HostPackageRequirement],
) -> Result<(), ProviderManifestError> {
    let mut seen = BTreeSet::new();
    for package in packages {
        if package.name.is_empty()
            || package.name.chars().any(char::is_whitespace)
            || package.version.trim().is_empty()
            || semver::VersionReq::parse(&package.version).is_err()
        {
            return Err(ProviderManifestError::new(
                "provider host package must have a name and valid version range",
            ));
        }
        if !seen.insert(&package.name) {
            return Err(ProviderManifestError::new(format!(
                "provider host package is duplicated: {}",
                package.name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ProviderManifest;
    use std::path::PathBuf;

    #[test]
    fn reads_committed_provider_manifests_in_the_production_model() {
        let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-manifest-schema-1");
        for case in ["bun-clock", "bun-http-client", "node-filesystem"] {
            let raw = std::fs::read_to_string(artifacts.join(case).join("provider.json")).unwrap();
            ProviderManifest::from_json(&raw).unwrap();
        }
    }

    #[test]
    fn rejects_unknown_fields_and_invalid_host_ranges() {
        let raw = include_str!(
            "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-http-client/provider.json"
        );
        assert!(ProviderManifest::from_json(
            &raw.replace("\"requires\":", "\"dynamic\": true, \"requires\":")
        )
        .unwrap_err()
        .to_string()
        .contains("unknown field `dynamic`"));
        assert!(
            ProviderManifest::from_json(&raw.replace("^7.0.0", "latest"))
                .unwrap_err()
                .to_string()
                .contains("valid version range")
        );
    }
}
