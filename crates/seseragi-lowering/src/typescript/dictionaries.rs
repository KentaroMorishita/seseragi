use std::collections::BTreeMap;

use crate::CoreInstanceEvidence;

use super::instances::local_instance_expression_key;
use super::types::type_ref_from_core_type;
use super::TypeScriptExpr;

pub(super) fn local_dictionary_expression(
    evidence: &CoreInstanceEvidence,
    imported_values: &BTreeMap<String, String>,
    imported_types: &BTreeMap<String, String>,
) -> Option<TypeScriptExpr> {
    let CoreInstanceEvidence::Local {
        identity,
        type_arguments,
        evidence_arguments,
    } = evidence
    else {
        return None;
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
