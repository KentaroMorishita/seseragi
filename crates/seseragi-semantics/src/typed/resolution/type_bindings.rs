use crate::prelude::is_external_nominal_type;
use crate::{
    ExternalTypeBinding, ExternalTypeProvider, ResolvedModule, SymbolKind, SymbolNamespace,
};

pub(super) fn collect_external_type_bindings(
    resolved: &ResolvedModule,
) -> Vec<ExternalTypeBinding> {
    let mut bindings = Vec::new();
    // The implicit Prelude declaration owns this signature, even when its
    // types do not occur in a source annotation/import. Keep that provenance
    // for saved callables, public interfaces and the native admission proof.
    let uses_prelude = |operations: &[&str]| {
        resolved.references.iter().any(|reference| {
            reference.namespace == SymbolNamespace::Value
                && reference.target.is_some_and(|target| {
                    resolved.symbols.iter().any(|symbol| {
                        symbol.id == target
                            && symbol.kind == SymbolKind::Prelude
                            && symbol
                                .canonical
                                .as_deref()
                                .is_some_and(|canonical| operations.contains(&canonical))
                    })
                })
        })
    };
    if uses_prelude(&[
        "std/prelude::print",
        "std/prelude::println",
        "std/prelude::printValue",
    ]) {
        for (name, provider) in [
            ("String", "std/prelude"),
            ("Console", "std/console"),
            ("ConsoleError", "std/console"),
        ] {
            bindings.push(ExternalTypeBinding {
                arity: Some(0),
                spelling: name.into(),
                canonical: format!("std/prelude::{name}"),
                provider: Some(ExternalTypeProvider {
                    module: provider.into(),
                    export: name.into(),
                }),
            });
        }
    }
    if uses_prelude(&["std/prelude::readLine"]) {
        for name in ["Stdin", "StdinError"] {
            bindings.push(ExternalTypeBinding {
                arity: Some(0),
                spelling: name.into(),
                canonical: format!("std/prelude::{name}"),
                provider: Some(ExternalTypeProvider {
                    module: "std/stdin".into(),
                    export: name.into(),
                }),
            });
        }
    }
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
        let prelude_provider = (symbol.kind == SymbolKind::Prelude)
            .then(|| crate::prelude::provider_type_for_prelude(canonical))
            .flatten();
        if !is_external_nominal_type(canonical) && prelude_provider.is_none() {
            continue;
        }
        let provider = prelude_provider
            .as_ref()
            .map(|provider| ExternalTypeProvider {
                module: provider.module.into(),
                export: provider.export.into(),
            });
        let binding = ExternalTypeBinding {
            arity: provider.as_ref().map(|_| 0),
            spelling: reference.spelling.clone(),
            canonical: prelude_provider
                .map_or_else(|| canonical.clone(), |provider| provider.canonical.into()),
            provider,
        };
        if !bindings.contains(&binding) {
            bindings.push(binding);
        }
    }
    for import in resolved.imports.iter().filter(|import| import.in_scope) {
        let Some(scheme_bindings) = &import.scheme_type_bindings else {
            continue;
        };
        for scheme_binding in scheme_bindings {
            let mut binding = scheme_binding.clone();
            if let Some(explicit) = resolved.imports.iter().find(|candidate| {
                candidate.in_scope
                    && candidate.export.namespace == "type"
                    && candidate.export.symbol == binding.canonical
            }) {
                binding.spelling = explicit.local_name.clone();
            }
            if !bindings.contains(&binding) {
                bindings.push(binding);
            }
        }
    }
    for import in resolved.imports.iter().filter(|import| import.in_scope) {
        if import.export.namespace != "type" {
            continue;
        }
        let binding = ExternalTypeBinding {
            arity: Some(import.export.scheme.type_parameters.len() as u32),
            spelling: import.local_name.clone(),
            canonical: import.export.symbol.clone(),
            provider: Some(ExternalTypeProvider {
                module: import.module.clone(),
                export: import.export.name.clone(),
            }),
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
