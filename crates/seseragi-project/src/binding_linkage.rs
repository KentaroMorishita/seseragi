//! Runtime proof checks shared by native and virtual host adapters.
use seseragi_interop::{
    RuntimeArtifactIdentity, RuntimeLinkageDiagnostic, RuntimeLinkageEvidence, RuntimeTarget,
};
use std::collections::BTreeMap;

/// Legacy declaration-only metadata remains usable. Once an adapter supplies
/// rich evidence, every selected executable export is validated against freshly
/// read host bytes. Process hosts may defer missing modules to typed lazy loads.
pub fn validate_binding_runtime_linkage(
    metadata: &str,
    target: RuntimeTarget,
    allow_missing_runtime: bool,
    mut resolve: impl FnMut(&str) -> Result<RuntimeArtifactIdentity, RuntimeLinkageDiagnostic>,
) -> Result<(), Vec<RuntimeLinkageDiagnostic>> {
    let stale = |reason: String| {
        vec![RuntimeLinkageDiagnostic::new(
            "SES-F0201",
            Some(target),
            reason,
        )]
    };
    let metadata: serde_json::Value =
        serde_json::from_str(metadata).map_err(|error| stale(error.to_string()))?;
    let Some(value) = metadata.get("runtimeLinkage") else {
        return Ok(());
    };
    let evidence: RuntimeLinkageEvidence =
        serde_json::from_value(value.clone()).map_err(|error| stale(error.to_string()))?;
    let mut artifacts = BTreeMap::new();
    for (key, export) in &evidence.contract.exports {
        if let Ok(artifact) = resolve(&export.specifier) {
            artifacts.insert(key.clone(), artifact);
        }
    }
    match evidence
        .contract
        .validate_for(&evidence.graph, target, &artifacts)
    {
        Ok(()) => Ok(()),
        Err(errors) => {
            let errors = errors
                .into_iter()
                .filter(|error| !(allow_missing_runtime && error.code == "SES-F0203"))
                .collect::<Vec<_>>();
            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }
}

/// Virtual adapters verify the real declaration/settings snapshot as well as
/// the contract's embedded graph. A self-consistent old graph is still stale.
pub fn validate_binding_runtime_linkage_inputs(
    metadata: &str,
    files: &BTreeMap<String, String>,
    settings: Option<&str>,
    target: RuntimeTarget,
) -> Result<(), Vec<RuntimeLinkageDiagnostic>> {
    use sha2::{Digest, Sha256};
    let stale = |reason: String| {
        vec![RuntimeLinkageDiagnostic::new(
            "SES-F0201",
            Some(target),
            reason,
        )]
    };
    let metadata: serde_json::Value =
        serde_json::from_str(metadata).map_err(|error| stale(error.to_string()))?;
    let Some(value) = metadata.get("runtimeLinkage") else {
        return Ok(());
    };
    let evidence: RuntimeLinkageEvidence =
        serde_json::from_value(value.clone()).map_err(|error| stale(error.to_string()))?;
    for (path, expected) in &evidence.contract.declaration.source_digests {
        let actual = files
            .get(path)
            .map(|source| format!("{:x}", Sha256::digest(source.as_bytes())));
        if actual.as_ref() != Some(expected) {
            return Err(stale(format!(
                "runtime linkage declaration `{path}` changed or is missing; regenerate bindings"
            )));
        }
    }
    if settings
        .map(|source| format!("{:x}", Sha256::digest(source.as_bytes())))
        .as_ref()
        != Some(&evidence.contract.declaration.settings_digest)
    {
        return Err(stale(
            "runtime linkage binding settings changed or are missing; regenerate bindings".into(),
        ));
    }
    Ok(())
}

/// A generated source is part of the verified artifact, not editable input.
/// Older declaration-only metadata has no digest and keeps its prior behavior.
pub fn validate_generated_binding_source(
    metadata: &str,
    source: Option<&str>,
    target: RuntimeTarget,
) -> Result<(), Vec<RuntimeLinkageDiagnostic>> {
    use sha2::{Digest, Sha256};
    let stale = |reason: &str| {
        vec![RuntimeLinkageDiagnostic::new(
            "SES-F0201",
            Some(target),
            reason,
        )]
    };
    let metadata: serde_json::Value = serde_json::from_str(metadata)
        .map_err(|_| stale("generated binding metadata is invalid"))?;
    let expected = metadata
        .get("sourceDigest")
        .and_then(serde_json::Value::as_str);
    if metadata.get("sourceDigest").is_none() && metadata.get("runtimeLinkage").is_none() {
        return Ok(());
    }
    let actual = source.map(|source| format!("{:x}", Sha256::digest(source.as_bytes())));
    if expected.is_none() || actual.as_deref() != expected {
        return Err(stale(
            "generated binding source changed or is missing; regenerate bindings",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_source_digest_preserves_legacy_and_rejects_missing_rich_proof() {
        use sha2::{Digest, Sha256};
        let source = "pub fn api x = x\n";
        let metadata =
            serde_json::json!({"sourceDigest":format!("{:x}",Sha256::digest(source.as_bytes()))})
                .to_string();
        assert!(
            validate_generated_binding_source(&metadata, Some(source), RuntimeTarget::Browser)
                .is_ok()
        );
        assert!(validate_generated_binding_source(
            &metadata,
            Some("edited"),
            RuntimeTarget::Browser
        )
        .is_err());
        assert!(
            validate_generated_binding_source(&metadata, None, RuntimeTarget::Browser).is_err()
        );
        assert!(validate_generated_binding_source("{}", None, RuntimeTarget::Browser).is_ok());
        assert!(validate_generated_binding_source(
            r#"{"runtimeLinkage":{}}"#,
            Some(source),
            RuntimeTarget::Browser
        )
        .is_err());
    }
}
