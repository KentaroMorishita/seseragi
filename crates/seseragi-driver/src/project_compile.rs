use crate::{
    compile_linked_module_with_output_paths, generated_output_paths, CompiledModule,
    LinkedCompileError,
};
use seseragi_project::{
    link_module, LinkError, LinkTargetError, ModuleGraph, ModuleGraphError, ModuleLinkTarget,
};
use seseragi_syntax::{
    parse_diagnostics, parse_unlinked_module_interface, DiagnosticArtifact, DiagnosticSeverity,
};
use std::collections::BTreeMap;

mod outputs;
mod validation;

#[cfg(test)]
mod package_scope_tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectModuleInput {
    pub source_name: String,
    pub module_id: String,
    pub source: String,
    pub output_path: String,
    pub package_scope: Option<String>,
}

impl ProjectModuleInput {
    pub fn new(
        source_name: impl Into<String>,
        module_id: impl Into<String>,
        source: impl Into<String>,
        output_path: impl Into<String>,
    ) -> Self {
        Self {
            source_name: source_name.into(),
            module_id: module_id.into(),
            source: source.into(),
            output_path: output_path.into(),
            package_scope: None,
        }
    }

    /// Assigns an opaque project-owned package scope for visibility linking.
    /// Modules in different scopes expose only their public interface.
    pub fn with_package_scope(mut self, package_scope: impl Into<String>) -> Self {
        self.package_scope = Some(package_scope.into());
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledProject {
    pub order: Vec<String>,
    pub modules: BTreeMap<String, CompiledModule>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectCompileError {
    Graph(ModuleGraphError<String>),
    DuplicateInput {
        module: String,
    },
    UnexpectedInput {
        module: String,
    },
    MissingInput {
        module: String,
    },
    DuplicateOutputPath {
        path: String,
        first_module: String,
        second_module: String,
    },
    GraphImportMismatch {
        module: String,
        graph_specifiers: Vec<String>,
        source_specifiers: Vec<String>,
    },
    Diagnostics {
        module: String,
        diagnostics: DiagnosticArtifact,
    },
    Link {
        module: String,
        errors: Vec<LinkError>,
    },
    LinkTarget {
        module: String,
        error: LinkTargetError,
    },
    OutputPlan {
        module: String,
        error: crate::TypeScriptOutputPlanError,
    },
    Compile {
        module: String,
        error: LinkedCompileError,
    },
}

/// Compiles a closed project graph in dependency order.
///
/// The caller owns filesystem discovery and supplies one source plus one
/// generated output path per graph node. Source import specifiers are matched
/// only against the graph's labeled edges; no path or module identity is
/// inferred inside the compiler pipeline.
pub fn compile_project(
    graph: ModuleGraph<String>,
    input_iter: impl IntoIterator<Item = ProjectModuleInput>,
) -> Result<CompiledProject, ProjectCompileError> {
    let order = graph
        .topological_order()
        .map_err(ProjectCompileError::Graph)?;
    let inputs = validation::index_project_inputs(&order, input_iter)?;

    let mut compiled: BTreeMap<String, CompiledModule> = BTreeMap::new();
    for module in &order {
        let input = inputs.get(module).expect("graph input was validated");
        let diagnostics = parse_diagnostics(input.source_name.clone(), &input.source);
        if has_errors(&diagnostics) {
            return Err(ProjectCompileError::Diagnostics {
                module: module.clone(),
                diagnostics,
            });
        }
        let unlinked = parse_unlinked_module_interface(
            input.source_name.clone(),
            input.module_id.clone(),
            &input.source,
        );
        validation::ensure_graph_imports_match(
            module,
            graph
                .dependencies_for(module)
                .expect("graph order contains only registered modules"),
            &unlinked,
        )?;
        let mut targets = BTreeMap::new();
        for (specifier, dependency) in graph
            .dependencies_for(module)
            .expect("graph order contains only registered modules")
        {
            let dependency_input = inputs.get(&dependency).expect("graph input was validated");
            let dependency_compiled = compiled
                .get(&dependency)
                .expect("topological order compiles dependencies first");
            let dependency_unlinked = parse_unlinked_module_interface(
                dependency_input.source_name.clone(),
                dependency_input.module_id.clone(),
                &dependency_input.source,
            );
            let dependency_interface = dependency_compiled
                .typed_interface
                .clone()
                .into_link_interface();
            let same_package = match (&input.package_scope, &dependency_input.package_scope) {
                (None, None) => true,
                (Some(importer), Some(dependency)) => importer == dependency,
                _ => false,
            };
            let target = if same_package {
                ModuleLinkTarget::same_package(dependency_unlinked.header, dependency_interface)
                    .map_err(|error| ProjectCompileError::LinkTarget {
                        module: module.clone(),
                        error,
                    })?
            } else {
                ModuleLinkTarget::external(dependency_interface)
            };
            targets.insert(specifier, target);
        }
        let linked =
            link_module(unlinked, &targets).map_err(|errors| ProjectCompileError::Link {
                module: module.clone(),
                errors,
            })?;
        // Inferred public contracts can carry a nominal type from a transitive
        // source provider without inventing a direct Seseragi import edge.
        // Every reachable provider compiled earlier in the closed graph is
        // therefore available under its project-owned output path. Unrelated
        // predecessor modules do not become implicit import edges.
        let provider_outputs =
            outputs::reachable_compiled_module_outputs(module, &graph, &inputs, &compiled);
        let output_plan = crate::plan_typescript_outputs(&input.output_path, provider_outputs)
            .map_err(|error| ProjectCompileError::OutputPlan {
                module: module.clone(),
                error,
            })?;
        let output_paths = generated_output_paths(&input.output_path).map_err(|error| {
            ProjectCompileError::OutputPlan {
                module: module.clone(),
                error,
            }
        })?;
        let compiled_module = compile_linked_module_with_output_paths(
            linked,
            &input.source,
            &output_plan,
            output_paths,
        )
        .map_err(|error| ProjectCompileError::Compile {
            module: module.clone(),
            error,
        })?;
        compiled.insert(module.clone(), compiled_module);
    }

    Ok(CompiledProject {
        order,
        modules: compiled,
    })
}

fn has_errors(diagnostics: &DiagnosticArtifact) -> bool {
    diagnostics
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_dependency_and_entry_through_the_linked_pipeline() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/game::main".to_owned(),
                [("./domain".to_owned(), "fixture/game::domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/game::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/game::domain",
                    "pub fn increment value: Int -> Int = value + 1\n",
                    "dist/game/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/game::main",
                    "import { increment as next } from \"./domain\"\n\npub fn run value: Int -> Int = next value\n",
                    "dist/game/main.js",
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            project.order,
            ["fixture/game::domain", "fixture/game::main"]
        );
        let main = project.modules.get("fixture/game::main").unwrap();
        assert!(main.generated.typescript.contains("from \"./domain.js\""));
        assert_eq!(
            main.generated.metadata.outputs.typescript,
            "dist/game/main.ts"
        );
        assert_eq!(
            main.generated.metadata.outputs.source_map,
            "dist/game/main.ts.map"
        );
        assert_eq!(main.generated.source_map.file, "dist/game/main.ts");
    }

    #[test]
    fn rejects_a_graph_input_with_parse_errors_before_linking() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/game::main".to_owned(), [])
            .unwrap();

        let error = compile_project(
            graph,
            [ProjectModuleInput::new(
                "main.ssrg",
                "fixture/game::main",
                "pub let answer: Int =\n",
                "dist/main.js",
            )],
        )
        .unwrap_err();
        assert!(matches!(error, ProjectCompileError::Diagnostics { .. }));
    }

    #[test]
    fn requires_an_esm_javascript_output_path_for_each_project_module() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/game::main".to_owned(), [])
            .unwrap();

        let error = compile_project(
            graph,
            [ProjectModuleInput::new(
                "main.ssrg",
                "fixture/game::main",
                "pub let answer: Int = 42\n",
                "dist/main.ts",
            )],
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ProjectCompileError::OutputPlan {
                error: crate::TypeScriptOutputPlanError::InvalidGeneratedOutputPath { .. },
                ..
            }
        ));
    }

    #[test]
    fn compiles_an_imported_adt_pattern_through_the_same_project_pipeline() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/rps::main".to_owned(),
                [("./domain".to_owned(), "fixture/rps::domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/rps::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/rps::domain",
                    "pub type Hand =\n  | Rock\n  | Paper\n  | Scissors\n",
                    "dist/rps/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/rps::main",
                    "import { Hand, Rock } from \"./domain\"\npub fn isRock hand: Hand -> Bool =\n  match hand {\n    Rock -> True\n    _ -> False\n  }\n",
                    "dist/rps/main.js",
                ),
            ],
        )
        .unwrap();

        let main = project.modules.get("fixture/rps::main").unwrap();
        assert_eq!(main.typed_hir.module_dependencies.len(), 1);
        assert!(main.generated.typescript.contains("from \"./domain.js\""));
        assert!(main.generated.typescript.contains("export const isRock"));
    }

    #[test]
    fn imports_a_direct_dependency_show_dictionary_from_generated_metadata() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/game::main".to_owned(),
                [("./domain".to_owned(), "fixture/game::domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/game::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/game::domain",
                    "type InternalError deriving Show =\n  | Internal\n\npub type ImportedError deriving Show =\n  | Message String\n",
                    "dist/game/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/game::main",
                    "import { ImportedError } from \"./domain\"\n\npub type AppError deriving Show =\n  | Invalid ImportedError\n",
                    "dist/game/main.js",
                ),
            ],
        )
        .unwrap();

        let domain = project.modules.get("fixture/game::domain").unwrap();
        let main = project.modules.get("fixture/game::main").unwrap();
        let domain_instance = domain
            .generated
            .metadata
            .instances
            .iter()
            .find(|instance| instance.identity == "Show<fixture/game::domain::ImportedError>")
            .unwrap();
        let main_instance = &main.generated.metadata.instances[0];
        assert_eq!(
            domain_instance.identity,
            "Show<fixture/game::domain::ImportedError>"
        );
        assert_eq!(main_instance.identity, "Show<fixture/game::main::AppError>");
        assert_ne!(domain_instance.identity, main_instance.identity);
        assert_eq!(domain_instance.dictionary_export, "__ssrg$instance$Show$1");
        assert!(main.generated.typescript.contains(
            "import { type ImportedError, __ssrg$instance$Show$1 } from \"./domain.js\""
        ));
        assert!(main
            .generated
            .typescript
            .contains("__ssrg$instance$Show$1.show(value.value)"));
    }

    #[test]
    fn imports_a_public_inherent_method_with_its_nominal_owner() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/method::main".to_owned(),
                [("./domain".to_owned(), "fixture/method::domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/method::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/method::domain",
                    "pub opaque struct Box<A> {\n  value: A,\n}\n\npub fn box<A> value: A -> Box<A> = Box { value }\n\nimpl<A> Box<A> {\n  pub fn get self: Box<A> -> A = self.value\n\n  pub fn map self: Box<A> -> transform: (A -> A) -> Box<A> =\n    Box { value: transform self.value }\n}\n",
                    "dist/method/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/method::main",
                    "import { box } from \"./domain\"\n\npub fn run value: Int -> Int =\n  ((box value).map (\\item -> item + item)).get\n",
                    "dist/method/main.js",
                ),
            ],
        )
        .unwrap();

        let domain = project.modules.get("fixture/method::domain").unwrap();
        let owner = domain
            .typed_interface
            .exports
            .iter()
            .find(|export| export.namespace == "type" && export.name == "Box")
            .unwrap();
        let method = owner
            .methods
            .iter()
            .find(|method| method.name == "get")
            .unwrap();
        assert_eq!(method.scheme.type_parameters.len(), 1);
        assert_eq!(owner.methods.len(), 2);
        assert!(!domain
            .typed_interface
            .exports
            .iter()
            .any(|export| { export.namespace == "value" && export.name == "get" }));
        assert!(domain
            .generated
            .typescript
            .contains("export const __ssrg$method$Box$get"));
        assert!(domain
            .generated
            .typescript
            .contains("export const __ssrg$method$Box$map"));

        let main = project.modules.get("fixture/method::main").unwrap();
        assert!(main
            .generated
            .typescript
            .contains("__ssrg$method$Box$get as get"));
        assert!(main
            .generated
            .typescript
            .contains("__ssrg$method$Box$map as map"));
        assert!(main.generated.typescript.contains("get(map(box(value))"));
    }
}

#[cfg(test)]
#[path = "project_compile/provider_tests.rs"]
mod provider_tests;
