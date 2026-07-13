use crate::{ResolvedModule, SymbolId, TypedEffect};
use seseragi_syntax::InterfaceType;
use std::collections::{BTreeMap, BTreeSet};

use super::super::semantic_types::SemanticValueType;
use super::imported_types::{flatten_function, ImportedTypeContext};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportedEffectFunction {
    pub(crate) symbol: String,
    pub(crate) parameters: Vec<SemanticValueType>,
    pub(crate) effect: TypedEffect,
}

pub(super) fn collect_imported_effects(
    resolved: &ResolvedModule,
) -> BTreeMap<SymbolId, ImportedEffectFunction> {
    let types = ImportedTypeContext::new(resolved);
    let no_type_parameters = BTreeSet::new();
    resolved
        .imports
        .iter()
        .filter_map(|import| {
            let export = &import.export;
            let scheme_type_bindings = import.scheme_type_bindings.as_deref()?;
            if !import.in_scope
                || export.declaration_kind.as_deref() != Some("effect-function")
                || !export.scheme.type_parameters.is_empty()
                || !export.scheme.constraints.is_empty()
            {
                return None;
            }
            let (parameter_types, result) = flatten_function(export.scheme.type_ref.clone());
            let parameters = parameter_types
                .into_iter()
                .map(|type_ref| {
                    types.semantic_value(
                        type_ref,
                        &import.module,
                        &no_type_parameters,
                        scheme_type_bindings,
                    )
                })
                .collect::<Option<Vec<_>>>()?;
            let InterfaceType::Named { name, arguments } = result else {
                return None;
            };
            let [environment, failure, success] = arguments.as_slice() else {
                return None;
            };
            if name != "Effect" {
                return None;
            }
            let typed = |type_ref: &InterfaceType| {
                types
                    .semantic_value(
                        type_ref.clone(),
                        &import.module,
                        &no_type_parameters,
                        scheme_type_bindings,
                    )
                    .map(|value| value.type_ref)
            };
            Some((
                import.symbol,
                ImportedEffectFunction {
                    symbol: export.symbol.clone(),
                    parameters,
                    effect: TypedEffect {
                        environment: typed(environment)?,
                        failure: typed(failure)?,
                        success: typed(success)?,
                    },
                },
            ))
        })
        .collect()
}
