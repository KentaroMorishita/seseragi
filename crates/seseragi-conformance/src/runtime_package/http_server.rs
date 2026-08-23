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
    let manifests = [
        read_manifest(root, "bun-http-server")?,
        read_manifest(root, "node-http-server")?,
    ];
    let application = fs::read_to_string(root.join("runtime/ts/probes/http-server-application.ts"))
        .map_err(|error| format!("failed to read HTTP server application probe: {error}"))?;
    if application.contains("runtime-bun") || application.contains("runtime-node") {
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
    seseragi_runtime::stage_typescript_package(&staging)?;
    for probe in ["http-server-application.ts", "http-server-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let result = [
        ("bun-process", "bun", "seseragi/runtime-bun#http-server"),
        ("node-process", "node", "seseragi/runtime-node#http-server"),
    ]
    .into_iter()
    .try_for_each(|(target, command, expected_provider)| {
        let selected = select(&contract, &manifests, target)?;
        if selected.provider != expected_provider {
            return Err(format!(
                "{target} selected unexpected provider {}",
                selected.provider
            ));
        }
        run_probe(&staging, command, target, available_port()?, &selected)
    });
    let cleanup = fs::remove_dir_all(&staging)
        .map_err(|error| format!("failed to clean HTTP server staging: {error}"));
    result.and(cleanup)
}

fn select(
    contract: &ProviderContract,
    manifests: &[ProviderManifest; 2],
    target: &str,
) -> Result<seseragi_provider::ProviderSelectionMetadata, String> {
    let candidates = manifests
        .iter()
        .cloned()
        .map(|manifest| ProviderCandidate {
            package: ProviderPackageMetadata {
                version: env!("CARGO_PKG_VERSION").to_owned(),
                source_identity: format!("toolchain:{}", manifest.identity),
                content_digest: format!("sha256:{}", manifest.identity),
            },
            artifact_digest: format!("sha256:{}-manifest", manifest.identity),
            host_packages: Vec::new(),
            contract: contract.clone(),
            manifest,
            visibility: CandidateVisibility::ToolchainBuiltin,
        })
        .collect::<Vec<_>>();
    let service = contract.identity.clone();
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
        &candidates,
        &ProviderResolutionContext {
            target: target.to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::new(),
        },
    )
    .map_err(|error| format!("failed to resolve {target} HTTP server provider: {error}"))?;
    resolution
        .selected
        .into_iter()
        .next()
        .ok_or_else(|| format!("{target} HTTP server resolution returned no selection"))
}

fn run_probe(
    staging: &Path,
    command: &str,
    target: &str,
    port: u16,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    let entry = if command == "node" {
        let build = Command::new("bun")
            .args([
                "build",
                "http-server-provider.ts",
                "--target=node",
                "--outfile=http-server-provider.mjs",
            ])
            .current_dir(staging)
            .output()
            .map_err(|error| format!("failed to bundle Node HTTP server probe: {error}"))?;
        if !build.status.success() {
            return Err(format!(
                "failed to bundle Node HTTP server probe: {}",
                String::from_utf8_lossy(&build.stderr)
            ));
        }
        "http-server-provider.mjs"
    } else {
        "http-server-provider.ts"
    };
    let output = Command::new(command)
        .arg(entry)
        .current_dir(staging)
        .env("SESERAGI_HTTP_SERVER_PROVIDER", &selected.provider)
        .env("SESERAGI_HTTP_SERVER_SERVICE", &selected.service)
        .env("SESERAGI_HTTP_SERVER_MODULE", &selected.entry_module)
        .env("SESERAGI_HTTP_SERVER_EXPORT", &selected.entry_export)
        .env("SESERAGI_HTTP_SERVER_PORT", port.to_string())
        .env("SESERAGI_HTTP_SERVER_TARGET", target)
        .output()
        .map_err(|error| format!("failed to run HTTP server probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "HTTP server provider probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let expected = format!("HTTP server provider probe passed: {target}\n");
    if output.stdout != expected.as_bytes() {
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

fn read_manifest(root: &Path, name: &str) -> Result<ProviderManifest, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-manifest-schema-1")
            .join(name)
            .join("provider.json"),
    )
    .map_err(|error| format!("failed to read Bun HTTP server manifest: {error}"))?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_effectful_http_server_execution_and_lifecycle() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        check_http_server(&root).unwrap();
    }
}
