use super::*;

pub(super) fn read_line_callable() -> TopLevelPureFunction {
    let primitive = |name: &str| TypedType::ExternalNamed {
        name: name.into(),
        canonical: format!("std/prelude::{name}"),
        arguments: vec![],
    };
    let parameter = canonical_unit_value_type();
    let result = TypedType::Named {
        name: "Effect".into(),
        arguments: vec![
            TypedType::Record {
                closed: true,
                fields: vec![crate::TypedRecordField {
                    name: "stdin".into(),
                    optional: false,
                    type_ref: primitive("Stdin"),
                }],
            },
            primitive("StdinError"),
            TypedType::Named {
                name: "Maybe".into(),
                arguments: vec![primitive("String")],
            },
        ],
    };
    TopLevelPureFunction {
        symbol: "std/prelude::readLine".into(),
        trait_identity: None,
        trait_method: None,
        type_parameters: vec![],
        constraints: vec![],
        constraint_identities: vec![],
        semantic_parameters: vec![parameter.key],
        parameters: vec![parameter.type_ref],
        semantic_result: standard_scheme_value(&result, &[]).key,
        result,
    }
}
