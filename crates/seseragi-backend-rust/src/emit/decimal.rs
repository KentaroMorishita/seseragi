use super::*;

fn signature(name: &str) -> Option<(CoreType, CoreType)> {
    let named = |name: &str, arguments| CoreType::Named {
        name: name.into(),
        arguments,
    };
    let decimal = named("std/decimal::Decimal", vec![]);
    let text = named("std/prelude::String", vec![]);
    Some(match name {
        "std/decimal::parse" => (
            text,
            named(
                "std/prelude::Either",
                vec![named("std/decimal::DecimalParseError", vec![]), decimal],
            ),
        ),
        "std/decimal::fromInt" => (named("std/prelude::Int", vec![]), decimal),
        _ => return None,
    })
}

impl Backend<'_> {
    pub(super) fn decimal_key_dictionary(
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
        let function = match identity.as_str() {
            "std/decimal::Eq" => "native_decimal_eq_dictionary()",
            "std/decimal::Hash" => "native_decimal_hash_dictionary()",
            _ => return Ok(None),
        };
        if self.program_wasm_library || !type_arguments.is_empty() || !evidence_arguments.is_empty()
        {
            return Err(
                "Rust backend: unproven native Decimal dictionary type, children or target".into(),
            );
        }
        if identity == "std/decimal::Eq" {
            self.validate_eq_evidence(selected, scope)?;
        } else {
            self.validate_hash_evidence(selected, scope)?;
        }
        self.capabilities.insert("decimal".into());
        Ok(Some(function.into()))
    }
    pub(super) fn decimal_show_dictionary(
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
        if identity != "Show<std/decimal::Decimal>" {
            return Ok(None);
        }
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: Decimal Show constraint arity".into());
        };
        if !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || !lists::standard_trait(selected, "Show", actual)
        {
            return Err("Rust backend: unproven Decimal Show evidence".into());
        }
        self.decimal_type(
            actual,
            &CoreType::Named {
                name: "std/decimal::Decimal".into(),
                arguments: vec![],
            },
            scope,
        )?;
        self.capabilities.insert("decimal".into());
        Ok(Some("native_decimal_show_dictionary()".into()))
    }
    pub(super) fn validate_decimal_show_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if !matches!(&selected.evidence,CoreInstanceEvidence::Standard {identity,..} if identity=="Show<std/decimal::Decimal>")
        {
            return Ok(());
        }
        if dispatch.trait_identity != "std/prelude::Show"
            || dispatch.method != "show"
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven Decimal Show dispatch".into());
        }
        self.decimal_type(
            actual,
            &CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/decimal::Decimal".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Named {
                    name: "std/prelude::String".into(),
                    arguments: vec![],
                }),
            },
            scope,
        )
    }
    pub(super) fn validate_decimal_show_call(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(());
        };
        if !evidence.iter().any(|selected| matches!(&selected.evidence,CoreInstanceEvidence::Standard {identity,..} if identity=="Show<std/decimal::Decimal>")) {return Ok(());}
        let [selected] = evidence.as_slice() else {
            return Err("Rust backend: Decimal Show dictionary arity".into());
        };
        let dispatch = trait_dispatch
            .as_ref()
            .ok_or("Rust backend: missing Decimal Show dispatch")?;
        if arguments.len() > 1
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Decimal Show source stages".into());
        }
        let full = if let Some(argument) = arguments.first() {
            CoreType::Function {
                parameter: Box::new(
                    local::method_argument_type(argument)
                        .ok_or("Rust backend: missing Decimal Show argument type")?,
                ),
                result: Box::new(type_ref.clone()),
            }
        } else {
            type_ref.clone()
        };
        self.validate_decimal_show_value(dispatch, selected, &full, scope)
    }
    pub(super) fn decimal_type(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err("program WASM: native Decimal requires a target-specific ABI proof".into());
        }
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: unproven canonical Decimal argument/result type".into());
        }
        Ok(())
    }

    fn decimal_callable(name: &str) -> String {
        format!(
            "V::fun(vec![], |_, value| native_decimal_operation({}, value))",
            quoted(name)
        )
    }

    pub(super) fn decimal_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameter, result)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: Decimal operation with unsupported evidence".into());
        }
        self.decimal_type(
            type_ref,
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            },
            scope,
        )?;
        self.capabilities.insert("decimal".into());
        Ok(Some(Self::decimal_callable(name)))
    }

    pub(super) fn decimal_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(None);
        };
        let Some((parameter, result)) = signature(callee) else {
            return Ok(None);
        };
        if scope.contains(callee) {
            return Ok(None);
        }
        if arguments.len() > 1
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Decimal source application".into());
        }
        let Some(argument) = arguments.first() else {
            return self.decimal_reference(callee, evidence, type_ref, scope);
        };
        let actual = local::method_argument_type(argument)
            .ok_or("Rust backend: missing Decimal argument type")?;
        self.decimal_type(&actual, &parameter, scope)?;
        self.decimal_type(type_ref, &result, scope)?;
        self.capabilities.insert("decimal".into());
        self.direct_calls += 1;
        Ok(Some(format!(
            "native_decimal_operation({}, {})",
            quoted(callee),
            self.expr(argument, scope)?
        )))
    }
}
