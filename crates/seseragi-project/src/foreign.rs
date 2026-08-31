use crate::ManifestForeignTypescript;
use serde_json::Value;
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
    let package_json: Value = serde_json::from_str(&package_json_source)
        .map_err(|error| format!("invalid `{}`: {error}", package_json_path.display()))?;
    let declared_name = package_json
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&package_name);
    if declared_name != package_name {
        return Err(format!(
            "host package `{package_name}` declares a different name `{declared_name}`"
        ));
    }
    let version = package_json
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("host package `{package_name}` has no version"))?;
    let target = package_entry(&package_json, subpath.as_deref())
        .ok_or_else(|| format!("host package `{package_name}` does not export `{specifier}`"))?;
    let source = resolve_module_file(&dependency_root.join(target.trim_start_matches("./")))?;
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

fn package_entry(package: &Value, subpath: Option<&str>) -> Option<String> {
    let key = subpath.map_or(".".to_owned(), |subpath| format!("./{subpath}"));
    if let Some(exports) = package.get("exports") {
        let selected = if exports.is_string() || exports.get("import").is_some() {
            (key == ".").then_some(exports)
        } else {
            exports.get(&key)
        };
        if let Some(value) = selected.and_then(export_target) {
            return Some(value.to_owned());
        }
        return None;
    }
    if let Some(subpath) = subpath {
        return Some(subpath.to_owned());
    }
    package
        .get("module")
        .or_else(|| package.get("main"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| Some("index.js".to_owned()))
}

fn export_target(value: &Value) -> Option<&str> {
    value.as_str().or_else(|| {
        value
            .get("import")
            .or_else(|| value.get("browser"))
            .or_else(|| value.get("default"))
            .and_then(export_target)
    })
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
