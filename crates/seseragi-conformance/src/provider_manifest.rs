use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_manifest_case(case: &Path) -> Result<(), String> {
    let path = case.join("provider.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider manifest: {error}"))?;
    check_provider_manifest(&raw)
}

fn check_provider_manifest(raw: &str) -> Result<(), String> {
    let manifest: ProviderManifest = serde_json::from_str(raw)
        .map_err(|error| format!("failed to parse provider manifest: {error}"))?;
    if manifest.schema != 1 {
        return Err("provider manifest must use schema 1".to_owned());
    }
    if manifest.kind != "runtime-provider" {
        return Err("provider manifest kind must be runtime-provider".to_owned());
    }
    check_provider_identity(&manifest.identity)?;
    check_type_identity(&manifest.service, "provider service identity")?;
    if manifest.contract_version.major == 0 {
        return Err("provider contract version major must be greater than zero".to_owned());
    }
    check_kebab_identifier(&manifest.backend.family, "provider backend family")?;
    if manifest.backend.abi_major == 0 {
        return Err("provider backend ABI major must be greater than zero".to_owned());
    }
    check_unique_identifiers(&manifest.targets, "provider target")?;
    if manifest.targets.is_empty() {
        return Err("provider targets must not be empty".to_owned());
    }
    check_module_specifier(&manifest.entry.module)?;
    check_typescript_export(&manifest.entry.export)?;
    check_unique_dotted_identifiers(
        &manifest.requires.runtime_features,
        "provider runtime feature",
    )?;
    check_host_packages(&manifest.requires.host_packages)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderManifest {
    schema: u32,
    kind: String,
    identity: String,
    service: String,
    #[serde(rename = "contractVersion")]
    contract_version: ContractVersion,
    backend: Backend,
    targets: Vec<String>,
    entry: Entry,
    requires: Requirements,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct ContractVersion {
    major: u32,
    minor: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Backend {
    family: String,
    #[serde(rename = "abiMajor")]
    abi_major: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    module: String,
    export: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirements {
    #[serde(rename = "runtimeFeatures")]
    runtime_features: Vec<String>,
    #[serde(rename = "hostPackages")]
    host_packages: Vec<HostPackage>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HostPackage {
    name: String,
    version: String,
}

fn check_provider_identity(identity: &str) -> Result<(), String> {
    let Some((package, provider)) = identity.split_once('#') else {
        return Err("provider identity must contain one # separator".to_owned());
    };
    if provider.contains('#') || package.is_empty() || provider.is_empty() {
        return Err("provider identity must contain one # separator".to_owned());
    }
    for segment in package.split('/') {
        check_kebab_identifier(segment, "provider identity package")?;
    }
    check_kebab_identifier(provider, "provider identity name")
}

fn check_type_identity(identity: &str, label: &str) -> Result<(), String> {
    let Some((module, symbol)) = identity.rsplit_once("::") else {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    };
    let segments = module
        .split("::")
        .flat_map(|part| part.split('/'))
        .collect::<Vec<_>>();
    if segments.len() < 2 || symbol.is_empty() {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    }
    for segment in segments {
        check_kebab_identifier(segment, label)?;
    }
    let mut chars = symbol.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        || !chars.all(|character| character.is_ascii_alphanumeric())
    {
        return Err(format!("{label} symbol must use UpperCamelCase"));
    }
    Ok(())
}

fn check_module_specifier(specifier: &str) -> Result<(), String> {
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || specifier.ends_with('/')
        || specifier.contains("//")
        || specifier.contains('\0')
    {
        return Err("provider entry.module must be a canonical package specifier".to_owned());
    }
    let segments = specifier.split('/').collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err("provider entry.module must include a package export".to_owned());
    }
    for segment in segments {
        check_kebab_identifier(segment, "provider entry.module")?;
    }
    Ok(())
}

fn check_typescript_export(export: &str) -> Result<(), String> {
    let mut chars = export.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        || !chars.all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err("provider entry.export must be a TypeScript identifier".to_owned());
    }
    Ok(())
}

fn check_unique_identifiers(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        check_kebab_identifier(value, label)?;
        if !seen.insert(value) {
            return Err(format!("{label} is duplicated: {value}"));
        }
    }
    Ok(())
}

fn check_unique_dotted_identifiers(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.is_empty() {
            return Err(format!("{label} must not be empty"));
        }
        for segment in value.split('.') {
            check_kebab_identifier(segment, label)?;
        }
        if !seen.insert(value) {
            return Err(format!("{label} is duplicated: {value}"));
        }
    }
    Ok(())
}

fn check_host_packages(packages: &[HostPackage]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for package in packages {
        if package.name.is_empty()
            || package.name.chars().any(char::is_whitespace)
            || package.version.trim().is_empty()
        {
            return Err("provider host package must have a name and version range".to_owned());
        }
        if !seen.insert(&package.name) {
            return Err(format!(
                "provider host package is duplicated: {}",
                package.name
            ));
        }
    }
    Ok(())
}

fn check_kebab_identifier(value: &str, label: &str) -> Result<(), String> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(format!("{label} must use lowercase kebab-case"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_provider_manifest, check_provider_manifest_case, ContractVersion};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
    struct Candidate<'a> {
        identity: &'a str,
        service: &'a str,
        contract: ContractVersion,
        target: &'a str,
        backend: &'a str,
        abi_major: u32,
        runtime_features: &'a [&'a str],
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Resolution<'a> {
        Selected(&'a str),
        Diagnostic(&'static str),
    }

    fn resolve<'a>(
        requirements: &[(&str, ContractVersion)],
        candidates: &'a [Candidate<'a>],
        explicit: &BTreeMap<&str, &str>,
        defaults: &BTreeMap<&str, &str>,
        target: &str,
        backend: &str,
        abi_major: u32,
        runtime_features: &BTreeSet<&str>,
    ) -> Resolution<'a> {
        let service = requirements[0].0;
        let required = requirements[0].1;
        if requirements.iter().any(|(candidate_service, version)| {
            *candidate_service != service || version.major != required.major
        }) {
            return Resolution::Diagnostic("SES-K0207 provider.requirement-conflict");
        }
        let required_minor = requirements
            .iter()
            .map(|(_, version)| version.minor)
            .max()
            .unwrap_or(required.minor);
        let service_candidates = candidates
            .iter()
            .filter(|candidate| candidate.service == service)
            .collect::<Vec<_>>();
        if service_candidates.is_empty() {
            return Resolution::Diagnostic("SES-K0201 provider.missing");
        }

        let pinned = explicit.get(service).or_else(|| defaults.get(service));
        if let Some(identity) = pinned {
            let Some(candidate) = service_candidates
                .iter()
                .copied()
                .find(|candidate| candidate.identity == *identity)
            else {
                return Resolution::Diagnostic("SES-K0208 provider.selection-unavailable");
            };
            return compatibility(
                candidate,
                required.major,
                required_minor,
                target,
                backend,
                abi_major,
                runtime_features,
            )
            .map_or_else(Resolution::Diagnostic, |_| {
                Resolution::Selected(candidate.identity)
            });
        }

        let compatible = service_candidates
            .iter()
            .copied()
            .filter(|candidate| {
                compatibility(
                    candidate,
                    required.major,
                    required_minor,
                    target,
                    backend,
                    abi_major,
                    runtime_features,
                )
                .is_ok()
            })
            .collect::<Vec<_>>();
        match compatible.as_slice() {
            [candidate] => Resolution::Selected(candidate.identity),
            [] => {
                let code = compatibility(
                    service_candidates[0],
                    required.major,
                    required_minor,
                    target,
                    backend,
                    abi_major,
                    runtime_features,
                )
                .unwrap_err();
                Resolution::Diagnostic(code)
            }
            _ => Resolution::Diagnostic("SES-K0202 provider.ambiguous"),
        }
    }

    fn compatibility(
        candidate: &Candidate<'_>,
        required_major: u32,
        required_minor: u32,
        target: &str,
        backend: &str,
        abi_major: u32,
        runtime_features: &BTreeSet<&str>,
    ) -> Result<(), &'static str> {
        if candidate.target != target {
            return Err("SES-K0203 provider.target-mismatch");
        }
        if candidate.contract.major != required_major || candidate.contract.minor < required_minor {
            return Err("SES-K0204 provider.contract-mismatch");
        }
        if candidate.backend != backend || candidate.abi_major != abi_major {
            return Err("SES-K0205 provider.abi-mismatch");
        }
        if !candidate
            .runtime_features
            .iter()
            .all(|feature| runtime_features.contains(feature))
        {
            return Err("SES-K0206 provider.runtime-feature-mismatch");
        }
        Ok(())
    }

    fn version(major: u32, minor: u32) -> ContractVersion {
        ContractVersion { major, minor }
    }

    fn candidate<'a>(identity: &'a str, service: &'a str, target: &'a str) -> Candidate<'a> {
        Candidate {
            identity,
            service,
            contract: version(1, 0),
            target,
            backend: "typescript",
            abi_major: 1,
            runtime_features: &["foreign.task-load"],
        }
    }

    fn features() -> BTreeSet<&'static str> {
        BTreeSet::from(["foreign.task-load"])
    }

    #[test]
    fn accepts_committed_clock_http_and_filesystem_manifests() {
        let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-manifest-schema-1");
        for case in ["bun-clock", "bun-http-client", "node-filesystem"] {
            check_provider_manifest_case(&artifacts.join(case)).unwrap();
        }
    }

    #[test]
    fn rejects_unknown_manifest_fields() {
        let raw = include_str!(
            "../../../examples/spec/artifacts/provider-manifest-schema-1/bun-clock/provider.json"
        );
        let raw = raw.replace("\"requires\":", "\"dynamic\": true, \"requires\":");
        assert!(check_provider_manifest(&raw)
            .unwrap_err()
            .contains("unknown field `dynamic`"));
    }

    #[test]
    fn selection_is_explicit_then_default_then_unique() {
        let candidates = [
            candidate(
                "seseragi/runtime-bun#http",
                "std/http::HttpClient",
                "bun-process",
            ),
            candidate("acme/undici#http", "std/http::HttpClient", "bun-process"),
            candidate(
                "seseragi/runtime-node#fs",
                "std/fs::FileSystem",
                "node-process",
            ),
        ];
        let http = [("std/http::HttpClient", version(1, 0))];
        let fs = [("std/fs::FileSystem", version(1, 0))];
        let explicit = BTreeMap::from([("std/http::HttpClient", "acme/undici#http")]);
        assert_eq!(
            resolve(
                &http,
                &candidates,
                &explicit,
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Selected("acme/undici#http")
        );
        let defaults = BTreeMap::from([("std/http::HttpClient", "seseragi/runtime-bun#http")]);
        assert_eq!(
            resolve(
                &http,
                &candidates,
                &BTreeMap::new(),
                &defaults,
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Selected("seseragi/runtime-bun#http")
        );
        assert_eq!(
            resolve(
                &fs,
                &candidates,
                &BTreeMap::new(),
                &BTreeMap::new(),
                "node-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Selected("seseragi/runtime-node#fs")
        );
    }

    #[test]
    fn rejects_missing_ambiguous_and_unavailable_explicit_selection() {
        let candidates = [
            candidate(
                "seseragi/runtime-bun#http",
                "std/http::HttpClient",
                "bun-process",
            ),
            candidate("acme/undici#http", "std/http::HttpClient", "bun-process"),
        ];
        let http = [("std/http::HttpClient", version(1, 0))];
        let clock = [("std/clock::Clock", version(1, 0))];
        assert_eq!(
            resolve(
                &clock,
                &candidates,
                &BTreeMap::new(),
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0201 provider.missing")
        );
        assert_eq!(
            resolve(
                &http,
                &candidates,
                &BTreeMap::new(),
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0202 provider.ambiguous")
        );
        let explicit = BTreeMap::from([("std/http::HttpClient", "hidden/provider#http")]);
        assert_eq!(
            resolve(
                &http,
                &candidates,
                &explicit,
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0208 provider.selection-unavailable")
        );
    }

    #[test]
    fn reports_each_compatibility_boundary_without_fallback() {
        let mut candidate = candidate(
            "seseragi/runtime-bun#clock",
            "std/clock::Clock",
            "bun-process",
        );
        let requirements = [("std/clock::Clock", version(1, 1))];
        let explicit = BTreeMap::from([("std/clock::Clock", "seseragi/runtime-bun#clock")]);
        assert_eq!(
            resolve(
                &requirements,
                &[candidate.clone()],
                &explicit,
                &BTreeMap::new(),
                "node-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0203 provider.target-mismatch")
        );
        assert_eq!(
            resolve(
                &requirements,
                &[candidate.clone()],
                &explicit,
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0204 provider.contract-mismatch")
        );
        candidate.contract = version(1, 1);
        assert_eq!(
            resolve(
                &requirements,
                &[candidate.clone()],
                &explicit,
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                2,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0205 provider.abi-mismatch")
        );
        assert_eq!(
            resolve(
                &requirements,
                &[candidate],
                &explicit,
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &BTreeSet::new(),
            ),
            Resolution::Diagnostic("SES-K0206 provider.runtime-feature-mismatch")
        );
    }

    #[test]
    fn rejects_transitive_requirement_major_conflicts() {
        let candidates = [candidate(
            "seseragi/runtime-bun#clock",
            "std/clock::Clock",
            "bun-process",
        )];
        assert_eq!(
            resolve(
                &[
                    ("std/clock::Clock", version(1, 0)),
                    ("std/clock::Clock", version(2, 0)),
                ],
                &candidates,
                &BTreeMap::new(),
                &BTreeMap::new(),
                "bun-process",
                "typescript",
                1,
                &features(),
            ),
            Resolution::Diagnostic("SES-K0207 provider.requirement-conflict")
        );
    }
}
