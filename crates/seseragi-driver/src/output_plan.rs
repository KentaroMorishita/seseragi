use seseragi_lowering::TypeScriptOutputPlan;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeScriptModuleOutput {
    module: String,
    path: String,
}

impl TypeScriptModuleOutput {
    pub fn new(module: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeScriptOutputPlanError {
    InvalidImporterPath { path: String },
    InvalidDependencyPath { module: String, path: String },
    DuplicateModule { module: String },
    DuplicateOutputPath { path: String },
}

/// Converts project-owned generated output paths into importer-relative ESM
/// specifiers. It does not choose output paths or inspect the filesystem.
pub fn plan_typescript_outputs(
    importer_path: &str,
    dependencies: impl IntoIterator<Item = TypeScriptModuleOutput>,
) -> Result<TypeScriptOutputPlan, TypeScriptOutputPlanError> {
    let importer = path_segments(importer_path).ok_or_else(|| {
        TypeScriptOutputPlanError::InvalidImporterPath {
            path: importer_path.to_owned(),
        }
    })?;
    let importer_directory = &importer[..importer.len() - 1];
    let mut modules = BTreeSet::new();
    let mut paths = BTreeSet::from([importer_path.to_owned()]);
    let mut specifiers = BTreeMap::new();

    for dependency in dependencies {
        if !modules.insert(dependency.module.clone()) {
            return Err(TypeScriptOutputPlanError::DuplicateModule {
                module: dependency.module,
            });
        }
        let target = path_segments(&dependency.path).ok_or_else(|| {
            TypeScriptOutputPlanError::InvalidDependencyPath {
                module: dependency.module.clone(),
                path: dependency.path.clone(),
            }
        })?;
        if !paths.insert(dependency.path.clone()) {
            return Err(TypeScriptOutputPlanError::DuplicateOutputPath {
                path: dependency.path,
            });
        }
        specifiers.insert(
            dependency.module,
            relative_specifier(importer_directory, &target),
        );
    }

    Ok(TypeScriptOutputPlan::new(specifiers))
}

fn path_segments(path: &str) -> Option<Vec<&str>> {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return None;
    }
    let segments = path.split('/').collect::<Vec<_>>();
    (!segments
        .iter()
        .any(|segment| matches!(*segment, "" | "." | "..")))
    .then_some(segments)
}

fn relative_specifier(from_directory: &[&str], target: &[&str]) -> String {
    let shared = from_directory
        .iter()
        .zip(target)
        .take_while(|(left, right)| left == right)
        .count();
    let mut segments = vec![".."; from_directory.len() - shared];
    segments.extend_from_slice(&target[shared..]);
    let relative = segments.join("/");
    if relative.starts_with("../") {
        relative
    } else {
        format!("./{relative}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_relative_specifiers_from_project_chosen_output_paths() {
        let plan = plan_typescript_outputs(
            "dist/game/main.js",
            [
                TypeScriptModuleOutput::new("game/domain", "dist/game/domain.js"),
                TypeScriptModuleOutput::new("shared/text", "dist/shared/text.js"),
            ],
        )
        .unwrap();

        assert_eq!(plan.specifier_for("game/domain"), Some("./domain.js"));
        assert_eq!(plan.specifier_for("shared/text"), Some("../shared/text.js"));
    }

    #[test]
    fn rejects_ambiguous_or_noncanonical_output_paths() {
        assert_eq!(
            plan_typescript_outputs(
                "main.js",
                [
                    TypeScriptModuleOutput::new("first", "domain.js"),
                    TypeScriptModuleOutput::new("second", "domain.js"),
                ],
            )
            .unwrap_err(),
            TypeScriptOutputPlanError::DuplicateOutputPath {
                path: "domain.js".to_owned()
            }
        );
        assert!(matches!(
            plan_typescript_outputs("../main.js", std::iter::empty::<TypeScriptModuleOutput>()),
            Err(TypeScriptOutputPlanError::InvalidImporterPath { .. })
        ));
    }
}
