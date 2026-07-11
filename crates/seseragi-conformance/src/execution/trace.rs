use std::fs;
use std::path::Path;

pub(super) const OBSERVATION_FILE: &str = ".seseragi-operation-trace.json";

pub(super) fn read_observation(execution_dir: &Path) -> Result<serde_json::Value, String> {
    let path = execution_dir.join(OBSERVATION_FILE);
    let raw = fs::read_to_string(&path).map_err(|error| {
        format!("capture-console did not write its operation trace before terminating: {error}")
    })?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("capture-console wrote an invalid operation trace: {error}"))
}

#[cfg(test)]
mod tests {
    use super::read_observation;
    use std::path::Path;

    #[test]
    fn reports_a_missing_trace_sidecar() {
        let error = read_observation(Path::new("target/trace-observation-does-not-exist"))
            .expect_err("missing operation traces must fail");

        assert!(error.contains("capture-console"));
    }
}
