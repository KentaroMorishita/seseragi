use crate::cst::{parse_cst, CstNode};
pub use crate::interface_model::{
    InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceInstance,
    InterfaceMethod, InterfaceOperator, InterfaceRecordField, InterfaceScheme, InterfaceType,
    ModuleInterface,
};
use crate::surface::parse_surface_ast;

mod dependencies;
mod exports;
mod header;
mod instances;
mod methods;
mod types;

use dependencies::dependency_from_surface_import;
use exports::{exports_from_surface_decl, operator_from_surface_decl};
use header::{empty_module_header, module_header_from_surface};
pub use header::{ModuleHeader, ModuleHeaderName};
use instances::instance_from_surface_decl;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportOccurrence {
    pub specifier: String,
    pub origin: crate::ByteSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnlinkedModuleInterface {
    pub header: ModuleHeader,
    pub interface: ModuleInterface,
    pub imports: Vec<crate::SurfaceImport>,
}

pub fn parse_module_interface(source_name: impl Into<String>, source: &str) -> ModuleInterface {
    let source_name = source_name.into();
    let module_name = module_name_from_source_name(&source_name);
    let mut unlinked = parse_unlinked_module_interface(&source_name, module_name, source);
    unlinked.interface.dependencies = unlinked
        .imports
        .into_iter()
        .map(|import| {
            dependency_from_surface_import(&unlinked.interface.module, &source_name, import)
        })
        .collect();
    unlinked.interface
}

/// Produces local public interface data without inventing dependency module or
/// symbol identities. A project resolver links `imports` to canonical module
/// interfaces before semantic analysis.
pub fn parse_unlinked_module_interface(
    source_name: impl Into<String>,
    module_id: impl Into<String>,
    source: &str,
) -> UnlinkedModuleInterface {
    let source_name = source_name.into();
    let module_id = module_id.into();
    parse_unlinked_module_interface_inner(source_name, module_id, source)
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
    let unlinked = parse_unlinked_module_interface(source_name, module_id, source);
    if unlinked.imports.is_empty() {
        return Ok(unlinked.interface);
    }
    Err(unlinked
        .imports
        .into_iter()
        .map(|import| ImportOccurrence {
            specifier: import.specifier,
            origin: import.span,
        })
        .collect())
}

fn parse_unlinked_module_interface_inner(
    source_name: String,
    module_name: String,
    source: &str,
) -> UnlinkedModuleInterface {
    let source_file = source_file_from_source_name(&source_name);
    let cst = parse_cst(source_file.clone(), source);
    if !cst.errors.is_empty() {
        return UnlinkedModuleInterface {
            header: empty_module_header(module_name.clone(), cst.source.clone()),
            interface: empty_interface(module_name, cst.source),
            imports: Vec::new(),
        };
    }

    let surface_module = parse_surface_ast(source_file.clone(), source);
    if cst_type_declaration_count(&cst.root)
        != surface_module
            .declarations
            .iter()
            .filter(|declaration| matches!(declaration, crate::SurfaceDecl::Type { .. }))
            .count()
    {
        return UnlinkedModuleInterface {
            header: empty_module_header(module_name.clone(), cst.source.clone()),
            interface: empty_interface(module_name, cst.source),
            imports: Vec::new(),
        };
    }

    let header = module_header_from_surface(&module_name, &surface_module);
    let imports = surface_module.imports;
    let interface = ModuleInterface {
        schema: 1,
        module: module_name.clone(),
        source: surface_module.source.clone(),
        dependencies: Vec::new(),
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
    UnlinkedModuleInterface {
        header,
        interface,
        imports,
    }
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
mod header_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod type_ref_tests;
