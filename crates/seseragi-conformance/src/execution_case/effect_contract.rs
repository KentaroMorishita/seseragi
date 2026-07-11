use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{InterfaceRecordField, InterfaceType};

use super::environment::parse_required_environment_fields;

pub(super) fn validate_effect_entry_contract(
    case: &Path,
    run: &serde_json::Value,
    entry_export: &str,
) -> Result<(), String> {
    let relative_path = run
        .pointer("/entry/typedInterface")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "run.json entry.typedInterface is required for Effect invocation".to_owned()
        })?;
    let interface_path = case.join(relative_path);
    let raw = fs::read_to_string(&interface_path)
        .map_err(|error| format!("failed to read execution typed interface: {error}"))?;
    let interface: TypedModuleInterface = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse execution typed interface: {error}"))?;
    if interface.schema != 1 || interface.stage != "typed-interface" {
        return Err(
            "execution entry.typedInterface must reference a schema 1 typed-interface artifact"
                .to_owned(),
        );
    }
    let entry_module = run
        .pointer("/entry/module")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "run.json entry.module is missing".to_owned())?;
    if interface.module != entry_module {
        return Err(format!(
            "execution typed interface module mismatch: expected {entry_module}, got {}",
            interface.module
        ));
    }

    let export = interface
        .exports
        .iter()
        .find(|export| export.namespace == "value" && export.name == entry_export)
        .ok_or_else(|| {
            format!("execution entry export {entry_export} is missing from entry.typedInterface")
        })?;
    let environment = effect_environment(&export.scheme.type_ref, entry_export)?;
    let required = parse_required_environment_fields(run)?;
    compare_required_environment(&environment, &required, entry_export)
}

fn compare_required_environment(
    environment: &BTreeMap<String, String>,
    required: &BTreeMap<String, String>,
    entry_export: &str,
) -> Result<(), String> {
    if environment == required {
        return Ok(());
    }
    Err(format!(
        "run.json requiredEnvironment does not match typed interface Effect R for {entry_export}: expected {environment:?}, got {required:?}"
    ))
}

fn effect_environment(
    entry_type: &InterfaceType,
    entry_export: &str,
) -> Result<BTreeMap<String, String>, String> {
    let InterfaceType::Function { result, .. } = entry_type else {
        return Err(format!(
            "execution Effect entry {entry_export} must have a function type"
        ));
    };
    let mut result = result.as_ref();
    while let InterfaceType::Function {
        result: next_result,
        ..
    } = result
    {
        result = next_result;
    }
    let InterfaceType::Named { name, arguments } = result else {
        return Err(format!(
            "execution Effect entry {entry_export} function result must be Effect<R, E, A>"
        ));
    };
    if name != "Effect" || arguments.len() != 3 {
        return Err(format!(
            "execution Effect entry {entry_export} function result must be Effect<R, E, A>"
        ));
    }
    let InterfaceType::Record { closed, fields } = &arguments[0] else {
        return Err(format!(
            "execution Effect entry {entry_export} environment R must be a closed record"
        ));
    };
    if !closed {
        return Err(format!(
            "execution Effect entry {entry_export} environment R must be a closed record"
        ));
    }
    canonical_record_fields(fields, entry_export)
}

fn canonical_record_fields(
    fields: &[InterfaceRecordField],
    entry_export: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut canonical = BTreeMap::new();
    for field in fields {
        if field.optional {
            return Err(format!(
                "execution Effect entry {entry_export} environment field {} cannot be optional",
                field.name
            ));
        }
        let spelling = canonical_type_spelling(&field.type_ref)?;
        if canonical.insert(field.name.clone(), spelling).is_some() {
            return Err(format!(
                "execution Effect entry {entry_export} environment has duplicate field {}",
                field.name
            ));
        }
    }
    Ok(canonical)
}

fn canonical_type_spelling(type_ref: &InterfaceType) -> Result<String, String> {
    match type_ref {
        InterfaceType::Named { name, arguments } => {
            render_type_application(name, arguments, canonical_type_spelling)
        }
        InterfaceType::Apply {
            constructor,
            arguments,
        } => render_type_application(constructor, arguments, canonical_type_spelling),
        InterfaceType::Tuple { elements } => Ok(format!(
            "({})",
            render_types(elements, canonical_type_spelling)?.join(", ")
        )),
        InterfaceType::Function { parameter, result } => Ok(format!(
            "{} -> {}",
            parenthesized_function_type(parameter)?,
            canonical_type_spelling(result)?
        )),
        InterfaceType::Record { closed, fields } => {
            let mut rendered = fields
                .iter()
                .map(|field| {
                    let optional = if field.optional { "?" } else { "" };
                    Ok(format!(
                        "{}{optional}: {}",
                        field.name,
                        canonical_type_spelling(&field.type_ref)?
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            rendered.sort();
            let tail = if *closed { "" } else { ", .." };
            Ok(format!("{{ {}{tail} }}", rendered.join(", ")))
        }
        InterfaceType::TypeConstructor { name, .. } => Ok(name.clone()),
        InterfaceType::Hole => {
            Err("execution Effect environment contains an unresolved type hole".to_owned())
        }
    }
}

fn render_type_application(
    name: &str,
    arguments: &[InterfaceType],
    render: fn(&InterfaceType) -> Result<String, String>,
) -> Result<String, String> {
    if arguments.is_empty() {
        return Ok(name.to_owned());
    }
    Ok(format!(
        "{name}<{}>",
        render_types(arguments, render)?.join(", ")
    ))
}

fn render_types(
    types: &[InterfaceType],
    render: fn(&InterfaceType) -> Result<String, String>,
) -> Result<Vec<String>, String> {
    types.iter().map(render).collect()
}

fn parenthesized_function_type(type_ref: &InterfaceType) -> Result<String, String> {
    let rendered = canonical_type_spelling(type_ref)?;
    if matches!(type_ref, InterfaceType::Function { .. }) {
        Ok(format!("({rendered})"))
    } else {
        Ok(rendered)
    }
}

#[cfg(test)]
mod tests;
