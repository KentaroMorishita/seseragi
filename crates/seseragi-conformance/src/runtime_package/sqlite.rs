use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_sqlite(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifest = read_manifest(root)?;
    audit_builtin_driver(root, &manifest)?;
    let selected = select(&contract, &manifest, "bun-process")?;
    if selected.provider != "seseragi/runtime-sqlite#bun" {
        return Err(format!(
            "bun-process selected unexpected SQLite provider {}",
            selected.provider
        ));
    }
    if select(&contract, &manifest, "node-process").is_ok() {
        return Err("SQLite Bun Provider must not resolve for node-process".to_owned());
    }

    let staging =
        std::env::temp_dir().join(format!("seseragi-sqlite-provider-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    seseragi_runtime::stage_typescript_package(&staging)?;
    fs::copy(
        root.join("runtime/ts/probes/sqlite-provider.ts"),
        staging.join("sqlite-provider.ts"),
    )
    .map_err(|error| format!("failed to stage SQLite provider probe: {error}"))?;
    let result = run_probe(&staging, &selected);
    let cleanup = fs::remove_dir_all(&staging).map_err(|error| error.to_string());
    result.and(cleanup)
}

fn audit_builtin_driver(root: &Path, manifest: &ProviderManifest) -> Result<(), String> {
    let entry = fs::read_to_string(root.join("runtime/providers/sqlite/bun.ts"))
        .map_err(|error| format!("failed to read SQLite Provider entry: {error}"))?;
    if !entry.contains("from \"bun:sqlite\"") {
        return Err("SQLite Provider must wrap the Bun built-in driver".to_owned());
    }
    if !manifest.requires.host_packages.is_empty() {
        return Err("SQLite Bun Provider must not declare an external host package".to_owned());
    }
    Ok(())
}

fn select(
    contract: &ProviderContract,
    manifest: &ProviderManifest,
    target: &str,
) -> Result<seseragi_provider::ProviderSelectionMetadata, String> {
    let candidate = ProviderCandidate {
        package: ProviderPackageMetadata {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            source_identity: "toolchain:seseragi-sqlite-bun".to_owned(),
            content_digest: "sha256:seseragi-sqlite-bun".to_owned(),
        },
        artifact_digest: "sha256:seseragi-sqlite-bun-manifest".to_owned(),
        host_packages: vec![],
        contract: contract.clone(),
        manifest: manifest.clone(),
        visibility: CandidateVisibility::RootDirectDependency,
    };
    let resolution = resolve_providers(
        &[RequiredService {
            requirement: ServiceRequirement {
                field: "sqlite".to_owned(),
                service: contract.identity.clone(),
            },
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![RequirementTrace {
                package: "fixture/sqlite-application".to_owned(),
                module: "fixture/sqlite-application::main".to_owned(),
                source: "src/main.ssrg".to_owned(),
                start: 0,
                end: 10,
            }],
        }],
        &[candidate],
        &ProviderResolutionContext {
            target: target.to_owned(),
            backend_family: "typescript".to_owned(),
            backend_abi_major: 1,
            runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
            explicit: BTreeMap::new(),
            defaults: BTreeMap::new(),
        },
    )
    .map_err(|error| format!("failed to resolve {target} SQLite Provider: {error}"))?;
    resolution
        .selected
        .into_iter()
        .next()
        .ok_or_else(|| format!("{target} SQLite resolution returned no selection"))
}

fn run_probe(
    staging: &Path,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    let output = Command::new("bun")
        .arg("sqlite-provider.ts")
        .current_dir(staging)
        .env("SESERAGI_SQLITE_PROVIDER", &selected.provider)
        .env("SESERAGI_SQLITE_SERVICE", &selected.service)
        .env("SESERAGI_SQLITE_MODULE", &selected.entry_module)
        .env("SESERAGI_SQLITE_EXPORT", &selected.entry_export)
        .output()
        .map_err(|error| format!("failed to run SQLite Provider probe: {error}"))?;
    if !output.status.success() || output.stdout != b"SQLite provider probe passed: bun-process\n" {
        return Err(format!(
            "SQLite Provider probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/sqlite/contract.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderContract::from_json(&raw).map_err(|error| error.to_string())
}

fn read_manifest(root: &Path) -> Result<ProviderManifest, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-manifest-schema-1/sqlite-bun/provider.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}
