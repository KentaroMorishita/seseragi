use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const BACKEND_NAMESPACES: &[&str] = &[
    "browser",
    "bun",
    "deno",
    "javascript",
    "native",
    "node",
    "typescript",
    "wasi",
];

pub(crate) fn check_provider_contract_case(case: &Path) -> Result<(), String> {
    let path = case.join("contract.json");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read provider contract: {error}"))?;
    let document = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse provider contract: {error}"))?;
    check_provider_contract(&document)
}

fn check_provider_contract(document: &Value) -> Result<(), String> {
    let contract = object(document, "provider contract")?;
    exact_fields(
        contract,
        &[
            "schema",
            "kind",
            "identity",
            "version",
            "requirement",
            "operations",
        ],
        "provider contract",
    )?;
    if contract.get("schema").and_then(Value::as_u64) != Some(1) {
        return Err("provider contract must use schema 1".to_owned());
    }
    if string(contract, "kind", "provider contract")? != "provider-contract" {
        return Err("provider contract kind must be provider-contract".to_owned());
    }

    let identity = string(contract, "identity", "provider contract")?;
    check_type_identity(identity, "provider contract identity")?;
    check_version(contract.get("version"))?;
    check_requirement(contract.get("requirement"), identity)?;

    let operations = contract
        .get("operations")
        .and_then(Value::as_array)
        .ok_or_else(|| "provider contract operations must be an array".to_owned())?;
    if operations.is_empty() {
        return Err("provider contract operations must not be empty".to_owned());
    }
    let mut identities = BTreeSet::new();
    for (index, operation) in operations.iter().enumerate() {
        check_operation(operation, index, identity, &mut identities)?;
    }
    Ok(())
}

fn check_version(value: Option<&Value>) -> Result<(), String> {
    let version = object_at(value, "provider contract version")?;
    exact_fields(version, &["major", "minor"], "provider contract version")?;
    let major = unsigned(version, "major", "provider contract version")?;
    unsigned(version, "minor", "provider contract version")?;
    if major == 0 {
        return Err("provider contract version major must be greater than zero".to_owned());
    }
    Ok(())
}

fn check_requirement(value: Option<&Value>, identity: &str) -> Result<(), String> {
    let requirement = object_at(value, "provider contract requirement")?;
    exact_fields(
        requirement,
        &["field", "type"],
        "provider contract requirement",
    )?;
    let field = string(requirement, "field", "provider contract requirement")?;
    check_lower_camel(field, "provider contract requirement field")?;
    let required_type = string(requirement, "type", "provider contract requirement")?;
    check_type_identity(required_type, "provider contract requirement type")?;
    if required_type != identity {
        return Err(format!(
            "provider contract requirement type {required_type} must match contract identity {identity}"
        ));
    }
    Ok(())
}

fn check_operation(
    value: &Value,
    index: usize,
    service_identity: &str,
    identities: &mut BTreeSet<String>,
) -> Result<(), String> {
    let label = format!("provider contract operation {index}");
    let operation = object(value, &label)?;
    exact_fields(
        operation,
        &[
            "identity",
            "kind",
            "input",
            "success",
            "failure",
            "portability",
            "summary",
        ],
        &label,
    )?;
    let identity = string(operation, "identity", &label)?;
    let prefix = format!("{service_identity}#");
    let Some(name) = identity.strip_prefix(&prefix) else {
        return Err(format!(
            "{label} identity {identity} must start with {prefix}"
        ));
    };
    check_lower_camel(name, &format!("{label} name"))?;
    if !identities.insert(identity.to_owned()) {
        return Err(format!(
            "provider contract operation identity is duplicated: {identity}"
        ));
    }
    match string(operation, "kind", &label)? {
        "one-shot" | "resource" | "subscription" => {}
        kind => return Err(format!("{label} kind is not supported: {kind}")),
    }
    check_logical_type(
        operation
            .get("input")
            .ok_or_else(|| format!("{label} input is missing"))?,
        &format!("{label} input"),
    )?;
    check_logical_type(
        operation
            .get("success")
            .ok_or_else(|| format!("{label} success is missing"))?,
        &format!("{label} success"),
    )?;
    check_logical_type(
        operation
            .get("failure")
            .ok_or_else(|| format!("{label} failure is missing"))?,
        &format!("{label} failure"),
    )?;
    check_portability(operation.get("portability"), &label)?;
    if string(operation, "summary", &label)?.trim().is_empty() {
        return Err(format!("{label} summary must not be empty"));
    }
    Ok(())
}

fn check_logical_type(value: &Value, label: &str) -> Result<(), String> {
    let logical_type = object(value, label)?;
    let kind = string(logical_type, "kind", label)?;
    match kind {
        "unit" | "never" => exact_fields(logical_type, &["kind"], label),
        "primitive" => {
            exact_fields(logical_type, &["kind", "name"], label)?;
            match string(logical_type, "name", label)? {
                "bool" | "bytes" | "float" | "int" | "string" => Ok(()),
                name => Err(format!("{label} primitive is not supported: {name}")),
            }
        }
        "named" => {
            exact_fields(logical_type, &["kind", "identity"], label)?;
            check_type_identity(string(logical_type, "identity", label)?, label)
        }
        "array" => {
            exact_fields(logical_type, &["kind", "items"], label)?;
            check_logical_type(
                logical_type
                    .get("items")
                    .ok_or_else(|| format!("{label} items is missing"))?,
                &format!("{label} items"),
            )
        }
        "record" => {
            exact_fields(logical_type, &["kind", "fields"], label)?;
            let fields = logical_type
                .get("fields")
                .and_then(Value::as_array)
                .ok_or_else(|| format!("{label} fields must be an array"))?;
            let mut names = BTreeSet::new();
            for (index, field) in fields.iter().enumerate() {
                let field_label = format!("{label} field {index}");
                let field = object(field, &field_label)?;
                exact_fields(field, &["name", "type"], &field_label)?;
                let name = string(field, "name", &field_label)?;
                check_lower_camel(name, &format!("{field_label} name"))?;
                if !names.insert(name.to_owned()) {
                    return Err(format!("{label} field name is duplicated: {name}"));
                }
                check_logical_type(
                    field
                        .get("type")
                        .ok_or_else(|| format!("{field_label} type is missing"))?,
                    &format!("{field_label} type"),
                )?;
            }
            Ok(())
        }
        _ => Err(format!("{label} kind is not supported: {kind}")),
    }
}

fn check_portability(value: Option<&Value>, operation: &str) -> Result<(), String> {
    let label = format!("{operation} portability");
    let portability = object_at(value, &label)?;
    match string(portability, "kind", &label)? {
        "portable" => exact_fields(portability, &["kind"], &label),
        "target-extension" => {
            exact_fields(portability, &["kind", "target"], &label)?;
            check_kebab_identifier(string(portability, "target", &label)?, &label)
        }
        kind => Err(format!("{label} kind is not supported: {kind}")),
    }
}

fn check_type_identity(identity: &str, label: &str) -> Result<(), String> {
    let Some((module, symbol)) = identity.rsplit_once("::") else {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    };
    if module.is_empty() || symbol.is_empty() {
        return Err(format!(
            "{label} must contain a canonical module and symbol"
        ));
    }
    let segments = module
        .split("::")
        .flat_map(|part| part.split('/'))
        .collect::<Vec<_>>();
    let Some(first) = segments.first().copied() else {
        return Err(format!("{label} module is missing"));
    };
    if BACKEND_NAMESPACES.contains(&first) {
        return Err(format!("{label} uses backend-specific namespace {first}"));
    }
    if segments.len() < 2 {
        return Err(format!("{label} must include a module path"));
    }
    for segment in segments {
        check_kebab_identifier(segment, label)?;
    }
    check_upper_camel(symbol, label)
}

fn check_kebab_identifier(value: &str, label: &str) -> Result<(), String> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(format!("{label} must use lowercase kebab-case segments"));
    }
    Ok(())
}

fn check_lower_camel(value: &str, label: &str) -> Result<(), String> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase())
        || !chars.all(|character| character.is_ascii_alphanumeric())
    {
        return Err(format!("{label} must use lowerCamelCase"));
    }
    Ok(())
}

fn check_upper_camel(value: &str, label: &str) -> Result<(), String> {
    let mut chars = value.chars();
    if !chars
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
        || !chars.all(|character| character.is_ascii_alphanumeric())
    {
        return Err(format!("{label} symbol must use UpperCamelCase"));
    }
    Ok(())
}

fn object<'a>(value: &'a Value, label: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{label} must be an object"))
}

fn object_at<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a Map<String, Value>, String> {
    object(value.ok_or_else(|| format!("{label} is missing"))?, label)
}

fn exact_fields(object: &Map<String, Value>, allowed: &[&str], label: &str) -> Result<(), String> {
    for field in object.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(format!("{label} contains unknown field {field}"));
        }
    }
    for field in allowed {
        if !object.contains_key(*field) {
            return Err(format!("{label} field {field} is missing"));
        }
    }
    Ok(())
}

fn string<'a>(object: &'a Map<String, Value>, field: &str, label: &str) -> Result<&'a str, String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} {field} must be a string"))
}

fn unsigned(object: &Map<String, Value>, field: &str, label: &str) -> Result<u64, String> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{label} {field} must be an unsigned integer"))
}

#[cfg(test)]
mod tests {
    use super::{check_provider_contract, check_provider_contract_case};
    use serde_json::{json, Value};
    use std::path::PathBuf;

    fn valid_contract() -> Value {
        json!({
            "schema": 1,
            "kind": "provider-contract",
            "identity": "std/clock::Clock",
            "version": { "major": 1, "minor": 0 },
            "requirement": { "field": "clock", "type": "std/clock::Clock" },
            "operations": [{
                "identity": "std/clock::Clock#now",
                "kind": "one-shot",
                "input": { "kind": "unit" },
                "success": { "kind": "named", "identity": "std/time::Instant" },
                "failure": { "kind": "never" },
                "portability": { "kind": "portable" },
                "summary": "Observe the current monotonic instant."
            }]
        })
    }

    #[test]
    fn accepts_committed_clock_and_filesystem_contracts() {
        let artifacts = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/spec/artifacts/provider-contract-schema-1");
        for case in ["clock", "filesystem"] {
            check_provider_contract_case(&artifacts.join(case)).unwrap();
        }
    }

    #[test]
    fn accepts_package_and_module_qualified_type_identities() {
        let mut contract = valid_contract();
        contract["identity"] = json!("acme/payments::service::Payments");
        contract["requirement"]["field"] = json!("payments");
        contract["requirement"]["type"] = json!("acme/payments::service::Payments");
        contract["operations"][0]["identity"] = json!("acme/payments::service::Payments#charge");
        contract["operations"][0]["success"] = json!({
            "kind": "named",
            "identity": "acme/payments::model::Receipt"
        });
        check_provider_contract(&contract).unwrap();
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut contract = valid_contract();
        contract
            .as_object_mut()
            .unwrap()
            .insert("typescript".to_owned(), json!("Promise"));
        assert!(check_provider_contract(&contract)
            .unwrap_err()
            .contains("unknown field typescript"));
    }

    #[test]
    fn rejects_duplicate_operation_identities() {
        let mut contract = valid_contract();
        let duplicate = contract["operations"][0].clone();
        contract["operations"]
            .as_array_mut()
            .unwrap()
            .push(duplicate);
        assert!(check_provider_contract(&contract)
            .unwrap_err()
            .contains("operation identity is duplicated"));
    }

    #[test]
    fn rejects_backend_specific_logical_types() {
        let mut contract = valid_contract();
        contract["operations"][0]["success"] = json!({
            "kind": "named",
            "identity": "typescript/promise::Promise"
        });
        assert!(check_provider_contract(&contract)
            .unwrap_err()
            .contains("backend-specific namespace typescript"));
    }
}
