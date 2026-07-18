use super::{ProjectCompileError, ProjectModuleInput};
use seseragi_project::is_standard_module;
use seseragi_syntax::UnlinkedModuleInterface;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn index_project_inputs(
    order: &[String],
    input_iter: impl IntoIterator<Item = ProjectModuleInput>,
) -> Result<BTreeMap<String, ProjectModuleInput>, ProjectCompileError> {
    let graph_modules = order.iter().cloned().collect::<BTreeSet<_>>();
    let mut inputs = BTreeMap::new();
    let mut output_owners = BTreeMap::new();

    for input in input_iter {
        let module = input.module_id.clone();
        if inputs.contains_key(&module) {
            return Err(ProjectCompileError::DuplicateInput { module });
        }
        if !graph_modules.contains(&module) {
            return Err(ProjectCompileError::UnexpectedInput { module });
        }
        if let Some(first_module) = output_owners.insert(input.output_path.clone(), module.clone())
        {
            return Err(ProjectCompileError::DuplicateOutputPath {
                path: input.output_path,
                first_module,
                second_module: module,
            });
        }
        inputs.insert(module, input);
    }

    for module in order {
        if !inputs.contains_key(module) {
            return Err(ProjectCompileError::MissingInput {
                module: module.clone(),
            });
        }
    }
    Ok(inputs)
}

pub(super) fn ensure_graph_imports_match(
    module: &str,
    graph_dependencies: Vec<(String, String)>,
    unlinked: &UnlinkedModuleInterface,
) -> Result<(), ProjectCompileError> {
    let graph_specifiers = graph_dependencies
        .into_iter()
        .map(|(specifier, _)| specifier)
        .collect::<BTreeSet<_>>();
    let source_specifiers = unlinked
        .imports
        .iter()
        .filter(|import| !is_standard_module(&import.specifier))
        .map(|import| import.specifier.clone())
        .collect::<BTreeSet<_>>();
    if graph_specifiers == source_specifiers {
        return Ok(());
    }
    Err(ProjectCompileError::GraphImportMismatch {
        module: module.to_owned(),
        graph_specifiers: graph_specifiers.into_iter().collect(),
        source_specifiers: source_specifiers.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use seseragi_syntax::parse_unlinked_module_interface;

    #[test]
    fn rejects_extra_project_input_and_global_output_collisions() {
        let order = ["fixture/main".to_owned(), "fixture/domain".to_owned()];
        let duplicate_paths = [
            ProjectModuleInput::new("main.ssrg", "fixture/main", "", "dist/main.js"),
            ProjectModuleInput::new("domain.ssrg", "fixture/domain", "", "dist/main.js"),
        ];
        assert!(matches!(
            index_project_inputs(&order, duplicate_paths),
            Err(ProjectCompileError::DuplicateOutputPath { .. })
        ));
        assert!(matches!(
            index_project_inputs(
                &order,
                [ProjectModuleInput::new(
                    "extra.ssrg",
                    "fixture/extra",
                    "",
                    "dist/extra.js"
                )]
            ),
            Err(ProjectCompileError::UnexpectedInput { .. })
        ));
    }

    #[test]
    fn rejects_graph_edges_that_are_not_present_in_the_source() {
        let unlinked = parse_unlinked_module_interface(
            "main.ssrg",
            "fixture/main",
            "pub let answer: Int = 42\n",
        );
        assert_eq!(
            ensure_graph_imports_match(
                "fixture/main",
                vec![("./domain".to_owned(), "fixture/domain".to_owned())],
                &unlinked,
            ),
            Err(ProjectCompileError::GraphImportMismatch {
                module: "fixture/main".to_owned(),
                graph_specifiers: vec!["./domain".to_owned()],
                source_specifiers: Vec::new(),
            })
        );
    }
}
