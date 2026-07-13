use std::collections::{BTreeMap, BTreeSet};

/// A deterministic module dependency graph.
///
/// The graph stores logical module identities and does not read source files or
/// choose generated output paths. Those concerns stay with the project loader
/// and output planner respectively.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleGraph<T> {
    dependencies: BTreeMap<T, BTreeMap<String, T>>,
}

impl<T> Default for ModuleGraph<T> {
    fn default() -> Self {
        Self {
            dependencies: BTreeMap::new(),
        }
    }
}

impl<T> ModuleGraph<T>
where
    T: Clone + Ord,
{
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds one module and the logical modules it imports.
    pub fn add_module(
        &mut self,
        module: T,
        dependencies: impl IntoIterator<Item = (String, T)>,
    ) -> Result<(), ModuleGraphError<T>> {
        if self.dependencies.contains_key(&module) {
            return Err(ModuleGraphError::DuplicateModule { module });
        }
        let mut edges = BTreeMap::new();
        for (specifier, dependency) in dependencies {
            if edges.insert(specifier.clone(), dependency).is_some() {
                return Err(ModuleGraphError::DuplicateSpecifier { module, specifier });
            }
        }
        self.dependencies.insert(module, edges);
        Ok(())
    }

    pub fn dependencies_for(&self, module: &T) -> Option<Vec<(String, T)>> {
        self.dependencies.get(module).map(|dependencies| {
            dependencies
                .iter()
                .map(|(specifier, dependency)| (specifier.clone(), dependency.clone()))
                .collect()
        })
    }

    pub fn dependency_for(&self, module: &T, specifier: &str) -> Option<&T> {
        self.dependencies.get(module)?.get(specifier)
    }

    /// Returns dependencies before their importers, with deterministic order
    /// for otherwise independent modules.
    pub fn topological_order(&self) -> Result<Vec<T>, ModuleGraphError<T>> {
        for (module, dependencies) in &self.dependencies {
            if let Some((_, dependency)) = dependencies
                .iter()
                .find(|(_, dependency)| !self.dependencies.contains_key(*dependency))
            {
                return Err(ModuleGraphError::MissingModule {
                    module: module.clone(),
                    dependency: dependency.clone(),
                });
            }
        }

        let mut pending = self.dependencies.clone();
        let mut ready = pending
            .iter()
            .filter(|(_, dependencies)| dependencies.is_empty())
            .map(|(module, _)| module.clone())
            .collect::<BTreeSet<_>>();
        let mut order = Vec::with_capacity(pending.len());

        while let Some(module) = ready.pop_first() {
            order.push(module.clone());
            for (candidate, dependencies) in &mut pending {
                let before = dependencies.len();
                dependencies.retain(|_, dependency| dependency != &module);
                let removed = dependencies.len() != before;
                if removed && dependencies.is_empty() {
                    ready.insert(candidate.clone());
                }
            }
        }

        if order.len() != pending.len() {
            let modules = cycle_witness(&pending)
                .expect("a closed graph with no complete topological ordering has a cycle");
            return Err(ModuleGraphError::Cycle { modules });
        }
        Ok(order)
    }
}

/// Finds one deterministic cycle in the dependency graph.
///
/// `topological_order` calls this only after Kahn's algorithm has left at
/// least one edge behind. That remainder can include modules which merely
/// depend on a cycle, so reporting all remaining modules would make the
/// diagnostic misleading. The DFS stack gives us just the back-edge witness.
fn cycle_witness<T>(dependencies: &BTreeMap<T, BTreeMap<String, T>>) -> Option<Vec<T>>
where
    T: Clone + Ord,
{
    fn visit<T>(
        module: &T,
        dependencies: &BTreeMap<T, BTreeMap<String, T>>,
        completed: &mut BTreeSet<T>,
        stack: &mut Vec<T>,
        stack_positions: &mut BTreeMap<T, usize>,
    ) -> Option<Vec<T>>
    where
        T: Clone + Ord,
    {
        if let Some(start) = stack_positions.get(module) {
            return Some(stack[*start..].to_vec());
        }
        if completed.contains(module) {
            return None;
        }

        stack_positions.insert(module.clone(), stack.len());
        stack.push(module.clone());
        for dependency in dependencies.get(module)?.values() {
            if let Some(witness) =
                visit(dependency, dependencies, completed, stack, stack_positions)
            {
                return Some(witness);
            }
        }
        stack_positions.remove(module);
        stack.pop();
        completed.insert(module.clone());
        None
    }

    let mut completed = BTreeSet::new();
    let mut stack = Vec::new();
    let mut stack_positions = BTreeMap::new();
    for module in dependencies.keys() {
        if let Some(witness) = visit(
            module,
            dependencies,
            &mut completed,
            &mut stack,
            &mut stack_positions,
        ) {
            return Some(witness);
        }
    }
    None
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleGraphError<T> {
    DuplicateModule { module: T },
    DuplicateSpecifier { module: T, specifier: String },
    MissingModule { module: T, dependency: T },
    Cycle { modules: Vec<T> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_dependencies_before_importers_deterministically() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "main",
                [
                    ("./domain".to_owned(), "domain"),
                    ("./input".to_owned(), "input"),
                ],
            )
            .unwrap();
        graph
            .add_module("input", [("./shared".to_owned(), "shared")])
            .unwrap();
        graph
            .add_module("domain", [("./shared".to_owned(), "shared")])
            .unwrap();
        graph.add_module("shared", []).unwrap();

        assert_eq!(
            graph.topological_order().unwrap(),
            ["shared", "domain", "input", "main"]
        );
    }

    #[test]
    fn reports_missing_dependency_without_synthesizing_a_node() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("main", [("./missing".to_owned(), "missing")])
            .unwrap();

        assert_eq!(
            graph.topological_order().unwrap_err(),
            ModuleGraphError::MissingModule {
                module: "main",
                dependency: "missing"
            }
        );
    }

    #[test]
    fn reports_all_nodes_remaining_in_a_cycle() {
        let mut graph = ModuleGraph::new();
        graph
            .add_module("main", [("./domain".to_owned(), "domain")])
            .unwrap();
        graph
            .add_module("domain", [("./main".to_owned(), "main")])
            .unwrap();

        assert_eq!(
            graph.topological_order().unwrap_err(),
            ModuleGraphError::Cycle {
                modules: vec!["domain", "main"]
            }
        );
    }

    #[test]
    fn reports_a_cycle_witness_without_downstream_blocked_modules() {
        let mut graph = ModuleGraph::new();
        graph.add_module("C", [("./a".to_owned(), "A")]).unwrap();
        graph.add_module("A", [("./b".to_owned(), "B")]).unwrap();
        graph.add_module("B", [("./a".to_owned(), "A")]).unwrap();

        assert_eq!(
            graph.topological_order().unwrap_err(),
            ModuleGraphError::Cycle {
                modules: vec!["A", "B"]
            }
        );
    }

    #[test]
    fn rejects_duplicate_module_registration() {
        let mut graph = ModuleGraph::new();
        graph.add_module("main", []).unwrap();

        assert_eq!(
            graph.add_module("main", []),
            Err(ModuleGraphError::DuplicateModule { module: "main" })
        );
    }

    #[test]
    fn rejects_duplicate_dependency_specifiers() {
        let mut graph = ModuleGraph::new();
        assert_eq!(
            graph.add_module(
                "main",
                [
                    ("./domain".to_owned(), "first"),
                    ("./domain".to_owned(), "second"),
                ],
            ),
            Err(ModuleGraphError::DuplicateSpecifier {
                module: "main",
                specifier: "./domain".to_owned()
            })
        );
    }
}
