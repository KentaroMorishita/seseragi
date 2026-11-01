use super::*;

impl Backend<'_> {
    pub(super) fn validate_standard_composite(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<bool> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(false);
        };
        let Some((provider, class)) = identity.rsplit_once("::") else {
            return Ok(false);
        };
        let (owner, count) = match (provider, class) {
            ("std/array", "Eq") => ("std/prelude::Array", 1),
            ("std/list", "Eq") => ("std/prelude::List", 1),
            ("std/maybe", "Eq") => ("std/prelude::Maybe", 1),
            ("std/either", "Eq") => ("std/prelude::Either", 2),
            ("std/map", "Eq") => ("std/map::Map", 2),
            ("std/set", "Eq") => ("std/set::Set", 1),
            ("std/non-empty-list", "Eq" | "Ord" | "Hash") => {
                ("std/non-empty-list::NonEmptyList", 1)
            }
            _ => return Ok(false),
        };
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: standard composite constraint arity mismatch".into());
        };
        if selected.constraint.name != class
            || selected.constraint.trait_identity.as_deref()
                != Some(format!("std/prelude::{class}").as_str())
            || self.declared_method_type_kind(actual, scope) != Some(0)
        {
            return Err("Rust backend: standard composite trait or kind mismatch".into());
        }
        let CoreType::Named { name, arguments } = self.local_canonical_type(actual, scope)? else {
            return Err("Rust backend: standard composite requires a named type".into());
        };
        if name != owner
            || arguments.len() != count
            || type_arguments
                .iter()
                .map(|ty| self.local_canonical_type(ty, scope))
                .collect::<Result<Vec<_>>>()?
                != arguments
        {
            return Err("Rust backend: standard composite provider or factory mismatch".into());
        }
        let contracts = match provider {
            "std/map" => vec![
                ("Eq", &arguments[0]),
                ("Hash", &arguments[0]),
                ("Eq", &arguments[1]),
            ],
            "std/set" => vec![("Eq", &arguments[0]), ("Hash", &arguments[0])],
            _ => arguments.iter().map(|ty| (class, ty)).collect(),
        };
        if contracts.len() != evidence_arguments.len() {
            return Err("Rust backend: standard composite child arity mismatch".into());
        }
        for ((class, ty), child) in contracts.iter().zip(evidence_arguments) {
            if child.constraint.name != *class
                || child.constraint.trait_identity.as_deref()
                    != Some(format!("std/prelude::{class}").as_str())
                || child.constraint.arguments.len() != 1
                || self.local_canonical_type(&child.constraint.arguments[0], scope)? != **ty
            {
                return Err("Rust backend: standard composite child trait or type mismatch".into());
            }
            match *class {
                "Eq" => self.validate_eq_evidence(child, scope)?,
                "Ord" => self.validate_ord_evidence(child, scope)?,
                "Hash" => self.validate_hash_evidence(child, scope)?,
                _ => unreachable!(),
            }
        }
        Ok(true)
    }

    pub(super) fn standard_composite_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        {
            let primitive = match identity.as_str() {
                "std/bool::Ord" => Some("Bool"),
                "std/string::Ord" => Some("String"),
                "std/unit::Ord" => Some("Unit"),
                _ => None,
            };
            if let Some(primitive) = primitive {
                if self.program_wasm_library {
                    return Err(
                        "program WASM: native Ord dictionary needs a target-specific ABI proof"
                            .into(),
                    );
                }
                let [ty] = selected.constraint.arguments.as_slice() else {
                    return Err("Rust backend: primitive Ord constraint arity mismatch".into());
                };
                if selected.constraint.name != "Ord"
                    || selected.constraint.trait_identity.as_deref() != Some("std/prelude::Ord")
                    || !type_arguments.is_empty()
                    || !evidence_arguments.is_empty()
                    || self.local_canonical_type(ty, scope)?
                        != (CoreType::Named {
                            name: format!("std/prelude::{primitive}"),
                            arguments: vec![],
                        })
                {
                    return Err("Rust backend: primitive Ord provider mismatch".into());
                }
                return Ok(Some(format!(
                    "native_standard_ord_dictionary({})",
                    quoted(primitive)
                )));
            }
        }
        if !self.validate_standard_composite(selected, scope)? {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native composite dictionary needs a target-specific ABI proof"
                    .into(),
            );
        }
        let CoreInstanceEvidence::Standard {
            identity,
            evidence_arguments,
            ..
        } = &selected.evidence
        else {
            unreachable!()
        };
        let children = self.evidence(evidence_arguments, scope)?;
        Ok(Some(format!(
            "native_standard_composite_dictionary({}, {children})",
            quoted(identity)
        )))
    }
}
