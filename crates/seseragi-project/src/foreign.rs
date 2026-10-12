use crate::ManifestForeignTypescript;
use seseragi_interop::{select_package_export, RuntimeLinkageDiagnostic, RuntimeTarget};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedForeignTypescriptModule {
    exact_identity: String,
    source: PathBuf,
    content_digest: String,
}

impl ResolvedForeignTypescriptModule {
    pub fn exact_identity(&self) -> &str {
        &self.exact_identity
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
}

pub fn resolve_foreign_typescript_module(
    package_root: &Path,
    configuration: Option<&ManifestForeignTypescript>,
    importer: &Path,
    specifier: &str,
) -> Result<ResolvedForeignTypescriptModule, String> {
    resolve_foreign_typescript_module_for_target(
        package_root,
        configuration,
        importer,
        specifier,
        RuntimeTarget::Node,
    )
    .map_err(|error| error.to_string())
}

/// Resolves the same foreign module identity for the explicitly selected host.
/// Conditional exports are selected before opening a file or executing code.
pub fn resolve_foreign_typescript_module_for_target(
    package_root: &Path,
    configuration: Option<&ManifestForeignTypescript>,
    importer: &Path,
    specifier: &str,
    target: RuntimeTarget,
) -> Result<ResolvedForeignTypescriptModule, RuntimeLinkageDiagnostic> {
    resolve_for_target(package_root, configuration, importer, specifier, target)
        .map_err(|reason| RuntimeLinkageDiagnostic::new("SES-F0203", Some(target), reason))
}

fn resolve_for_target(
    package_root: &Path,
    configuration: Option<&ManifestForeignTypescript>,
    importer: &Path,
    specifier: &str,
    target: RuntimeTarget,
) -> Result<ResolvedForeignTypescriptModule, String> {
    let package_root = fs::canonicalize(package_root)
        .map_err(|error| format!("failed to resolve package root: {error}"))?;
    if specifier.starts_with('.') {
        let source = importer
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(specifier);
        let source = resolve_module_file(&source)?;
        if !source.starts_with(&package_root) {
            return Err(format!(
                "foreign TypeScript module `{specifier}` resolves outside package root"
            ));
        }
        let relative = portable_path(
            source
                .strip_prefix(&package_root)
                .expect("resolved source is inside package root"),
        )?;
        return resolved(format!("workspace:{relative}"), source);
    }
    if specifier.starts_with("node:") {
        if target == RuntimeTarget::Browser {
            return Err(format!(
                "foreign runtime `{specifier}` is unavailable for browser target"
            ));
        }
        return Ok(ResolvedForeignTypescriptModule {
            exact_identity: specifier.to_owned(),
            source: PathBuf::new(),
            content_digest: "sha256:builtin".to_owned(),
        });
    }

    let configuration = configuration.ok_or_else(|| {
        format!("bare foreign TypeScript specifier `{specifier}` requires [foreign.typescript]")
    })?;
    let manifest_path = package_root.join(configuration.manifest.as_str());
    let host_root = manifest_path.parent().ok_or_else(|| {
        "foreign TypeScript manifest must have a package-relative parent".to_owned()
    })?;
    validate_declared_host_input(&package_root, &manifest_path, "manifest")?;
    validate_declared_host_input(
        &package_root,
        &package_root.join(configuration.lockfile.as_str()),
        "lockfile",
    )?;

    let (package_name, subpath) = split_bare_specifier(specifier)?;
    let dependency_root = host_root.join("node_modules").join(&package_name);
    let dependency_root = fs::canonicalize(&dependency_root)
        .map_err(|error| format!("failed to resolve host package `{package_name}`: {error}"))?;
    let package_json_path = dependency_root.join("package.json");
    let package_json_source = fs::read_to_string(&package_json_path)
        .map_err(|error| format!("failed to read `{}`: {error}", package_json_path.display()))?;
    let selected = select_package_export(
        &package_json_source,
        &package_name,
        subpath.as_deref(),
        target,
    )
    .map_err(|error| error.reason)?;
    let version = &selected.version;
    let target_path = &selected.path;
    let source = resolve_module_file(&dependency_root.join(target_path.trim_start_matches("./")))?;
    if !source.starts_with(&dependency_root) {
        return Err(format!(
            "host package export `{specifier}` escapes its package"
        ));
    }
    let relative = portable_path(
        source
            .strip_prefix(&dependency_root)
            .expect("resolved export is inside package root"),
    )?;
    resolved(format!("npm:{package_name}@{version}/{relative}"), source)
}

fn resolved(
    exact_identity: String,
    source: PathBuf,
) -> Result<ResolvedForeignTypescriptModule, String> {
    let bytes = fs::read(&source).map_err(|error| {
        format!(
            "failed to read foreign module `{}`: {error}",
            source.display()
        )
    })?;
    Ok(ResolvedForeignTypescriptModule {
        exact_identity,
        source,
        content_digest: format!("sha256:{:x}", Sha256::digest(bytes)),
    })
}

fn validate_declared_host_input(root: &Path, path: &Path, kind: &str) -> Result<(), String> {
    let resolved = fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to resolve foreign host {kind} `{}`: {error}",
            path.display()
        )
    })?;
    if !resolved.starts_with(root) || !resolved.is_file() {
        return Err(format!(
            "foreign host {kind} must be a file inside package root: {}",
            path.display()
        ));
    }
    Ok(())
}

fn split_bare_specifier(specifier: &str) -> Result<(String, Option<String>), String> {
    let parts = specifier.split('/').collect::<Vec<_>>();
    let package_segments = if specifier.starts_with('@') { 2 } else { 1 };
    if parts.len() < package_segments
        || parts[..package_segments]
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return Err(format!(
            "invalid bare foreign TypeScript specifier `{specifier}`"
        ));
    }
    let package = parts[..package_segments].join("/");
    let subpath = (parts.len() > package_segments).then(|| parts[package_segments..].join("/"));
    Ok((package, subpath))
}

fn resolve_module_file(path: &Path) -> Result<PathBuf, String> {
    let mut candidates = vec![path.to_owned()];
    if path.extension().is_none() {
        for extension in ["mjs", "js", "cjs", "mts", "ts", "cts"] {
            candidates.push(path.with_extension(extension));
        }
        for extension in ["mjs", "js", "cjs", "mts", "ts", "cts"] {
            candidates.push(path.join(format!("index.{extension}")));
        }
    }
    for candidate in candidates {
        if let Ok(resolved) = fs::canonicalize(&candidate) {
            if resolved.is_file() {
                return Ok(resolved);
            }
        }
    }
    Err(format!(
        "foreign TypeScript module `{}` does not resolve to a file",
        path.display()
    ))
}

fn portable_path(path: &Path) -> Result<String, String> {
    path.components()
        .map(|component| {
            let std::path::Component::Normal(value) = component else {
                return Err("foreign module identity is not package-relative".to_owned());
            };
            value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| "foreign module identity is not valid UTF-8".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

/// Target preflight over the already loaded package graph. This does not
/// duplicate semantic conversion and does not run the foreign module.
pub fn validate_foreign_runtime_target(
    project: &crate::LoadedLocalProject,
    target: RuntimeTarget,
) -> Result<(), Vec<RuntimeLinkageDiagnostic>> {
    let mut diagnostics = Vec::new();
    for (identity, module) in project.modules() {
        let Some(package) = project.packages().package(identity.package()) else {
            continue;
        };
        let surface = seseragi_syntax::parse_surface_ast(
            module.source_path().to_string_lossy(),
            module.source(),
        );
        for foreign in surface.foreign_modules {
            let resolved = resolve_foreign_typescript_module_for_target(
                package.root(),
                package.manifest().foreign_typescript.as_ref(),
                module.source_path(),
                &foreign.specifier,
                target,
            );
            let result = resolved.and_then(|resolved| {
                let metadata_path = module.source_path().with_extension("binding.json");
                if let Ok(metadata) = fs::read_to_string(metadata_path) {
                    crate::validate_binding_runtime_linkage(
                        &metadata,
                        target,
                        target != RuntimeTarget::Browser,
                        |specifier| {
                            resolve_foreign_typescript_module_for_target(
                                package.root(),
                                package.manifest().foreign_typescript.as_ref(),
                                module.source_path(),
                                specifier,
                                target,
                            )
                            .map(|artifact| {
                                seseragi_interop::RuntimeArtifactIdentity {
                                    exact_identity: artifact.exact_identity().to_owned(),
                                    content_digest: artifact.content_digest().to_owned(),
                                }
                            })
                        },
                    )
                    .map_err(|mut errors| errors.remove(0))?;
                }
                if target == RuntimeTarget::Browser {
                    validate_browser_closure(
                        package.root(),
                        package.manifest().foreign_typescript.as_ref(),
                        resolved.source(),
                    )
                } else {
                    Ok(())
                }
            });
            if let Err(mut diagnostic) = result {
                diagnostic.symbol = Some(format!(
                    "{}#{}-{}",
                    module.source_path().display(),
                    foreign.span.start,
                    foreign.span.end
                ));
                diagnostics.push(diagnostic);
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn validate_browser_closure(
    package_root: &Path,
    configuration: Option<&ManifestForeignTypescript>,
    source: &Path,
) -> Result<(), RuntimeLinkageDiagnostic> {
    let mut pending = vec![source.to_owned()];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(path) = pending.pop() {
        if !visited.insert(path.clone()) {
            continue;
        }
        let source = fs::read_to_string(&path).map_err(|e| {
            RuntimeLinkageDiagnostic::new(
                "SES-F0203",
                Some(RuntimeTarget::Browser),
                format!("failed to read browser runtime {}: {e}", path.display()),
            )
        })?;
        for specifier in crate::runtime_import_specifiers(&source)? {
            let resolved = resolve_foreign_typescript_module_for_target(
                package_root,
                configuration,
                &path,
                &specifier,
                RuntimeTarget::Browser,
            )?;
            pending.push(resolved.source().to_owned());
        }
    }
    Ok(())
}

/// Supplies target availability from real host files. Browser availability also
/// requires its complete static import closure, rather than a resolvable root.
pub fn resolve_foreign_runtime_artifact_for_target(
    package_root: &Path,
    configuration: Option<&ManifestForeignTypescript>,
    importer: &Path,
    specifier: &str,
    target: RuntimeTarget,
) -> Result<seseragi_interop::RuntimeArtifactIdentity, RuntimeLinkageDiagnostic> {
    let resolved = resolve_foreign_typescript_module_for_target(
        package_root,
        configuration,
        importer,
        specifier,
        target,
    )?;
    if target == RuntimeTarget::Browser {
        validate_browser_closure(package_root, configuration, resolved.source())?;
    }
    Ok(seseragi_interop::RuntimeArtifactIdentity {
        exact_identity: resolved.exact_identity().to_owned(),
        content_digest: resolved.content_digest().to_owned(),
    })
}
