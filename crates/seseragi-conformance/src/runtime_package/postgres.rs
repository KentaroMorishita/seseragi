use seseragi_provider::{
    resolve_providers, CandidateVisibility, ContractVersion, ProviderCandidate, ProviderContract,
    ProviderManifest, ProviderPackageMetadata, ProviderResolutionContext, RequiredService,
    RequirementTrace, ResolvedHostPackage, ServiceRequirement,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

pub(super) fn check_postgres(root: &Path) -> Result<(), String> {
    let contract = read_contract(root)?;
    let manifest = read_manifest(root)?;
    audit_external_driver(root)?;
    let application = fs::read_to_string(root.join("runtime/ts/probes/postgres-application.ts"))
        .map_err(|error| format!("failed to read PostgreSQL application probe: {error}"))?;
    if application.contains("runtime-postgres") || application.contains("pg-cursor") {
        return Err("PostgreSQL application must not expose provider identity".to_owned());
    }
    let staging =
        std::env::temp_dir().join(format!("seseragi-postgres-provider-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
    seseragi_runtime::stage_typescript_package(&staging)?;
    for probe in ["postgres-application.ts", "postgres-provider.ts"] {
        fs::copy(
            root.join("runtime/ts/probes").join(probe),
            staging.join(probe),
        )
        .map_err(|error| format!("failed to stage {probe}: {error}"))?;
    }
    let result = ["bun-process", "node-process"]
        .into_iter()
        .try_for_each(|target| {
            let selected = select(&contract, &manifest, target)?;
            if selected.provider != "seseragi/runtime-postgres#pg" {
                return Err(format!(
                    "{target} selected unexpected PostgreSQL provider {}",
                    selected.provider
                ));
            }
            run_probe(&staging, target, &selected)
        });
    let cleanup = fs::remove_dir_all(&staging).map_err(|error| error.to_string());
    result.and(cleanup)
}

fn audit_external_driver(root: &Path) -> Result<(), String> {
    let entry = fs::read_to_string(root.join("runtime/providers/postgres/pg.ts"))
        .map_err(|error| format!("failed to read PostgreSQL provider entry: {error}"))?;
    if !entry.contains("from \"pg\"") || !entry.contains("from \"pg-cursor\"") {
        return Err("PostgreSQL provider must wrap pg and pg-cursor".to_owned());
    }
    let package: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("runtime/providers/package.json"))
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    for dependency in ["pg", "pg-cursor"] {
        if package
            .pointer(&format!("/dependencies/{dependency}"))
            .and_then(|value| value.as_str())
            .is_none()
        {
            return Err(format!(
                "PostgreSQL provider package dependency is missing: {dependency}"
            ));
        }
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
            source_identity: "toolchain:seseragi-postgres-pg".to_owned(),
            content_digest: "sha256:seseragi-postgres-pg".to_owned(),
        },
        artifact_digest: "sha256:seseragi-postgres-pg-manifest".to_owned(),
        host_packages: vec![
            ResolvedHostPackage {
                name: "pg".to_owned(),
                version: "8.23.0".to_owned(),
                source_identity: "registry:pg@8.23.0".to_owned(),
                content_digest: "sha256:pg-8.23.0".to_owned(),
            },
            ResolvedHostPackage {
                name: "pg-cursor".to_owned(),
                version: "2.22.0".to_owned(),
                source_identity: "registry:pg-cursor@2.22.0".to_owned(),
                content_digest: "sha256:pg-cursor-2.22.0".to_owned(),
            },
        ],
        contract: contract.clone(),
        manifest: manifest.clone(),
        visibility: CandidateVisibility::RootDirectDependency,
    };
    let resolution = resolve_providers(
        &[RequiredService {
            requirement: ServiceRequirement {
                field: "postgres".to_owned(),
                service: contract.identity.clone(),
            },
            contract_version: ContractVersion { major: 1, minor: 0 },
            traces: vec![RequirementTrace {
                package: "fixture/postgres-application".to_owned(),
                module: "fixture/postgres-application::main".to_owned(),
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
    .map_err(|error| format!("failed to resolve {target} PostgreSQL provider: {error}"))?;
    resolution
        .selected
        .into_iter()
        .next()
        .ok_or_else(|| format!("{target} PostgreSQL resolution returned no selection"))
}

fn run_probe(
    staging: &Path,
    target: &str,
    selected: &seseragi_provider::ProviderSelectionMetadata,
) -> Result<(), String> {
    let (command, entry) = if target == "node-process" {
        let build = Command::new("bun")
            .args([
                "build",
                "postgres-provider.ts",
                "--target=node",
                "--outfile=postgres-provider.mjs",
            ])
            .current_dir(staging)
            .output()
            .map_err(|error| format!("failed to bundle Node PostgreSQL probe: {error}"))?;
        if !build.status.success() {
            return Err(format!(
                "failed to bundle Node PostgreSQL probe: {}",
                String::from_utf8_lossy(&build.stderr)
            ));
        }
        ("node", "postgres-provider.mjs")
    } else {
        ("bun", "postgres-provider.ts")
    };
    let output = Command::new(command)
        .arg(entry)
        .current_dir(staging)
        .env("SESERAGI_POSTGRES_PROVIDER", &selected.provider)
        .env("SESERAGI_POSTGRES_SERVICE", &selected.service)
        .env("SESERAGI_POSTGRES_MODULE", &selected.entry_module)
        .env("SESERAGI_POSTGRES_EXPORT", &selected.entry_export)
        .env("SESERAGI_POSTGRES_TARGET", target)
        .output()
        .map_err(|error| format!("failed to run {target} PostgreSQL probe: {error}"))?;
    let expected = format!("PostgreSQL provider probe passed: {target}\n");
    if !output.status.success() || output.stdout != expected.as_bytes() {
        return Err(format!(
            "{target} PostgreSQL probe failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn read_contract(root: &Path) -> Result<ProviderContract, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-contract-schema-1/postgres/contract.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderContract::from_json(&raw).map_err(|error| error.to_string())
}

fn read_manifest(root: &Path) -> Result<ProviderManifest, String> {
    let raw = fs::read_to_string(
        root.join("examples/spec/artifacts/provider-manifest-schema-1/postgres-pg/provider.json"),
    )
    .map_err(|error| error.to_string())?;
    ProviderManifest::from_json(&raw).map_err(|error| error.to_string())
}
