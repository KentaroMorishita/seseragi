use seseragi_driver::{
    CandidateVisibility, ProjectProviderConfiguration, ProviderCandidate,
    ProviderCompatibilityContext, ProviderContract, ProviderManifest, ProviderPackageMetadata,
    ProviderResolutionContext,
};
use std::collections::{BTreeMap, BTreeSet};

const HTTP_SERVER_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/http-server/contract.json"
);
const BUN_HTTP_SERVER_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-http-server/provider.json"
);

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
