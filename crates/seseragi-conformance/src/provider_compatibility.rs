use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_compatibility_case(case: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(case.join("contract.json"))
        .map_err(|error| format!("failed to read provider compatibility contract: {error}"))?;
    check_provider_compatibility(&raw)
}

fn check_provider_compatibility(raw: &str) -> Result<(), String> {
    let contract: CompatibilityContract = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider compatibility contract: {error}"))?;
    if contract.schema != 1
        || contract.kind != "provider-compatibility-contract"
        || contract.identity != "seseragi/provider-compatibility"
        || contract.version != 1
    {
        return Err("provider compatibility contract must identify schema 1 version 1".to_owned());
    }
    if contract.handshake
        != [
            "artifact-schema",
            "target-extension",
            "service-contract",
            "backend-abi",
            "runtime-package",
            "compiler-features",
            "provider-conformance",
        ]
    {
        return Err("provider compatibility handshake order is not canonical".to_owned());
    }
    check_versions(&contract.versions)?;
    check_changes(&contract.changes)?;
    check_diagnostics(&contract.diagnostics)?;
    check_cases(&contract.conformance_cases)?;
    check_backends(&contract.backends)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityContract {
    schema: u32,
    kind: String,
    identity: String,
    version: u32,
    handshake: Vec<String>,
    versions: Versions,
    changes: Changes,
    diagnostics: Vec<Diagnostic>,
    #[serde(rename = "conformanceCases")]
    conformance_cases: Vec<String>,
    backends: Vec<BackendExample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Versions {
    #[serde(rename = "artifactSchema")]
    artifact_schema: String,
    #[serde(rename = "serviceContract")]
    service_contract: String,
    #[serde(rename = "backendAbi")]
    backend_abi: String,
    #[serde(rename = "runtimePackage")]
    runtime_package: String,
    compiler: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Changes {
    additive: Vec<String>,
    breaking: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Diagnostic {
    code: String,
    label: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackendExample {
    backend: String,
    #[serde(rename = "applicationApi")]
    application_api: String,
    #[serde(rename = "providerBoundary")]
    provider_boundary: String,
}

fn check_versions(versions: &Versions) -> Result<(), String> {
    if versions.artifact_schema != "exact-supported-major"
        || versions.service_contract != "same-major-provider-minor-at-least-required"
        || versions.backend_abi != "exact-major"
        || versions.runtime_package != "locked-semver-and-content-digest"
        || versions.compiler != "declared-feature-support"
    {
        return Err("provider compatibility version roles are not canonical".to_owned());
    }
    Ok(())
}

fn check_changes(changes: &Changes) -> Result<(), String> {
    let additive = BTreeSet::from([
        "optional-artifact-metadata",
        "new-target-extension",
        "new-service-minor-operation",
        "new-conformance-case-within-existing-semantics",
    ]);
    let breaking = BTreeSet::from([
        "remove-or-rename-operation",
        "change-logical-type-or-outcome",
        "weaken-cancellation-resource-or-backpressure",
        "change-abi-value-or-call-shape",
    ]);
    if changes
        .additive
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != additive
        || changes
            .breaking
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != breaking
    {
        return Err("provider compatibility change classification is incomplete".to_owned());
    }
    Ok(())
}

fn check_diagnostics(diagnostics: &[Diagnostic]) -> Result<(), String> {
    let actual = diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.label.as_str()))
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        ("SES-K0204", "provider.contract-mismatch"),
        ("SES-K0205", "provider.abi-mismatch"),
        ("SES-K0209", "provider.extension-mismatch"),
        ("SES-K0210", "provider.runtime-mismatch"),
        ("SES-K0211", "provider.compiler-mismatch"),
        ("SES-K0212", "provider.conformance-mismatch"),
    ]);
    if actual != expected {
        return Err("provider compatibility diagnostics are incomplete".to_owned());
    }
    Ok(())
}

fn check_cases(cases: &[String]) -> Result<(), String> {
    let expected = BTreeSet::from([
        "success",
        "typed-failure",
        "defect",
        "cancellation",
        "cleanup",
        "concurrency",
        "invalid-value",
        "target-mismatch",
        "abi-mismatch",
        "ambiguity",
    ]);
    if cases.iter().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return Err("provider conformance case model is incomplete".to_owned());
    }
    Ok(())
}

fn check_backends(backends: &[BackendExample]) -> Result<(), String> {
    let names = backends
        .iter()
        .map(|backend| backend.backend.as_str())
        .collect::<BTreeSet<_>>();
    if names != BTreeSet::from(["bun", "future", "node"])
        || backends.iter().any(|backend| {
            backend.application_api != "unchanged-provider-contract"
                || backend.provider_boundary.is_empty()
        })
    {
        return Err("provider backend substitution examples are incomplete".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_compatibility, check_provider_compatibility_case};
    use std::path::PathBuf;

    fn fixture() -> &'static str {
        include_str!(
            "../../../examples/spec/artifacts/provider-compatibility-schema-1/core/contract.json"
        )
    }

    #[test]
    fn accepts_committed_compatibility_contract() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-compatibility-schema-1/core");
        check_provider_compatibility_case(&case).unwrap();
    }

    #[test]
    fn rejects_unknown_fields_and_reordered_handshake() {
        let raw = fixture().replace("\"handshake\":", "\"fallback\": true, \"handshake\":");
        assert!(check_provider_compatibility(&raw)
            .unwrap_err()
            .contains("unknown field `fallback`"));
        let raw = fixture().replace(
            "\"artifact-schema\",\n    \"target-extension\"",
            "\"target-extension\",\n    \"artifact-schema\"",
        );
        assert!(check_provider_compatibility(&raw)
            .unwrap_err()
            .contains("handshake order"));
    }

    #[test]
    fn rejects_missing_conformance_outcomes() {
        let raw = fixture().replace("    \"cleanup\",\n", "");
        assert!(check_provider_compatibility(&raw)
            .unwrap_err()
            .contains("case model"));
    }
}
