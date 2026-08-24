use super::*;
use crate::ProviderManifest;
use serde_json::json;

fn contract(service: &str, field: &str) -> ProviderContract {
    ProviderContract::from_json(
        &json!({
            "schema": 1,
            "kind": "provider-contract",
            "identity": service,
            "version": { "major": 1, "minor": 0 },
            "requirement": { "field": field, "type": service },
            "operations": [{
                "identity": format!("{service}#run"),
                "kind": "one-shot",
                "input": { "kind": "unit" },
                "success": { "kind": "unit" },
                "failure": { "kind": "never" },
                "portability": { "kind": "portable" },
                "summary": "Run the service operation."
            }]
        })
        .to_string(),
    )
    .unwrap()
}

fn manifest(identity: &str, service: &str, target: &str) -> ProviderManifest {
    ProviderManifest::from_json(
        &json!({
            "schema": 1,
            "kind": "runtime-provider",
            "identity": identity,
            "service": service,
            "contractVersion": { "major": 1, "minor": 0 },
            "backend": { "family": "typescript", "abiMajor": 1 },
            "targets": [target],
            "entry": { "module": "acme/provider/entry", "export": "provider" },
            "requires": { "runtimeFeatures": ["foreign.task-load"], "hostPackages": [] }
        })
        .to_string(),
    )
    .unwrap()
}

fn candidate(identity: &str, service: &str, field: &str, target: &str) -> ProviderCandidate {
    ProviderCandidate {
        manifest: manifest(identity, service, target),
        contract: contract(service, field),
        visibility: CandidateVisibility::ToolchainBuiltin,
        package: ProviderPackageMetadata {
            version: "1.2.3".to_owned(),
            source_identity: "registry:acme/provider@1.2.3".to_owned(),
            content_digest: "sha256:package".to_owned(),
        },
        artifact_digest: "sha256:artifact".to_owned(),
        host_packages: Vec::new(),
    }
}

fn requirement(service: &str, field: &str, major: u64, minor: u64) -> RequiredService {
    RequiredService {
        requirement: ServiceRequirement {
            field: field.to_owned(),
            service: service.to_owned(),
        },
        contract_version: ContractVersion { major, minor },
        traces: vec![RequirementTrace {
            package: "acme/app".to_owned(),
            module: "acme/app::main".to_owned(),
            source: "src/main.ssrg".to_owned(),
            start: 10,
            end: 15,
        }],
    }
}

fn context(target: &str) -> ProviderResolutionContext {
    ProviderResolutionContext {
        target: target.to_owned(),
        backend_family: "typescript".to_owned(),
        backend_abi_major: 1,
        runtime_features: BTreeSet::from(["foreign.task-load".to_owned()]),
        explicit: BTreeMap::new(),
        defaults: BTreeMap::new(),
    }
}

#[test]
fn selects_explicit_then_default_then_unique_without_catalog_order_tiebreaking() {
    let service = "std/http::HttpClient";
    let candidates = [
        candidate("acme/undici#http", service, "http", "bun-process"),
        candidate("seseragi/runtime-bun#http", service, "http", "bun-process"),
    ];
    let required = [requirement(service, "http", 1, 0)];
    let ambiguous = resolve_providers(&required, &candidates, &context("bun-process")).unwrap_err();
    assert_eq!(ambiguous.code(), "SES-K0202");

    let mut explicit = context("bun-process");
    explicit
        .explicit
        .insert(service.to_owned(), "acme/undici#http".to_owned());
    assert_eq!(
        resolve_providers(&required, &candidates, &explicit)
            .unwrap()
            .selected[0]
            .provider,
        "acme/undici#http"
    );

    let mut default = context("bun-process");
    default
        .defaults
        .insert(service.to_owned(), "seseragi/runtime-bun#http".to_owned());
    assert_eq!(
        resolve_providers(&required, &candidates, &default)
            .unwrap()
            .selected[0]
            .provider,
        "seseragi/runtime-bun#http"
    );
}

#[test]
fn reports_missing_selection_and_compatibility_boundaries_without_fallback() {
    let service = "std/clock::Clock";
    let required = [requirement(service, "clock", 1, 0)];
    assert_eq!(
        resolve_providers(&required, &[], &context("bun-process"))
            .unwrap_err()
            .code(),
        "SES-K0201"
    );

    let candidates = [candidate(
        "seseragi/runtime-bun#clock",
        service,
        "clock",
        "bun-process",
    )];
    let mut explicit = context("node-process");
    explicit
        .explicit
        .insert(service.to_owned(), "seseragi/runtime-bun#clock".to_owned());
    assert_eq!(
        resolve_providers(&required, &candidates, &explicit)
            .unwrap_err()
            .code(),
        "SES-K0203"
    );
    explicit
        .explicit
        .insert(service.to_owned(), "hidden/provider#clock".to_owned());
    assert_eq!(
        resolve_providers(&required, &candidates, &explicit)
            .unwrap_err()
            .code(),
        "SES-K0208"
    );
}

#[test]
fn merges_minor_versions_and_rejects_transitive_major_conflicts() {
    let service = "std/clock::Clock";
    let candidates = [candidate(
        "seseragi/runtime-bun#clock",
        service,
        "clock",
        "bun-process",
    )];
    let requirements = [
        requirement(service, "clock", 1, 0),
        requirement(service, "clock", 1, 1),
    ];
    assert_eq!(
        resolve_providers(&requirements, &candidates, &context("bun-process"))
            .unwrap_err()
            .code(),
        "SES-K0204"
    );
    let conflict = [
        requirement(service, "clock", 1, 0),
        requirement(service, "clock", 2, 0),
    ];
    assert_eq!(
        resolve_providers(&conflict, &candidates, &context("bun-process"))
            .unwrap_err()
            .code(),
        "SES-K0207"
    );
}

#[test]
fn emits_stable_lock_and_build_metadata_without_absolute_paths() {
    let service = "std/fs::FileSystem";
    let selected = resolve_providers(
        &[requirement(service, "fileSystem", 1, 0)],
        &[candidate(
            "seseragi/runtime-node#filesystem",
            service,
            "fileSystem",
            "node-process",
        )],
        &context("node-process"),
    )
    .unwrap();
    let lock = serde_json::to_string(&selected.lock).unwrap();
    let build = serde_json::to_string(&selected.build).unwrap();
    assert!(lock.contains("sha256:artifact"));
    assert!(lock.contains("registry:acme/provider@1.2.3"));
    assert!(lock.contains("requiredContractVersion"));
    assert!(lock.contains("providerContractVersion"));
    assert!(!lock.contains("/Users/") && !build.contains("/Users/"));
    assert_eq!(selected.lock.providers, selected.build.providers);
    let project = selected.lock.project_lock_selections();
    assert_eq!(project.len(), 1);
    assert_eq!(project[0].service, service);
    assert_eq!(project[0].required_contract, "1.0");
    assert_eq!(project[0].provider_contract, "1.0");
    assert_eq!(project[0].artifact_digest, "sha256:artifact");
}

#[test]
fn reports_incompatible_candidates_in_identity_order() {
    let service = "std/clock::Clock";
    let required = [requirement(service, "clock", 1, 0)];
    let candidates = [
        candidate("zeta/provider#clock", service, "clock", "browser"),
        candidate("alpha/provider#clock", service, "clock", "browser"),
    ];
    let error = resolve_providers(&required, &candidates, &context("bun-process")).unwrap_err();
    let ProviderResolutionError::NoCompatible { rejections, .. } = error else {
        panic!("expected incompatible provider error");
    };
    assert_eq!(rejections[0].provider, "alpha/provider#clock");
    assert_eq!(rejections[1].provider, "zeta/provider#clock");
}

#[test]
fn carries_requirement_target_and_selection_origin_in_errors() {
    let service = "std/clock::Clock";
    let candidates = [candidate(
        "seseragi/runtime-bun#clock",
        service,
        "clock",
        "bun-process",
    )];
    let mut selected = context("node-process");
    selected
        .defaults
        .insert(service.to_owned(), "seseragi/runtime-bun#clock".to_owned());
    let error = resolve_providers(
        &[requirement(service, "clock", 1, 0)],
        &candidates,
        &selected,
    )
    .unwrap_err();
    let ProviderResolutionError::Incompatible {
        context, selection, ..
    } = error
    else {
        panic!("expected selected provider incompatibility");
    };
    assert_eq!(context.service, service);
    assert_eq!(context.target, "node-process");
    assert_eq!(context.traces[0].source, "src/main.ssrg");
    assert_eq!(selection, ProviderSelectionSource::ToolchainDefault);
}

#[test]
fn rejects_absolute_package_sources_from_lock_metadata() {
    let service = "std/clock::Clock";
    for source in ["/tmp/provider", "path:relative/provider", "C:/provider"] {
        let mut provider = candidate(
            "seseragi/runtime-bun#clock",
            service,
            "clock",
            "bun-process",
        );
        provider.package.source_identity = source.to_owned();
        assert_eq!(
            resolve_providers(
                &[requirement(service, "clock", 1, 0)],
                &[provider],
                &context("bun-process"),
            )
            .unwrap_err()
            .code(),
            "SES-K0200"
        );
    }
}
