//! Product artifact description for the O02 build pipeline.
//!
//! This model is separate from the build-directory ownership marker. Producers
//! populate it from the compiler graph and actual output files; it must never
//! contain temporary staging paths or infer retention from a zero byte count.

use serde::{Deserialize, Serialize};
use seseragi_project::BuildProfile;

pub const ARTIFACT_MANIFEST_FILE: &str = "artifact-manifest.json";
pub const ARTIFACT_MANIFEST_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactManifest {
    pub schema: u32,
    pub profile: BuildProfile,
    pub target: ArtifactTarget,
    pub entry_module: String,
    /// Relative path of the published executable entry.
    pub entry: String,
    pub provenance: ArtifactProvenance,
    /// Compiler-produced module inventory before final bundler elimination.
    pub generated_modules: Vec<GeneratedArtifactModule>,
    /// Exact files published in the artifact, excluding this manifest itself.
    pub files: Vec<ArtifactFile>,
    pub sizes: ArtifactSizes,
    pub source_map: ArtifactSourceMap,
    /// None means retention analysis has not been performed. An empty result
    /// is only valid after analysis proves that no runtime module is retained.
    pub runtime_retention: Option<Vec<RetainedRuntimeModule>>,
    pub reachability: Option<seseragi_driver::ApplicationReachability>,
    pub bundles: Option<Vec<ArtifactBundle>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactTarget {
    Process,
    Web,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactProvenance {
    pub compiler_version: String,
    pub runtime_version: String,
    /// Deterministic digest assigned by the canonical artifact producer.
    pub build_id: String,
    pub bundler_version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedArtifactModule {
    pub module: String,
    pub exports: Vec<String>,
    pub runtime_requirements: Vec<String>,
    pub typescript_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactFile {
    /// Forward-slash path relative to the published artifact directory.
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactSizes {
    pub generated_typescript_bytes: u64,
    /// None means that the corresponding production stage did not run.
    pub bundled_javascript_bytes: Option<u64>,
    pub minified_javascript_bytes: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "lowercase")]
pub enum ArtifactSourceMap {
    Emit { files: Vec<String> },
    Omit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedRuntimeModule {
    pub module: String,
    /// Stable semantic reasons, independent of bundler chunk or pass names.
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactBundle {
    pub path: String,
    pub entry: bool,
    pub modules: Vec<String>,
}

struct BundleEvidence {
    runtime: Vec<RetainedRuntimeModule>,
    bundles: Vec<ArtifactBundle>,
}

/// Describe the completed staging directory before its atomic publication.
pub(crate) fn write_manifest<'a>(
    directory: &std::path::Path,
    target: crate::BuildTarget,
    entry_module: &str,
    modules: impl Iterator<Item = &'a seseragi_driver::CompiledModule>,
    reachability: Option<seseragi_driver::ApplicationReachability>,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    let modules = modules.collect::<Vec<_>>();
    let entry = modules
        .iter()
        .find(|module| module.generated.metadata.module == entry_module)
        .ok_or_else(|| "artifact graph omitted its entry module".to_owned())?;
    let profile = BuildProfile::parse(&entry.generated.metadata.profile)?;
    let mut generated_modules = modules
        .iter()
        .map(|module| {
            let metadata = &module.generated.metadata;
            let mut exports = metadata.exports.clone();
            exports.sort();
            exports.dedup();
            let mut runtime_requirements = metadata.runtime.requirements.clone();
            runtime_requirements.sort();
            runtime_requirements.dedup();
            GeneratedArtifactModule {
                module: metadata.module.clone(),
                exports,
                runtime_requirements,
                typescript_bytes: module.generated.typescript.len() as u64,
            }
        })
        .collect::<Vec<_>>();
    generated_modules.sort_by(|left, right| left.module.cmp(&right.module));
    let evidence = read_bundle_retention(directory, &modules)?;
    let mut files = Vec::new();
    inventory(directory, directory, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let maps = files
        .iter()
        .filter(|file| file.path.ends_with(".map"))
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let executable = if target == crate::BuildTarget::Web {
        "assets/app.js"
    } else if directory.join("entry.js").is_file() {
        "entry.js"
    } else {
        "entry.ts"
    };
    let bundled_javascript_bytes = if executable.ends_with(".js") {
        Some(
            files
                .iter()
                .find(|file| file.path == executable)
                .ok_or("web artifact omitted its JavaScript entry")?
                .bytes,
        )
    } else {
        None
    };
    let runtime_package: serde_json::Value =
        serde_json::from_str(include_str!("../../../runtime/ts/package.json"))
            .map_err(|error| error.to_string())?;
    let mut manifest = ArtifactManifest {
        schema: ARTIFACT_MANIFEST_SCHEMA,
        profile,
        target: match target {
            crate::BuildTarget::Process => ArtifactTarget::Process,
            crate::BuildTarget::Web => ArtifactTarget::Web,
        },
        entry_module: entry_module.to_owned(),
        entry: executable.to_owned(),
        provenance: ArtifactProvenance {
            compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
            runtime_version: runtime_package["version"]
                .as_str()
                .ok_or("runtime package omitted version")?
                .to_owned(),
            build_id: String::new(),
            bundler_version: if bundled_javascript_bytes.is_some() {
                let output = std::process::Command::new("bun")
                    .arg("--version")
                    .output()
                    .map_err(|error| error.to_string())?;
                if !output.status.success() {
                    return Err("failed to identify bundler version".to_owned());
                }
                Some(
                    String::from_utf8(output.stdout)
                        .map_err(|error| error.to_string())?
                        .trim()
                        .to_owned(),
                )
            } else {
                None
            },
        },
        sizes: ArtifactSizes {
            generated_typescript_bytes: generated_modules
                .iter()
                .map(|module| module.typescript_bytes)
                .sum(),
            bundled_javascript_bytes,
            minified_javascript_bytes: None,
        },
        generated_modules,
        files,
        source_map: if maps.is_empty() {
            ArtifactSourceMap::Omit
        } else {
            ArtifactSourceMap::Emit { files: maps }
        },
        runtime_retention: evidence.as_ref().map(|evidence| evidence.runtime.clone()),
        bundles: evidence.map(|evidence| evidence.bundles),
        reachability,
    };
    // Hash canonical compact schema-order JSON with an empty buildId. This is
    // an artifact identity, not a claim to identify every unused source input.
    let identity = serde_json::to_vec(&manifest).map_err(|error| error.to_string())?;
    manifest.provenance.build_id = format!("{:x}", Sha256::digest(identity));
    let mut json = serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?;
    json.push('\n');
    std::fs::write(directory.join(ARTIFACT_MANIFEST_FILE), json)
        .map_err(|error| format!("failed to write artifact manifest: {error}"))
}

fn inventory(
    root: &std::path::Path,
    directory: &std::path::Path,
    files: &mut Vec<ArtifactFile>,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_dir() {
            inventory(root, &path, files)?;
        } else if kind.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|error| error.to_string())?
                .to_str()
                .ok_or("artifact path must be UTF-8")?
                .replace('\\', "/");
            if relative == ARTIFACT_MANIFEST_FILE {
                return Err("public asset collides with artifact-manifest.json".to_owned());
            }
            let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
            files.push(ArtifactFile {
                path: relative,
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&bytes)),
            });
        } else {
            return Err("artifact contains a non-regular file".to_owned());
        }
    }
    Ok(())
}

const BUNDLE_METADATA: &str = ".seseragi-bundle-meta.json";
fn read_bundle_retention(
    directory: &std::path::Path,
    modules: &[&seseragi_driver::CompiledModule],
) -> Result<Option<BundleEvidence>, String> {
    let path = directory.join(BUNDLE_METADATA);
    if !path.exists() {
        return Ok(None);
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    std::fs::remove_file(path).map_err(|error| error.to_string())?;
    let contract: serde_json::Value =
        serde_json::from_str(include_str!("../../../runtime/ts/retention.json"))
            .map_err(|error| error.to_string())?;
    let mut retained = std::collections::BTreeMap::new();
    let outputs = metadata["outputs"]
        .as_object()
        .ok_or("bundler omitted output evidence")?;
    let mut bundles = Vec::new();
    for (output_path, output) in outputs {
        let inputs = output["inputs"]
            .as_object()
            .ok_or("bundler omitted retained input evidence")?;
        let base = metadata["seseragiOutputDirectory"]
            .as_str()
            .unwrap_or(".")
            .trim_start_matches("./");
        let relative = output_path.trim_start_matches("./");
        let output_path = if base.is_empty() || base == "." {
            relative.to_owned()
        } else {
            format!("{base}/{relative}")
        };

        if output_path.starts_with('/') || output_path.split('/').any(|part| part == "..") {
            return Err("bundler output path escaped artifact root".to_owned());
        }
        let mut source_modules = Vec::new();
        for (path, evidence) in inputs {
            if evidence["bytesInOutput"]
                .as_u64()
                .ok_or("bundler omitted retained byte evidence")?
                == 0
            {
                continue;
            }
            let path = path.trim_start_matches("./");
            if let Some(module) = modules.iter().find(|module| {
                module
                    .generated
                    .metadata
                    .outputs
                    .typescript
                    .trim_start_matches("./")
                    == path
            }) {
                source_modules.push(module.generated.metadata.module.clone());
            }

            if let Some(source) = path.strip_prefix("node_modules/@seseragi/runtime/") {
                let class = contract["modules"][source]
                    .as_str()
                    .ok_or_else(|| format!("runtime retention contract omitted {source}"))?;
                retained.insert(
                    format!(
                        "@seseragi/runtime/{}",
                        source.trim_start_matches("src/").trim_end_matches(".ts")
                    ),
                    class.to_owned(),
                );
            } else if let Some(source) = path.strip_prefix("node_modules/seseragi/") {
                retained.insert(
                    format!("seseragi/{}", source.trim_end_matches(".ts")),
                    "provider-resource-bootstrap".to_owned(),
                );
            }
        }
        source_modules.sort();
        source_modules.dedup();
        bundles.push(ArtifactBundle {
            path: output_path,
            entry: output["entryPoint"].is_string(),
            modules: source_modules,
        });
    }
    bundles.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(Some(BundleEvidence {
        runtime: retained
            .into_iter()
            .map(|(module, class)| RetainedRuntimeModule {
                module,
                reasons: vec![class, "referenced-by-bundled-code".to_owned()],
            })
            .collect(),
        bundles,
    }))
}
