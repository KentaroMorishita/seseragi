use seseragi_provider::ProviderContract;
use std::fs;
use std::path::Path;

pub(crate) fn check_provider_contract_case(case: &Path) -> Result<(), String> {
    let path = case.join("contract.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider contract: {error}"))?;
    ProviderContract::from_json(&raw)
        .map(|_| ())
        .map_err(|error| format!("invalid provider contract: {error}"))
}

#[cfg(test)]
mod tests {
    use super::check_provider_contract_case;
    use std::path::PathBuf;

    #[test]
    fn conformance_uses_the_production_parser_for_committed_contracts() {
        let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-contract-schema-1");
        for case in ["clock", "filesystem", "http-server", "bun-http-extension"] {
            check_provider_contract_case(&artifacts.join(case)).unwrap();
        }
    }
}
