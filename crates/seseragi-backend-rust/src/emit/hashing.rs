use super::*;

impl Backend<'_> {
    pub(super) fn validate_hash_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        let [argument] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: Hash requires one type argument".into());
        };
        if selected.constraint.trait_identity.as_deref() != Some("std/prelude::Hash")
            || !self.valid_constraint_label(&selected.constraint)
            || self.declared_method_type_kind(argument, scope) != Some(0)
        {
            return Err("Rust backend: unproven canonical Hash constraint".into());
        }
        self.validate_parameter_evidence(selected, scope)?;
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
            std::slice::from_ref(selected),
            scope,
        )?;
        if self.validate_standard_composite(selected, scope)? {
            return Ok(());
        }
        if self.validate_derived_structural_evidence(selected, scope)? {
            return Ok(());
        }
        if let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        {
            let canonical = match identity.as_str() {
                "std/int::Hash" => "std/prelude::Int",
                "std/bool::Hash" => "std/prelude::Bool",
                "std/char::Hash" => "std/prelude::Char",
                "std/string::Hash" => "std/prelude::String",
                "std/unit::Hash" => "std/prelude::Unit",
                "std/decimal::Hash" => "std/decimal::Decimal",
                "std/big-int::Hash" => "std/big-int::BigInt",
                _ => return Err("Rust backend: unsupported canonical Hash provider".into()),
            };
            if !type_arguments.is_empty()
                || !evidence_arguments.is_empty()
                || self.local_canonical_type(argument, scope)?
                    != (CoreType::Named {
                        name: canonical.into(),
                        arguments: vec![],
                    })
            {
                return Err("Rust backend: canonical Hash provider type or child mismatch".into());
            }
        }
        Ok(())
    }

    pub(super) fn primitive_hash_dictionary(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(None);
        };
        if !matches!(
            identity.as_str(),
            "std/int::Hash"
                | "std/bool::Hash"
                | "std/char::Hash"
                | "std/string::Hash"
                | "std/unit::Hash"
        ) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err("program WASM: native Hash requires a target-specific ABI proof".into());
        }
        self.validate_hash_evidence(selected, scope)?;
        Ok(Some(format!(
            "native_hash_dictionary({})",
            quoted(identity)
        )))
    }

    pub(super) fn validate_hash_call(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(());
        };
        let is_hash = dispatch.trait_identity == "std/prelude::Hash"
            || evidence.first().is_some_and(|e| {
                e.constraint.trait_identity.as_deref() == Some("std/prelude::Hash")
            });
        if !is_hash {
            return Ok(());
        }
        let [selected] = evidence.as_slice() else {
            return Err("Rust backend: Hash requires one primary dictionary".into());
        };
        if dispatch.trait_identity != "std/prelude::Hash"
            || dispatch.method != "hash"
            || dispatch.method_contract.is_some()
            || arguments.len() > 1
            || scope.has_unproven_type_scope()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Hash source stages".into());
        }
        self.validate_hash_evidence(selected, scope)?;
        let mut full = type_ref.clone();
        for argument in arguments.iter().rev() {
            full = CoreType::Function {
                parameter: Box::new(
                    local::method_argument_type(argument)
                        .ok_or("Rust backend: missing Hash source argument type")?,
                ),
                result: Box::new(full),
            };
        }
        let expected = CoreType::Function {
            parameter: Box::new(
                self.local_canonical_type(&selected.constraint.arguments[0], scope)?,
            ),
            result: Box::new(CoreType::Named {
                name: "std/prelude::Int".into(),
                arguments: vec![],
            }),
        };
        if self.declared_method_type_kind(&full, scope) != Some(0)
            || self.local_canonical_type(&full, scope)? != expected
        {
            return Err("Rust backend: Hash argument or Int result contract mismatch".into());
        }
        Ok(())
    }

    pub(super) fn validate_hash_instance(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<()> {
        if !instance.identity.starts_with("std/prelude::Hash<") {
            return Ok(());
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven Hash implementation".into());
        };
        let [method] = methods.as_slice() else {
            return Err("Rust backend: Hash requires one method".into());
        };
        if instance.arguments.len() != 1
            || instance.supertrait_count != 0
            || method.name != "hash"
            || !method.type_parameters.is_empty()
            || !method.constraints.is_empty()
            || method.parameters.len() != 1
        {
            return Err("Rust backend: unproven Hash implementation source arity".into());
        }
        if self.declared_method_type_kind(&instance.arguments[0], scope) != Some(0) {
            return Err("Rust backend: unproven Hash implementation argument kind".into());
        }
        self.match_local_type(
            &instance.arguments[0],
            &method.parameters[0].type_ref,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )?;
        let result = local::method_argument_type(&method.body)
            .ok_or("Rust backend: missing Hash implementation result")?;
        if self.local_canonical_type(&result, scope)?
            != (CoreType::Named {
                name: "std/prelude::Int".into(),
                arguments: vec![],
            })
        {
            return Err("Rust backend: Hash implementation must return canonical Int".into());
        }
        Ok(())
    }
}
