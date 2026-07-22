use std::collections::BTreeMap;

use crate::{
    prelude_ops::runtime_prelude_dictionary_for_identity,
    show_ops::runtime_show_dictionary_for_identity, CoreInstanceEvidence,
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
    if let CoreInstanceEvidence::Standard { identity } = evidence {
        let local_name = runtime_show_dictionary_for_identity(identity)
            .map(|dictionary| dictionary.local_name)
            .or_else(|| {
                runtime_prelude_dictionary_for_identity(identity)
                    .map(|dictionary| dictionary.local_name)
            })?;
        return Some(TypeScriptExpr::RuntimeReference {
            name: local_name.to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materializes_a_registered_standard_show_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "Show<std/prelude::String>".to_owned(),
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
    fn materializes_a_registered_standard_arithmetic_dictionary() {
        let expression = local_dictionary_expression(
            &CoreInstanceEvidence::Standard {
                identity: "std/int::Add".to_owned(),
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
