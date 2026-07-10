use crate::surface::{SurfaceDecl, Visibility};

use super::{nominal_type_export, InterfaceExport, InterfaceScheme, InterfaceType};

pub(super) fn exports_from_type_decl(
    module_name: &str,
    declaration: &SurfaceDecl,
) -> Vec<InterfaceExport> {
    let SurfaceDecl::Type {
        visibility,
        opaque,
        name,
        type_parameters,
        variants,
        span,
        ..
    } = declaration
    else {
        return Vec::new();
    };
    if *visibility != Visibility::Public {
        return Vec::new();
    }

    let owner = format!("{module_name}::{name}");
    let mut exports = vec![nominal_type_export(
        module_name,
        name,
        *visibility,
        if *opaque { "opaque-type" } else { "type" },
        type_parameters,
        *span,
    )];
    if *opaque {
        return exports;
    }

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
    exports.extend(variants.iter().map(|variant| {
        let type_ref = variant.payload.as_ref().map_or_else(
            || result.clone(),
            |payload| InterfaceType::Function {
                parameter: Box::new(super::interface_type_from_type_ref(payload)),
                result: Box::new(result.clone()),
            },
        );
        InterfaceExport {
            symbol: format!("{module_name}::{}", variant.name),
            namespace: "value".to_owned(),
            name: variant.name.clone(),
            constructor_of: Some(owner.clone()),
            visibility: *visibility,
            declaration_kind: Some("constructor".to_owned()),
            declaration: variant.name_span,
            scheme: InterfaceScheme {
                type_parameters: type_parameters.clone(),
                constraints: Vec::new(),
                type_ref,
            },
            representation: None,
        }
    }));
    exports
}
