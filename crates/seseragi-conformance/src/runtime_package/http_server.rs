use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

pub(super) fn check_http_server(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifest = read_manifest(root)?;
    let service = contract.identity.clone();
    let provider = manifest.identity.clone();
    let resolution = resolve_providers(
        &[RequiredService {
            requirement: ServiceRequirement {
                field: "httpServer".to_owned(),
                service: service.clone(),
            },
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![RequirementTrace {
                package: "fixture/http-server-application".to_owned(),
                module: "fixture/http-server-application::main".to_owned(),
                source: "src/main.ssrg".to_owned(),
                start: 0,
                end: 10,
            }],
        }],
        &[ProviderCandidate {
            manifest,
            contract,
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
        }],
        &ProviderResolutionContext {
            target: "bun-process".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::from([(service, provider)]),
        },
    )
    .map_err(|error| format!("failed to resolve built-in HTTP server provider: {error}"))?;
    let selected = resolution
        .selected
        .first()
        .ok_or_else(|| "HTTP server resolution returned no selection".to_owned())?;

    let application = fs::read_to_string(root.join("runtime/ts/probes/http-server-application.ts"))
        .map_err(|error| format!("failed to read HTTP server application probe: {error}"))?;
    if application.contains("runtime-bun") || application.contains(&selected.provider) {
        return Err("HTTP server application must not expose provider identity".to_owned());
    }

    let staging = std::env::temp_dir().join(format!(
        "seseragi-http-server-provider-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .map_err(|error| format!("failed to clean HTTP server staging: {error}"))?;
    }
    fs::create_dir_all(&staging)
        .map_err(|error| format!("failed to create HTTP server staging: {error}"))?;
    let result = run_probe(root, &staging, selected);
    let cleanup = fs::remove_dir_all(&staging)
        .map_err(|error| format!("failed to clean HTTP server staging: {error}"));
    result.and(cleanup)
}

fn run_probe(
    root: &Path,
    staging: &Path,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    seseragi_runtime::stage_typescript_package(staging)?;
    for probe in ["http-server-application.ts", "http-server-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let port = available_port()?;
    let output = Command::new("bun")
        .arg("http-server-provider.ts")
        .current_dir(staging)
        .env("SESERAGI_HTTP_SERVER_PROVIDER", &selected.provider)
        .env("SESERAGI_HTTP_SERVER_SERVICE", &selected.service)
        .env("SESERAGI_HTTP_SERVER_MODULE", &selected.entry_module)
        .env("SESERAGI_HTTP_SERVER_EXPORT", &selected.entry_export)
        .env("SESERAGI_HTTP_SERVER_PORT", port.to_string())
        .output()
        .map_err(|error| format!("failed to run HTTP server probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "HTTP server provider probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"HTTP server provider probe passed\n" {
        return Err(format!(
            "HTTP server provider probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn available_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("failed to reserve HTTP server probe port: {error}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| format!("failed to read HTTP server probe port: {error}"))
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/http-server/contract.json"),
    )
    .map_err(|error| format!("failed to read HTTP server Contract: {error}"))?;
    ProviderContract::from_json(&raw).map_err(|error| error.to_string())
}

fn read_manifest(root: &Path) -> Result<ProviderManifest, String> {
    let raw =
        fs::read_to_string(root.join(
            "examples/spec/artifacts/provider-manifest-schema-1/bun-http-server/provider.json",
        ))
        .map_err(|error| format!("failed to read Bun HTTP server manifest: {error}"))?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}
