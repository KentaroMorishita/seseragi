use crate::surface::{SurfaceImport, SurfaceImportItem};

use super::{InterfaceDependency, InterfaceImport};

pub(super) fn dependency_from_surface_import(
    module_name: &str,
    source_name: &str,
    import: SurfaceImport,
) -> InterfaceDependency {
    let module = dependency_module_name(module_name, source_name, &import.specifier);
    InterfaceDependency {
        specifier: import.specifier,
        module: module.clone(),
        origin: import.span,
        imports: import
            .items
            .into_iter()
            .map(|item| interface_import_from_surface_item(&module, item))
            .collect(),
    }
}

fn interface_import_from_surface_item(module: &str, item: SurfaceImportItem) -> InterfaceImport {
    InterfaceImport {
        symbol: match item.namespace.as_str() {
            "operator" => format!("{module}::operator({})", item.name),
            "namespace" => format!("{module}::*"),
            _ => format!("{module}::{}", item.name),
        },
        namespace: item.namespace,
        name: item.name,
        local_name: item.alias,
    }
}

fn dependency_module_name(module_name: &str, source_name: &str, specifier: &str) -> String {
    if let Some(relative) = specifier.strip_prefix("./") {
        return format!("{module_name}/{relative}");
    }
    source_name
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/{specifier}"))
        .unwrap_or_else(|| specifier.to_owned())
}
