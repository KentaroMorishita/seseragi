//! Runtime module initialization follows source import order (depth first),
//! independently of the topological order used to check declarations.
use crate::CoreModule;
use std::collections::{BTreeMap, BTreeSet};

/// Canonical, once-only postorder for the source modules reachable from entry.
/// Runtime-provided standard modules are outside this source graph. Duplicate,
/// missing or cyclic source identities cannot establish an initialization order.
pub fn core_module_initialization_order(
    modules: &[&CoreModule],
    entry: &str,
) -> Option<Vec<String>> {
    let mut index = BTreeMap::new();
    for module in modules {
        if index.insert(module.module.as_str(), *module).is_some() {
            return None;
        }
    }
    fn visit<'a>(
        name: &'a str,
        index: &BTreeMap<&'a str, &'a CoreModule>,
        active: &mut BTreeSet<&'a str>,
        done: &mut BTreeSet<&'a str>,
        order: &mut Vec<String>,
    ) -> Option<()> {
        if done.contains(name) {
            return Some(());
        }
        if !active.insert(name) {
            return None;
        }
        let module = index.get(name)?;
        for dependency in &module.module_dependencies {
            if !dependency.module.starts_with("std/") {
                visit(&dependency.module, index, active, done, order)?;
            }
        }
        active.remove(name);
        done.insert(name);
        order.push(name.to_owned());
        Some(())
    }
    let root = *index.get_key_value(entry)?.0;
    let mut order = Vec::new();
    visit(
        root,
        &index,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut order,
    )?;
    Some(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreModuleDependency, SourceSpan};
    fn module(name: &str, dependencies: &[&str]) -> CoreModule {
        CoreModule {
            schema: 1,
            stage: "core".into(),
            module: name.into(),
            foreign_modules: vec![],
            external_type_bindings: vec![],
            module_dependencies: dependencies
                .iter()
                .map(|dependency| CoreModuleDependency {
                    specifier: format!("./{dependency}"),
                    module: (*dependency).into(),
                    imports: vec![],
                    origin: SourceSpan {
                        source: "test.ssrg".into(),
                        start: 0,
                        end: 1,
                    },
                })
                .collect(),
            adts: vec![],
            aliases: vec![],
            structs: vec![],
            instances: vec![],
            bindings: vec![],
            functions: vec![],
        }
    }
    #[test]
    fn source_import_postorder_preserves_siblings_and_deduplicates_diamonds() {
        let a = module("a", &["shared"]);
        let z = module("z", &["shared"]);
        let shared = module("shared", &[]);
        let main = module("main", &["z", "a", "z"]);
        assert_eq!(
            core_module_initialization_order(&[&a, &main, &shared, &z], "main").unwrap(),
            ["shared", "z", "a", "main"]
        );
        assert!(core_module_initialization_order(&[&a, &main, &z], "main").is_none());
        assert!(core_module_initialization_order(&[&a, &a, &shared], "a").is_none());
        let cycle = module("shared", &["a"]);
        assert!(core_module_initialization_order(&[&a, &cycle], "a").is_none());
    }
}
