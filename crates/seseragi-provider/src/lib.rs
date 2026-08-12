//! Backend-neutral Runtime Provider Contract artifacts.
//!
//! This crate owns the closed schemas, their semantic validation, and the
//! backend-neutral provider selection contract. Package graph discovery and
//! logical-value projection into a backend ABI remain outside this crate.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

mod compatibility;
mod manifest;
mod resolution;
mod target;

pub use compatibility::{
    validate_selected_provider_compatibility, validate_target_extensions,
    CompilerFeatureRequirement, ProviderCompatibilityContext, ProviderCompatibilityError,
    ProviderConformanceRequirement, RuntimePackageCompatibility, TargetExtensionRequirement,
};

pub use manifest::{
    HostPackageRequirement, ProviderBackend, ProviderEntry, ProviderManifest,
    ProviderManifestError, ProviderManifestKind, ProviderRequirements,
};
pub use resolution::{
    resolve_providers, CandidateRejection, CandidateRejectionReason, CandidateVisibility,
    ProviderBuildMetadata, ProviderCandidate, ProviderErrorContext, ProviderLockMetadata,
    ProviderPackageMetadata, ProviderResolution, ProviderResolutionContext,
    ProviderResolutionError, ProviderSelectionMetadata, ProviderSelectionSource, RequiredService,
    RequirementTrace, ResolvedHostPackage,
};
pub use target::{is_builtin_service, validate_provider_target, ProviderTargetMismatch};

const BACKEND_NAMESPACES: &[&str] = &[
    "browser",
    "bun",
    "deno",
    "javascript",
    "native",
    "node",
    "typescript",
    "wasi",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderContract {
    pub schema: u32,
    pub kind: ProviderContractKind,
    pub identity: String,
    pub version: ContractVersion,
    pub requirement: ContractRequirement,
    pub operations: Vec<ProviderOperation>,
}

impl ProviderContract {
    pub fn from_json(raw: &str) -> Result<Self, ProviderContractError> {
        let document: Value = serde_json::from_str(raw)
            .map_err(|error| ProviderContractError::new(format!("invalid JSON schema: {error}")))?;
        validate_closed_schema(&document)?;
        let contract: Self = serde_json::from_value(document)
            .map_err(|error| ProviderContractError::new(format!("invalid JSON schema: {error}")))?;
        contract.validate()?;
        Ok(contract)
    }

    pub fn operation(&self, identity: &str) -> Option<&ProviderOperation> {
        self.operations
            .iter()
            .find(|operation| operation.identity == identity)
    }

    /// Matches by canonical service identity. An application may bind the
    /// same service type under a non-canonical field name to distinguish two
    /// instances; field names therefore do not select provider candidates.
    pub fn provides_requirement(&self, requirement: &ServiceRequirement) -> bool {
        self.requirement.type_identity == requirement.service
    }

    fn validate(&self) -> Result<(), ProviderContractError> {
        if self.schema != 1 {
            return Err(ProviderContractError::new(
                "provider contract must use schema 1",
            ));
        }
        check_type_identity(&self.identity, "provider contract identity")?;
        if self.version.major == 0 {
            return Err(ProviderContractError::new(
                "provider contract version major must be greater than zero",
            ));
        }
        check_lower_camel(
            &self.requirement.field,
            "provider contract requirement field",
        )?;
        check_type_identity(
            &self.requirement.type_identity,
            "provider contract requirement type",
        )?;
        if self.requirement.type_identity != self.identity {
            return Err(ProviderContractError::new(format!(
                "provider contract requirement type {} must match contract identity {}",
                self.requirement.type_identity, self.identity
            )));
        }
        if self.operations.is_empty() {
            return Err(ProviderContractError::new(
                "provider contract operations must not be empty",
            ));
        }
        let mut identities = BTreeSet::new();
        for (index, operation) in self.operations.iter().enumerate() {
            operation.validate(index, &self.identity)?;
            if !identities.insert(&operation.identity) {
                return Err(ProviderContractError::new(format!(
                    "provider contract operation identity is duplicated: {}",
                    operation.identity
                )));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderContractKind {
    ProviderContract,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractVersion {
    pub major: u64,
    pub minor: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractRequirement {
    pub field: String,
    #[serde(rename = "type")]
    pub type_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderOperation {
    pub identity: String,
    pub kind: OperationKind,
    pub input: LogicalType,
    pub success: LogicalType,
    pub failure: LogicalType,
    pub portability: Portability,
    pub summary: String,
}

impl ProviderOperation {
    fn validate(&self, index: usize, service_identity: &str) -> Result<(), ProviderContractError> {
        let label = format!("provider contract operation {index}");
        let prefix = format!("{service_identity}#");
        let Some(name) = self.identity.strip_prefix(&prefix) else {
            return Err(ProviderContractError::new(format!(
                "{label} identity {} must start with {prefix}",
                self.identity
            )));
        };
        check_lower_camel(name, &format!("{label} name"))?;
        self.input.validate(&format!("{label} input"))?;
        self.success.validate(&format!("{label} success"))?;
        self.failure.validate(&format!("{label} failure"))?;
        self.portability
            .validate(&format!("{label} portability"), service_identity)?;
        if self.summary.trim().is_empty() {
            return Err(ProviderContractError::new(format!(
                "{label} summary must not be empty"
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationKind {
    OneShot,
    Resource,
    Subscription,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrimitiveType {
    Bool,
    Bytes,
    Float,
    Int,
    String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LogicalType {
    Unit,
    Never,
    Primitive { name: PrimitiveType },
    Named { identity: String },
    Array { items: Box<LogicalType> },
    Record { fields: Vec<LogicalRecordField> },
}

impl LogicalType {
    fn validate(&self, label: &str) -> Result<(), ProviderContractError> {
        match self {
            Self::Unit | Self::Never | Self::Primitive { .. } => Ok(()),
            Self::Named { identity } => check_type_identity(identity, label),
            Self::Array { items } => items.validate(&format!("{label} items")),
            Self::Record { fields } => {
                let mut names = BTreeSet::new();
                for (index, field) in fields.iter().enumerate() {
                    let field_label = format!("{label} field {index}");
                    check_lower_camel(&field.name, &format!("{field_label} name"))?;
                    if !names.insert(&field.name) {
                        return Err(ProviderContractError::new(format!(
                            "{label} field name is duplicated: {}",
                            field.name
                        )));
                    }
                    field.type_ref.validate(&format!("{field_label} type"))?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LogicalRecordField {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: LogicalType,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Portability {
    Portable,
    TargetExtension { target: String },
}

impl Portability {
    fn validate(&self, label: &str, service_identity: &str) -> Result<(), ProviderContractError> {
        match self {
            Self::Portable => {
                if identity_has_backend_namespace(service_identity) {
                    return Err(ProviderContractError::new(format!(
                        "{label} may not mark a target namespace as portable"
                    )));
                }
            }
            Self::TargetExtension { target } => {
                check_kebab_identifier(target, label)?;
                if !identity_module_segments(service_identity).contains(&target.as_str()) {
                    return Err(ProviderContractError::new(format!(
                        "{label} target {target} must appear in the service module identity"
                    )));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceRequirement {
    pub field: String,
    pub service: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderContractError {
    message: String,
}

impl ProviderContractError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ProviderContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProviderContractError {}

fn validate_closed_schema(document: &Value) -> Result<(), ProviderContractError> {
    reject_unknown_fields(
        document,
        "provider contract",
        &[
            "schema",
            "kind",
            "identity",
            "version",
            "requirement",
            "operations",
        ],
    )?;
    if let Some(version) = document.get("version") {
        reject_unknown_fields(version, "provider contract version", &["major", "minor"])?;
    }
    if let Some(requirement) = document.get("requirement") {
        reject_unknown_fields(
            requirement,
            "provider contract requirement",
            &["field", "type"],
        )?;
    }
    if let Some(operations) = document.get("operations").and_then(Value::as_array) {
        for (index, operation) in operations.iter().enumerate() {
            let label = format!("provider contract operation {index}");
            reject_unknown_fields(
                operation,
                &label,
                &[
                    "identity",
                    "kind",
                    "input",
                    "success",
                    "failure",
                    "portability",
                    "summary",
                ],
            )?;
            for field in ["input", "success", "failure"] {
                if let Some(logical_type) = operation.get(field) {
                    validate_closed_logical_type(logical_type, &format!("{label} {field}"))?;
                }
            }
            if let Some(portability) = operation.get("portability") {
                let fields = match portability.get("kind").and_then(Value::as_str) {
                    Some("portable") => &["kind"][..],
                    Some("target-extension") => &["kind", "target"][..],
                    _ => continue,
                };
                reject_unknown_fields(portability, &format!("{label} portability"), fields)?;
            }
        }
    }
    Ok(())
}

fn validate_closed_logical_type(value: &Value, label: &str) -> Result<(), ProviderContractError> {
    let fields = match value.get("kind").and_then(Value::as_str) {
        Some("unit" | "never") => &["kind"][..],
        Some("primitive") => &["kind", "name"][..],
        Some("named") => &["kind", "identity"][..],
        Some("array") => &["kind", "items"][..],
        Some("record") => &["kind", "fields"][..],
        _ => return Ok(()),
    };
    reject_unknown_fields(value, label, fields)?;
    match value.get("kind").and_then(Value::as_str) {
        Some("array") => {
            if let Some(items) = value.get("items") {
                validate_closed_logical_type(items, &format!("{label} items"))?;
            }
        }
        Some("record") => {
            if let Some(record_fields) = value.get("fields").and_then(Value::as_array) {
                for (index, field) in record_fields.iter().enumerate() {
                    let field_label = format!("{label} field {index}");
                    reject_unknown_fields(field, &field_label, &["name", "type"])?;
                    if let Some(type_ref) = field.get("type") {
                        validate_closed_logical_type(type_ref, &format!("{field_label} type"))?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn reject_unknown_fields(
    value: &Value,
    label: &str,
    allowed: &[&str],
) -> Result<(), ProviderContractError> {
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
    {
        return Err(ProviderContractError::new(format!(
            "{label} contains unknown field {field}"
        )));
    }
    Ok(())
}

pub(crate) fn valid_type_identity(identity: &str, label: &str) -> Result<(), String> {
    let Some((module, symbol)) = identity.rsplit_once("::") else {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    };
    if module.is_empty() || symbol.is_empty() {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    }
    let segments = module
        .split("::")
        .flat_map(|part| part.split('/'))
        .collect::<Vec<_>>();
    let Some(first) = segments.first().copied() else {
        return Err(format!("{label} module is missing"));
    };
    if BACKEND_NAMESPACES.contains(&first) {
        return Err(format!("{label} uses backend-specific namespace {first}"));
    }
    if segments.len() < 2 {
        return Err(format!("{label} must include a module path"));
    }
    for segment in segments {
        valid_kebab_identifier(segment, label)?;
    }
    check_upper_camel(symbol, label).map_err(|error| error.to_string())
}

fn check_type_identity(identity: &str, label: &str) -> Result<(), ProviderContractError> {
    valid_type_identity(identity, label).map_err(ProviderContractError::new)
}

fn identity_has_backend_namespace(identity: &str) -> bool {
    identity_module_segments(identity)
        .iter()
        .any(|segment| BACKEND_NAMESPACES.contains(segment))
}

fn identity_module_segments(identity: &str) -> Vec<&str> {
    identity
        .rsplit_once("::")
        .map(|(module, _)| {
            module
                .split("::")
                .flat_map(|part| part.split('/'))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn valid_kebab_identifier(value: &str, label: &str) -> Result<(), String> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(format!("{label} must use lowercase kebab-case segments"));
    }
    Ok(())
}

fn check_kebab_identifier(value: &str, label: &str) -> Result<(), ProviderContractError> {
    valid_kebab_identifier(value, label).map_err(ProviderContractError::new)
}

fn check_lower_camel(value: &str, label: &str) -> Result<(), ProviderContractError> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| character.is_ascii_alphanumeric())
    {
        return Err(ProviderContractError::new(format!(
            "{label} must use lowerCamelCase"
        )));
    }
    Ok(())
}

fn check_upper_camel(value: &str, label: &str) -> Result<(), ProviderContractError> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        || !chars.all(|character| character.is_ascii_alphanumeric())
    {
        return Err(ProviderContractError::new(format!(
            "{label} symbol must use UpperCamelCase"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
