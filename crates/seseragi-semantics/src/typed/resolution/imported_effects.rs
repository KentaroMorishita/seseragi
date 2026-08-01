use crate::{ResolvedModule, SymbolId};
use std::collections::BTreeMap;

use super::super::functions::TopLevelPureFunction;
use super::imported_types::ImportedTypeContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImportedEffectFunction {
    pub(crate) signature: TopLevelPureFunction,
}

pub(super) fn collect_imported_effects(
    resolved: &ResolvedModule,
) -> BTreeMap<SymbolId, ImportedEffectFunction> {
    let types = ImportedTypeContext::new(resolved);
    resolved
        .imports
        .iter()
        .filter(|import| {
            import.in_scope && import.export.declaration_kind.as_deref() == Some("effect-function")
        })
        .filter_map(|import| {
            let signature = super::imports::imported_callable(&types, import)?;
            matches!(
                &signature.result,
                crate::TypedType::Named { name, arguments }
                    if name == "Effect" && arguments.len() == 3
            )
            .then_some((import.symbol, ImportedEffectFunction { signature }))
        })
        .collect()
}
