use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_child_process(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifests = [
        read_manifest(root, "bun-child-process")?,
        read_manifest(root, "node-child-process")?,
    ];
    let application =
        fs::read_to_string(root.join("runtime/ts/probes/child-process-application.ts"))
            .map_err(|error| format!("failed to read child process application probe: {error}"))?;
    if application.contains("runtime-bun") || application.contains("runtime-node") {
        return Err("child process application must not expose provider identity".to_owned());
    }
    let staging = std::env::temp_dir().join(format!(
        "seseragi-child-process-provider-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    seseragi_runtime::stage_typescript_package(&staging)?;
    for probe in ["child-process-application.ts", "child-process-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let result = [
        ("bun-process", "bun", "seseragi/runtime-bun#child-process"),
        (
            "node-process",
            "node",
            "seseragi/runtime-node#child-process",
        ),
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
        run_probe(&staging, command, target, &selected)
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
                field: "childProcesses".to_owned(),
                service: contract.identity.clone(),
            },
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![RequirementTrace {
                package: "fixture/child-process-application".to_owned(),
                module: "fixture/child-process-application::main".to_owned(),
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
    .map_err(|error| format!("failed to resolve {target} child process: {error}"))?;
    resolution
        .selected
        .into_iter()
        .next()
        .ok_or_else(|| format!("{target} child process resolution returned no selection"))
}

fn run_probe(
    staging: &Path,
    command: &str,
    target: &str,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    let entry = if command == "node" {
        let build = Command::new("bun")
            .args([
                "build",
                "child-process-provider.ts",
                "--target=node",
                "--outfile=child-process-provider.mjs",
            ])
            .current_dir(staging)
            .output()
            .map_err(|error| format!("failed to bundle Node child process probe: {error}"))?;
        if !build.status.success() {
            return Err(format!(
                "failed to bundle Node child process probe: {}",
                String::from_utf8_lossy(&build.stderr)
            ));
        }
        "child-process-provider.mjs"
    } else {
        "child-process-provider.ts"
    };
    let output = Command::new(command)
        .arg(entry)
        .current_dir(staging)
        .env("SESERAGI_CHILD_PROCESS_PROVIDER", &selected.provider)
        .env("SESERAGI_CHILD_PROCESS_SERVICE", &selected.service)
        .env("SESERAGI_CHILD_PROCESS_MODULE", &selected.entry_module)
        .env("SESERAGI_CHILD_PROCESS_EXPORT", &selected.entry_export)
        .env("SESERAGI_CHILD_PROCESS_TARGET", target)
        .output()
        .map_err(|error| format!("failed to run {target} child process probe: {error}"))?;
    let expected = format!("child process provider probe passed: {target}\n");
    if !output.status.success() || output.stdout != expected.as_bytes() {
        return Err(format!(
            "{target} child process probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/child-process/contract.json"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bun_and_node_child_process_execution_and_lifecycle() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        check_child_process(&root).unwrap();
    }
}
