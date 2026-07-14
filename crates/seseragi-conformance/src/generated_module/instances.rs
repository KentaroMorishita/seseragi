use std::collections::BTreeSet;

use serde_json::Value;

pub(super) fn check_generated_instances(
    generated_module: &Value,
    typescript: &str,
) -> Result<(), String> {
    let Some(instances) = generated_module.get("instances") else {
        return Ok(());
    };
    let instances = instances
        .as_array()
        .ok_or_else(|| "generated module instances must be an array".to_owned())?;
    if instances.is_empty() {
        return Ok(());
    }

    let public_exports = generated_module
        .get("exports")
        .and_then(Value::as_array)
        .ok_or_else(|| "generated module exports must be an array".to_owned())?;
    let mut selected = BTreeSet::new();
    for (index, instance) in instances.iter().enumerate() {
        let label = format!("generated module instances[{index}]");
        let instance = instance
            .as_object()
            .ok_or_else(|| format!("{label} must be an object"))?;
        let identity = required_string(instance.get("identity"), &format!("{label}.identity"))?;
        let trait_name = required_string(instance.get("trait"), &format!("{label}.trait"))?;
        let arguments = instance
            .get("arguments")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("{label}.arguments must be an array"))?;
        if arguments.is_empty() {
            return Err(format!("{label}.arguments must not be empty"));
        }
        for (argument_index, argument) in arguments.iter().enumerate() {
            let argument_label = format!("{label}.arguments[{argument_index}]");
            let argument = argument
                .as_object()
                .ok_or_else(|| format!("{argument_label} must be an object"))?;
            required_string(argument.get("kind"), &format!("{argument_label}.kind"))?;
        }
        if trait_name == "Show" {
            required_string(
                instance.get("typeIdentity"),
                &format!("{label}.typeIdentity"),
            )?;
        }
        let dictionary_export = required_string(
            instance.get("dictionaryExport"),
            &format!("{label}.dictionaryExport"),
        )?;
        if !valid_typescript_binding_identifier(dictionary_export) {
            return Err(format!(
                "{label}.dictionaryExport must be a compiler-safe TypeScript identifier"
            ));
        }
        if !selected.insert(identity) {
            return Err(format!(
                "generated module has duplicate instance metadata identity {identity}"
            ));
        }
        if public_exports
            .iter()
            .any(|export| export.as_str() == Some(dictionary_export))
        {
            return Err(format!(
                "generated dictionary export {dictionary_export} must not appear in source-public metadata.exports"
            ));
        }
        if !typescript_exports_const(typescript, dictionary_export) {
            return Err(format!(
                "generated dictionary export {dictionary_export} is missing from TypeScript output"
            ));
        }
    }
    Ok(())
}

fn required_string<'a>(value: Option<&'a Value>, label: &str) -> Result<&'a str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label} must be a non-empty string"))
}

fn valid_typescript_binding_identifier(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic())
        || !characters.all(|character| {
            character == '_' || character == '$' || character.is_ascii_alphanumeric()
        })
    {
        return false;
    }
    !matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "implements"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn typescript_exports_const(typescript: &str, name: &str) -> bool {
    let prefix = format!("export const {name}");
    typescript.lines().any(|line| {
        line.strip_prefix(&prefix).is_some_and(|suffix| {
            suffix.starts_with(':')
                || suffix.starts_with(' ')
                || suffix.starts_with('=')
                || suffix.starts_with('<')
        })
    })
}

#[cfg(test)]
mod tests;
