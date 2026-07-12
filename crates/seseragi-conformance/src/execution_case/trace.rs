pub(crate) fn expected_trace(run: &serde_json::Value) -> Result<Option<serde_json::Value>, String> {
    let Some(trace) = run.pointer("/expected/trace") else {
        return Ok(None);
    };
    validate_trace(trace, "run.json expected.trace")?;
    Ok(Some(trace.clone()))
}

pub(crate) fn trace_stdout(trace: Option<&serde_json::Value>) -> Result<Option<String>, String> {
    let Some(trace) = trace else {
        return Ok(None);
    };
    let events = validate_trace(trace, "run.json expected.trace")?;
    Ok(Some(
        events
            .iter()
            .filter_map(|event| event.get("stdout").and_then(serde_json::Value::as_str))
            .collect(),
    ))
}

pub(crate) fn compare_trace(
    expected: Option<&serde_json::Value>,
    actual: Option<&serde_json::Value>,
) -> Result<(), String> {
    if let Some(actual) = actual {
        validate_trace(actual, "capture-console operation trace")?;
    }
    let Some(expected) = expected else {
        return Ok(());
    };
    let Some(actual) = actual else {
        return Err("execution capture-console operation trace is missing".to_owned());
    };
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "execution operation trace mismatch: expected {expected}, got {actual}"
        ))
    }
}

fn validate_trace<'a>(
    value: &'a serde_json::Value,
    label: &str,
) -> Result<&'a Vec<serde_json::Value>, String> {
    let events = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?;
    for (index, event) in events.iter().enumerate() {
        let event_label = format!("{label}[{index}]");
        let object = event
            .as_object()
            .ok_or_else(|| format!("{event_label} must be an object"))?;
        if object.len() != 4 {
            return Err(format!(
                "{event_label} must contain exactly service, operation, arguments, and stdout"
            ));
        }
        required_string(object, "service", &event_label)?;
        required_string(object, "operation", &event_label)?;
        object
            .get("arguments")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("{event_label}.arguments must be an array"))?;
        object
            .get("stdout")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{event_label}.stdout is missing"))?;
    }
    Ok(events)
}

fn required_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
    label: &str,
) -> Result<(), String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(|_| ())
        .ok_or_else(|| format!("{label}.{field} is missing"))
}

#[cfg(test)]
mod tests {
    use super::{compare_trace, expected_trace, trace_stdout};
    use serde_json::json;

    fn trace() -> serde_json::Value {
        json!([{
            "service": "console",
            "operation": "println",
            "arguments": ["hello"],
            "stdout": "hello\n"
        }])
    }

    #[test]
    fn validates_and_reconstructs_expected_stdout() {
        let value = json!({ "expected": { "trace": trace() } });
        let expected = expected_trace(&value).unwrap();

        assert_eq!(
            trace_stdout(expected.as_ref()).unwrap().as_deref(),
            Some("hello\n")
        );
    }

    #[test]
    fn compares_the_actual_operation_sequence() {
        let expected = trace();
        compare_trace(Some(&expected), Some(&expected)).unwrap();

        let actual = json!([{
            "service": "console",
            "operation": "print",
            "arguments": ["hello"],
            "stdout": "hello"
        }]);
        assert!(compare_trace(Some(&expected), Some(&actual))
            .unwrap_err()
            .contains("operation trace mismatch"));
        assert!(compare_trace(Some(&expected), None)
            .unwrap_err()
            .contains("trace is missing"));
    }

    #[test]
    fn permits_an_unasserted_host_trace() {
        assert_eq!(expected_trace(&json!({ "expected": {} })).unwrap(), None);
        compare_trace(None, Some(&trace())).unwrap();

        let malformed = json!([{ "service": "console" }]);
        assert!(compare_trace(None, Some(&malformed))
            .unwrap_err()
            .contains("exactly service"));
    }

    #[test]
    fn rejects_malformed_trace_events() {
        let error = expected_trace(&json!({
            "expected": {
                "trace": [{
                    "service": "console",
                    "operation": "println",
                    "arguments": []
                }]
            }
        }))
        .unwrap_err();

        assert!(error.contains("exactly service"));
    }
}
