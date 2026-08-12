use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_filesystem(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifests = [
        read_manifest(root, "bun-filesystem")?,
        read_manifest(root, "node-filesystem")?,
    ];
    let application = fs::read_to_string(root.join("runtime/ts/probes/filesystem-application.ts"))
        .map_err(|error| format!("failed to read filesystem application probe: {error}"))?;
    if application.contains("runtime-bun") || application.contains("runtime-node") {
        return Err("filesystem application must not expose provider identity".to_owned());
    }
    let staging = std::env::temp_dir().join(format!(
        "seseragi-filesystem-provider-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    seseragi_runtime::stage_typescript_package(&staging)?;
    for probe in ["filesystem-application.ts", "filesystem-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let fixture = staging.join("fixture.txt");
    fs::write(&fixture, b"seseragi-filesystem")
        .map_err(|error| format!("failed to write filesystem fixture: {error}"))?;
    let result = [
        ("bun-process", "bun", "seseragi/runtime-bun#filesystem"),
        ("node-process", "node", "seseragi/runtime-node#filesystem"),
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
        run_probe(&staging, &fixture, command, target, &selected)
    });
    let cleanup = fs::remove_dir_all(&staging).map_err(|error| error.to_string());
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
    let resolution = resolve_providers(
        &[RequiredService {
            requirement: ServiceRequirement {
                field: "fileSystem".to_owned(),
                service: contract.identity.clone(),
            },
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![RequirementTrace {
                package: "fixture/filesystem-application".to_owned(),
                module: "fixture/filesystem-application::main".to_owned(),
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
    .map_err(|error| format!("failed to resolve {target} filesystem: {error}"))?;
    resolution
        .selected
        .into_iter()
        .next()
        .ok_or_else(|| format!("{target} filesystem resolution returned no selection"))
}

fn run_probe(
    staging: &Path,
    fixture: &Path,
    command: &str,
    target: &str,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    let entry = if command == "node" {
        let build = Command::new("bun")
            .args([
                "build",
                "filesystem-provider.ts",
                "--target=node",
                "--outfile=filesystem-provider.mjs",
            ])
            .current_dir(staging)
            .output()
            .map_err(|error| format!("failed to bundle Node filesystem probe: {error}"))?;
        if !build.status.success() {
            return Err(format!(
                "failed to bundle Node filesystem probe: {}",
                String::from_utf8_lossy(&build.stderr)
            ));
        }
        "filesystem-provider.mjs"
    } else {
        "filesystem-provider.ts"
    };
    let output = Command::new(command)
        .arg(entry)
        .current_dir(staging)
        .env("SESERAGI_FILESYSTEM_PROVIDER", &selected.provider)
        .env("SESERAGI_FILESYSTEM_SERVICE", &selected.service)
        .env("SESERAGI_FILESYSTEM_MODULE", &selected.entry_module)
        .env("SESERAGI_FILESYSTEM_EXPORT", &selected.entry_export)
        .env("SESERAGI_FILESYSTEM_TARGET", target)
        .env("SESERAGI_FILESYSTEM_FIXTURE", fixture)
        .output()
        .map_err(|error| format!("failed to run {target} filesystem probe: {error}"))?;
    let expected = format!("filesystem provider probe passed: {target}\n");
    if !output.status.success() || output.stdout != expected.as_bytes() {
        return Err(format!(
            "{target} filesystem probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/filesystem/contract.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderContract::from_json(&raw).map_err(|error| error.to_string())
}

fn read_manifest(root: &Path, name: &str) -> Result<ProviderManifest, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-manifest-schema-1")
            .join(name)
            .join("provider.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}
