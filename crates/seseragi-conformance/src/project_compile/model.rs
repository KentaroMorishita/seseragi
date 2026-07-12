use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectCompileCase {
    pub(crate) modules: Vec<ProjectCompileModule>,
    pub(crate) expected_order: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectCompileModule {
    pub(crate) id: String,
    pub(crate) source: String,
    pub(crate) output: String,
    pub(crate) artifacts: String,
    pub(crate) imports: Vec<ProjectImport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectImport {
    pub(crate) specifier: String,
    pub(crate) module: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectDescriptor {
    schema: u32,
    kind: String,
    modules: Vec<ProjectModuleDescriptor>,
    #[serde(rename = "expectedOrder")]
    expected_order: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectModuleDescriptor {
    id: String,
    source: String,
    output: String,
    artifacts: String,
    imports: Vec<ProjectImportDescriptor>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectImportDescriptor {
    specifier: String,
    module: String,
}

/// Reads a closed project compile descriptor without invoking the compiler.
pub(crate) fn load_project_compile_case(case: &Path) -> Result<ProjectCompileCase, String> {
    let descriptor_path = case.join("project.json");
    let raw = fs::read_to_string(&descriptor_path)
        .map_err(|error| format!("failed to read project.json: {error}"))?;
    let descriptor: ProjectDescriptor = serde_json::from_str(&raw)
        .map_err(|error| format!("failed to parse project.json: {error}"))?;
    if descriptor.schema != 1 {
        return Err("project.json must use schema 1".to_owned());
    }
    if descriptor.kind != "closed-compile" {
        return Err("project.json kind must be closed-compile".to_owned());
    }
    if descriptor.modules.is_empty() {
        return Err("project.json must contain at least one module".to_owned());
    }

    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut outputs = BTreeSet::new();
    let mut artifact_directories = BTreeSet::new();
    let mut modules = Vec::with_capacity(descriptor.modules.len());
    for module in descriptor.modules {
        if module.id.is_empty() || !ids.insert(module.id.clone()) {
            return Err(format!(
                "project.json has duplicate or empty module id `{}`",
                module.id
            ));
        }
        validate_case_path("source", &module.source)?;
        validate_case_path("output", &module.output)?;
        validate_case_path("artifacts", &module.artifacts)?;
        if !module.source.ends_with(".ssrg") {
            return Err(format!(
                "project.json source must end in .ssrg: {}",
                module.source
            ));
        }
        if !module.output.ends_with(".js") {
            return Err(format!(
                "project.json output must end in .js: {}",
                module.output
            ));
        }
        if !sources.insert(module.source.clone()) {
            return Err(format!(
                "project.json has duplicate source path {}",
                module.source
            ));
        }
        if !outputs.insert(module.output.clone()) {
            return Err(format!(
                "project.json has duplicate output path {}",
                module.output
            ));
        }
        if !artifact_directories.insert(module.artifacts.clone()) {
            return Err(format!(
                "project.json has duplicate artifacts directory {}",
                module.artifacts
            ));
        }
        let source_path = case.join(&module.source);
        if !source_path.is_file() {
            return Err(format!("project source is missing: {}", module.source));
        }
        let mut import_specifiers = BTreeSet::new();
        let imports = module
            .imports
            .into_iter()
            .map(|import| {
                if import.specifier.is_empty()
                    || !import_specifiers.insert(import.specifier.clone())
                {
                    return Err(format!(
                        "project module {} has duplicate or empty import specifier `{}`",
                        module.id, import.specifier
                    ));
                }
                if import.module.is_empty() {
                    return Err(format!(
                        "project module {} has an import with an empty target module",
                        module.id
                    ));
                }
                Ok(ProjectImport {
                    specifier: import.specifier,
                    module: import.module,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        modules.push(ProjectCompileModule {
            id: module.id,
            source: module.source,
            output: module.output,
            artifacts: module.artifacts,
            imports,
        });
    }

    let expected_order = descriptor.expected_order;
    if expected_order.len() != modules.len()
        || expected_order.iter().collect::<BTreeSet<_>>().len() != modules.len()
        || expected_order.iter().any(|module| !ids.contains(module))
    {
        return Err(
            "project.json expectedOrder must contain every module id exactly once".to_owned(),
        );
    }
    for module in &modules {
        for import in &module.imports {
            if !ids.contains(&import.module) {
                return Err(format!(
                    "project module {} imports undeclared module {}",
                    module.id, import.module
                ));
            }
        }
    }

    Ok(ProjectCompileCase {
        modules,
        expected_order,
    })
}

fn validate_case_path(label: &str, value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value
            .split('/')
            .any(|segment| matches!(segment, "" | "." | ".."))
    {
        return Err(format!(
            "project.json {label} must be a canonical relative path: {value}"
        ));
    }
    Ok(())
}
