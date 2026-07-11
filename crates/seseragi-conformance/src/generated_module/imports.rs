use crate::runtime_abi::{runtime_imports, runtime_type_imports, RuntimeImport};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn check_generated_runtime_imports(
    root: &Path,
    typescript_ir: &Value,
    typescript: &str,
) -> Result<(), String> {
    check_value_imports(&runtime_imports(root)?, typescript_ir, typescript)?;
    check_type_imports(&runtime_type_imports(root)?, typescript_ir, typescript)
}

fn check_value_imports(
    runtime_imports: &BTreeMap<String, RuntimeImport>,
    typescript_ir: &Value,
    typescript: &str,
) -> Result<(), String> {
    let Some(imports) = typescript_ir.get("imports") else {
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
        let local = required_string_field(
            import,
            "local",
            &format!("TypeScriptIr imports[{index}].local"),
        )?;
        let runtime_import = runtime_imports.get(feature).ok_or_else(|| {
            format!("TypeScriptIr import feature {feature} has no runtime ABI import")
        })?;
        if !typescript
            .lines()
            .any(|line| line_imports_value_binding(line, runtime_import, local))
        {
            return Err(format!(
                "generated TypeScript import for runtime feature {feature} does not match ABI"
            ));
        }
    }
    Ok(())
}

fn line_imports_value_binding(line: &str, runtime_import: &RuntimeImport, local: &str) -> bool {
    line.starts_with("import { ")
        && line.contains(&format!("{} as {local}", runtime_import.export_name))
        && line.ends_with(&format!("from \"{}\"", runtime_import.module))
}

fn check_type_imports(
    runtime_type_imports: &BTreeMap<String, RuntimeImport>,
    typescript_ir: &Value,
    typescript: &str,
) -> Result<(), String> {
    let Some(imports) = typescript_ir.get("typeImports") else {
        return Ok(());
    };
    let imports = imports
        .as_array()
        .ok_or_else(|| "TypeScriptIr typeImports must be an array".to_owned())?;

    for (index, import) in imports.iter().enumerate() {
        let feature = required_string_field(
            import,
            "feature",
            &format!("TypeScriptIr typeImports[{index}].feature"),
        )?;
        let local = required_string_field(
            import,
            "local",
            &format!("TypeScriptIr typeImports[{index}].local"),
        )?;
        let runtime_import = runtime_type_imports.get(feature).ok_or_else(|| {
            format!("TypeScriptIr type import feature {feature} has no runtime ABI typeImport")
        })?;
        if !typescript
            .lines()
            .any(|line| line_imports_type_binding(line, runtime_import, local))
        {
            return Err(format!(
                "generated TypeScript type import for runtime feature {feature} does not match ABI"
            ));
        }
    }
    Ok(())
}

fn line_imports_type_binding(line: &str, runtime_import: &RuntimeImport, local: &str) -> bool {
    let module_suffix = format!(" }} from \"{}\"", runtime_import.module);
    let Some(specifiers) = line
        .strip_prefix("import { ")
        .and_then(|line| line.strip_suffix(&module_suffix))
    else {
        return false;
    };
    let expected = format!("type {} as {local}", runtime_import.export_name);
    specifiers
        .split(", ")
        .any(|specifier| specifier == expected)
}

fn required_string_field<'a>(
    value: &'a Value,
    field: &str,
    label: &str,
) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label} must be a non-empty string"))
}

#[cfg(test)]
mod tests {
    use super::{check_type_imports, line_imports_type_binding, line_imports_value_binding};
    use crate::runtime_abi::RuntimeImport;
    use std::collections::BTreeMap;

    #[test]
    fn accepts_grouped_generated_import_from_runtime_abi_mapping() {
        let runtime_import = RuntimeImport {
            module: "@seseragi/runtime/console".to_owned(),
            export_name: "println".to_owned(),
        };

        assert!(line_imports_value_binding(
            "import { print as _ssrg_console_print, println as _ssrg_console_println } from \"@seseragi/runtime/console\"",
            &runtime_import,
            "_ssrg_console_println"
        ));
    }

    #[test]
    fn accepts_grouped_inline_type_import_from_runtime_abi_mapping() {
        let runtime_import = RuntimeImport {
            module: "@seseragi/runtime/console".to_owned(),
            export_name: "ConsoleError".to_owned(),
        };

        assert!(line_imports_type_binding(
            "import { println as _ssrg_console_println, type ConsoleError as ConsoleError } from \"@seseragi/runtime/console\"",
            &runtime_import,
            "ConsoleError"
        ));
        assert!(!line_imports_type_binding(
            "import { ConsoleError as ConsoleError } from \"@seseragi/runtime/console\"",
            &runtime_import,
            "ConsoleError"
        ));
    }

    #[test]
    fn rejects_type_import_that_does_not_match_runtime_abi() {
        let imports = BTreeMap::from([(
            "effect.console.error".to_owned(),
            RuntimeImport {
                module: "@seseragi/runtime/console".to_owned(),
                export_name: "ConsoleError".to_owned(),
            },
        )]);
        let typescript_ir = serde_json::json!({
            "typeImports": [{
                "feature": "effect.console.error",
                "local": "ConsoleError"
            }]
        });

        assert_eq!(
            check_type_imports(
                &imports,
                &typescript_ir,
                "import { type StdinError as ConsoleError } from \"@seseragi/runtime/stdin\""
            ),
            Err("generated TypeScript type import for runtime feature effect.console.error does not match ABI".to_owned())
        );
    }
}
