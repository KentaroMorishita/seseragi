use crate::surface::{SurfaceDecl, Visibility};

use super::methods::{
    function_interface_type, interface_constraint_from_surface, interface_method_from_surface,
};
use super::types::interface_type_from_type_ref;
use super::{InterfaceExport, InterfaceOperator, InterfaceScheme, InterfaceType};

mod adt;
mod newtype;

pub(super) fn exports_from_surface_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Vec<InterfaceExport> {
    if matches!(declaration, SurfaceDecl::Type { .. }) {
        return adt::exports_from_type_decl(module_name, declaration);
    }
    if matches!(declaration, SurfaceDecl::Newtype { .. }) {
        return newtype::exports_from_newtype_decl(module_name, declaration);
    }
    export_from_surface_decl(module_name, declaration)
        .into_iter()
        .collect()
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
                constructor_of: None,
                visibility: *visibility,
                declaration_kind: None,
                declaration: *span,
                scheme: InterfaceScheme {
                    type_parameters: Vec::new(),
                    constraints: Vec::new(),
                    type_ref,
                },
                methods: Vec::new(),
                representation: None,
            })
        }
        SurfaceDecl::Fn {
            visibility,
            name,
            type_parameters,
            parameters,
            return_type,
            constraints,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::{name}"),
            namespace: "value".to_owned(),
            name: name.clone(),
            constructor_of: None,
            visibility: *visibility,
            declaration_kind: Some("function".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: constraints
                    .iter()
                    .map(interface_constraint_from_surface)
                    .collect(),
                type_ref: function_interface_type(parameters, return_type),
            },
            methods: Vec::new(),
            representation: None,
        }),
        SurfaceDecl::Alias {
            visibility,
            name,
            type_parameters,
            target,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::{name}"),
            namespace: "type".to_owned(),
            name: name.clone(),
            constructor_of: None,
            visibility: *visibility,
            declaration_kind: Some("alias".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: Vec::new(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.clone(),
                    arity: type_parameters.len() as u32,
                },
            },
            methods: Vec::new(),
            representation: Some(interface_type_from_type_ref(target)),
        }),
        SurfaceDecl::Struct {
            visibility,
            opaque,
            name,
            type_parameters,
            span,
            ..
        } if *visibility == Visibility::Public => Some(nominal_type_export(
            module_name,
            name,
            *visibility,
            if *opaque { "opaque-struct" } else { "struct" },
            type_parameters,
            *span,
        )),
        SurfaceDecl::Trait {
            visibility,
            name,
            type_parameters,
            constraints,
            methods,
            span,
            ..
        } if *visibility == Visibility::Public => Some(InterfaceExport {
            symbol: format!("{module_name}::trait({name})"),
            namespace: "trait".to_owned(),
            name: name.clone(),
            constructor_of: None,
            visibility: *visibility,
            declaration_kind: Some("trait".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: constraints
                    .iter()
                    .map(interface_constraint_from_surface)
                    .collect(),
                type_ref: InterfaceType::TypeConstructor {
                    name: name.clone(),
                    arity: type_parameters.len() as u32,
                },
            },
            methods: methods.iter().map(interface_method_from_surface).collect(),
            representation: None,
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
            constructor_of: None,
            visibility: *visibility,
            declaration_kind: Some("custom-operator".to_owned()),
            declaration: *span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: constraints
                    .iter()
                    .map(interface_constraint_from_surface)
                    .collect(),
                type_ref: function_interface_type(parameters, return_type),
            },
            methods: Vec::new(),
            representation: None,
        }),
        _ => None,
    }
}

fn nominal_type_export(
    module_name: &str,
    name: &str,
    visibility: Visibility,
    declaration_kind: &str,
    type_parameters: &[String],
    span: crate::surface::ByteSpan,
) -> InterfaceExport {
    InterfaceExport {
        symbol: format!("{module_name}::{name}"),
        namespace: "type".to_owned(),
        name: name.to_owned(),
        constructor_of: None,
        visibility,
        declaration_kind: Some(declaration_kind.to_owned()),
        declaration: span,
        scheme: InterfaceScheme {
            type_parameters: type_parameters.to_vec(),
            constraints: Vec::new(),
            type_ref: InterfaceType::TypeConstructor {
                name: name.to_owned(),
                arity: type_parameters.len() as u32,
            },
        },
        methods: Vec::new(),
        representation: None,
    }
}

pub(super) fn operator_from_surface_decl(
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
