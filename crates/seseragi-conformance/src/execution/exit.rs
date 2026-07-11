use std::fs;
use std::path::Path;

pub(super) const OBSERVATION_FILE: &str = ".seseragi-effect-exit.json";

pub(super) fn read_observation(execution_dir: &Path) -> Result<serde_json::Value, String> {
    let path = execution_dir.join(OBSERVATION_FILE);
    let raw = fs::read_to_string(&path).map_err(|error| {
        format!("effect execution did not write its exit observation before terminating: {error}")
    })?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("effect execution wrote an invalid exit observation: {error}"))
}

#[cfg(test)]
mod tests {
    use super::read_observation;
    use std::path::Path;

    #[test]
    fn reports_when_effect_terminates_before_observation() {
        let error = read_observation(Path::new("target/exit-observation-does-not-exist"))
            .expect_err("missing observations must fail");

        assert!(error.contains("before terminating"));
    }
}
