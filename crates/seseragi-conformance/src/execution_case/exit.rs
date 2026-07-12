use crate::execution::Invocation;

pub(crate) fn expected_observation(
    run: &serde_json::Value,
    invocation: &Invocation,
) -> Result<Option<serde_json::Value>, String> {
    if !matches!(invocation, Invocation::Effect { .. }) {
        return Ok(None);
    }

    let expected = run
        .pointer("/expected/exit")
        .ok_or_else(|| "run.json expected.exit is required for Effect invocation".to_owned())?;
    validate_observation(expected, "run.json expected.exit")?;
    Ok(Some(expected.clone()))
}

pub(crate) fn compare_observation(
    expected: Option<&serde_json::Value>,
    actual: Option<&serde_json::Value>,
) -> Result<(), String> {
    match (expected, actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(actual)) => {
            validate_observation(actual, "effect exit observation")?;
            if actual == expected {
                Ok(())
            } else {
                Err(format!(
                    "execution Effect exit mismatch: expected {expected}, got {actual}"
                ))
            }
        }
        (Some(_), None) => Err("execution Effect exit observation is missing".to_owned()),
        (None, Some(_)) => Err("pure execution unexpectedly produced an Effect exit".to_owned()),
    }
}

fn validate_observation(value: &serde_json::Value, label: &str) -> Result<(), String> {
    match value.get("kind").and_then(|kind| kind.as_str()) {
        Some("success") if value.get("value").is_some() => Ok(()),
        Some("failure") if value.get("error").is_some() => Ok(()),
        Some("success") => Err(format!("{label} success value is missing")),
        Some("failure") => Err(format!("{label} failure error is missing")),
        Some(other) => Err(format!("{label} kind {other} is not supported")),
        None => Err(format!("{label} kind is missing")),
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_observation, expected_observation};
    use crate::execution::{Invocation, InvocationArgument};
    use serde_json::json;

    fn effect_invocation() -> Invocation {
        Invocation::Effect {
            arguments: vec![InvocationArgument::Unit],
        }
    }

    #[test]
    fn requires_exit_only_for_effect_invocations() {
        let error = expected_observation(&json!({ "expected": {} }), &effect_invocation())
            .expect_err("Effect execution must declare its exit");
        assert!(error.contains("required for Effect"));

        let pure = Invocation::PureJson {
            arguments: vec![InvocationArgument::Unit],
        };
        assert_eq!(
            expected_observation(&json!({ "expected": {} }), &pure).unwrap(),
            None
        );
    }

    #[test]
    fn compares_success_and_failure_payloads_without_rewriting_them() {
        let success = json!({ "kind": "success", "value": { "tag": "Rock" } });
        compare_observation(Some(&success), Some(&success)).unwrap();

        let failure = json!({
            "kind": "failure",
            "error": { "tag": "UnknownHand", "value": "lizard" }
        });
        compare_observation(Some(&failure), Some(&failure)).unwrap();

        let mismatch = json!({ "kind": "success", "value": "Unit" });
        assert!(compare_observation(Some(&success), Some(&mismatch))
            .unwrap_err()
            .contains("exit mismatch"));
    }

    #[test]
    fn rejects_incomplete_exit_shapes() {
        let error = expected_observation(
            &json!({ "expected": { "exit": { "kind": "failure" } } }),
            &effect_invocation(),
        )
        .unwrap_err();

        assert!(error.contains("failure error is missing"));
    }
}
