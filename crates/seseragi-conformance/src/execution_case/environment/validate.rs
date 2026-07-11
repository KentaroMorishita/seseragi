use std::collections::{BTreeMap, HashSet};

use super::{EnvironmentBinding, EnvironmentPlan, HostAdapter};

pub(crate) fn parse_environment_plan(
    run: &serde_json::Value,
    required_for_invocation: bool,
) -> Result<EnvironmentPlan, String> {
    let required = run.get("requiredEnvironment");
    let host = run.get("hostEnvironment");
    match (required, host) {
        (None, None) if !required_for_invocation => return Ok(EnvironmentPlan::empty()),
        (None, None) => {
            return Err(
                "run.json requiredEnvironment and hostEnvironment are required for Effect invocation"
                    .to_owned(),
            );
        }
        (None, Some(_)) => return Err("run.json requiredEnvironment is missing".to_owned()),
        (Some(_), None) => return Err("run.json hostEnvironment is missing".to_owned()),
        (Some(_), Some(_)) => {}
    }

    let required_by_name = parse_required_environment(required.expect("checked above"))?;
    parse_host_environment(host.expect("checked above"), &required_by_name)
}

fn parse_required_environment(
    value: &serde_json::Value,
) -> Result<BTreeMap<String, String>, String> {
    let required = value
        .as_object()
        .ok_or_else(|| "run.json requiredEnvironment must be an object".to_owned())?;
    if required.get("kind").and_then(serde_json::Value::as_str) != Some("record") {
        return Err("run.json requiredEnvironment must be a record".to_owned());
    }
    if required.get("closed").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err("run.json requiredEnvironment must be closed".to_owned());
    }
    let fields = required
        .get("fields")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "run.json requiredEnvironment.fields must be an array".to_owned())?;
    let mut by_name = BTreeMap::new();
    for (index, field) in fields.iter().enumerate() {
        let field = field.as_object().ok_or_else(|| {
            format!("run.json requiredEnvironment.fields[{index}] must be an object")
        })?;
        let label = format!("run.json requiredEnvironment.fields[{index}]");
        let name = required_string(field, "name", &label)?;
        let service_type = required_string(field, "type", &label)?;
        if by_name
            .insert(name.to_owned(), service_type.to_owned())
            .is_some()
        {
            return Err(format!(
                "run.json requiredEnvironment has duplicate field {name}"
            ));
        }
    }
    Ok(by_name)
}

fn parse_host_environment(
    value: &serde_json::Value,
    required_by_name: &BTreeMap<String, String>,
) -> Result<EnvironmentPlan, String> {
    let host = value
        .as_object()
        .ok_or_else(|| "run.json hostEnvironment must be an object".to_owned())?;
    let host_closed = host
        .get("closed")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| "run.json hostEnvironment.closed must be a boolean".to_owned())?;
    let services = host
        .get("services")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "run.json hostEnvironment.services must be an array".to_owned())?;
    let mut seen_fields = HashSet::new();
    let mut bindings = Vec::with_capacity(services.len());
    for (index, service) in services.iter().enumerate() {
        let service = service.as_object().ok_or_else(|| {
            format!("run.json hostEnvironment.services[{index}] must be an object")
        })?;
        let label = format!("run.json hostEnvironment.services[{index}]");
        let field = required_string(service, "field", &label)?;
        let service_type = required_string(service, "type", &label)?;
        let adapter_name = required_string(service, "adapter", &label)?;
        if !seen_fields.insert(field.to_owned()) {
            return Err(format!(
                "run.json hostEnvironment has duplicate service field {field}"
            ));
        }
        let adapter = HostAdapter::parse(adapter_name)?;
        validate_service(
            field,
            service_type,
            adapter_name,
            adapter,
            host_closed,
            required_by_name,
        )?;
        bindings.push(EnvironmentBinding::new(field.to_owned(), adapter));
    }

    for (field, service_type) in required_by_name {
        if !seen_fields.contains(field) {
            return Err(format!(
                "run.json hostEnvironment is missing required service {field}: {service_type}"
            ));
        }
    }
    Ok(EnvironmentPlan::new(bindings))
}

fn validate_service(
    field: &str,
    service_type: &str,
    adapter_name: &str,
    adapter: HostAdapter,
    host_closed: bool,
    required_by_name: &BTreeMap<String, String>,
) -> Result<(), String> {
    if adapter.service_type() != service_type {
        return Err(format!(
            "run.json hostEnvironment service {field} uses adapter {adapter_name} for type {service_type}; expected {}",
            adapter.service_type()
        ));
    }
    match required_by_name.get(field) {
        Some(required_type) if required_type != service_type => Err(format!(
            "run.json hostEnvironment service {field} has type {service_type}; required type is {required_type}"
        )),
        None if host_closed => Err(format!(
            "run.json closed hostEnvironment has extra service field {field}"
        )),
        _ => Ok(()),
    }
}

fn required_string<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
    label: &str,
) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label}.{field} is missing"))
}
