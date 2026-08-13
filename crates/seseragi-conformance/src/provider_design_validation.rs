use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_design_validation_case(case: &Path) -> Result<(), String> {
    let raw = fs::read_to_string(case.join("validation.json"))
        .map_err(|error| format!("failed to read provider design validation: {error}"))?;
    let validation: DesignValidation = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse provider design validation: {error}"))?;
    if validation.schema != 1
        || validation.kind != "provider-design-validation"
        || validation.identity != "seseragi/provider-system"
    {
        return Err("provider design validation envelope is not canonical".to_owned());
    }
    check_capabilities(&validation.capabilities)?;
    check_set(
        &validation.diagnostics,
        &["missing", "ambiguous", "target", "contract", "abi"],
    )?;
    check_set(
        &validation.conformance,
        &[
            "success",
            "typed-failure",
            "defect",
            "cancellation",
            "cleanup",
            "concurrency",
            "invalid-value",
            "mismatch",
            "ambiguity",
        ],
    )?;
    let expected = [
        "contract-artifact",
        "manifest-resolution",
        "typescript-bridge",
        "target-diagnostics",
        "provider-package",
        "clock",
        "http-server",
        "http-client-node",
        "filesystem",
        "postgresql",
        "conformance-guide",
    ];
    if validation.handoff != expected {
        return Err("provider implementation handoff order is not canonical".to_owned());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignValidation {
    schema: u32,
    kind: String,
    identity: String,
    capabilities: Vec<Capability>,
    diagnostics: Vec<String>,
    conformance: Vec<String>,
    handoff: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Capability {
    name: String,
    #[serde(rename = "applicationApi")]
    application_api: String,
    contract: String,
    abi: String,
    provider: String,
    properties: Vec<String>,
}

fn check_capabilities(capabilities: &[Capability]) -> Result<(), String> {
    let expected = BTreeMap::from([
        (
            "clock",
            (
                "std/clock::{now,sleep}",
                "std/clock::Clock#{now,sleep}",
                BTreeSet::from(["one-shot", "cancellation"]),
            ),
        ),
        (
            "http-client",
            (
                "std/http::{sendBytes,sendEmpty}",
                "std/http::HttpClient#send",
                BTreeSet::from(["request-response", "copied-bytes", "cancellation"]),
            ),
        ),
        (
            "http-server",
            (
                "std/http/server::{listen,serveOnce,close}",
                "std/http/server::HttpServer#{listen,close}",
                BTreeSet::from(["handler", "resource", "shutdown"]),
            ),
        ),
        (
            "filesystem",
            (
                "std/fs::{readBytes,readChunks}",
                "std/fs::FileSystem#{openRead,read,close}",
                BTreeSet::from(["opaque-handle", "bytes", "resource", "cleanup"]),
            ),
        ),
        (
            "postgresql",
            (
                "PostgreSQL-specific package API",
                "acme/postgres::Postgres#{openPool,query,openCursor,fetch,closeCursor,closePool}",
                BTreeSet::from(["external-driver", "pool", "row", "cursor"]),
            ),
        ),
    ]);
    let mut actual = BTreeMap::new();
    for capability in capabilities {
        let api = capability.application_api.to_ascii_lowercase();
        if api.contains("provider")
            || api.contains("bun")
            || api.contains("node")
            || capability.contract.is_empty()
            || capability.abi != "seseragi/provider-abi/typescript@1"
            || capability.provider.is_empty()
        {
            return Err(format!(
                "provider design capability leaks implementation details: {}",
                capability.name
            ));
        }
        actual.insert(
            capability.name.as_str(),
            (
                capability.application_api.as_str(),
                capability.contract.as_str(),
                capability
                    .properties
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
            ),
        );
    }
    if actual != expected {
        return Err("provider design must cover all five capability shapes".to_owned());
    }
    Ok(())
}

fn check_set(values: &[String], expected: &[&str]) -> Result<(), String> {
    if values.iter().map(String::as_str).collect::<BTreeSet<_>>()
        != expected.iter().copied().collect::<BTreeSet<_>>()
    {
        return Err("provider design coverage set is incomplete".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_provider_design_validation_case;
    use std::path::PathBuf;

    #[test]
    fn accepts_committed_design_validation() {
        let case = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-design-validation-schema-1/system");
        check_provider_design_validation_case(&case).unwrap();
    }
}
