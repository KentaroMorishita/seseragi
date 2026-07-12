use seseragi_lowering::{GeneratedOutputPaths, TypeScriptOutputPlan};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeScriptModuleOutput {
    module: String,
    path: String,
    instance_exports: Vec<TypeScriptInstanceOutput>,
}

/// A compiler-produced dictionary export available from one generated module.
///
/// `identity` is a semantic instance identity, not a trait or type-name
/// fallback. The driver carries it from dependency metadata to the backend
/// output plan without deriving a dictionary name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeScriptInstanceOutput {
    identity: String,
    dictionary_export: String,
}

impl TypeScriptInstanceOutput {
    pub fn new(identity: impl Into<String>, dictionary_export: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
            dictionary_export: dictionary_export.into(),
        }
    }
}

impl TypeScriptModuleOutput {
    pub fn new(module: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            path: path.into(),
            instance_exports: Vec::new(),
        }
    }

    pub fn with_instance_exports(
        mut self,
        instance_exports: impl IntoIterator<Item = TypeScriptInstanceOutput>,
    ) -> Self {
        self.instance_exports.extend(instance_exports);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeScriptOutputPlanError {
    InvalidImporterPath {
        path: String,
    },
    InvalidDependencyPath {
        module: String,
        path: String,
    },
    InvalidGeneratedOutputPath {
        path: String,
    },
    DuplicateModule {
        module: String,
    },
    DuplicateOutputPath {
        path: String,
    },
    DuplicateInstanceIdentity {
        module: String,
        identity: String,
    },
    DuplicateInstanceExport {
        module: String,
        dictionary_export: String,
    },
}

/// Converts a project-owned ESM `.js` output path to the TypeScript artifact
/// paths written by a development/compiler host. The emitted ESM import stays
/// `.js`; Bun and TypeScript resolve the staged `.ts` implementation.
pub fn generated_output_paths(
    output_path: &str,
) -> Result<GeneratedOutputPaths, TypeScriptOutputPlanError> {
    if path_segments(output_path).is_none() || !output_path.ends_with(".js") {
        return Err(TypeScriptOutputPlanError::InvalidGeneratedOutputPath {
            path: output_path.to_owned(),
        });
    }
    let stem = output_path
        .strip_suffix(".js")
        .expect("the suffix was checked");
    if stem.is_empty() {
        return Err(TypeScriptOutputPlanError::InvalidGeneratedOutputPath {
            path: output_path.to_owned(),
        });
    }
    Ok(GeneratedOutputPaths::new(
        format!("{stem}.ts"),
        format!("{stem}.ts.map"),
    ))
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
    let mut instance_exports = Vec::new();

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
            dependency.module.clone(),
            relative_specifier(importer_directory, &target),
        );

        let mut identities = BTreeSet::new();
        let mut dictionary_exports = BTreeSet::new();
        for instance in dependency.instance_exports {
            if !identities.insert(instance.identity.clone()) {
                return Err(TypeScriptOutputPlanError::DuplicateInstanceIdentity {
                    module: dependency.module.clone(),
                    identity: instance.identity,
                });
            }
            if !dictionary_exports.insert(instance.dictionary_export.clone()) {
                return Err(TypeScriptOutputPlanError::DuplicateInstanceExport {
                    module: dependency.module.clone(),
                    dictionary_export: instance.dictionary_export,
                });
            }
            instance_exports.push((
                (dependency.module.clone(), instance.identity),
                instance.dictionary_export,
            ));
        }
    }

    Ok(TypeScriptOutputPlan::new(specifiers).with_instance_exports(instance_exports))
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
    fn carries_semantic_instance_exports_from_each_provider() {
        let plan = plan_typescript_outputs(
            "dist/game/main.js",
            [
                TypeScriptModuleOutput::new("fixture/game::domain", "dist/game/domain.js")
                    .with_instance_exports([TypeScriptInstanceOutput::new(
                        "Show<fixture/game::domain::ImportedError>",
                        "__ssrg$instance$Show$0",
                    )]),
            ],
        )
        .unwrap();

        assert_eq!(
            plan.instance_export_for(
                "fixture/game::domain",
                "Show<fixture/game::domain::ImportedError>"
            ),
            Some("__ssrg$instance$Show$0")
        );
        assert_eq!(
            plan.instance_export_for(
                "fixture/game::domain",
                "Show<fixture/game::domain::OtherError>"
            ),
            None
        );
    }

    #[test]
    fn rejects_duplicate_instance_identity_or_export_within_one_provider() {
        let duplicate_identity = plan_typescript_outputs(
            "dist/game/main.js",
            [
                TypeScriptModuleOutput::new("fixture/game::domain", "dist/game/domain.js")
                    .with_instance_exports([
                        TypeScriptInstanceOutput::new(
                            "Show<fixture/game::domain::ImportedError>",
                            "__ssrg$instance$Show$0",
                        ),
                        TypeScriptInstanceOutput::new(
                            "Show<fixture/game::domain::ImportedError>",
                            "__ssrg$instance$Show$1",
                        ),
                    ]),
            ],
        )
        .unwrap_err();
        assert_eq!(
            duplicate_identity,
            TypeScriptOutputPlanError::DuplicateInstanceIdentity {
                module: "fixture/game::domain".to_owned(),
                identity: "Show<fixture/game::domain::ImportedError>".to_owned(),
            }
        );

        let duplicate_export = plan_typescript_outputs(
            "dist/game/main.js",
            [
                TypeScriptModuleOutput::new("fixture/game::domain", "dist/game/domain.js")
                    .with_instance_exports([
                        TypeScriptInstanceOutput::new(
                            "Show<fixture/game::domain::ImportedError>",
                            "__ssrg$instance$Show$0",
                        ),
                        TypeScriptInstanceOutput::new(
                            "Show<fixture/game::domain::OtherError>",
                            "__ssrg$instance$Show$0",
                        ),
                    ]),
            ],
        )
        .unwrap_err();
        assert_eq!(
            duplicate_export,
            TypeScriptOutputPlanError::DuplicateInstanceExport {
                module: "fixture/game::domain".to_owned(),
                dictionary_export: "__ssrg$instance$Show$0".to_owned(),
            }
        );
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

    #[test]
    fn derives_typescript_artifact_paths_without_rewriting_esm_specifiers() {
        assert_eq!(
            generated_output_paths("dist/game/main.js").unwrap(),
            GeneratedOutputPaths::new("dist/game/main.ts", "dist/game/main.ts.map")
        );
        assert!(matches!(
            generated_output_paths("dist/game/main.ts"),
            Err(TypeScriptOutputPlanError::InvalidGeneratedOutputPath { .. })
        ));
    }
}
