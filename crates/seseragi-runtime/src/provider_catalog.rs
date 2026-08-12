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
    let contract = ProviderContract::from_json(HTTP_SERVER_CONTRACT)
        .map_err(|error| format!("invalid built-in HTTP server contract: {error}"))?;
    let manifest = ProviderManifest::from_json(BUN_HTTP_SERVER_MANIFEST)
        .map_err(|error| format!("invalid built-in Bun HTTP server manifest: {error}"))?;
    let service = contract.identity.clone();
    let provider = manifest.identity.clone();
    let candidate = ProviderCandidate {
        manifest,
        contract: contract.clone(),
        visibility: CandidateVisibility::ToolchainBuiltin,
        package: ProviderPackageMetadata {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            source_identity: format!(
                "toolchain:seseragi/runtime-bun@{}",
                env!("CARGO_PKG_VERSION")
            ),
            content_digest: "sha256:committed-runtime-bun-http-server".to_owned(),
        },
        artifact_digest: "sha256:committed-bun-http-server-manifest".to_owned(),
        host_packages: Vec::new(),
    };
    Ok(ProjectProviderConfiguration {
        entry_module: String::new(),
        contracts: vec![contract],
        candidates: vec![candidate],
        context: ProviderResolutionContext {
            target: "bun-process".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::from([(service, provider)]),
        },
        transitive_requirements: Vec::new(),
        compatibility: ProviderCompatibilityContext::default(),
    })
}

/// Returns the toolchain-owned provider catalog shared by Web builds and the
/// Playground browser execution boundary.
pub fn browser_provider_configuration() -> Result<ProjectProviderConfiguration, String> {
    provider_configuration(
        "browser",
        [
            ("clock", CLOCK_CONTRACT, BROWSER_CLOCK_MANIFEST),
            (
                "http-client",
                HTTP_CLIENT_CONTRACT,
                BROWSER_HTTP_CLIENT_MANIFEST,
            ),
        ],
    )
}

fn provider_configuration<const N: usize>(
    target: &str,
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
                    "toolchain:seseragi/runtime-{target}@{}",
                    env!("CARGO_PKG_VERSION")
                ),
                content_digest: format!("sha256:committed-runtime-{target}"),
            },
            artifact_digest: format!("sha256:committed-{target}-{name}-manifest"),
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
