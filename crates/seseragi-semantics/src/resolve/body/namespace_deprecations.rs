use crate::ResolvedNamespaceDeprecation;
use seseragi_project::{LinkedDependency, LinkedImport};
use seseragi_syntax::ModuleInterface;

pub(super) fn linked(
    dependencies: &[LinkedDependency],
    interface: &ModuleInterface,
) -> Vec<ResolvedNamespaceDeprecation> {
    let mut result = Vec::new();
    for dependency in dependencies {
        for import in &dependency.imports {
            let LinkedImport::Namespace { local_name, .. } = import else {
                continue;
            };
            for metadata in &dependency.interface.namespace_deprecations {
                let path = format!("{local_name}.{}", metadata.name);
                let reexport_origin = interface.namespace_deprecations.iter().find_map(|entry| {
                    (entry.symbol == metadata.symbol && entry.name == path)
                        .then_some(entry.publication)
                        .flatten()
                });
                result.push(ResolvedNamespaceDeprecation {
                    metadata: metadata.clone(),
                    path,
                    own_declaration: None,
                    reexport_origin,
                });
            }
        }
    }
    result
}
