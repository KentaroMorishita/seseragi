use crate::cst::parse_cst;
pub use crate::interface_model::{
    InterfaceConstraint, InterfaceDependency, InterfaceExport, InterfaceImport, InterfaceInstance,
    InterfaceOperator, InterfaceScheme, InterfaceType, ModuleInterface,
};
use crate::surface::{parse_surface_ast, SurfaceDecl, SurfaceImport, TypeRef, Visibility};

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

fn export_from_surface_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Option<InterfaceExport> {
    match declaration {
        SurfaceDecl::Let {
            visibility,
            name,
            type_ref,
            span,
            ..
        }
        | SurfaceDecl::EffectFn {
            visibility,
            name,
            return_type: type_ref,
            span,
            ..
        } if *visibility == Visibility::Public => {
            let type_ref = type_ref.as_ref().map(interface_type_from_type_ref)?;
            Some(InterfaceExport {
                symbol: format!("{module_name}::{name}"),
                namespace: "value".to_owned(),
                name: name.clone(),
                visibility: *visibility,
                declaration_kind: None,
                declaration: *span,
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref,
                },
                representation: None,
            })
        }
        SurfaceDecl::Newtype {
            visibility,
            name,
            representation,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::{name}"),
            namespace: "type".to_owned(),
            name: name.clone(),
            visibility: *visibility,
            declaration_kind: Some("newtype".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: Vec::new(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.clone(),
                    arity: 0,
                },
            },
            representation: Some(interface_type_from_type_ref(representation)),
        }),
        SurfaceDecl::Operator {
            visibility,
            type_parameters,
            spelling,
            parameters,
            return_type,
            constraints,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::operator({spelling})"),
            namespace: "operator".to_owned(),
            name: spelling.clone(),
            visibility: *visibility,
            declaration_kind: Some("custom-operator".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: constraints
                    .iter()
                    .map(|name| InterfaceConstraint { name: name.clone() })
                    .collect(),
                type_ref: operator_interface_type(parameters, return_type),
            },
            representation: None,
        }),
        _ => None,
    }
}

fn dependency_from_surface_import(
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
            .map(|item| InterfaceImport {
                symbol: match item.namespace.as_str() {
                    "operator" => format!("{module}::operator({})", item.name),
                    "namespace" => format!("{module}::*"),
                    _ => format!("{module}::{}", item.name),
                },
                namespace: item.namespace,
                name: item.name,
                local_name: item.alias,
            })
            .collect(),
    }
}

fn operator_from_surface_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Option<InterfaceOperator> {
    match declaration {
        SurfaceDecl::Operator {
            visibility,
            fixity,
            precedence,
            spelling,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceOperator {
            symbol: format!("{module_name}::operator({spelling})"),
            spelling: spelling.clone(),
            fixity: fixity.clone(),
            precedence: *precedence,
            origin: *span,
        }),
        _ => None,
    }
}

fn instance_from_surface_decl(declaration: SurfaceDecl) -> Option<InterfaceInstance> {
    match declaration {
        SurfaceDecl::Instance {
            type_parameters,
            trait_name,
            arguments,
            constraints,
            span,
        } => Some(InterfaceInstance {
            trait_name: trait_name.clone(),
            type_parameters,
            head: InterfaceType::Apply {
                constructor: trait_name,
                arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
            },
            constraints: constraints
                .iter()
                .map(|name| InterfaceConstraint { name: name.clone() })
                .collect(),
            origin: span,
        }),
        _ => None,
    }
}

fn operator_interface_type(
    parameters: &[crate::surface::SurfaceParameter],
    return_type: &TypeRef,
) -> InterfaceType {
    parameters.iter().rev().fold(
        interface_type_from_type_ref(return_type),
        |result, parameter| InterfaceType::Function {
            parameter: Box::new(interface_type_from_type_ref(&parameter.type_ref)),
            result: Box::new(result),
        },
    )
}

fn interface_type_from_type_ref(type_ref: &TypeRef) -> InterfaceType {
    match type_ref {
        TypeRef::Named {
            name, arguments, ..
        } => InterfaceType::Named {
            name: name.clone(),
            arguments: arguments.iter().map(interface_type_from_type_ref).collect(),
        },
    }
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

fn dependency_module_name(module_name: &str, source_name: &str, specifier: &str) -> String {
    if let Some(relative) = specifier.strip_prefix("./") {
        return format!("{module_name}/{relative}");
    }
    source_name
        .rsplit_once('/')
        .map(|(parent, _)| format!("{parent}/{specifier}"))
        .unwrap_or_else(|| specifier.to_owned())
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
