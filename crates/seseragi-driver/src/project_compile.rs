use crate::{
    compile::compile_analyzed_module_with_output_paths, generated_output_paths, CompiledModule,
    LinkedCompileError,
};
use seseragi_project::{
    link_module, standard_module_target, LinkError, LinkTargetError, ModuleGraph, ModuleGraphError,
    ModuleLinkTarget,
};
use seseragi_semantics::{
    analysis_document, analyze_linked_module, analyze_linked_module_recovering, AnalysisDocument,
    AnalyzedModule,
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
pub struct AnalyzedProject {
    pub order: Vec<String>,
    pub documents: BTreeMap<String, AnalysisDocument>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectModuleDiagnostics {
    pub module: String,
    pub diagnostics: DiagnosticArtifact,
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
        modules: Vec<ProjectModuleDiagnostics>,
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
    let mut frontend = analyze_project_frontend(&graph, &order, &inputs, false)?;
    if !frontend.diagnostics.is_empty() {
        return Err(ProjectCompileError::Diagnostics {
            modules: frontend.diagnostics,
        });
    }

    let mut compiled: BTreeMap<String, CompiledModule> = BTreeMap::new();
    for module in &order {
        let input = inputs.get(module).expect("graph input was validated");
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
        let analyzed = frontend
            .analyzed
            .remove(module)
            .expect("error-free frontend analyzed every module");
        let compiled_module = compile_analyzed_module_with_output_paths(
            analyzed,
            &input.source,
            &output_plan,
            output_paths,
        )
        .map_err(|error| ProjectCompileError::Compile {
            module: module.clone(),
            error: LinkedCompileError::TypeScriptPlan(error),
        })?;
        compiled.insert(module.clone(), compiled_module);
    }

    Ok(CompiledProject {
        order,
        modules: compiled,
    })
}

/// Analyzes a closed project graph in dependency order without lowering or
/// code generation.
///
/// Like [`compile_project`], the caller owns source discovery and graph
/// identity. Public interfaces from already-analyzed dependencies are linked
/// before each document is resolved and typed, so imported symbols and types
/// are available to browser and editor queries.
pub fn analyze_project(
    graph: ModuleGraph<String>,
    input_iter: impl IntoIterator<Item = ProjectModuleInput>,
) -> Result<AnalyzedProject, ProjectCompileError> {
    let order = graph
        .topological_order()
        .map_err(ProjectCompileError::Graph)?;
    let inputs = validation::index_project_inputs(&order, input_iter)?;
    let frontend = analyze_project_frontend(&graph, &order, &inputs, true)?;
    if frontend.has_parse_errors {
        return Err(ProjectCompileError::Diagnostics {
            modules: frontend.diagnostics,
        });
    }

    let documents = frontend
        .analyzed
        .into_iter()
        .map(|(module, analyzed)| {
            let document =
                analysis_document(analyzed.diagnostics, analyzed.resolved, &analyzed.typed_hir);
            (module, document)
        })
        .collect();
    Ok(AnalyzedProject { order, documents })
}

struct ProjectFrontend {
    analyzed: BTreeMap<String, AnalyzedModule>,
    diagnostics: Vec<ProjectModuleDiagnostics>,
    has_parse_errors: bool,
}

fn analyze_project_frontend(
    graph: &ModuleGraph<String>,
    order: &[String],
    inputs: &BTreeMap<String, ProjectModuleInput>,
    retain_semantic_recovery: bool,
) -> Result<ProjectFrontend, ProjectCompileError> {
    let mut parsed = BTreeMap::new();
    let mut diagnostics_by_module = BTreeMap::new();
    let mut has_parse_errors = false;

    for module in order {
        let input = inputs.get(module).expect("graph input was validated");
        let diagnostics = parse_diagnostics(input.source_name.clone(), &input.source);
        if has_errors(&diagnostics) {
            has_parse_errors = true;
            diagnostics_by_module.insert(module.clone(), diagnostics);
            continue;
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
        parsed.insert(module.clone(), (diagnostics, unlinked));
    }

    let mut analyzed: BTreeMap<String, AnalyzedModule> = BTreeMap::new();
    for module in order {
        let Some((diagnostics, unlinked)) = parsed.remove(module) else {
            continue;
        };
        let dependencies = graph
            .dependencies_for(module)
            .expect("graph order contains only registered modules");
        if dependencies
            .iter()
            .any(|(_, dependency)| !analyzed.contains_key(dependency))
        {
            continue;
        }

        let input = inputs.get(module).expect("graph input was validated");
        let mut targets = unlinked
            .imports
            .iter()
            .filter_map(|import| {
                standard_module_target(&import.specifier)
                    .map(|target| (import.specifier.clone(), target))
            })
            .collect::<BTreeMap<_, _>>();
        for (specifier, dependency) in dependencies {
            let dependency_input = inputs.get(&dependency).expect("graph input was validated");
            let dependency_analyzed = analyzed
                .get(&dependency)
                .expect("invalid dependencies were filtered before linking");
            let dependency_unlinked = parse_unlinked_module_interface(
                dependency_input.source_name.clone(),
                dependency_input.module_id.clone(),
                &dependency_input.source,
            );
            let dependency_interface = dependency_analyzed
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
        let analysis = if retain_semantic_recovery {
            analyze_linked_module_recovering(diagnostics, linked, &input.source)
        } else {
            analyze_linked_module(diagnostics, linked, &input.source)
        };
        match analysis {
            Ok(document) => {
                if has_errors(&document.diagnostics) {
                    diagnostics_by_module.insert(module.clone(), document.diagnostics.clone());
                }
                analyzed.insert(module.clone(), document);
            }
            Err(diagnostics) => {
                diagnostics_by_module.insert(module.clone(), diagnostics);
            }
        }
    }

    let diagnostics = order
        .iter()
        .filter_map(|module| {
            diagnostics_by_module
                .remove(module)
                .map(|diagnostics| ProjectModuleDiagnostics {
                    module: module.clone(),
                    diagnostics,
                })
        })
        .collect();
    Ok(ProjectFrontend {
        analyzed,
        diagnostics,
        has_parse_errors,
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
    fn preserves_external_result_types_for_imported_component_calls() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/components::parent".to_owned(),
                [("./child".to_owned(), "fixture/components::child".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/components::child".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "child.ssrg",
                    "fixture/components::child",
                    "import * as html from \"std/web/html\"\n\nstruct ChildState { count: Int }\n\ntype ChildAction =\n  | Increment\n\npub fn component action: Effect<{}, Never, Unit> -> html.Html<Effect<{}, Never, Unit>> =\n  html.button { onClick: action, children: \"Child\" }\n",
                    "dist/components/child.js",
                ),
                ProjectModuleInput::new(
                    "parent.ssrg",
                    "fixture/components::parent",
                    "import * as html from \"std/web/html\"\nimport { component } from \"./child\"\n\npub fn parent action: Effect<{}, Never, Unit> -> html.Html<Effect<{}, Never, Unit>> =\n  html.section { children: [component action] }\n",
                    "dist/components/parent.js",
                ),
            ],
        )
        .unwrap();

        let child = project.modules.get("fixture/components::child").unwrap();
        let component = child
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "component")
            .unwrap();
        let seseragi_syntax::InterfaceType::Function { result, .. } = &component.scheme.type_ref
        else {
            panic!("component must expose a function contract");
        };
        assert!(matches!(
            result.as_ref(),
            seseragi_syntax::InterfaceType::ExternalNamed {
                canonical,
                provider_module,
                provider_export,
                ..
            } if canonical == "std/web/html::Html"
                && provider_module == "std/web/html"
                && provider_export == "Html"
        ));
        assert!(child
            .typed_interface
            .exports
            .iter()
            .all(|export| export.name != "ChildState" && export.name != "ChildAction"));

        let parent = project.modules.get("fixture/components::parent").unwrap();
        assert!(parent.generated.typescript.contains("from \"./child.js\""));
        assert!(parent.generated.typescript.contains("component(action)"));
    }

    #[test]
    fn expands_imported_public_aliases_without_runtime_edges() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/alias::main".to_owned(),
                [("./domain".to_owned(), "fixture/alias::domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("fixture/alias::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/alias::domain",
                    "pub alias Pair<A> = { left: A, right: A }\n\npub fn pair<A> value: A -> Pair<A> = { left: value, right: value }\n",
                    "dist/alias/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/alias::main",
                    "import { Pair, pair } from \"./domain\"\n\npub fn duplicate value: Int -> Pair<Int> = pair value\n",
                    "dist/alias/main.js",
                ),
            ],
        )
        .unwrap();

        let domain = project.modules.get("fixture/alias::domain").unwrap();
        let alias = domain
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "Pair")
            .unwrap();
        assert!(matches!(
            alias.representation,
            Some(seseragi_syntax::InterfaceType::Record { .. })
        ));
        let main = project.modules.get("fixture/alias::main").unwrap();
        assert!(main.generated.typescript.contains("pair(value)"));
        assert!(!main.generated.typescript.contains("type Pair"));
    }

    #[test]
    fn expands_imported_higher_kinded_aliases_with_standard_and_user_constructors() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/hkt-alias::main".to_owned(),
                [(
                    "./domain".to_owned(),
                    "fixture/hkt-alias::domain".to_owned(),
                )],
            )
            .unwrap();
        graph
            .add_module("fixture/hkt-alias::domain".to_owned(), [])
            .unwrap();

        let project = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/hkt-alias::domain",
                    concat!(
                        "pub type Box<A> = | Boxed A\n",
                        "pub alias StateT<S, M<_>, A> = S -> M<(A, S)>\n",
                        "pub fn keepMaybe value: StateT<Int, Maybe, String> -> StateT<Int, Maybe, String> = value\n",
                        "pub fn keepEither value: StateT<Int, Either<String, _>, Int> -> StateT<Int, Either<String, _>, Int> = value\n",
                        "pub fn keepBox value: StateT<Int, Box, String> -> StateT<Int, Box, String> = value\n",
                    ),
                    "dist/hkt-alias/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/hkt-alias::main",
                    concat!(
                        "import { Box, StateT, keepMaybe, keepEither } from \"./domain\"\n",
                        "pub fn useMaybe value: StateT<Int, Maybe, String> -> StateT<Int, Maybe, String> = keepMaybe value\n",
                        "pub fn useEither value: StateT<Int, Either<String, _>, Int> -> StateT<Int, Either<String, _>, Int> = keepEither value\n",
                        "pub fn useBox value: StateT<Int, Box, String> -> StateT<Int, Box, String> = value\n",
                    ),
                    "dist/hkt-alias/main.js",
                ),
            ],
        )
        .unwrap();

        let domain = project.modules.get("fixture/hkt-alias::domain").unwrap();
        let state = domain
            .typed_interface
            .exports
            .iter()
            .find(|export| export.name == "StateT")
            .expect("StateT export");
        assert_eq!(
            state.scheme.type_parameters[1],
            seseragi_syntax::TypeParameter::constructor("M", 1)
        );
        let main = project.modules.get("fixture/hkt-alias::main").unwrap();
        assert!(main.generated.typescript.contains("keepMaybe(value)"));
        assert!(main.generated.typescript.contains("keepEither(value)"));
        assert!(main.generated.typescript.contains("useBox"));
        assert!(!main.generated.typescript.contains("type StateT"));
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
    fn aggregates_parse_and_semantic_diagnostics_in_stable_project_order() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("fixture/z-semantic".to_owned(), [])
            .unwrap();
        graph.add_module("fixture/a-parse".to_owned(), []).unwrap();
        let inputs = vec![
            ProjectModuleInput::new(
                "z-semantic.ssrg",
                "fixture/z-semantic",
                "pub fn wrong unit: Unit -> Int = \"wrong\"\n",
                "dist/z-semantic.js",
            ),
            ProjectModuleInput::new(
                "a-parse.ssrg",
                "fixture/a-parse",
                "pub let broken: Int =\n",
                "dist/a-parse.js",
            ),
        ];

        let compiled = compile_project(graph.clone(), inputs.clone()).unwrap_err();
        let analyzed = analyze_project(graph, inputs).unwrap_err();
        assert_eq!(compiled, analyzed);

        let ProjectCompileError::Diagnostics { modules } = compiled else {
            panic!("expected aggregated project diagnostics, received {compiled:#?}");
        };
        assert_eq!(
            modules
                .iter()
                .map(|diagnostics| diagnostics.module.as_str())
                .collect::<Vec<_>>(),
            ["fixture/a-parse", "fixture/z-semantic"]
        );
        assert!(modules[0]
            .diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key.starts_with("parser.")));
        assert!(modules[1]
            .diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0101"));
    }

    #[test]
    fn compile_collects_every_shared_semantic_diagnostic_from_analysis() {
        let mut graph = ModuleGraph::new();
        graph.add_module("fixture/z-last".to_owned(), []).unwrap();
        graph.add_module("fixture/a-first".to_owned(), []).unwrap();
        let inputs = vec![
            ProjectModuleInput::new(
                "z-last.ssrg",
                "fixture/z-last",
                "pub fn wrong unit: Unit -> Int = \"last\"\n",
                "dist/z-last.js",
            ),
            ProjectModuleInput::new(
                "a-first.ssrg",
                "fixture/a-first",
                "pub fn wrong unit: Unit -> Bool = 1\n",
                "dist/a-first.js",
            ),
        ];

        let analyzed = analyze_project(graph.clone(), inputs.clone()).unwrap();
        let compiled = compile_project(graph, inputs).unwrap_err();
        let ProjectCompileError::Diagnostics { modules } = compiled else {
            panic!("expected aggregated semantic diagnostics, received {compiled:#?}");
        };

        assert_eq!(
            modules
                .iter()
                .map(|diagnostics| diagnostics.module.as_str())
                .collect::<Vec<_>>(),
            analyzed
                .order
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
        );
        for diagnostics in modules {
            assert_eq!(
                diagnostics.diagnostics,
                analyzed
                    .documents
                    .get(&diagnostics.module)
                    .expect("analysis document for diagnostic module")
                    .diagnostics
            );
        }
    }

    #[test]
    fn rejects_imported_opaque_struct_construction_before_codegen() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/opaque-struct::main".to_owned(),
                [(
                    "./domain".to_owned(),
                    "fixture/opaque-struct::domain".to_owned(),
                )],
            )
            .unwrap();
        graph
            .add_module("fixture/opaque-struct::domain".to_owned(), [])
            .unwrap();

        let error = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "fixture/opaque-struct::domain",
                    "pub opaque struct Secret { value: Int }\n\npub fn secret value: Int -> Secret = Secret { value }\n",
                    "dist/opaque-struct/domain.js",
                ),
                ProjectModuleInput::new(
                    "main.ssrg",
                    "fixture/opaque-struct::main",
                    "import { Secret } from \"./domain\"\n\npub fn forge unit: Unit -> Secret = Secret {}\n",
                    "dist/opaque-struct/main.js",
                ),
            ],
        )
        .unwrap_err();

        let ProjectCompileError::Diagnostics { modules } = error else {
            panic!("expected project diagnostics, received {error:#?}");
        };
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].module, "fixture/opaque-struct::main");
        let diagnostics = &modules[0].diagnostics;
        assert!(diagnostics.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "SES-T0101"
                && diagnostic.message_key == "struct.representation-private"
        }));
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

    #[test]
    fn analyzes_imported_symbols_and_types_across_the_project_graph() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "playground/main".to_owned(),
                [("./domain".to_owned(), "playground/domain".to_owned())],
            )
            .unwrap();
        graph
            .add_module("playground/domain".to_owned(), [])
            .unwrap();
        let main_source = "import { double } from \"./domain\"\n\nlet answer = double 21\n";

        let project = analyze_project(
            graph,
            [
                ProjectModuleInput::new(
                    "domain.ssrg",
                    "playground/domain",
                    "pub fn double value: Int -> Int = value + value\n",
                    "domain.js",
                ),
                ProjectModuleInput::new("main.ssrg", "playground/main", main_source, "main.js"),
            ],
        )
        .unwrap();

        assert_eq!(project.order, ["playground/domain", "playground/main"]);
        let main = project.documents.get("playground/main").unwrap();
        let imported = main_source.rfind("double 21").unwrap();
        assert_eq!(main.symbol_at(imported).unwrap().name, "double");
        assert_eq!(main.type_at(imported).unwrap().type_name, "Int -> Int");
        assert_eq!(
            main.symbol_at(imported).unwrap().module,
            "playground/domain"
        );
    }
}

#[cfg(test)]
#[path = "project_compile/provider_tests.rs"]
mod provider_tests;
