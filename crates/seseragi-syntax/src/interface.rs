use crate::cst::parse_cst;
pub use crate::interface_model::{
    InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceInstance,
    InterfaceOperator, InterfaceRecordField, InterfaceScheme, InterfaceType, ModuleInterface,
};
use crate::surface::parse_surface_ast;

mod dependencies;
mod exports;
mod instances;
mod types;

use dependencies::dependency_from_surface_import;
use exports::{export_from_surface_decl, operator_from_surface_decl};
use instances::instance_from_surface_decl;

pub fn parse_module_interface(source_name: impl Into<String>, source: &str) -> ModuleInterface {
    let source_name = source_name.into();
    let module_name = module_name_from_source_name(&source_name);
    let source_file = source_file_from_source_name(&source_name);
    let cst = parse_cst(source_file.clone(), source);
    if !cst.errors.is_empty() {
        return ModuleInterface {
            schema: 1,
            module: module_name,
            source: cst.source,
            dependencies: Vec::new(),
            exports: Vec::new(),
            operators: Vec::new(),
            instances: Vec::new(),
        };
    }

    let surface_module = parse_surface_ast(source_file.clone(), source);

    let interface = ModuleInterface {
        schema: 1,
        module: module_name.clone(),
        source: surface_module.source.clone(),
        dependencies: surface_module
            .imports
            .into_iter()
            .map(|import| dependency_from_surface_import(&module_name, &source_name, import))
            .collect(),
        exports: surface_module
            .declarations
            .iter()
            .filter_map(|declaration| export_from_surface_decl(&module_name, declaration))
            .collect(),
        operators: surface_module
            .declarations
            .iter()
            .filter_map(|declaration| operator_from_surface_decl(&module_name, declaration))
            .collect(),
        instances: surface_module
            .declarations
            .into_iter()
            .filter_map(instance_from_surface_decl)
            .collect(),
    };
    interface
}

fn module_name_from_source_name(source_name: &str) -> String {
    let normalized = source_name.replace('\\', "/");
    let source_file = source_file_from_source_name(&normalized);
    let parent = normalized.rsplit_once('/').map(|(parent, _)| parent);

    match parent {
        Some(parent) if !parent.is_empty() && parent != "." => parent.to_owned(),
        _ => source_file
            .rsplit_once('.')
            .map(|(stem, _)| stem.to_owned())
            .unwrap_or(source_file),
    }
}

fn source_file_from_source_name(source_name: &str) -> String {
    source_name
        .replace('\\', "/")
        .rsplit_once('/')
        .map(|(_, file)| file.to_owned())
        .unwrap_or_else(|| source_name.to_owned())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod type_ref_tests;
