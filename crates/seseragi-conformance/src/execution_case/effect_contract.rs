use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use seseragi_semantics::TypedModuleInterface;
use seseragi_syntax::{InterfaceRecordField, InterfaceType};

use super::environment::parse_required_environment_fields;

mod failure;
mod model;

#[cfg(test)]
pub(crate) use model::DictionaryImport;
pub(crate) use model::{EffectEntryContract, FailureRenderer};

pub(super) fn validate_effect_entry_contract(
    case: &Path,
    run: &serde_json::Value,
    entry_export: &str,
) -> Result<EffectEntryContract, String> {
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
    let effect = effect_type(&export.scheme.type_ref, entry_export)?;
    let environment = environment_fields(effect.environment, entry_export)?;
    let required = parse_required_environment_fields(run)?;
    compare_required_environment(&environment, &required, entry_export)?;

    let compiled_module = read_compiled_module(case, run)?;
    if compiled_module
        .get("module")
        .and_then(serde_json::Value::as_str)
        != Some(entry_module)
    {
        return Err(format!(
            "execution compiled module does not match typed interface module {entry_module}"
        ));
    }
    failure::resolve_effect_entry_contract(&interface, effect.failure, &compiled_module)
}

fn read_compiled_module(case: &Path, run: &serde_json::Value) -> Result<serde_json::Value, String> {
    let relative_path = run
        .pointer("/entry/compiledModule")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "run.json entry.compiledModule is required for Effect invocation".to_owned()
        })?;
    let raw = fs::read_to_string(case.join(relative_path))
        .map_err(|error| format!("failed to read execution generated module: {error}"))?;
    serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse execution generated module: {error}"))
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

#[cfg(test)]
fn effect_environment(
    entry_type: &InterfaceType,
    entry_export: &str,
) -> Result<BTreeMap<String, String>, String> {
    let effect = effect_type(entry_type, entry_export)?;
    environment_fields(effect.environment, entry_export)
}

struct EffectType<'a> {
    environment: &'a InterfaceType,
    failure: &'a InterfaceType,
}

fn effect_type<'a>(
    entry_type: &'a InterfaceType,
    entry_export: &str,
) -> Result<EffectType<'a>, String> {
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
    Ok(EffectType {
        environment: &arguments[0],
        failure: &arguments[1],
    })
}

fn environment_fields(
    environment: &InterfaceType,
    entry_export: &str,
) -> Result<BTreeMap<String, String>, String> {
    let InterfaceType::Record { closed, fields } = environment else {
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
