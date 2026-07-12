use std::collections::{BTreeMap, BTreeSet};

/// A deterministic module dependency graph.
///
/// The graph stores logical module identities and does not read source files or
/// choose generated output paths. Those concerns stay with the project loader
/// and output planner respectively.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleGraph<T> {
    dependencies: BTreeMap<T, BTreeSet<T>>,
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
        dependencies: impl IntoIterator<Item = T>,
    ) -> Result<(), ModuleGraphError<T>> {
        if self.dependencies.contains_key(&module) {
            return Err(ModuleGraphError::DuplicateModule { module });
        }
        self.dependencies
            .insert(module, dependencies.into_iter().collect());
        Ok(())
    }

    /// Returns dependencies before their importers, with deterministic order
    /// for otherwise independent modules.
    pub fn topological_order(&self) -> Result<Vec<T>, ModuleGraphError<T>> {
        for (module, dependencies) in &self.dependencies {
            if let Some(dependency) = dependencies
                .iter()
                .find(|dependency| !self.dependencies.contains_key(*dependency))
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
                if dependencies.remove(&module) && dependencies.is_empty() {
                    ready.insert(candidate.clone());
                }
            }
        }

        if order.len() != pending.len() {
            let modules = pending
                .into_iter()
                .filter_map(|(module, dependencies)| (!dependencies.is_empty()).then_some(module))
                .collect();
            return Err(ModuleGraphError::Cycle { modules });
        }
        Ok(order)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleGraphError<T> {
    DuplicateModule { module: T },
    MissingModule { module: T, dependency: T },
    Cycle { modules: Vec<T> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_dependencies_before_importers_deterministically() {
        let mut graph = ModuleGraph::new();
        graph.add_module("main", ["domain", "input"]).unwrap();
        graph.add_module("input", ["shared"]).unwrap();
        graph.add_module("domain", ["shared"]).unwrap();
        graph.add_module("shared", []).unwrap();

        assert_eq!(
            graph.topological_order().unwrap(),
            ["shared", "domain", "input", "main"]
        );
    }

    #[test]
    fn reports_missing_dependency_without_synthesizing_a_node() {
        let mut graph = ModuleGraph::new();
        graph.add_module("main", ["missing"]).unwrap();

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
        graph.add_module("main", ["domain"]).unwrap();
        graph.add_module("domain", ["main"]).unwrap();

        assert_eq!(
            graph.topological_order().unwrap_err(),
            ModuleGraphError::Cycle {
                modules: vec!["domain", "main"]
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
}
