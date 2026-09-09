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

/// Describe the completed staging directory before its atomic publication.
pub(crate) fn write_manifest<'a>(
    directory: &std::path::Path,
    target: crate::BuildTarget,
    entry_module: &str,
    modules: impl Iterator<Item = &'a seseragi_driver::CompiledModule>,
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
    let mut files = Vec::new();
    inventory(directory, directory, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let maps = files
        .iter()
        .filter(|file| file.path.ends_with(".map"))
        .map(|file| file.path.clone())
        .collect::<Vec<_>>();
    let bundled_javascript_bytes = if target == crate::BuildTarget::Web {
        Some(
            files
                .iter()
                .find(|file| file.path == "assets/app.js")
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
        entry: if target == crate::BuildTarget::Web {
            "assets/app.js"
        } else {
            "entry.ts"
        }
        .to_owned(),
        provenance: ArtifactProvenance {
            compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
            runtime_version: runtime_package["version"]
                .as_str()
                .ok_or("runtime package omitted version")?
                .to_owned(),
            build_id: String::new(),
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
        runtime_retention: None,
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
