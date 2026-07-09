use crate::pipeline::{interface_source_name, parse_typescript_ir_json};
use crate::runtime_abi::runtime_feature_ids;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(crate) fn check_typescript_ir_json(root: &Path, case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_path = case.join("typescript-ir.json");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected = fs::read_to_string(&expected_path)
        .map_err(|error| format!("failed to read expected TypeScriptIr: {error}"))?;
    let actual_value = parse_typescript_ir_json(interface_source_name(case)?, &source)?;
    let expected_value: Value = serde_json::from_str(&expected)
        .map_err(|error| format!("failed to parse expected TypeScriptIr: {error}"))?;

    if actual_value != expected_value {
        return Err("TypeScriptIr artifact mismatch".to_owned());
    }
    check_runtime_requirements_exist(root, &actual_value)?;
    check_runtime_import_consistency(&actual_value)
}

fn check_runtime_requirements_exist(root: &Path, ir: &Value) -> Result<(), String> {
    let available = runtime_feature_ids(root)?;
    let requirements = required_string_array(
        ir,
        "runtimeRequirements",
        "TypeScriptIr runtimeRequirements",
    )?;
    for requirement in requirements {
        if !available.contains(requirement) {
            return Err(format!(
                "TypeScriptIr requires unknown runtime feature {requirement}"
            ));
        }
    }
    Ok(())
}

fn check_runtime_import_consistency(ir: &Value) -> Result<(), String> {
    let requirements = required_string_array(
        ir,
        "runtimeRequirements",
        "TypeScriptIr runtimeRequirements",
    )?;
    let mut requirement_set = BTreeSet::new();
    for requirement in requirements {
        if !requirement_set.insert(requirement) {
            return Err("TypeScriptIr runtimeRequirements contains duplicate feature".to_owned());
        }
    }

    let Some(imports) = ir.get("imports") else {
        return Ok(());
    };
    let imports = imports
        .as_array()
        .ok_or_else(|| "TypeScriptIr imports must be an array".to_owned())?;
    for (index, import) in imports.iter().enumerate() {
        let feature = required_string_field(
            import,
            "feature",
            &format!("TypeScriptIr imports[{index}].feature"),
        )?;
        required_string_field(
            import,
            "local",
            &format!("TypeScriptIr imports[{index}].local"),
        )?;
        if !requirement_set.contains(feature) {
            return Err(format!(
                "TypeScriptIr import feature `{feature}` is missing from runtimeRequirements"
            ));
        }
    }
    Ok(())
}

fn required_string_array<'a>(
    value: &'a Value,
    field: &str,
    label: &str,
) -> Result<Vec<&'a str>, String> {
    let array = value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{label} must be an array"))?;
    let mut strings = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate() {
        strings.push(
            item.as_str()
                .ok_or_else(|| format!("{label}[{index}] must be a string"))?,
        );
    }
    Ok(strings)
}

fn required_string_field<'a>(
    value: &'a Value,
    field: &str,
    label: &str,
) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} must be a string"))
        .and_then(|text| {
            if text.is_empty() {
                Err(format!("{label} must not be empty"))
            } else {
                Ok(text)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::check_runtime_import_consistency;
    use serde_json::json;

    #[test]
    fn accepts_imports_declared_as_runtime_requirements() {
        let ir = json!({
            "runtimeRequirements": ["core.unit", "effect.console.println"],
            "imports": [
                { "feature": "effect.console.println", "local": "_ssrg_console_println" }
            ]
        });

        assert!(check_runtime_import_consistency(&ir).is_ok());
    }

    #[test]
    fn rejects_imports_missing_from_runtime_requirements() {
        let ir = json!({
            "runtimeRequirements": ["core.unit"],
            "imports": [
                { "feature": "effect.console.println", "local": "_ssrg_console_println" }
            ]
        });

        let error = check_runtime_import_consistency(&ir).unwrap_err();
        assert!(error.contains("missing from runtimeRequirements"));
    }

    #[test]
    fn rejects_duplicate_runtime_requirements() {
        let ir = json!({
            "runtimeRequirements": ["core.unit", "core.unit"]
        });

        let error = check_runtime_import_consistency(&ir).unwrap_err();
        assert!(error.contains("duplicate"));
    }
}
