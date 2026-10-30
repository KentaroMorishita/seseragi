use super::*;

impl Backend<'_> {
    pub(super) fn big_int_trait_dictionary(
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
            "std/big-int::Eq" => "native_big_int_eq_dictionary()",
            "std/big-int::Hash" => "native_big_int_hash_dictionary()",
            "std/big-int::Ord" => "native_big_int_ord_dictionary()",
            "std/big-int::Zero" => "native_big_int_constant_dictionary(false)",
            "std/big-int::One" => "native_big_int_constant_dictionary(true)",
            _ => return Ok(None),
        };
        if self.program_wasm_library || !type_arguments.is_empty() || !evidence_arguments.is_empty()
        {
            return Err(
                "Rust backend: unproven native BigInt dictionary type, children or target".into(),
            );
        }
        if identity == "std/big-int::Eq" {
            self.validate_eq_evidence(selected, scope)?;
        } else if identity == "std/big-int::Hash" {
            self.validate_hash_evidence(selected, scope)?;
        } else if identity == "std/big-int::Ord" {
            self.validate_ord_evidence(selected, scope)?;
        } else {
            let [actual] = selected.constraint.arguments.as_slice() else {
                return Err("Rust backend: BigInt constant constraint arity".into());
            };
            let name = if identity == "std/big-int::Zero" {
                "Zero"
            } else {
                "One"
            };
            if !lists::standard_trait(selected, name, actual) {
                return Err("Rust backend: unproven BigInt constant trait owner".into());
            }
            self.big_int_type(
                actual,
                &CoreType::Named {
                    name: "std/big-int::BigInt".into(),
                    arguments: vec![],
                },
                scope,
            )?;
        }
        self.capabilities.insert("big-int".into());
        Ok(Some(function.into()))
    }

    pub(super) fn validate_big_int_dispatch(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
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
        let Some(selected) = evidence.first() else {
            return Ok(());
        };
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(());
        };
        let (trait_identity, method) = match identity.as_str() {
            "std/big-int::Eq" => ("std/prelude::Eq", "eq"),
            "std/big-int::Hash" => ("std/prelude::Hash", "hash"),
            "std/big-int::Ord" => ("std/prelude::Ord", "compare"),
            "std/big-int::Zero" => ("std/prelude::Zero", "zero"),
            "std/big-int::One" => ("std/prelude::One", "one"),
            _ => return Ok(()),
        };
        if evidence.len() != 1
            || dispatch.trait_identity != trait_identity
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: BigInt dictionary dispatch mismatch".into());
        }
        if matches!(method, "zero" | "one") {
            if arguments.len() > 1
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven BigInt constant source stages".into());
            }
            let full = if let Some(argument) = arguments.first() {
                CoreType::Function {
                    parameter: Box::new(
                        local::method_argument_type(argument)
                            .ok_or("Rust backend: unproven BigInt constant Unit argument")?,
                    ),
                    result: Box::new(type_ref.clone()),
                }
            } else {
                type_ref.clone()
            };
            self.validate_big_int_constant_value(dispatch, selected, &full, scope)?;
        } else if method == "compare" {
            if arguments.len() > 2
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven BigInt Ord source stages".into());
            }
            let full = arguments
                .iter()
                .rev()
                .try_fold(type_ref.clone(), |result, argument| {
                    Ok::<_, RustBackendError>(CoreType::Function {
                        parameter: Box::new(
                            local::method_argument_type(argument)
                                .ok_or("Rust backend: unproven BigInt Ord argument")?,
                        ),
                        result: Box::new(result),
                    })
                })?;
            self.validate_ord_method_value(dispatch, selected, &full, scope)?;
        }
        Ok(())
    }

    pub(super) fn validate_big_int_constant_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(());
        };
        let (name, method) = match identity.as_str() {
            "std/big-int::Zero" => ("Zero", "zero"),
            "std/big-int::One" => ("One", "one"),
            _ => return Ok(()),
        };
        if dispatch.trait_identity != format!("std/prelude::{name}")
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven BigInt constant dispatch".into());
        }
        self.big_int_type(
            actual,
            &CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/prelude::Unit".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Named {
                    name: "std/big-int::BigInt".into(),
                    arguments: vec![],
                }),
            },
            scope,
        )
    }
    pub(super) fn big_int_comparison_binary(
        &mut self,
        operator: &str,
        left: &CoreExpr,
        right: &CoreExpr,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !comparisons::is_comparison(operator) {
            return Ok(None);
        }
        let standard = evidence
            .first()
            .and_then(|selected| match &selected.evidence {
                CoreInstanceEvidence::Standard { identity, .. } => Some(identity.as_str()),
                _ => None,
            });
        if standard != Some("std/big-int::Ord") {
            let big_int_operand = [left, right].iter().any(|argument| {
                let Some(actual) = local::method_argument_type(argument) else { return false; };
                matches!(self.local_canonical_type(&actual, scope), Ok(CoreType::Named { name, arguments }) if name == "std/big-int::BigInt" && arguments.is_empty())
            });
            if (standard.is_some() || evidence.is_empty()) && big_int_operand {
                return Err(
                    "Rust backend: BigInt operand requires its selected canonical Ord provider"
                        .into(),
                );
            }
            return Ok(None);
        }
        let [selected] = evidence else {
            return Err("Rust backend: BigInt comparison dictionary arity".into());
        };
        self.validate_ord_evidence(selected, scope)?;
        self.comparison_result_type(type_ref, 2, evidence, scope)?;
        for argument in [left, right] {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: unproven BigInt comparison argument")?;
            self.big_int_type(
                &actual,
                &CoreType::Named {
                    name: "std/big-int::BigInt".into(),
                    arguments: vec![],
                },
                scope,
            )?;
        }
        let dictionary = self
            .big_int_trait_dictionary(selected, scope)?
            .expect("proved BigInt Ord");
        let stage = self.fresh();
        Ok(Some(format!("{{ let {stage}=({dictionary}).field(\"compare\").apply({}); native_ordering_comparison({}, {stage}.apply({})) }}", self.expr(left,scope)?,quoted(operator),self.expr(right,scope)?)))
    }

    pub(super) fn big_int_display_dictionary(
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
        let method = match identity.as_str() {
            "Show<std/big-int::BigInt>" => "Show",
            "Debug<std/big-int::BigInt>" => "Debug",
            _ => return Ok(None),
        };
        let [actual] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: BigInt Show constraint arity".into());
        };
        if !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || !lists::standard_trait(selected, method, actual)
        {
            return Err("Rust backend: unproven BigInt Show evidence".into());
        }
        self.big_int_type(
            actual,
            &CoreType::Named {
                name: "std/big-int::BigInt".into(),
                arguments: vec![],
            },
            scope,
        )?;
        self.capabilities.insert("big-int".into());
        Ok(Some(format!(
            "native_big_int_display_dictionary({})",
            method == "Debug"
        )))
    }
    pub(super) fn validate_big_int_display_value(
        &self,
        dispatch: &seseragi_lowering::CoreTraitDispatch,
        selected: &CoreCallEvidence,
        actual: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let method = match &selected.evidence {
            CoreInstanceEvidence::Standard { identity, .. }
                if identity == "Show<std/big-int::BigInt>" =>
            {
                "show"
            }
            CoreInstanceEvidence::Standard { identity, .. }
                if identity == "Debug<std/big-int::BigInt>" =>
            {
                "debug"
            }
            _ => return Ok(()),
        };
        let expected = if method == "show" {
            "std/prelude::Show"
        } else {
            "std/prelude::Debug"
        };
        if dispatch.trait_identity != expected
            || dispatch.method != method
            || dispatch.method_contract.is_some()
        {
            return Err("Rust backend: unproven BigInt display dispatch".into());
        }
        self.big_int_type(
            actual,
            &CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/big-int::BigInt".into(),
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
    pub(super) fn validate_big_int_display_call(
        &self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<()> {
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
        if !evidence.iter().any(|selected| matches!(&selected.evidence,CoreInstanceEvidence::Standard {identity,..} if matches!(identity.as_str(),"Show<std/big-int::BigInt>"|"Debug<std/big-int::BigInt>"))) {return Ok(());}
        let [selected] = evidence.as_slice() else {
            return Err("Rust backend: BigInt Show dictionary arity".into());
        };
        let dispatch = trait_dispatch
            .as_ref()
            .ok_or("Rust backend: missing BigInt Show dispatch")?;
        if arguments.len() > 1
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven BigInt Show source stages".into());
        }
        let full = if let Some(argument) = arguments.first() {
            CoreType::Function {
                parameter: Box::new(
                    local::method_argument_type(argument)
                        .ok_or("Rust backend: missing BigInt Show argument type")?,
                ),
                result: Box::new(type_ref.clone()),
            }
        } else {
            type_ref.clone()
        };
        self.validate_big_int_display_value(dispatch, selected, &full, scope)
    }
}
