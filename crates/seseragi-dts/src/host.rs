//! Host package identity resolution over caller-provided manifest text.
use crate::{ConvertError, HostModuleIdentity};
use std::path::Path;

/// The reader supplies already resolved package metadata. It can read native
/// files or a virtual workspace; this function never loads executable code.
pub fn resolve_host_module_identity(
    host_manifest: &str,
    host_manifest_source: &str,
    specifier: &str,
    mut read_manifest: impl FnMut(&str) -> Result<String, String>,
) -> Result<HostModuleIdentity, ConvertError> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return Ok(HostModuleIdentity {
            specifier: specifier.to_owned(),
            exact_identity: format!("workspace:{specifier}"),
        });
    }
    let package_name = if specifier.starts_with('@') {
        specifier.split('/').take(2).collect::<Vec<_>>().join("/")
    } else {
        specifier.split('/').next().unwrap_or(specifier).to_owned()
    };
    let subpath = specifier
        .strip_prefix(&package_name)
        .unwrap_or_default()
        .trim_start_matches('/');
    let root_manifest: serde_json::Value =
        serde_json::from_str(host_manifest_source).map_err(|error| {
            ConvertError::new(format!(
                "invalid foreign host manifest {host_manifest}: {error}"
            ))
        })?;
    let manifest = if root_manifest.get("name").and_then(|value| value.as_str())
        == Some(package_name.as_str())
    {
        root_manifest
    } else {
        let path = Path::new(host_manifest)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join("node_modules")
            .join(&package_name)
            .join("package.json")
            .to_string_lossy()
            .replace('\\', "/");
        let source = read_manifest(&path).map_err(ConvertError::new)?;
        serde_json::from_str(&source).map_err(|error| {
            ConvertError::new(format!(
                "invalid resolved foreign package manifest {path}: {error}"
            ))
        })?
    };
    let resolved_name = manifest
        .get("name")
        .and_then(|value| value.as_str())
        .ok_or_else(|| ConvertError::new("resolved foreign package manifest has no string name"))?;
    let version = manifest
        .get("version")
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            ConvertError::new("resolved foreign package manifest has no string version")
        })?;
    if resolved_name != package_name {
        return Err(ConvertError::new(format!(
            "foreign specifier `{specifier}` resolved package `{resolved_name}` instead of `{package_name}`"
        )));
    }
    let exact_identity = if subpath.is_empty() {
        format!("{resolved_name}@{version}")
    } else {
        format!("{resolved_name}@{version}/{subpath}")
    };
    Ok(HostModuleIdentity {
        specifier: specifier.to_owned(),
        exact_identity,
    })
}
