use super::digest::{package_digests, relative_source};
use super::{
    parse_lockfile, LockDependency, LockError, LockForeignModule, LockPackage, LockSourceKind,
    Lockfile, LOCKFILE_NAME, LOCK_SCHEMA, STANDARD_LIBRARY_VERSION, TIMEZONE_DATABASE_VERSION,
    UNICODE_VERSION,
};
use crate::{
    discover_local_package_graph, parse_manifest, resolve_foreign_typescript_module,
    ManifestDependency, ManifestLayout, PackageIdentity, IMPLEMENTED_LANGUAGE_VERSION,
};
use semver::Version;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn generate_lockfile(root: impl AsRef<Path>) -> Result<Lockfile, LockError> {
    let root = fs::canonicalize(root.as_ref())
        .map_err(|source| LockError::io("canonicalize package root", root.as_ref(), source))?;
    let graph = discover_local_package_graph(&root)
        .map_err(|error| LockError::PackageGraph(format!("{}: {error}", error.code())))?;
    let mut identities = BTreeMap::<PackageIdentity, String>::new();
    for (identity, package) in graph.packages() {
        let source_kind = if identity == graph.root() {
            LockSourceKind::Workspace
        } else {
            LockSourceKind::Path
        };
        let source = if source_kind == LockSourceKind::Workspace {
            ".".to_owned()
        } else {
            relative_source(&root, package.root())?
        };
        identities.insert(identity.clone(), lock_id(identity, source_kind, &source));
    }
    let mut packages = Vec::new();
    for (identity, package) in graph.packages() {
        let source_kind = if identity == graph.root() {
            LockSourceKind::Workspace
        } else {
            LockSourceKind::Path
        };
        let source = if source_kind == LockSourceKind::Workspace {
            ".".to_owned()
        } else {
            relative_source(&root, package.root())?
        };
        let digests = package_digests(package.root(), &package.manifest().layout)?;
        let dependencies = graph
            .graph()
            .dependencies_for(identity)
            .unwrap_or_default()
            .into_iter()
            .map(|(import, target)| {
                Ok(LockDependency {
                    import,
                    package: identities
                        .get(&target)
                        .ok_or_else(|| {
                            LockError::PackageGraph(
                                "package graph contains a dangling edge".to_owned(),
                            )
                        })?
                        .clone(),
                })
            })
            .collect::<Result<Vec<_>, LockError>>()?;
        packages.push(LockPackage {
            id: identities[identity].clone(),
            name: identity.name().clone(),
            version: identity.version().clone(),
            source_kind,
            source,
            manifest_digest: digests.manifest,
            content_digest: digests.content,
            dependencies,
        });
    }
    packages.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
    let mut foreign_modules = Vec::new();
    for (identity, package) in graph.packages() {
        foreign_modules.extend(lock_foreign_modules(
            package.root(),
            &package.manifest().layout,
            package.manifest().foreign_typescript.as_ref(),
            &identities[identity],
        )?);
    }
    foreign_modules.sort_by(|left, right| {
        (
            left.package.as_bytes(),
            left.declaration.as_bytes(),
            left.specifier.as_bytes(),
        )
            .cmp(&(
                right.package.as_bytes(),
                right.declaration.as_bytes(),
                right.specifier.as_bytes(),
            ))
    });
    Ok(Lockfile {
        schema: LOCK_SCHEMA,
        language: Version::parse(IMPLEMENTED_LANGUAGE_VERSION)
            .expect("implemented language version is valid SemVer"),
        standard_library: Version::parse(STANDARD_LIBRARY_VERSION)
            .expect("standard library version is valid SemVer"),
        unicode: UNICODE_VERSION.to_owned(),
        timezone_database: TIMEZONE_DATABASE_VERSION.to_owned(),
        root: identities[graph.root()].clone(),
        packages,
        providers: Vec::new(),
        foreign_modules,
    })
}

pub fn read_and_validate_lockfile(root: impl AsRef<Path>) -> Result<Lockfile, LockError> {
    read_and_validate(root.as_ref(), true)
}

pub fn read_and_validate_development_lockfile(
    root: impl AsRef<Path>,
) -> Result<Lockfile, LockError> {
    read_and_validate(root.as_ref(), false)
}

fn read_and_validate(root: &Path, check_content: bool) -> Result<Lockfile, LockError> {
    let path = root.join(LOCKFILE_NAME);
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(LockError::Missing)
        }
        Err(source) => return Err(LockError::io("read lockfile", path, source)),
    };
    let actual = parse_lockfile(&source)?;
    validate_toolchain(&actual)?;
    if actual
        .packages
        .iter()
        .any(|package| package.source_kind == LockSourceKind::Registry)
    {
        validate_local_manifest_contract(root, &actual, check_content).map_err(as_stale)?;
        Ok(actual)
    } else {
        let expected = generate_lockfile(root).map_err(as_stale)?;
        stale_reason(&actual, &expected, check_content)
            .map(|reason| Err(LockError::Stale(reason)))
            .unwrap_or_else(|| Ok(actual))
    }
}

fn as_stale(error: LockError) -> LockError {
    match error {
        LockError::Stale(_) | LockError::Missing => error,
        error => LockError::Stale(error.to_string()),
    }
}

fn validate_toolchain(lockfile: &Lockfile) -> Result<(), LockError> {
    let expected_language = Version::parse(IMPLEMENTED_LANGUAGE_VERSION).unwrap();
    let expected_standard = Version::parse(STANDARD_LIBRARY_VERSION).unwrap();
    if lockfile.language != expected_language {
        return Err(LockError::Stale(format!(
            "language version is {}, expected {expected_language}",
            lockfile.language
        )));
    }
    if lockfile.standard_library != expected_standard {
        return Err(LockError::Stale(format!(
            "standard library version is {}, expected {expected_standard}",
            lockfile.standard_library
        )));
    }
    if lockfile.unicode != UNICODE_VERSION {
        return Err(LockError::Stale(format!(
            "Unicode database is {}, expected {UNICODE_VERSION}",
            lockfile.unicode
        )));
    }
    if lockfile.timezone_database != TIMEZONE_DATABASE_VERSION {
        return Err(LockError::Stale(format!(
            "timezone database is {}, expected {TIMEZONE_DATABASE_VERSION}",
            lockfile.timezone_database
        )));
    }
    Ok(())
}

fn stale_reason(actual: &Lockfile, expected: &Lockfile, check_content: bool) -> Option<String> {
    if actual.root != expected.root {
        return Some(format!(
            "root is `{}`, expected `{}`",
            actual.root, expected.root
        ));
    }
    if actual.packages.len() != expected.packages.len() {
        return Some("package set differs from the manifest graph".to_owned());
    }
    for expected_package in &expected.packages {
        let Some(actual_package) = actual
            .packages
            .iter()
            .find(|package| package.id == expected_package.id)
        else {
            return Some(format!("package `{}` is missing", expected_package.id));
        };
        if actual_package.name != expected_package.name
            || actual_package.version != expected_package.version
            || actual_package.source_kind != expected_package.source_kind
            || actual_package.source != expected_package.source
        {
            return Some(format!(
                "package identity `{}` changed",
                expected_package.id
            ));
        }
        if actual_package.manifest_digest != expected_package.manifest_digest {
            return Some(format!(
                "manifest digest for `{}` changed",
                expected_package.id
            ));
        }
        if check_content && actual_package.content_digest != expected_package.content_digest {
            return Some(format!(
                "content digest for `{}` changed",
                expected_package.id
            ));
        }
        if actual_package.dependencies != expected_package.dependencies {
            return Some(format!(
                "dependency edges for `{}` changed",
                expected_package.id
            ));
        }
    }
    if actual.foreign_modules != expected.foreign_modules {
        return Some("foreign TypeScript exact identities or declarations changed".to_owned());
    }
    None
}

fn lock_foreign_modules(
    package_root: &Path,
    layout: &ManifestLayout,
    configuration: Option<&crate::ManifestForeignTypescript>,
    package: &str,
) -> Result<Vec<LockForeignModule>, LockError> {
    let source_root = package_root.join(layout.source.as_str());
    if !source_root.exists() {
        return Ok(Vec::new());
    }
    let mut pending = vec![source_root];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| LockError::io("read foreign source root", &directory, error))?;
        for entry in entries {
            let entry = entry
                .map_err(|error| LockError::io("read foreign source entry", &directory, error))?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| LockError::io("inspect foreign source entry", &path, error))?;
            if metadata.file_type().is_symlink() {
                return Err(LockError::PackageGraph(format!(
                    "foreign source `{}` is a symlink",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file()
                && path.extension().and_then(|value| value.to_str()) == Some("ssrg")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut locked = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)
            .map_err(|error| LockError::io("read foreign source", &path, error))?;
        let surface = seseragi_syntax::parse_surface_ast(path.to_string_lossy(), &source);
        if !surface.foreign_modules.is_empty() && configuration.is_none() {
            return Err(LockError::PackageGraph(format!(
                "package `{package}` uses foreign TypeScript but has no [foreign.typescript] manifest input"
            )));
        }
        let relative = path.strip_prefix(package_root).map_err(|_| {
            LockError::PackageGraph("foreign source is outside package root".to_owned())
        })?;
        let relative = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        for foreign in surface.foreign_modules {
            let resolved = resolve_foreign_typescript_module(
                package_root,
                configuration,
                &path,
                &foreign.specifier,
            )
            .map_err(LockError::PackageGraph)?;
            let declaration = format!("{relative}#{}-{}", foreign.span.start, foreign.span.end);
            let bytes = source
                .as_bytes()
                .get(foreign.span.start..foreign.span.end)
                .ok_or_else(|| {
                    LockError::PackageGraph("foreign declaration span is invalid".to_owned())
                })?;
            locked.push(LockForeignModule {
                package: package.to_owned(),
                declaration,
                specifier: foreign.specifier,
                exact_identity: resolved.exact_identity().to_owned(),
                declaration_digest: format!("sha256:{:x}", Sha256::digest(bytes)),
                content_digest: resolved.content_digest().to_owned(),
            });
        }
    }
    Ok(locked)
}

fn lock_id(identity: &PackageIdentity, kind: LockSourceKind, source: &str) -> String {
    format!(
        "{}@{}#{}:{source}",
        identity.name().as_str(),
        identity.version(),
        kind.as_str()
    )
}

fn validate_local_manifest_contract(
    root: &Path,
    lockfile: &Lockfile,
    check_content: bool,
) -> Result<(), LockError> {
    let root = fs::canonicalize(root)
        .map_err(|source| LockError::io("canonicalize package root", root, source))?;
    let packages = lockfile
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package))
        .collect::<BTreeMap<_, _>>();
    let mut expected_foreign_modules = Vec::new();
    for package in &lockfile.packages {
        if package.source_kind == LockSourceKind::Registry {
            continue;
        }
        let package_root = match package.source_kind {
            LockSourceKind::Workspace => root.clone(),
            LockSourceKind::Path => {
                fs::canonicalize(root.join(&package.source)).map_err(|source| {
                    LockError::io(
                        "canonicalize locked path package",
                        root.join(&package.source),
                        source,
                    )
                })?
            }
            LockSourceKind::Registry => unreachable!(),
        };
        let manifest_path = package_root.join("seseragi.toml");
        let source = fs::read_to_string(&manifest_path)
            .map_err(|error| LockError::io("read manifest", &manifest_path, error))?;
        let manifest = parse_manifest(&source).map_err(|error| {
            LockError::Stale(format!(
                "manifest `{}` is invalid: {error}",
                manifest_path.display()
            ))
        })?;
        expected_foreign_modules.extend(lock_foreign_modules(
            &package_root,
            &manifest.layout,
            manifest.foreign_typescript.as_ref(),
            &package.id,
        )?);
        if manifest.package.name != package.name || manifest.package.version != package.version {
            return Err(LockError::Stale(format!(
                "manifest identity for `{}` changed",
                package.id
            )));
        }
        if !manifest.package.language.matches(&lockfile.language) {
            return Err(LockError::Stale(format!(
                "package `{}` language range `{}` does not include {}",
                package.id,
                manifest.package.language.as_str(),
                lockfile.language
            )));
        }
        let digests = package_digests(&package_root, &manifest.layout)?;
        if digests.manifest != package.manifest_digest {
            return Err(LockError::Stale(format!(
                "manifest digest for `{}` changed",
                package.id
            )));
        }
        if check_content && digests.content != package.content_digest {
            return Err(LockError::Stale(format!(
                "content digest for `{}` changed",
                package.id
            )));
        }
        if manifest.dependencies.len() != package.dependencies.len() {
            return Err(LockError::Stale(format!(
                "dependency edges for `{}` changed",
                package.id
            )));
        }
        for (key, dependency) in &manifest.dependencies {
            let edge = package
                .dependencies
                .iter()
                .find(|edge| edge.import == key.as_str())
                .ok_or_else(|| {
                    LockError::Stale(format!(
                        "dependency `{}` for `{}` is missing",
                        key.as_str(),
                        package.id
                    ))
                })?;
            let target = packages[edge.package.as_str()];
            if target.name != *dependency.package() {
                return Err(LockError::Stale(format!(
                    "dependency `{}` for `{}` names `{}` instead of `{}`",
                    key.as_str(),
                    package.id,
                    target.name.as_str(),
                    dependency.package().as_str()
                )));
            }
            match dependency {
                ManifestDependency::Registry { version, .. } => {
                    if target.source_kind != LockSourceKind::Registry
                        || !version.matches(&target.version)
                    {
                        return Err(LockError::Stale(format!(
                            "registry dependency `{}` for `{}` does not match locked {}",
                            key.as_str(),
                            package.id,
                            target.version
                        )));
                    }
                }
                ManifestDependency::Path { path, .. } => {
                    if target.source_kind != LockSourceKind::Path {
                        return Err(LockError::Stale(format!(
                            "path dependency `{}` for `{}` is not locked as path",
                            key.as_str(),
                            package.id
                        )));
                    }
                    let declared =
                        fs::canonicalize(package_root.join(path.as_str())).map_err(|source| {
                            LockError::io(
                                "canonicalize manifest path dependency",
                                package_root.join(path.as_str()),
                                source,
                            )
                        })?;
                    let locked = fs::canonicalize(root.join(&target.source)).map_err(|source| {
                        LockError::io(
                            "canonicalize locked path package",
                            root.join(&target.source),
                            source,
                        )
                    })?;
                    if declared != locked {
                        return Err(LockError::Stale(format!(
                            "path dependency `{}` for `{}` resolves to another source",
                            key.as_str(),
                            package.id
                        )));
                    }
                }
            }
        }
    }
    expected_foreign_modules.sort_by(|left, right| {
        (
            left.package.as_bytes(),
            left.declaration.as_bytes(),
            left.specifier.as_bytes(),
        )
            .cmp(&(
                right.package.as_bytes(),
                right.declaration.as_bytes(),
                right.specifier.as_bytes(),
            ))
    });
    if lockfile.foreign_modules != expected_foreign_modules {
        return Err(LockError::Stale(
            "foreign TypeScript exact identities or declarations changed".to_owned(),
        ));
    }
    Ok(())
}
