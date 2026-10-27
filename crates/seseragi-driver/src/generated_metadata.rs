use seseragi_lowering::{GeneratedBundle, GeneratedDeprecation};
use seseragi_semantics::TypedModuleInterface;
use std::collections::BTreeMap;

/// Attach checked public metadata at the orchestration boundary. Deprecation
/// never participates in Core IR, runtime ABI, evidence or generated code.
pub(crate) fn retain_deprecations(
    interface: &TypedModuleInterface,
    generated: &mut GeneratedBundle,
) {
    let mut entries = BTreeMap::new();
    for namespace in &interface.namespace_deprecations {
        let entry = GeneratedDeprecation {
            identity: namespace.symbol.clone(),
            namespace: "namespace".to_owned(),
            name: namespace.name.clone(),
            owner: None,
            deprecation: namespace.deprecation.clone(),
        };
        entries.insert(
            (
                entry.namespace.clone(),
                entry.name.clone(),
                entry.identity.clone(),
            ),
            entry,
        );
    }
    for export in &interface.exports {
        if let Some(deprecation) = &export.deprecation {
            // Constructor exports never inherit their owner's notice.
            if export.constructor_of.is_none() {
                let entry = GeneratedDeprecation {
                    identity: export.symbol.clone(),
                    namespace: export.namespace.clone(),
                    name: export.name.clone(),
                    owner: None,
                    deprecation: deprecation.clone(),
                };
                entries.insert(
                    (
                        entry.namespace.clone(),
                        entry.name.clone(),
                        entry.identity.clone(),
                    ),
                    entry,
                );
            }
        }
        for method in &export.methods {
            if let Some(deprecation) = &method.deprecation {
                let entry = GeneratedDeprecation {
                    identity: format!("{}::{}", export.symbol, method.name),
                    namespace: "method".to_owned(),
                    name: method.name.clone(),
                    owner: Some(export.symbol.clone()),
                    deprecation: deprecation.clone(),
                };
                entries.insert(
                    (
                        entry.namespace.clone(),
                        entry.name.clone(),
                        entry.identity.clone(),
                    ),
                    entry,
                );
            }
        }
    }
    let entries = entries.into_values().collect::<Vec<_>>();
    generated.metadata.deprecations.clone_from(&entries);
    generated.source_map.deprecations = entries;
}
