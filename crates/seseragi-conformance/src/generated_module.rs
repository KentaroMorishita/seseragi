use crate::pipeline::{emit_generated_module, interface_source_name, parse_typescript_ir_json};
use crate::runtime_abi::{runtime_feature_ids, runtime_helper_imports};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn check_generated_module(root: &Path, case: &Path) -> Result<(), String> {
    let source_path = case.join("main.ssrg");
    let expected_metadata_path = case.join("generated-module.json");
    let expected_typescript_path = case.join("main.ts");
    let expected_source_map_path = case.join("main.ts.map");
    let source = fs::read_to_string(&source_path)
        .map_err(|error| format!("failed to read source: {error}"))?;
    let expected_metadata = fs::read_to_string(&expected_metadata_path)
        .map_err(|error| format!("failed to read expected GeneratedModule: {error}"))?;
    let expected_typescript = fs::read_to_string(&expected_typescript_path)
        .map_err(|error| format!("failed to read expected main.ts: {error}"))?;
    let expected_source_map = fs::read_to_string(&expected_source_map_path)
        .map_err(|error| format!("failed to read expected main.ts.map: {error}"))?;
    let bundle = emit_generated_module(interface_source_name(case)?, &source)?;

    let actual_metadata_value = serde_json::to_value(&bundle.metadata)
        .map_err(|error| format!("failed to encode GeneratedModule: {error}"))?;
    let expected_metadata_value: serde_json::Value = serde_json::from_str(&expected_metadata)
        .map_err(|error| format!("failed to parse expected GeneratedModule: {error}"))?;
    if actual_metadata_value != expected_metadata_value {
        return Err("GeneratedModule artifact mismatch".to_owned());
    }
    check_generated_runtime_requirements(root, &actual_metadata_value)?;
    if bundle.typescript != expected_typescript {
        return Err("main.ts artifact mismatch".to_owned());
    }
    check_generated_exports(&actual_metadata_value, &bundle.typescript)?;
    check_generated_runtime_imports(
        root,
        &parse_typescript_ir_json(interface_source_name(case)?, &source)?,
        &bundle.typescript,
    )?;

    let actual_source_map_value = serde_json::to_value(&bundle.source_map)
        .map_err(|error| format!("failed to encode SourceMap: {error}"))?;
    let expected_source_map_value: serde_json::Value =
        serde_json::from_str(&expected_source_map)
            .map_err(|error| format!("failed to parse expected main.ts.map: {error}"))?;
    if actual_source_map_value != expected_source_map_value {
        return Err("main.ts.map artifact mismatch".to_owned());
    }
    check_generated_source_map(&actual_metadata_value, &actual_source_map_value, &source)?;
    Ok(())
}

fn check_generated_runtime_imports(
    root: &Path,
    typescript_ir: &Value,
    typescript: &str,
) -> Result<(), String> {
    let helper_imports = runtime_helper_imports(root)?;
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
        let helper = helper_imports.get(feature).ok_or_else(|| {
            format!("TypeScriptIr import feature {feature} has no runtime helper ABI import")
        })?;
        if !typescript
            .lines()
            .any(|line| typescript_line_imports_helper(line, helper, local))
        {
            return Err(format!(
                "generated TypeScript import for runtime feature {feature} does not match ABI"
            ));
        }
    }
    Ok(())
}

fn typescript_line_imports_helper(
    line: &str,
    helper: &crate::runtime_abi::RuntimeHelperImport,
    local: &str,
) -> bool {
    line.starts_with("import { ")
        && line.contains(&format!("{} as {local}", helper.export_name))
        && line.ends_with(&format!("from \"{}\"", helper.module))
}

fn check_generated_source_map(
    generated_module: &serde_json::Value,
    source_map: &serde_json::Value,
    source: &str,
) -> Result<(), String> {
    if source_map
        .pointer("/version")
        .and_then(|value| value.as_u64())
        != Some(3)
    {
        return Err("generated source map version must be 3".to_owned());
    }
    if source_map.pointer("/file").and_then(|value| value.as_str()) != Some("main.ts") {
        return Err("generated source map file must be main.ts".to_owned());
    }
    let module = generated_module
        .pointer("/module")
        .and_then(|value| value.as_str())
        .ok_or_else(|| "generated module name must be a string".to_owned())?;
    let expected_source_uri = format!("seseragi://{module}");
    let sources = source_map
        .pointer("/sources")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated source map sources must be an array".to_owned())?;
    if sources.len() != 1 || sources[0].as_str() != Some(expected_source_uri.as_str()) {
        return Err("generated source map sources must contain the Seseragi module URI".to_owned());
    }
    let sources_content = source_map
        .pointer("/sourcesContent")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated source map sourcesContent must be an array".to_owned())?;
    if sources_content.len() != 1 || sources_content[0].as_str() != Some(source) {
        return Err("generated source map sourcesContent must preserve source text".to_owned());
    }
    Ok(())
}

fn check_generated_exports(
    generated_module: &serde_json::Value,
    typescript: &str,
) -> Result<(), String> {
    let exports = generated_module
        .pointer("/exports")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated module exports must be an array".to_owned())?;

    for export in exports {
        let export = export
            .as_str()
            .ok_or_else(|| "generated module export name must be a string".to_owned())?;
        if !typescript_exports_name(typescript, export) {
            return Err(format!(
                "generated module export {export} is missing from TypeScript output"
            ));
        }
    }
    Ok(())
}

fn typescript_exports_name(typescript: &str, name: &str) -> bool {
    typescript.contains(&format!("export const {name}"))
        || typescript.contains(&format!("export function {name}"))
        || typescript.contains(&format!("export {{ {name}"))
        || typescript.contains(&format!(", {name}"))
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

fn check_generated_runtime_requirements(
    root: &Path,
    generated_module: &serde_json::Value,
) -> Result<(), String> {
    let available = runtime_feature_ids(root)?;
    let requirements = generated_module
        .pointer("/runtime/requirements")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "generated module runtime.requirements must be an array".to_owned())?;

    for requirement in requirements {
        let requirement = requirement
            .as_str()
            .ok_or_else(|| "generated module runtime requirement must be a string".to_owned())?;
        if !available.contains(requirement) {
            return Err(format!(
                "generated module requires unknown runtime feature {requirement}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::typescript_line_imports_helper;
    use crate::runtime_abi::RuntimeHelperImport;

    #[test]
    fn accepts_grouped_generated_import_from_runtime_abi_mapping() {
        let helper = RuntimeHelperImport {
            module: "@seseragi/runtime/console".to_owned(),
            export_name: "println".to_owned(),
        };

        assert!(typescript_line_imports_helper(
            "import { print as _ssrg_console_print, println as _ssrg_console_println } from \"@seseragi/runtime/console\"",
            &helper,
            "_ssrg_console_println"
        ));
    }
}
