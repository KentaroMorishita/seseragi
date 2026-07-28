use std::collections::BTreeMap;

use crate::{
    display_ops::runtime_display_dictionary_for_identity,
    prelude_ops::runtime_prelude_dictionary_for_identity, CoreInstanceEvidence,
};

use super::instances::local_instance_expression_key;
use super::types::type_ref_from_core_type;
use super::TypeScriptExpr;

pub(super) fn local_dictionary_expression(
    evidence: &CoreInstanceEvidence,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> Option<TypeScriptExpr> {
    if let CoreInstanceEvidence::Parameter { index } = evidence {
        return Some(TypeScriptExpr::Identifier {
            name: super::evidence_parameter_name(*index),
        });
    }
    if let CoreInstanceEvidence::Standard {
        identity,
        type_arguments,
        evidence_arguments,
    } = evidence
    {
        let local_name = runtime_display_dictionary_for_identity(identity)
            .map(|dictionary| dictionary.local_name)
            .or_else(|| {
                runtime_prelude_dictionary_for_identity(identity)
                    .map(|dictionary| dictionary.local_name)
            })?;
        if type_arguments.is_empty() && evidence_arguments.is_empty() {
            return Some(TypeScriptExpr::RuntimeReference {
                name: local_name.to_owned(),
            });
        }
        let arguments = evidence_arguments
            .iter()
            .map(|evidence| {
                local_dictionary_expression(&evidence.evidence, imported_values, imported_types)
            })
            .collect::<Option<Vec<_>>>()?;
        return Some(TypeScriptExpr::TypeApplicationCall {
            callee: local_name.to_owned(),
            type_arguments: type_arguments
                .iter()
                .map(|type_ref| type_ref_from_core_type(type_ref, imported_types))
                .collect(),
            arguments: structural_dictionary_arguments(identity, type_arguments, arguments)?,
        });
    }
    let (identity, type_arguments, evidence_arguments) = match evidence {
        CoreInstanceEvidence::Local {
            identity,
            type_arguments,
            evidence_arguments,
        }
        | CoreInstanceEvidence::Imported {
            identity,
            type_arguments,
            evidence_arguments,
            ..
        } => (identity, type_arguments, evidence_arguments),
        CoreInstanceEvidence::Standard { .. } | CoreInstanceEvidence::Parameter { .. } => {
            return None;
        }
    };
    let callee = imported_values
        .get(&local_instance_expression_key(identity))?
        .clone();
    let arguments = evidence_arguments
        .iter()
        .map(|evidence| {
            local_dictionary_expression(&evidence.evidence, imported_values, imported_types)
        })
        .collect::<Option<Vec<_>>>()?;
    if type_arguments.is_empty() && arguments.is_empty() {
        return Some(TypeScriptExpr::Identifier { name: callee });
    }
    Some(TypeScriptExpr::TypeApplicationCall {
        callee,
        type_arguments: type_arguments
            .iter()
            .map(|type_ref| type_ref_from_core_type(type_ref, imported_types))
            .collect(),
        arguments,
    })
}

fn structural_dictionary_arguments(
    identity: &str,
    type_arguments: &[crate::CoreType],
    evidence_arguments: Vec<TypeScriptExpr>,
) -> Option<Vec<TypeScriptExpr>> {
    match identity {
        "std/tuple::Show" | "std/tuple::Debug" => {
            let [crate::CoreType::Tuple { elements }] = type_arguments else {
                return None;
            };
            (elements.len() == evidence_arguments.len()).then_some(evidence_arguments)
        }
        "std/record::Show" | "std/record::Debug" => {
            let [crate::CoreType::Record {
                closed: true,
                fields,
            }] = type_arguments
            else {
                return None;
            };
            if fields.len() != evidence_arguments.len() {
                return None;
            }
            let mut fields = fields.iter().collect::<Vec<_>>();
            fields.sort_by(|left, right| left.name.cmp(&right.name));
            let names = TypeScriptExpr::Tuple {
                elements: fields
                    .iter()
                    .map(|field| TypeScriptExpr::String {
                        value: field.name.clone(),
                    })
                    .collect(),
            };
            let optional = TypeScriptExpr::Tuple {
                elements: fields
                    .iter()
                    .map(|field| TypeScriptExpr::Boolean {
                        value: field.optional,
                    })
                    .collect(),
            };
            Some(
                [names, optional]
                    .into_iter()
                    .chain(evidence_arguments)
                    .collect(),
            )
        }
        _ => Some(evidence_arguments),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_a_registered_standard_show_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "Show<std/prelude::String>".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert_eq!(
            expression,
            Some(TypeScriptExpr::RuntimeReference {
                name: "_ssrg_show_stringShow".to_owned(),
            })
        );
    }

    #[test]
    fn materializes_a_registered_standard_debug_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "Debug<std/prelude::String>".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert_eq!(
            expression,
            Some(TypeScriptExpr::RuntimeReference {
                name: "_ssrg_debug_stringDebug".to_owned(),
            })
        );
    }

    #[test]
    fn materializes_a_standard_collection_factory_with_nested_evidence() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/array::Show".to_owned(),
                type_arguments: vec![crate::CoreType::Named {
                    name: "String".to_owned(),
                    arguments: Vec::new(),
                }],
                evidence_arguments: vec![crate::CoreCallEvidence {
                    constraint: crate::CoreInstanceConstraint {
                        name: "Show".to_owned(),
                        arguments: vec![crate::CoreType::Named {
                            name: "String".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    evidence: CoreInstanceEvidence::Standard {
                        identity: "Show<std/prelude::String>".to_owned(),
                        type_arguments: Vec::new(),
                        evidence_arguments: Vec::new(),
                    },
                }],
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert_eq!(
            expression,
            Some(TypeScriptExpr::TypeApplicationCall {
                callee: "_ssrg_show_arrayShow".to_owned(),
                type_arguments: vec![crate::TypeScriptType::String],
                arguments: vec![TypeScriptExpr::RuntimeReference {
                    name: "_ssrg_show_stringShow".to_owned(),
                }],
            })
        );
    }

    #[test]
    fn materializes_a_standard_collection_factory_from_scoped_evidence() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/maybe::Debug".to_owned(),
                type_arguments: vec![crate::CoreType::Named {
                    name: "A".to_owned(),
                    arguments: Vec::new(),
                }],
                evidence_arguments: vec![crate::CoreCallEvidence {
                    constraint: crate::CoreInstanceConstraint {
                        name: "Debug".to_owned(),
                        arguments: vec![crate::CoreType::Named {
                            name: "A".to_owned(),
                            arguments: Vec::new(),
                        }],
                    },
                    evidence: CoreInstanceEvidence::Parameter { index: 0 },
                }],
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert!(matches!(
            expression,
            Some(TypeScriptExpr::TypeApplicationCall {
                callee,
                arguments,
                ..
            }) if callee == "_ssrg_debug_maybeDebug"
                && matches!(
                    arguments.as_slice(),
                    [TypeScriptExpr::Identifier { name }]
                        if name == "__ssrg$evidence$0"
                )
        ));
    }

    #[test]
    fn materializes_structural_record_metadata_without_host_key_inspection() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/record::Debug".to_owned(),
                type_arguments: vec![crate::CoreType::Record {
                    closed: true,
                    fields: vec![
                        crate::CoreRecordField {
                            name: "zeta".to_owned(),
                            optional: true,
                            type_ref: crate::CoreType::Named {
                                name: "String".to_owned(),
                                arguments: Vec::new(),
                            },
                        },
                        crate::CoreRecordField {
                            name: "alpha".to_owned(),
                            optional: false,
                            type_ref: crate::CoreType::Named {
                                name: "Int".to_owned(),
                                arguments: Vec::new(),
                            },
                        },
                    ],
                }],
                evidence_arguments: vec![
                    crate::CoreCallEvidence {
                        constraint: crate::CoreInstanceConstraint {
                            name: "Debug".to_owned(),
                            arguments: vec![crate::CoreType::Named {
                                name: "Int".to_owned(),
                                arguments: Vec::new(),
                            }],
                        },
                        evidence: CoreInstanceEvidence::Standard {
                            identity: "Debug<std/prelude::Int>".to_owned(),
                            type_arguments: Vec::new(),
                            evidence_arguments: Vec::new(),
                        },
                    },
                    crate::CoreCallEvidence {
                        constraint: crate::CoreInstanceConstraint {
                            name: "Debug".to_owned(),
                            arguments: vec![crate::CoreType::Named {
                                name: "String".to_owned(),
                                arguments: Vec::new(),
                            }],
                        },
                        evidence: CoreInstanceEvidence::Standard {
                            identity: "Debug<std/prelude::String>".to_owned(),
                            type_arguments: Vec::new(),
                            evidence_arguments: Vec::new(),
                        },
                    },
                ],
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert!(matches!(
            expression,
            Some(TypeScriptExpr::TypeApplicationCall {
                callee,
                arguments,
                ..
            }) if callee == "_ssrg_debug_recordDebug"
                && matches!(
                    arguments.as_slice(),
                    [
                        TypeScriptExpr::Tuple { elements: names },
                        TypeScriptExpr::Tuple { elements: optional },
                        TypeScriptExpr::RuntimeReference { name: alpha },
                        TypeScriptExpr::RuntimeReference { name: zeta },
                    ] if matches!(
                        names.as_slice(),
                        [
                            TypeScriptExpr::String { value: first },
                            TypeScriptExpr::String { value: second },
                        ] if first == "alpha" && second == "zeta"
                    )
                        && matches!(
                            optional.as_slice(),
                            [
                                TypeScriptExpr::Boolean { value: false },
                                TypeScriptExpr::Boolean { value: true },
                            ]
                        )
                        && alpha == "_ssrg_debug_intDebug"
                        && zeta == "_ssrg_debug_stringDebug"
                )
        ));
    }

    #[test]
    fn materializes_a_registered_standard_arithmetic_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/int::Add".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert_eq!(
            expression,
            Some(TypeScriptExpr::RuntimeReference {
                name: "_ssrg_int_add".to_owned(),
            })
        );
    }

    #[test]
    fn materializes_a_registered_standard_prelude_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/either::Monad".to_owned(),
                type_arguments: Vec::new(),
                evidence_arguments: Vec::new(),
            },
            &BTreeMap::new(),
            &BTreeMap::new(),
        );

        assert_eq!(
            expression,
            Some(TypeScriptExpr::RuntimeReference {
                name: "_ssrg_either_monad".to_owned(),
            })
        );
    }
}
