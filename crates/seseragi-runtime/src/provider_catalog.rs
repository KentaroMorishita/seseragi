use serde::Serialize;
use seseragi_driver::{
    CandidateVisibility, ProjectProviderConfiguration, ProviderCandidate,
    ProviderCompatibilityContext, ProviderContract, ProviderManifest, ProviderPackageMetadata,
    ProviderResolution, ProviderResolutionContext, ResolvedHostPackage,
};
use sha2::{Digest, Sha256};
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
const STORAGE_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/storage/contract.json"
);
const BROWSER_STORAGE_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/browser-storage/provider.json"
);
const BUN_HTTP_CLIENT_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-http-client-native/provider.json"
);
const WEBSOCKET_CLIENT_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/websocket-client/contract.json"
);
const WEBSOCKET_SERVER_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/websocket-server/contract.json"
);
const BROWSER_WEBSOCKET_CLIENT_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/browser-websocket-client/provider.json"
);
const BUN_WEBSOCKET_CLIENT_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-websocket-client/provider.json"
);
const BUN_WEBSOCKET_SERVER_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-websocket-server/provider.json"
);
const POSTGRES_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/postgres/contract.json"
);
const POSTGRES_PG_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/postgres-pg/provider.json"
);
const SQLITE_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/sqlite/contract.json"
);
const SQLITE_BUN_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/sqlite-bun/provider.json"
);
const FILESYSTEM_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/filesystem/contract.json"
);
const BUN_FILESYSTEM_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-filesystem/provider.json"
);
const CHILD_PROCESS_CONTRACT: &str = include_str!(
    "../../../examples/spec/artifacts/provider-contract-schema-1/child-process/contract.json"
);
const BUN_CHILD_PROCESS_MANIFEST: &str = include_str!(
    "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-child-process/provider.json"
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
            (
                "websocket-client",
                WEBSOCKET_CLIENT_CONTRACT,
                BUN_WEBSOCKET_CLIENT_MANIFEST,
            ),
            (
                "websocket-server",
                WEBSOCKET_SERVER_CONTRACT,
                BUN_WEBSOCKET_SERVER_MANIFEST,
            ),
            ("postgres", POSTGRES_CONTRACT, POSTGRES_PG_MANIFEST),
            ("sqlite", SQLITE_CONTRACT, SQLITE_BUN_MANIFEST),
            ("filesystem", FILESYSTEM_CONTRACT, BUN_FILESYSTEM_MANIFEST),
            (
                "child-process",
                CHILD_PROCESS_CONTRACT,
                BUN_CHILD_PROCESS_MANIFEST,
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
                "websocket-client",
                WEBSOCKET_CLIENT_CONTRACT,
                BROWSER_WEBSOCKET_CLIENT_MANIFEST,
            ),
            (
                "navigation",
                NAVIGATION_CONTRACT,
                BROWSER_NAVIGATION_MANIFEST,
            ),
            ("storage", STORAGE_CONTRACT, BROWSER_STORAGE_MANIFEST),
        ],
    )
}

fn provider_configuration<const N: usize>(
    target: &str,
    runtime_package: &str,
    artifacts: [(&str, &str, &str); N],
) -> Result<ProjectProviderConfiguration, String> {
    let package_digest = provider_package_digest(&artifacts);
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
        let host_packages = if name == "postgres" {
            vec![
                ResolvedHostPackage {
                    name: "pg".to_owned(),
                    version: "8.23.0".to_owned(),
                    source_identity: "registry:npm/pg@8.23.0".to_owned(),
                    content_digest: sha256(b"pg@8.23.0"),
                },
                ResolvedHostPackage {
                    name: "pg-cursor".to_owned(),
                    version: "2.22.0".to_owned(),
                    source_identity: "registry:npm/pg-cursor@2.22.0".to_owned(),
                    content_digest: sha256(b"pg-cursor@2.22.0"),
                },
            ]
        } else {
            Vec::new()
        };
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
                content_digest: package_digest.clone(),
            },
            artifact_digest: sha256(manifest_json.as_bytes()),
            host_packages,
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

fn provider_package_digest<const N: usize>(artifacts: &[(&str, &str, &str); N]) -> String {
    let mut digest = Sha256::new();
    for (name, contract, manifest) in artifacts {
        for bytes in [name.as_bytes(), contract.as_bytes(), manifest.as_bytes()] {
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(bytes);
        }
    }
    format!("sha256:{:x}", digest.finalize())
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bun_process_catalog_includes_clock_http_server_and_filesystem_defaults() {
        let configuration = bun_process_provider_configuration().unwrap();
        assert_eq!(configuration.context.target, "bun-process");
        assert_eq!(configuration.contracts.len(), 9);
        assert_eq!(configuration.candidates.len(), 9);
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
                .get("std/fs::FileSystem")
                .map(String::as_str),
            Some("seseragi/runtime-bun#filesystem")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/child-process::ChildProcesses")
                .map(String::as_str),
            Some("seseragi/runtime-bun#child-process")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/http/server::HttpServer")
                .map(String::as_str),
            Some("seseragi/runtime-bun#http-server")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/websocket::WebSocketClient")
                .map(String::as_str),
            Some("seseragi/runtime-bun#websocket-client")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/websocket/server::WebSocketServer")
                .map(String::as_str),
            Some("seseragi/runtime-bun#websocket-server")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("seseragi/postgres::Postgres")
                .map(String::as_str),
            Some("seseragi/runtime-postgres#pg")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("seseragi/sqlite::Sqlite")
                .map(String::as_str),
            Some("seseragi/runtime-sqlite#bun")
        );
    }

    #[test]
    fn browser_catalog_includes_navigation_and_storage_defaults() {
        let configuration = browser_provider_configuration().unwrap();
        assert_eq!(configuration.context.target, "browser");
        assert_eq!(configuration.contracts.len(), 5);
        assert_eq!(configuration.candidates.len(), 5);
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/web/navigation::Navigation")
                .map(String::as_str),
            Some("seseragi/runtime-browser#navigation")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/web/storage::Storage")
                .map(String::as_str),
            Some("seseragi/runtime-browser#storage")
        );
        assert_eq!(
            configuration
                .context
                .defaults
                .get("std/websocket::WebSocketClient")
                .map(String::as_str),
            Some("seseragi/runtime-browser#websocket-client")
        );
    }
}
