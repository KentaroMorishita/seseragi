use super::*;

impl Backend<'_> {
    pub(super) fn standard_value_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let Some((provider, class)) = identity.rsplit_once("::") else {
            return Ok(None);
        };
        let display = matches!(class, "Show" | "Debug");
        let monoid = class == "Monoid";
        let (owner, slots) = match (provider, class) {
            ("std/either", "Show" | "Debug") => ("std/prelude::Either", 2),
            ("std/map", "Show" | "Debug") => ("std/map::Map", 2),
            ("std/set", "Show" | "Debug") => ("std/set::Set", 1),
            ("std/validation", "Show" | "Debug" | "Eq") => ("std/validation::Validation", 2),
            ("std/non-empty-list", "Show" | "Debug" | "Semigroup") => {
                ("std/non-empty-list::NonEmptyList", 1)
            }
            ("std/string", "Semigroup" | "Monoid") => ("std/prelude::String", 0),
            ("std/array", "Semigroup" | "Monoid") => ("std/prelude::Array", 1),
            ("std/list", "Semigroup" | "Monoid") => ("std/prelude::List", 1),
            ("std/maybe", "Semigroup" | "Monoid") => ("std/prelude::Maybe", 1),
            ("std/sum", "Semigroup" | "Monoid") => ("std/prelude::Sum", 1),
            ("std/product", "Semigroup" | "Monoid") => ("std/prelude::Product", 1),
            _ => return Ok(None),
        };
        if self.program_wasm_library {
            return Err(
                "program WASM: native standard dictionary needs a target-specific ABI proof".into(),
            );
        }
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: standard dictionary constraint arity mismatch".into());
        };
        if selected.constraint.name != class
            || selected.constraint.trait_identity.as_deref()
                != Some(format!("std/prelude::{class}").as_str())
            || self.declared_method_type_kind(actual, scope) != Some(0)
        {
            return Err("Rust backend: standard dictionary trait or kind mismatch".into());
        }
        let CoreType::Named { name, arguments } = self.local_canonical_type(actual, scope)? else {
            return Err("Rust backend: standard dictionary requires a named type".into());
        };
        if name != owner || arguments.len() != slots {
            return Err("Rust backend: standard dictionary provider type mismatch".into());
        }
        let factory =
            display || class == "Eq" || matches!(provider, "std/maybe" | "std/sum" | "std/product");
        let types = type_arguments
            .iter()
            .map(|ty| self.local_canonical_type(ty, scope))
            .collect::<Result<Vec<_>>>()?;
        if types != if factory { arguments.clone() } else { vec![] } {
            return Err("Rust backend: standard dictionary factory type mismatch".into());
        }
        let contracts = if display || class == "Eq" {
            arguments
                .iter()
                .map(|ty| (class, vec![ty.clone()]))
                .collect::<Vec<_>>()
        } else if provider == "std/maybe" {
            vec![("Semigroup", vec![arguments[0].clone()])]
        } else if matches!(provider, "std/sum" | "std/product") {
            let mut children = vec![];
            if monoid {
                children.push((
                    if provider == "std/sum" { "Zero" } else { "One" },
                    vec![arguments[0].clone()],
                ));
            }
            children.push((
                if provider == "std/sum" { "Add" } else { "Mul" },
                vec![arguments[0].clone(); 3],
            ));
            children
        } else {
            vec![]
        };
        if contracts.len() != evidence_arguments.len() {
            return Err("Rust backend: standard dictionary child arity mismatch".into());
        }
        for ((class, types), child) in contracts.iter().zip(evidence_arguments) {
            if child.constraint.name != *class
                || child.constraint.trait_identity.as_deref()
                    != Some(format!("std/prelude::{class}").as_str())
                || child
                    .constraint
                    .arguments
                    .iter()
                    .map(|ty| self.local_canonical_type(ty, scope))
                    .collect::<Result<Vec<_>>>()?
                    != *types
            {
                return Err(
                    "Rust backend: standard dictionary child type or trait mismatch".into(),
                );
            }
            if *class == "Eq" {
                self.validate_eq_evidence(child, scope)?;
            }
            if display {
                self.validate_display_child(child, class, scope)?;
            }
        }
        let children = self.evidence(evidence_arguments, scope)?;
        Ok(Some(if class == "Eq" {
            format!("native_validation_eq({children})")
        } else if display {
            format!(
                "native_standard_display_dictionary({}, {children}, {})",
                quoted(provider),
                class == "Debug"
            )
        } else {
            format!(
                "native_standard_append_dictionary({}, {children}, {monoid})",
                quoted(provider)
            )
        }))
    }
}
