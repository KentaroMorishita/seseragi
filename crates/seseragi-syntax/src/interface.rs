use crate::cst::{parse_cst, CstNode};
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
use exports::{exports_from_surface_decl, operator_from_surface_decl};
use instances::instance_from_surface_decl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportOccurrence {
    pub specifier: String,
    pub origin: crate::ByteSpan,
}

pub fn parse_module_interface(source_name: impl Into<String>, source: &str) -> ModuleInterface {
    let source_name = source_name.into();
    let module_name = module_name_from_source_name(&source_name);
    parse_module_interface_inner(source_name, module_name, false, source)
        .expect("the compatibility interface parser accepts unresolved imports")
}

/// Parses an import-free module with a caller-provided logical identity.
///
/// `source_name` remains a physical diagnostic/source label and does not
/// participate in symbol identity. Imported module identities must come from
/// a project resolver, so this entrypoint reports import occurrences instead
/// of manufacturing canonical dependency symbols from source spellings.
pub fn parse_import_free_module_interface(
    source_name: impl Into<String>,
    module_id: impl Into<String>,
    source: &str,
) -> Result<ModuleInterface, Vec<ImportOccurrence>> {
    let source_name = source_name.into();
    let module_id = module_id.into();
    parse_module_interface_inner(source_name, module_id, true, source)
}

fn parse_module_interface_inner(
    source_name: String,
    module_name: String,
    reject_imports: bool,
    source: &str,
) -> Result<ModuleInterface, Vec<ImportOccurrence>> {
    let source_file = source_file_from_source_name(&source_name);
    let cst = parse_cst(source_file.clone(), source);
    if !cst.errors.is_empty() {
        return Ok(empty_interface(module_name, cst.source));
    }

    let surface_module = parse_surface_ast(source_file.clone(), source);
    if cst_type_declaration_count(&cst.root)
        != surface_module
            .declarations
            .iter()
            .filter(|declaration| matches!(declaration, crate::SurfaceDecl::Type { .. }))
            .count()
    {
        return Ok(empty_interface(module_name, cst.source));
    }

    if reject_imports && !surface_module.imports.is_empty() {
        return Err(surface_module
            .imports
            .iter()
            .map(|import| ImportOccurrence {
                specifier: import.specifier.clone(),
                origin: import.span,
            })
            .collect());
    }

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
            .flat_map(|declaration| exports_from_surface_decl(&module_name, declaration))
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
    Ok(interface)
}

fn empty_interface(module: String, source: String) -> ModuleInterface {
    ModuleInterface {
        schema: 1,
        module,
        source,
        dependencies: Vec::new(),
        exports: Vec::new(),
        operators: Vec::new(),
        instances: Vec::new(),
    }
}

fn cst_type_declaration_count(root: &CstNode) -> usize {
    root.children
        .iter()
        .filter(|top| top.children.iter().any(|child| child.kind == "type-decl"))
        .count()
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
