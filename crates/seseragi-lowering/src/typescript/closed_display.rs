use super::dictionaries::local_dictionary_expression;
use super::types::TypeScriptTypeContext;
use super::TypeScriptExpr;
use crate::CoreCallEvidence;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A finite, compiler-certified factory owned by one generated module. The
/// getter keeps only its closed standard dictionary, never source values.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeScriptClosedDisplayDictionary {
    pub name: String,
    pub factory: TypeScriptExpr,
}

pub(super) fn selected_dictionary_expression(
    selected: &CoreCallEvidence,
    imported_values: &BTreeMap<String, String>,
    imported_types: &TypeScriptTypeContext,
) -> Option<TypeScriptExpr> {
    let factory = local_dictionary_expression(&selected.evidence, imported_values, imported_types)?;
    let Some(key) = crate::closed_standard_display_key(selected, imported_types.bound_type_names())
    else {
        return Some(factory);
    };
    let name = imported_types.closed_display_dictionary(key, factory);
    Some(TypeScriptExpr::Call {
        callee: name,
        arguments: Vec::new(),
        type_arguments: Vec::new(),
        checked_callee_type: None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn runtime_import_freshening_reaches_the_lazy_factory() {
        let source="fn _ssrg_tupleShow value: Int -> Int = value\nfn _ssrg_intShow value: Int -> Int = value\npub fn kernel value: Int -> String = show (value, value + 1, True)";
        let module = crate::lower_core_module_to_typescript_ir(crate::lower_typed_module(
            seseragi_semantics::type_module("display/main.ssrg", source),
        ));
        assert_eq!(module.closed_display_dictionaries.len(), 1);
        let import = module
            .imports
            .iter()
            .find(|import| import.feature == "core.tuple.show")
            .unwrap();
        assert_ne!(import.local, "_ssrg_tupleShow");
        let super::TypeScriptExpr::TypeApplicationCall { callee, .. } =
            &module.closed_display_dictionaries[0].factory
        else {
            panic!("factory")
        };
        assert_eq!(callee, &import.local);
    }
}
