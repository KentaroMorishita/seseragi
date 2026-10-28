use super::*;

/// These declarations belong to Prelude, independently of a caller's local
/// type names and binders. Direct Effect syntax and saved callable values use
/// the same ordinary application checker and the same one source parameter.
pub(super) fn callable(canonical: &str) -> Option<TopLevelPureFunction> {
    if !matches!(canonical, "std/prelude::print" | "std/prelude::println") {
        return None;
    }
    let primitive = |name: &str| TypedType::ExternalNamed {
        name: name.to_owned(),
        canonical: format!("std/prelude::{name}"),
        arguments: vec![],
    };
    let parameter = primitive("String");
    let result = TypedType::Named {
        name: "Effect".into(),
        arguments: vec![
            TypedType::Record {
                closed: true,
                fields: vec![crate::TypedRecordField {
                    name: "console".into(),
                    optional: false,
                    type_ref: primitive("Console"),
                }],
            },
            primitive("ConsoleError"),
            named_type("Unit"),
        ],
    };
    Some(TopLevelPureFunction {
        symbol: canonical.to_owned(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec![],
        constraints: vec![],
        constraint_identities: vec![],
        semantic_parameters: vec![standard_scheme_value(&parameter, &[]).key],
        semantic_result: standard_scheme_value(&result, &[]).key,
        parameters: vec![parameter],
        result,
    })
}
