use crate::prelude::is_external_nominal_type;
use crate::{ExternalTypeBinding, ResolvedModule, SymbolNamespace};

pub(super) fn collect_external_type_bindings(
    resolved: &ResolvedModule,
) -> Vec<ExternalTypeBinding> {
    let mut bindings = Vec::new();
    for reference in &resolved.references {
        if reference.namespace != SymbolNamespace::Type {
            continue;
        }
        let Some(symbol) = reference
            .target
            .and_then(|target| resolved.symbols.iter().find(|symbol| symbol.id == target))
        else {
            continue;
        };
        let Some(canonical) = &symbol.canonical else {
            continue;
        };
        if !is_external_nominal_type(canonical) {
            continue;
        }
        let binding = ExternalTypeBinding {
            spelling: reference.spelling.clone(),
            canonical: canonical.clone(),
        };
        if !bindings.contains(&binding) {
            bindings.push(binding);
        }
    }
    bindings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_prelude_runtime_type_canonical_identity() {
        let resolved = crate::resolve_module(
            "artifact/runtime-type/main.ssrg",
            "type AppError = | InputFailure StdinError\n",
        );

        assert!(collect_external_type_bindings(&resolved)
            .iter()
            .any(|binding| binding.spelling == "StdinError"
                && binding.canonical == "std/prelude::StdinError"));
    }

    #[test]
    fn excludes_local_shadow_from_external_nominal_bindings() {
        let resolved = crate::resolve_module(
            "artifact/runtime-type-shadow/main.ssrg",
            "type StdinError = | LocalStdinError\ntype AppError = | InputFailure StdinError\n",
        );

        let bindings = collect_external_type_bindings(&resolved);
        assert!(bindings.is_empty());
    }
}
