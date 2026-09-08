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
