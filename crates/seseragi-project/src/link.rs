use seseragi_syntax::{
    InterfaceDependency, InterfaceExport, InterfaceImport, ModuleInterface, SurfaceImportItem,
    UnlinkedModuleInterface,
};
use std::collections::{BTreeMap, BTreeSet};

mod model;

pub use model::{LinkError, LinkedDependency, LinkedImport, LinkedModule};

pub fn link_module(
    mut unlinked: UnlinkedModuleInterface,
    targets: &BTreeMap<String, ModuleInterface>,
) -> Result<LinkedModule, Vec<LinkError>> {
    let mut dependencies = Vec::with_capacity(unlinked.imports.len());
    let mut interface_dependencies = Vec::with_capacity(unlinked.imports.len());
    let mut names = BTreeSet::new();
    let mut errors = Vec::new();

    for import in unlinked.imports {
        let Some(target) = targets.get(&import.specifier) else {
            errors.push(LinkError::UnresolvedSpecifier {
                specifier: import.specifier,
                origin: import.span,
            });
            continue;
        };
        let mut linked_imports = Vec::new();
        let mut interface_imports = Vec::new();
        for item in &import.items {
            link_item(
                target,
                item,
                &mut names,
                &mut linked_imports,
                &mut interface_imports,
                &mut errors,
            );
        }
        interface_dependencies.push(InterfaceDependency {
            specifier: import.specifier.clone(),
            module: target.module.clone(),
            origin: import.span,
            imports: interface_imports,
        });
        dependencies.push(LinkedDependency {
            specifier: import.specifier,
            origin: import.span,
            interface: target.clone(),
            imports: linked_imports,
        });
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    unlinked.interface.dependencies = interface_dependencies;
    Ok(LinkedModule {
        interface: unlinked.interface,
        dependencies,
    })
}

fn link_item(
    target: &ModuleInterface,
    item: &SurfaceImportItem,
    names: &mut BTreeSet<(String, String)>,
    linked: &mut Vec<LinkedImport>,
    interface: &mut Vec<InterfaceImport>,
    errors: &mut Vec<LinkError>,
) {
    match item.namespace.as_str() {
        "namespace" => link_namespace(target, item, names, linked, interface, errors),
        "operator" => link_operator(target, item, names, linked, interface, errors),
        "value" => link_named(target, item, names, linked, interface, errors),
        other => errors.push(LinkError::UnsupportedImportNamespace {
            namespace: other.to_owned(),
            origin: item.name_span,
        }),
    }
}

fn link_namespace(
    target: &ModuleInterface,
    item: &SurfaceImportItem,
    names: &mut BTreeSet<(String, String)>,
    linked: &mut Vec<LinkedImport>,
    interface: &mut Vec<InterfaceImport>,
    errors: &mut Vec<LinkError>,
) {
    let Some(local_name) = item.alias.clone() else {
        errors.push(LinkError::MissingNamespaceAlias {
            origin: item.name_span,
        });
        return;
    };
    let origin = item.alias_span.unwrap_or(item.name_span);
    if !register_name("module", &local_name, origin, names, errors) {
        return;
    }
    linked.push(LinkedImport::Namespace {
        local_name: local_name.clone(),
        origin,
        module: target.module.clone(),
    });
    interface.push(InterfaceImport {
        namespace: "namespace".to_owned(),
        name: "*".to_owned(),
        symbol: format!("{}::*", target.module),
        local_name: Some(local_name),
    });
}

fn link_operator(
    target: &ModuleInterface,
    item: &SurfaceImportItem,
    names: &mut BTreeSet<(String, String)>,
    linked: &mut Vec<LinkedImport>,
    interface: &mut Vec<InterfaceImport>,
    errors: &mut Vec<LinkError>,
) {
    let export = target
        .exports
        .iter()
        .find(|export| export.namespace == "operator" && export.name == item.name);
    let operator = target
        .operators
        .iter()
        .find(|operator| operator.spelling == item.name);
    let Some((export, operator)) = export.zip(operator) else {
        errors.push(missing_export(target, item));
        return;
    };
    if !register_name("operator", &item.name, item.name_span, names, errors) {
        return;
    }
    linked.push(LinkedImport::Operator {
        spelling: item.name.clone(),
        origin: item.name_span,
        export: export.clone(),
        operator: operator.clone(),
    });
    interface.push(interface_import(export, &item.name, None));
}

fn link_named(
    target: &ModuleInterface,
    item: &SurfaceImportItem,
    names: &mut BTreeSet<(String, String)>,
    linked: &mut Vec<LinkedImport>,
    interface: &mut Vec<InterfaceImport>,
    errors: &mut Vec<LinkError>,
) {
    let exports = target
        .exports
        .iter()
        .filter(|export| export.name == item.name && export.namespace != "operator")
        .collect::<Vec<_>>();
    if exports.is_empty() {
        errors.push(missing_export(target, item));
        return;
    }
    let local_name = item.alias.as_deref().unwrap_or(&item.name);
    let origin = item.alias_span.unwrap_or(item.name_span);
    for export in exports {
        if !register_name(&export.namespace, local_name, origin, names, errors) {
            continue;
        }
        linked.push(LinkedImport::Symbol {
            local_name: local_name.to_owned(),
            origin,
            export: export.clone(),
        });
        interface.push(interface_import(
            export,
            &item.name,
            (local_name != item.name).then(|| local_name.to_owned()),
        ));
    }
}

fn interface_import(
    export: &InterfaceExport,
    imported_name: &str,
    local_name: Option<String>,
) -> InterfaceImport {
    InterfaceImport {
        namespace: export.namespace.clone(),
        name: imported_name.to_owned(),
        symbol: export.symbol.clone(),
        local_name,
    }
}

fn register_name(
    namespace: &str,
    local_name: &str,
    origin: seseragi_syntax::ByteSpan,
    names: &mut BTreeSet<(String, String)>,
    errors: &mut Vec<LinkError>,
) -> bool {
    if names.insert((namespace.to_owned(), local_name.to_owned())) {
        return true;
    }
    errors.push(LinkError::DuplicateImport {
        namespace: namespace.to_owned(),
        local_name: local_name.to_owned(),
        origin,
    });
    false
}

fn missing_export(target: &ModuleInterface, item: &SurfaceImportItem) -> LinkError {
    LinkError::MissingExport {
        module: target.module.clone(),
        name: item.name.clone(),
        origin: item.name_span,
    }
}

#[cfg(test)]
mod tests;
