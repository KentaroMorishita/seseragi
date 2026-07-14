use crate::surface::{SurfaceDecl, Visibility};

use super::super::types::interface_type_from_type_ref;
use super::super::{InterfaceExport, InterfaceScheme, InterfaceType};

pub(super) fn exports_from_newtype_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Vec<InterfaceExport> {
    let SurfaceDecl::Newtype {
        visibility,
        opaque,
        name,
        name_span,
        type_parameters,
        representation,
        span,
        ..
    } = declaration
    else {
        return Vec::new();
    };
    if *visibility != Visibility::Public {
        return Vec::new();
    }

    let symbol = format!("{module_name}::{name}");
    let result = InterfaceType::Named {
        name: name.clone(),
        arguments: type_parameters
            .iter()
            .map(|parameter| InterfaceType::Named {
                name: parameter.clone(),
                arguments: Vec::new(),
            })
            .collect(),
    };
    let mut exports = vec![InterfaceExport {
        symbol: symbol.clone(),
        namespace: "type".to_owned(),
        name: name.clone(),
        constructor_of: None,
        visibility: *visibility,
        declaration_kind: Some("newtype".to_owned()),
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
        representation: (!opaque).then(|| interface_type_from_type_ref(representation)),
    }];
    if !opaque {
        exports.push(InterfaceExport {
            symbol,
            namespace: "value".to_owned(),
            name: name.clone(),
            constructor_of: Some(format!("{module_name}::{name}")),
            visibility: *visibility,
            declaration_kind: Some("constructor".to_owned()),
            declaration: *name_span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: Vec::new(),
                type_ref: InterfaceType::Function {
                    parameter: Box::new(interface_type_from_type_ref(representation)),
                    result: Box::new(result),
                },
            },
            methods: Vec::new(),
            representation: None,
        });
    }
    exports
}
