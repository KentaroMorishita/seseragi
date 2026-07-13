use std::collections::BTreeSet;

/// Rejects unknown fields in the project-only nested execution schema before
/// delegating the shared execution values to their semantic parsers.
///
/// The legacy `run.json` lane intentionally keeps its existing compatibility
/// contract. Project descriptors are closed at every object boundary so a
/// misspelled assertion cannot be silently ignored by a shared `Value` parser.
pub(super) fn validate_nested_fields(document: &serde_json::Value) -> Result<(), String> {
    validate_invocation(document.get("invocation"))?;
    validate_required_environment(document.get("requiredEnvironment"))?;
    validate_host_environment(document.get("hostEnvironment"))?;
    validate_expected(document.get("expected"))
}

fn validate_invocation(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(invocation) = object(value) else {
        return Ok(());
    };
    reject_unknown_fields(
        invocation,
        "invocation",
        &["argument", "arguments", "effect", "pure"],
    )?;

    if let Some(arguments) = invocation
        .get("arguments")
        .and_then(serde_json::Value::as_array)
    {
        for (index, argument) in arguments.iter().enumerate() {
            let Some(argument) = argument.as_object() else {
                continue;
            };
            reject_unknown_fields(
                argument,
                &format!("invocation.arguments[{index}]"),
                &["type", "value"],
            )?;
        }
    }

    if let Some(effect) = object(invocation.get("effect")) {
        reject_unknown_fields(effect, "invocation.effect", &["cold", "rootScope"])?;
    }
    if let Some(pure) = object(invocation.get("pure")) {
        reject_unknown_fields(pure, "invocation.pure", &["result"])?;
    }
    Ok(())
}

fn validate_required_environment(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(required) = object(value) else {
        return Ok(());
    };
    reject_unknown_fields(
        required,
        "requiredEnvironment",
        &["kind", "closed", "fields"],
    )?;
    let Some(fields) = required.get("fields").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for (index, field) in fields.iter().enumerate() {
        let Some(field) = field.as_object() else {
            continue;
        };
        reject_unknown_fields(
            field,
            &format!("requiredEnvironment.fields[{index}]"),
            &["name", "type"],
        )?;
    }
    Ok(())
}

fn validate_host_environment(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(host) = object(value) else {
        return Ok(());
    };
    reject_unknown_fields(host, "hostEnvironment", &["closed", "services"])?;
    let Some(services) = host.get("services").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for (index, service) in services.iter().enumerate() {
        let Some(service) = service.as_object() else {
            continue;
        };
        reject_unknown_fields(
            service,
            &format!("hostEnvironment.services[{index}]"),
            &["field", "type", "adapter"],
        )?;
    }
    Ok(())
}

fn validate_expected(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(expected) = object(value) else {
        return Ok(());
    };
    validate_exit(expected.get("exit"))?;
    validate_trace(expected.get("trace"))
}

fn validate_exit(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(exit) = object(value) else {
        return Ok(());
    };
    let allowed = match exit.get("kind").and_then(serde_json::Value::as_str) {
        Some("success") => &["kind", "value"][..],
        Some("failure") => &["kind", "error"][..],
        _ => &["kind", "value", "error"][..],
    };
    reject_unknown_fields(exit, "expected.exit", allowed)
}

fn validate_trace(value: Option<&serde_json::Value>) -> Result<(), String> {
    let Some(events) = value.and_then(serde_json::Value::as_array) else {
        return Ok(());
    };
    for (index, event) in events.iter().enumerate() {
        let Some(event) = event.as_object() else {
            continue;
        };
        reject_unknown_fields(
            event,
            &format!("expected.trace[{index}]"),
            &["service", "operation", "arguments", "stdout"],
        )?;
    }
    Ok(())
}

fn object(
    value: Option<&serde_json::Value>,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    value.and_then(serde_json::Value::as_object)
}

fn reject_unknown_fields(
    object: &serde_json::Map<String, serde_json::Value>,
    label: &str,
    allowed: &[&str],
) -> Result<(), String> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(field.as_str()))
    {
        return Err(format!(
            "execution.json {label} has unknown field `{field}`"
        ));
    }
    Ok(())
}
