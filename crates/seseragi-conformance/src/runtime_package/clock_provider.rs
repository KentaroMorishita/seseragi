use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_clock_provider(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifest = read_manifest(root)?;
    let service = contract.identity.clone();
    let provider = manifest.identity.clone();
    let candidate = ProviderCandidate {
        manifest,
        contract,
        visibility: CandidateVisibility::ToolchainBuiltin,
        package: ProviderPackageMetadata {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            source_identity: format!(
                "toolchain:seseragi/runtime-bun@{}",
                env!("CARGO_PKG_VERSION")
            ),
            content_digest: "sha256:committed-runtime-bun-clock".to_owned(),
        },
        artifact_digest: "sha256:committed-bun-clock-provider-manifest".to_owned(),
        host_packages: Vec::new(),
    };
    let requirement = RequiredService {
        requirement: ServiceRequirement {
            field: "clock".to_owned(),
            service: service.clone(),
        },
        contract_version: ContractVersion { major: 1, minor: 0 },
        traces: vec![RequirementTrace {
            package: "fixture/clock-application".to_owned(),
            module: "fixture/clock-application::main".to_owned(),
            source: "src/main.ssrg".to_owned(),
            start: 0,
            end: 5,
        }],
    };
    let resolution = resolve_providers(
        &[requirement],
        &[candidate],
        &ProviderResolutionContext {
            target: "bun-process".to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::from([(service, provider)]),
        },
    )
    .map_err(|error| format!("failed to resolve built-in Clock provider: {error}"))?;
    let selected = resolution
        .selected
        .first()
        .ok_or_else(|| "Clock provider resolution returned no selection".to_owned())?;

    let application = fs::read_to_string(root.join("runtime/ts/probes/clock-application.ts"))
        .map_err(|error| format!("failed to read Clock application probe: {error}"))?;
    if application.contains("runtime-bun") || application.contains(&selected.provider) {
        return Err("Clock application probe must not expose provider identity".to_owned());
    }

    let staging =
        std::env::temp_dir().join(format!("seseragi-clock-provider-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .map_err(|error| format!("failed to clean Clock probe staging: {error}"))?;
    }
    fs::create_dir_all(&staging)
        .map_err(|error| format!("failed to create Clock probe staging: {error}"))?;
    let result = run_probe(root, &staging, selected);
    let cleanup = fs::remove_dir_all(&staging)
        .map_err(|error| format!("failed to clean Clock probe staging: {error}"));
    result.and(cleanup)
}

fn run_probe(
    root: &Path,
    staging: &Path,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    seseragi_runtime::stage_typescript_package(staging)?;
    for probe in ["clock-application.ts", "clock-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let output = Command::new("bun")
        .arg("clock-provider.ts")
        .current_dir(staging)
        .env("SESERAGI_CLOCK_PROVIDER", &selected.provider)
        .env("SESERAGI_CLOCK_SERVICE", &selected.service)
        .env("SESERAGI_CLOCK_MODULE", &selected.entry_module)
        .env("SESERAGI_CLOCK_EXPORT", &selected.entry_export)
        .output()
        .map_err(|error| format!("failed to run Clock provider probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Clock provider probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if output.stdout != b"clock provider probe passed\n" {
        return Err(format!(
            "Clock provider probe returned unexpected output: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/clock/contract.json"),
    )
    .map_err(|error| format!("failed to read Clock Contract: {error}"))?;
    ProviderContract::from_json(&raw).map_err(|error| error.to_string())
}

fn read_manifest(root: &Path) -> Result<ProviderManifest, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-manifest-schema-1/bun-clock/provider.json"),
    )
    .map_err(|error| format!("failed to read Bun Clock manifest: {error}"))?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}
