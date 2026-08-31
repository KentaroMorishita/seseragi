use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const METADATA_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GeneratedBindingValidationError {
    pub entry: String,
    pub message: String,
}

pub(crate) fn validate_generated_bindings(
    package_root: &Path,
    generated_root: &Path,
    bindings: &Path,
    host_manifest: &Path,
) -> Result<(), Vec<GeneratedBindingValidationError>> {
    let settings_path = package_root.join(bindings);
    let settings_source = read(&settings_path, "configuration")?;
    let config = toml::from_str::<FreshnessConfig>(&settings_source).map_err(|error| {
        vec![GeneratedBindingValidationError {
            entry: "configuration".to_owned(),
            message: format!(
                "invalid binding settings {}: {error}",
                settings_path.display()
            ),
        }]
    })?;
    if config.schema != 1 {
        return Err(vec![GeneratedBindingValidationError {
            entry: "configuration".to_owned(),
            message: format!(
                "unsupported binding settings schema {}; expected 1",
                config.schema
            ),
        }]);
    }
    let host_manifest_path = package_root.join(host_manifest);
    let host_manifest_source = read(&host_manifest_path, "configuration")?;
    let settings_digest = sha256(&settings_source);
    let mut errors = Vec::new();
    for (id, entry) in config.entries {
        let metadata_path = generated_root.join(format!("{}.binding.json", entry.output));
        let source_path = generated_root.join(format!("{}.ssrg", entry.output));
        let report_path = generated_root.join(format!("{}.report.json", entry.output));
        let metadata = fs::read_to_string(&metadata_path)
            .ok()
            .and_then(|source| serde_json::from_str::<BindingMetadata>(&source).ok());
        let Some(metadata) = metadata else {
            errors.push(GeneratedBindingValidationError {
                entry: id,
                message: format!(
                    "generated binding metadata is missing or invalid at {}; run `seseragi dts convert`",
                    metadata_path.display()
                ),
            });
            continue;
        };
        let input_digest = fs::read_to_string(package_root.join(&entry.declaration))
            .map(|source| sha256(&source))
            .unwrap_or_default();
        let host_exact_identity = resolve_host_exact_identity(
            package_root,
            host_manifest,
            &host_manifest_source,
            &entry.specifier,
        );
        let current = metadata.schema == METADATA_SCHEMA
            && metadata.generator.name == "seseragi-dts"
            && metadata.generator.version == env!("CARGO_PKG_VERSION")
            && metadata.entry == id
            && metadata.output == entry.output
            && metadata.specifier == entry.specifier
            && metadata.host_module.specifier == entry.specifier
            && host_exact_identity.as_ref() == Some(&metadata.host_module.exact_identity)
            && metadata.evaluation == entry.evaluation.as_str()
            && metadata.settings_digest == settings_digest
            && metadata.input_digest == input_digest
            && source_path.is_file()
            && report_path.is_file();
        if !current {
            errors.push(GeneratedBindingValidationError {
                entry: id,
                message: format!(
                    "generated binding `{}` is stale; run `seseragi dts convert`",
                    entry.output
                ),
            });
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn read(path: &Path, entry: &str) -> Result<String, Vec<GeneratedBindingValidationError>> {
    fs::read_to_string(path).map_err(|error| {
        vec![GeneratedBindingValidationError {
            entry: entry.to_owned(),
            message: format!("failed to read {}: {error}", path.display()),
        }]
    })
}

fn resolve_host_exact_identity(
    package_root: &Path,
    host_manifest: &Path,
    host_manifest_source: &str,
    specifier: &str,
) -> Option<String> {
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return Some(format!("workspace:{specifier}"));
    }
    let package_name = if specifier.starts_with('@') {
        specifier.split('/').take(2).collect::<Vec<_>>().join("/")
    } else {
        specifier.split('/').next()?.to_owned()
    };
    let subpath = specifier
        .strip_prefix(&package_name)?
        .trim_start_matches('/');
    let root: serde_json::Value = serde_json::from_str(host_manifest_source).ok()?;
    let manifest =
        if root.get("name").and_then(|value| value.as_str()) == Some(package_name.as_str()) {
            root
        } else {
            let path = package_root
                .join(host_manifest)
                .parent()?
                .join("node_modules")
                .join(&package_name)
                .join("package.json");
            serde_json::from_str(&fs::read_to_string(path).ok()?).ok()?
        };
    let resolved_name = manifest.get("name")?.as_str()?;
    if resolved_name != package_name {
        return None;
    }
    let version = manifest.get("version")?.as_str()?;
    Some(if subpath.is_empty() {
        format!("{resolved_name}@{version}")
    } else {
        format!("{resolved_name}@{version}/{subpath}")
    })
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[derive(Deserialize)]
struct FreshnessConfig {
    schema: u32,
    entries: BTreeMap<String, FreshnessEntry>,
}

#[derive(Deserialize)]
struct FreshnessEntry {
    declaration: String,
    specifier: String,
    output: String,
    #[serde(default)]
    evaluation: Evaluation,
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Evaluation {
    Pure,
    #[default]
    Task,
}

impl Evaluation {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Pure => "pure",
            Self::Task => "task",
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindingMetadata {
    schema: u32,
    generator: GeneratorIdentity,
    entry: String,
    output: String,
    specifier: String,
    host_module: HostModuleIdentity,
    evaluation: String,
    input_digest: String,
    settings_digest: String,
}

#[derive(Deserialize)]
struct GeneratorIdentity {
    name: String,
    version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostModuleIdentity {
    specifier: String,
    exact_identity: String,
}
