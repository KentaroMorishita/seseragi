use super::{
    LockDependency, LockError, LockHostPackage, LockPackage, LockProviderSelection, LockSourceKind,
    Lockfile, LOCK_SCHEMA,
};
use crate::PackageName;
use semver::Version;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
struct RawLockfile {
    schema: u64,
    language: String,
    standard_library: String,
    unicode: String,
    timezone_database: String,
    root: String,
    packages: Vec<RawPackage>,
    #[serde(default)]
    providers: Vec<RawProvider>,
}

#[derive(Deserialize)]
struct RawPackage {
    id: String,
    name: String,
    version: String,
    source_kind: String,
    source: String,
    manifest_digest: String,
    content_digest: String,
    #[serde(default)]
    dependencies: Vec<RawDependency>,
}

#[derive(Deserialize)]
struct RawDependency {
    import: String,
    package: String,
}

#[derive(Deserialize)]
struct RawProvider {
    field: String,
    service: String,
    required_contract: String,
    provider_contract: String,
    provider: String,
    package_version: String,
    package_source: String,
    package_digest: String,
    artifact_digest: String,
    backend: String,
    backend_abi_major: u64,
    target: String,
    entry_module: String,
    entry_export: String,
    #[serde(default)]
    runtime_features: Vec<String>,
    #[serde(default)]
    host_packages: Vec<RawHostPackage>,
}

#[derive(Deserialize)]
struct RawHostPackage {
    name: String,
    version: String,
    source: String,
    content_digest: String,
}

pub fn parse_lockfile(source: &str) -> Result<Lockfile, LockError> {
    let raw = toml::from_str::<RawLockfile>(source)
        .map_err(|error| LockError::InvalidToml(error.to_string()))?;
    if raw.schema != LOCK_SCHEMA {
        return Err(LockError::UnsupportedSchema(raw.schema));
    }
    let language = parse_version("language", &raw.language)?;
    let standard_library = parse_version("standard_library", &raw.standard_library)?;
    validate_release_id("unicode", &raw.unicode)?;
    validate_release_id("timezone_database", &raw.timezone_database)?;

    let mut packages = raw
        .packages
        .into_iter()
        .map(parse_package)
        .collect::<Result<Vec<_>, _>>()?;
    packages.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    validate_graph(&raw.root, &packages)?;
    let mut providers = raw
        .providers
        .into_iter()
        .map(parse_provider)
        .collect::<Result<Vec<_>, _>>()?;
    providers.sort_by(|left, right| {
        (
            left.service.as_bytes(),
            left.field.as_bytes(),
            left.target.as_bytes(),
            left.provider.as_bytes(),
        )
            .cmp(&(
                right.service.as_bytes(),
                right.field.as_bytes(),
                right.target.as_bytes(),
                right.provider.as_bytes(),
            ))
    });
    let mut provider_keys = BTreeSet::new();
    for provider in &providers {
        if !provider_keys.insert((
            provider.target.clone(),
            provider.field.clone(),
            provider.service.clone(),
        )) {
            return Err(invalid(
                "providers",
                format!(
                    "duplicate selection for field `{}` / service `{}` on `{}`",
                    provider.field, provider.service, provider.target
                ),
            ));
        }
    }
    Ok(Lockfile {
        schema: raw.schema,
        language,
        standard_library,
        unicode: raw.unicode,
        timezone_database: raw.timezone_database,
        root: raw.root,
        packages,
        providers,
    })
}

fn parse_provider(raw: RawProvider) -> Result<LockProviderSelection, LockError> {
    for (field, value) in [
        ("providers.field", raw.field.as_str()),
        ("providers.service", raw.service.as_str()),
        ("providers.provider", raw.provider.as_str()),
        ("providers.package_source", raw.package_source.as_str()),
        ("providers.backend", raw.backend.as_str()),
        ("providers.target", raw.target.as_str()),
        ("providers.entry_module", raw.entry_module.as_str()),
        ("providers.entry_export", raw.entry_export.as_str()),
    ] {
        if value.trim().is_empty() || absolute_path(value) {
            return Err(invalid(field, "must be non-empty and machine-independent"));
        }
    }
    parse_version("providers.package_version", &raw.package_version)?;
    validate_contract("providers.required_contract", &raw.required_contract)?;
    validate_contract("providers.provider_contract", &raw.provider_contract)?;
    validate_digest("providers.package_digest", &raw.package_digest)?;
    validate_digest("providers.artifact_digest", &raw.artifact_digest)?;
    let mut runtime_features = raw.runtime_features;
    runtime_features.sort();
    runtime_features.dedup();
    let mut host_packages = raw
        .host_packages
        .into_iter()
        .map(|package| {
            PackageName::parse(&package.name)
                .map_err(|error| invalid("providers.host_packages.name", error))?;
            parse_version("providers.host_packages.version", &package.version)?;
            if package.source.trim().is_empty() || absolute_path(&package.source) {
                return Err(invalid(
                    "providers.host_packages.source",
                    "must be non-empty and machine-independent",
                ));
            }
            validate_digest(
                "providers.host_packages.content_digest",
                &package.content_digest,
            )?;
            Ok(LockHostPackage {
                name: package.name,
                version: package.version,
                source: package.source,
                content_digest: package.content_digest,
            })
        })
        .collect::<Result<Vec<_>, LockError>>()?;
    host_packages.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    if host_packages
        .windows(2)
        .any(|packages| packages[0].name == packages[1].name)
    {
        return Err(invalid(
            "providers.host_packages",
            "duplicate host package name",
        ));
    }
    Ok(LockProviderSelection {
        field: raw.field,
        service: raw.service,
        required_contract: raw.required_contract,
        provider_contract: raw.provider_contract,
        provider: raw.provider,
        package_version: raw.package_version,
        package_source: raw.package_source,
        package_digest: raw.package_digest,
        artifact_digest: raw.artifact_digest,
        backend: raw.backend,
        backend_abi_major: raw.backend_abi_major,
        target: raw.target,
        entry_module: raw.entry_module,
        entry_export: raw.entry_export,
        runtime_features,
        host_packages,
    })
}

fn parse_package(raw: RawPackage) -> Result<LockPackage, LockError> {
    let name = PackageName::parse(&raw.name).map_err(|error| invalid("packages.name", error))?;
    let version = parse_version("packages.version", &raw.version)?;
    let source_kind = match raw.source_kind.as_str() {
        "workspace" => LockSourceKind::Workspace,
        "path" => LockSourceKind::Path,
        "registry" => LockSourceKind::Registry,
        value => {
            return Err(invalid(
                "packages.source_kind",
                format!("unknown kind `{value}`"),
            ))
        }
    };
    validate_source(source_kind, &raw.source)?;
    validate_digest("packages.manifest_digest", &raw.manifest_digest)?;
    validate_digest("packages.content_digest", &raw.content_digest)?;
    let mut dependencies = raw
        .dependencies
        .into_iter()
        .map(|dependency| {
            PackageName::parse(&dependency.import)
                .map_err(|error| invalid("packages.dependencies.import", error))?;
            Ok(LockDependency {
                import: dependency.import,
                package: dependency.package,
            })
        })
        .collect::<Result<Vec<_>, LockError>>()?;
    dependencies.sort_by(|left, right| left.import.as_bytes().cmp(right.import.as_bytes()));
    let package = LockPackage {
        id: raw.id,
        name,
        version,
        source_kind,
        source: raw.source,
        manifest_digest: raw.manifest_digest,
        content_digest: raw.content_digest,
        dependencies,
    };
    if package.id != package.canonical_id() {
        return Err(invalid(
            "packages.id",
            format!("expected canonical id `{}`", package.canonical_id()),
        ));
    }
    Ok(package)
}

fn validate_graph(root: &str, packages: &[LockPackage]) -> Result<(), LockError> {
    let mut ids = BTreeSet::new();
    let mut identities = BTreeMap::<(String, Version), String>::new();
    for package in packages {
        if !ids.insert(package.id.clone()) {
            return Err(LockError::DuplicatePackage(package.id.clone()));
        }
        let identity = (package.name.as_str().to_owned(), package.version.clone());
        if let Some(first) = identities.insert(identity.clone(), package.id.clone()) {
            if first != package.id {
                return Err(LockError::DuplicateIdentity(format!(
                    "{}@{}",
                    identity.0, identity.1
                )));
            }
        }
        let mut imports = BTreeSet::new();
        for dependency in &package.dependencies {
            if !imports.insert(dependency.import.clone()) {
                return Err(LockError::DuplicateDependency {
                    package: package.id.clone(),
                    import: dependency.import.clone(),
                });
            }
        }
    }
    let root_package = packages
        .iter()
        .find(|package| package.id == root)
        .ok_or_else(|| LockError::DanglingRoot(root.to_owned()))?;
    if root_package.source_kind != LockSourceKind::Workspace || root_package.source != "." {
        return Err(invalid(
            "root",
            "root must reference the workspace:. package",
        ));
    }
    if packages
        .iter()
        .filter(|package| package.source_kind == LockSourceKind::Workspace)
        .count()
        != 1
    {
        return Err(invalid(
            "packages.source_kind",
            "the lock graph must contain exactly one workspace package",
        ));
    }
    for package in packages {
        for dependency in &package.dependencies {
            if !ids.contains(&dependency.package) {
                return Err(LockError::DanglingDependency {
                    package: package.id.clone(),
                    dependency: dependency.package.clone(),
                });
            }
        }
    }
    let packages_by_id = packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit_package(root, &packages_by_id, &mut visiting, &mut visited)?;
    if visited.len() != packages.len() {
        let package = packages
            .iter()
            .find(|package| !visited.contains(package.id.as_str()))
            .expect("package count differs because at least one package is unreachable");
        return Err(invalid(
            "packages",
            format!("package `{}` is not reachable from root", package.id),
        ));
    }
    Ok(())
}

fn visit_package<'a>(
    id: &'a str,
    packages: &BTreeMap<&'a str, &'a LockPackage>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), LockError> {
    if visited.contains(id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(invalid(
            "packages.dependencies",
            format!("dependency graph contains a cycle through `{id}`"),
        ));
    }
    for dependency in &packages[id].dependencies {
        visit_package(&dependency.package, packages, visiting, visited)?;
    }
    visiting.remove(id);
    visited.insert(id);
    Ok(())
}

fn validate_source(kind: LockSourceKind, value: &str) -> Result<(), LockError> {
    let valid = match kind {
        LockSourceKind::Workspace => value == ".",
        LockSourceKind::Path => {
            !value.is_empty()
                && !value.starts_with('/')
                && !value.contains('\\')
                && !value
                    .split('/')
                    .any(|segment| segment.is_empty() || segment == ".")
                && !value
                    .split('/')
                    .next()
                    .is_some_and(|segment| segment.contains(':'))
        }
        LockSourceKind::Registry => PackageName::parse(value).is_ok(),
    };
    valid.then_some(()).ok_or_else(|| {
        invalid(
            "packages.source",
            format!("invalid {} source `{value}`", kind.as_str()),
        )
    })
}

fn parse_version(field: &str, value: &str) -> Result<Version, LockError> {
    Version::parse(value).map_err(|_| invalid(field, format!("invalid SemVer `{value}`")))
}

fn validate_release_id(field: &str, value: &str) -> Result<(), LockError> {
    (!value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
    .then_some(())
    .ok_or_else(|| invalid(field, format!("invalid release id `{value}`")))
}

fn validate_digest(field: &str, value: &str) -> Result<(), LockError> {
    let hex = value.strip_prefix("sha256:").unwrap_or_default();
    (hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')))
    .then_some(())
    .ok_or_else(|| {
        invalid(
            field,
            "expected sha256: followed by 64 lowercase hex digits",
        )
    })
}

fn validate_contract(field: &str, value: &str) -> Result<(), LockError> {
    let valid = value
        .split_once('.')
        .is_some_and(|(major, minor)| major.parse::<u64>().is_ok() && minor.parse::<u64>().is_ok());
    valid
        .then_some(())
        .ok_or_else(|| invalid(field, format!("invalid Contract version `{value}`")))
}

fn absolute_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with("path:")
        || value.as_bytes().get(1) == Some(&b':')
        || value.contains('\\')
}

fn invalid(field: impl Into<String>, reason: impl ToString) -> LockError {
    LockError::InvalidField {
        field: field.into(),
        reason: reason.to_string(),
    }
}
