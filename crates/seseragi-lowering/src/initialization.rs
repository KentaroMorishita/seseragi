//! Runtime module initialization follows source import order (depth first),
//! independently of the topological order used to check declarations.
use crate::CoreModule;
use std::collections::{BTreeMap, BTreeSet};

/// Whether importing this checked module must retain a runtime initialization
/// edge. Summaries are accumulated in checking order. Missing source summaries
/// stay conservative; task-only foreign wrappers do not eagerly load a host.
pub fn core_module_requires_initialization(
    module: &CoreModule,
    dependencies: &BTreeMap<String, bool>,
) -> bool {
    !module.bindings.is_empty()
        || module
            .foreign_modules
            .iter()
            .any(|foreign| foreign.pure_load)
        || module.module_dependencies.iter().any(|dependency| {
            !dependency.module.starts_with("std/")
                && dependencies
                    .get(&dependency.module)
                    .copied()
                    .unwrap_or(true)
        })
}

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
            trait_methods: vec![],
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

    #[test]
    fn initialization_summaries_preserve_values_and_pure_load_but_not_task_load() {
        let mut inert = module("inert", &[]);
        assert!(!core_module_requires_initialization(
            &inert,
            &BTreeMap::new()
        ));
        inert.foreign_modules.push(crate::CoreForeignModule {
            visibility: seseragi_syntax::Visibility::Public,
            language: "typescript".into(),
            specifier: "fixture-api".into(),
            pure_load: false,
            members: vec![],
            origin: SourceSpan {
                source: "inert.ssrg".into(),
                start: 0,
                end: 1,
            },
        });
        assert!(!core_module_requires_initialization(
            &inert,
            &BTreeMap::new()
        ));
        inert.foreign_modules[0].pure_load = true;
        assert!(core_module_requires_initialization(
            &inert,
            &BTreeMap::new()
        ));
        let values = crate::lower_typed_module(seseragi_semantics::type_module(
            "values",
            "let unused: Int = 9007199254740991 + 1\n",
        ));
        assert!(core_module_requires_initialization(
            &values,
            &BTreeMap::new()
        ));
        let facade = module("facade", &["inert", "values", "std/int"]);
        let mut summaries = BTreeMap::from([("inert".into(), false), ("values".into(), false)]);
        assert!(!core_module_requires_initialization(&facade, &summaries));
        summaries.insert("values".into(), true);
        assert!(core_module_requires_initialization(&facade, &summaries));
        summaries.remove("values");
        assert!(core_module_requires_initialization(&facade, &summaries));
    }
}
