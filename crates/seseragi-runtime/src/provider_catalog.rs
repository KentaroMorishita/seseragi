use serde::Serialize;
use seseragi_driver::{
    CandidateVisibility, ProjectProviderConfiguration, ProviderCandidate,
    ProviderCompatibilityContext, ProviderContract, ProviderManifest, ProviderPackageMetadata,
    ProviderResolution, ProviderResolutionContext,
};
use std::collections::{BTreeMap, BTreeSet};

const HTTP_SERVER_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/http-server/contract.json"
);
const BUN_HTTP_SERVER_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-http-server/provider.json"
);
const BUN_CLOCK_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-clock/provider.json"
);
const CLOCK_CONTRACT: &str =
    include_str!("../../../examples/spec/artifacts/provider-contract-schema-1/clock/contract.json");
const HTTP_CLIENT_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/http-client/contract.json"
);
const BROWSER_CLOCK_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/browser-clock/provider.json"
);
const BROWSER_HTTP_CLIENT_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/browser-http-client/provider.json"
);
const NAVIGATION_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/navigation/contract.json"
);
const BROWSER_NAVIGATION_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/browser-navigation/provider.json"
);
const BUN_HTTP_CLIENT_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-http-client-native/provider.json"
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserProviderSelection {
    pub provider: String,
    pub service: String,
    pub target: String,
    pub entry_module: String,
    pub entry_export: String,
}

/// Projects provider resolution metadata onto the browser runtime transport.
/// Diagnostic traces and package metadata intentionally remain toolchain-only.
pub fn browser_provider_selections(
    resolution: Option<&ProviderResolution>,
) -> Vec<BrowserProviderSelection> {
    resolution
        .into_iter()
        .flat_map(|resolution| &resolution.selected)
        .map(|selection| BrowserProviderSelection {
            provider: selection.provider.clone(),
            service: selection.service.clone(),
            target: selection.target.clone(),
            entry_module: selection.entry_module.clone(),
            entry_export: selection.entry_export.clone(),
        })
        .collect()
}

/// Returns the toolchain-owned provider catalog for Bun process execution.
///
/// The entry module is filled by the local-project compiler, so callers do not
/// need to duplicate logical package identity or provider identity.
pub fn bun_process_provider_configuration() -> Result<ProjectProviderConfiguration, String> {
    provider_configuration(
        "bun-process",
        "bun",
        [
            ("clock", CLOCK_CONTRACT, BUN_CLOCK_MANIFEST),
            (
                "http-client",
                HTTP_CLIENT_CONTRACT,
                BUN_HTTP_CLIENT_MANIFEST,
            ),
            (
                "http-server",
                HTTP_SERVER_CONTRACT,
                BUN_HTTP_SERVER_MANIFEST,
            ),
        ],
    )
}

/// Returns the toolchain-owned provider catalog shared by Web builds and the
/// Playground browser execution boundary.
pub fn browser_provider_configuration() -> Result<ProjectProviderConfiguration, String> {
    provider_configuration(
        "browser",
        "browser",
        [
            ("clock", CLOCK_CONTRACT, BROWSER_CLOCK_MANIFEST),
            (
                "http-client",
                HTTP_CLIENT_CONTRACT,
                BROWSER_HTTP_CLIENT_MANIFEST,
            ),
            (
                "navigation",
                NAVIGATION_CONTRACT,
                BROWSER_NAVIGATION_MANIFEST,
            ),
        ],
    )
}

fn provider_configuration<const N: usize>(
    target: &str,
    runtime_package: &str,
    artifacts: [(&str, &str, &str); N],
) -> Result<ProjectProviderConfiguration, String> {
    let mut contracts = Vec::with_capacity(N);
    let mut candidates = Vec::with_capacity(N);
    let mut defaults = BTreeMap::new();
    for (name, contract_json, manifest_json) in artifacts {
        let contract = ProviderContract::from_json(contract_json)
            .map_err(|error| format!("invalid built-in {name} contract: {error}"))?;
        let manifest = ProviderManifest::from_json(manifest_json)
            .map_err(|error| format!("invalid built-in {name} manifest: {error}"))?;
        let service = contract.identity.clone();
        let provider = manifest.identity.clone();
        candidates.push(ProviderCandidate {
            manifest,
            contract: contract.clone(),
            visibility: CandidateVisibility::ToolchainBuiltin,
            package: ProviderPackageMetadata {
                version: env!("CARGO_PKG_VERSION").to_owned(),
                source_identity: format!(
                    "toolchain:seseragi/runtime-{runtime_package}@{}",
                    env!("CARGO_PKG_VERSION")
                ),
                content_digest: format!("sha256:committed-runtime-{runtime_package}"),
            },
            artifact_digest: format!("sha256:committed-{runtime_package}-{name}-manifest"),
            host_packages: Vec::new(),
        });
        defaults.insert(service, provider);
        contracts.push(contract);
    }
    Ok(ProjectProviderConfiguration {
        entry_module: String::new(),
        contracts,
        candidates,
        context: ProviderResolutionContext {
            target: target.to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults,
        },
        transitive_requirements: Vec::new(),
        compatibility: ProviderCompatibilityContext::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bun_process_catalog_includes_clock_and_http_server_defaults() {
        let configuration = bun_process_provider_configuration().unwrap();
        assert_eq!(configuration.context.target, "bun-process");
        assert_eq!(configuration.contracts.len(), 3);
        assert_eq!(configuration.candidates.len(), 3);
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/http::HttpClient")
                .map(String::as_str),
            Some("seseragi/runtime-bun#http-client")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/clock::Clock")
                .map(String::as_str),
            Some("seseragi/runtime-bun#clock")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/http/server::HttpServer")
                .map(String::as_str),
            Some("seseragi/runtime-bun#http-server")
        );
    }

    #[test]
    fn browser_catalog_includes_navigation_default() {
        let configuration = browser_provider_configuration().unwrap();
        assert_eq!(configuration.context.target, "browser");
        assert_eq!(configuration.contracts.len(), 3);
        assert_eq!(configuration.candidates.len(), 3);
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/web/navigation::Navigation")
                .map(String::as_str),
            Some("seseragi/runtime-browser#navigation")
        );
    }
}
