//! Check retained semantic adapters against the policy that currently authorizes them.
use seseragi_interop::{ProjectionPolicySet, RuntimeLinkageEvidence};
use seseragi_projection::{generate_adapters, ProjectionArtifactEvidence, SemanticAdapters};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub(crate) fn validate_generated_projection(
    package_root: &Path,
    generated_root: &Path,
    settings: &str,
    output: &str,
    declaration: &str,
    policy_path: &str,
    metadata: &str,
) -> Result<(), String> {
    let metadata: serde_json::Value =
        serde_json::from_str(metadata).map_err(|error| error.to_string())?;
    let evidence: ProjectionArtifactEvidence = serde_json::from_value(
        metadata
            .get("projectionEvidence")
            .cloned()
            .ok_or("checked adapter has no persisted policy input evidence")?,
    )
    .map_err(|error| format!("invalid projection evidence: {error}"))?;
    evidence
        .graph
        .validate()
        .map_err(|error| error.to_string())?;
    let source = fs::read_to_string(package_root.join(policy_path))
        .map_err(|error| format!("failed to read projection policy `{policy_path}`: {error}"))?;
    if sha256(&source) != evidence.policy_source_digest {
        return Err("projection policy source changed after adapter generation".into());
    }
    let identity = evidence
        .graph
        .identity
        .as_ref()
        .ok_or("projection graph has no input identity")?;
    if identity.settings_digest != sha256(settings)
        || !identity.source_digests.contains_key(declaration)
        || metadata["settingsDigest"].as_str() != Some(identity.settings_digest.as_str())
        || metadata["inputDigest"].as_str()
            != identity.source_digests.get(declaration).map(String::as_str)
        || metadata["generator"]
            != serde_json::to_value(&identity.frontend).map_err(|e| e.to_string())?
        || identity.frontend.version != env!("CARGO_PKG_VERSION")
    {
        return Err("projection graph does not match active binding inputs".into());
    }
    for (path, digest) in &identity.source_digests {
        if Path::new(path)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("projection source is not a package-relative file".into());
        }
        let source = fs::read_to_string(package_root.join(path))
            .map_err(|error| format!("failed to read projection declaration `{path}`: {error}"))?;
        if sha256(&source) != *digest
            || evidence
                .graph
                .sources
                .get(path)
                .is_none_or(|record| record.byte_length != source.len())
        {
            return Err(format!("projection declaration `{path}` changed"));
        }
    }
    if let Some(runtime) = metadata.get("runtimeLinkage") {
        let runtime: RuntimeLinkageEvidence = serde_json::from_value(runtime.clone())
            .map_err(|error| format!("invalid runtime linkage evidence: {error}"))?;
        if runtime.graph != evidence.graph {
            return Err("projection graph differs from raw runtime linkage graph".into());
        }
    } else if identity.runtime_contract_digest.is_some() {
        return Err("projection graph has no corresponding runtime linkage evidence".into());
    }
    let policy: ProjectionPolicySet = serde_json::from_str(&source)
        .map_err(|error| format!("invalid projection policy JSON: {error}"))?;
    let expected = generate_adapters(&evidence.graph, &policy)?;
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(generated_root.join(format!("{output}.report.json")))
            .map_err(|error| format!("failed to read semantic adapter report: {error}"))?,
    )
    .map_err(|error| format!("invalid semantic adapter report: {error}"))?;
    if report["compatibility"]["graphDigest"].as_str() != Some(expected.graph_digest.as_str())
        || report["compatibility"]["identity"]
            != serde_json::to_value(identity).map_err(|e| e.to_string())?
    {
        return Err("projection graph differs from conversion report".into());
    }
    let actual_source = fs::read_to_string(generated_root.join(format!("{output}/semantic.ssrg")))
        .map_err(|error| format!("failed to read semantic adapter: {error}"))?;
    let actual: SemanticAdapters = serde_json::from_str(
        &fs::read_to_string(generated_root.join(format!("{output}.semantic.json")))
            .map_err(|error| format!("failed to read semantic adapter metadata: {error}"))?,
    )
    .map_err(|error| format!("invalid semantic adapter metadata: {error}"))?;
    if expected != actual || expected.semantic_source != actual_source {
        return Err("semantic adapter output differs from current policy; generated semantic module has been edited or is stale".into());
    }
    Ok(())
}

fn sha256(source: &str) -> String {
    format!("{:x}", Sha256::digest(source.as_bytes()))
}
