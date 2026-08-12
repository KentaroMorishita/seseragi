use seseragi_provider::ProviderManifest;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_manifest_case(case: &Path) -> Result<(), String> {
    let path = case.join("provider.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider manifest: {error}"))?;
    ProviderManifest::from_json(&raw)
        .map(|_| ())
        .map_err(|error| format!("invalid provider manifest: {error}"))
}

#[cfg(test)]
mod tests {
    use super::check_provider_manifest_case;
    use std::path::PathBuf;

    #[test]
    fn conformance_uses_the_production_parser_for_committed_manifests() {
        let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-manifest-schema-1");
        for case in [
            "bun-clock",
            "bun-filesystem",
            "bun-http-client",
            "bun-http-client-native",
            "bun-http-server",
            "node-http-client",
            "node-filesystem",
            "postgres-pg",
        ] {
            check_provider_manifest_case(&artifacts.join(case)).unwrap();
        }
    }
}
