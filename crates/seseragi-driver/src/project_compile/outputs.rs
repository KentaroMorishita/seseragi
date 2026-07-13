use super::ProjectModuleInput;
use crate::{CompiledModule, TypeScriptInstanceOutput, TypeScriptModuleOutput};
use seseragi_project::ModuleGraph;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn reachable_compiled_module_outputs(
    module: &str,
    graph: &ModuleGraph<String>,
    inputs: &BTreeMap<String, ProjectModuleInput>,
    compiled: &BTreeMap<String, CompiledModule>,
) -> Vec<TypeScriptModuleOutput> {
    reachable_modules(module, graph)
        .into_iter()
        .map(|provider| {
            let input = inputs
                .get(&provider)
                .expect("reachable project provider must retain its validated input");
            let compiled_module = compiled
                .get(&provider)
                .expect("topological order must compile reachable providers first");
            TypeScriptModuleOutput::new(provider, input.output_path.clone()).with_instance_exports(
                compiled_module
                    .generated
                    .metadata
                    .instances
                    .iter()
                    .map(|instance| {
                        TypeScriptInstanceOutput::new(
                            instance.identity.clone(),
                            instance.dictionary_export.clone(),
                        )
                    }),
            )
        })
        .collect()
}

fn reachable_modules(module: &str, graph: &ModuleGraph<String>) -> BTreeSet<String> {
    let mut reachable = BTreeSet::new();
    let module = module.to_owned();
    let mut pending = graph
        .dependencies_for(&module)
        .expect("compiled module must be present in the validated graph")
        .into_iter()
        .map(|(_, dependency)| dependency)
        .collect::<Vec<_>>();
    while let Some(provider) = pending.pop() {
        if !reachable.insert(provider.clone()) {
            continue;
        }
        pending.extend(
            graph
                .dependencies_for(&provider)
                .expect("reachable provider must be present in the validated graph")
                .into_iter()
                .map(|(_, dependency)| dependency),
        );
    }
    reachable
}

#[cfg(test)]
mod tests {
    use super::reachable_modules;
    use seseragi_project::ModuleGraph;
    use std::collections::BTreeSet;

    #[test]
    fn selects_only_transitively_reachable_providers() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "main".to_owned(),
                [("./facade".to_owned(), "facade".to_owned())],
            )
            .unwrap();
        graph
            .add_module(
                "facade".to_owned(),
                [("./provider".to_owned(), "provider".to_owned())],
            )
            .unwrap();
        graph.add_module("provider".to_owned(), []).unwrap();
        graph.add_module("unrelated".to_owned(), []).unwrap();

        assert_eq!(
            reachable_modules("main", &graph),
            BTreeSet::from(["facade".to_owned(), "provider".to_owned()])
        );
    }
}
