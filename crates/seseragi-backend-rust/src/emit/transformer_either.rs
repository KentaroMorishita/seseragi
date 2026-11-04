use super::*;

fn supported(name: &str) -> bool {
    matches!(
        name,
        "std/transformer/either::run"
            | "std/transformer/either::fromEither"
            | "std/transformer/either::lift"
    )
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn applied(constructor: &CoreType, argument: CoreType) -> Result<CoreType> {
    let mut applied = constructor.clone();
    match &mut applied {
        CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
            arguments.push(argument);
            Ok(applied)
        }
        CoreType::TypeFunction { .. } => {
            seseragi_syntax::type_functions::apply_type_function(constructor, &[argument])
                .ok_or_else(|| "Rust backend: unproven EitherT base application".into())
        }
        _ => Err("Rust backend: unproven EitherT base constructor".into()),
    }
}

impl Backend<'_> {
    fn either_transformer_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native EitherT operation needs a target-specific ABI proof".into(),
            );
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven EitherT source scope".into());
        }
        let mut full = type_ref.clone();
        for argument in arguments.iter().rev() {
            full = CoreType::Function {
                parameter: Box::new(
                    local::method_argument_type(argument)
                        .ok_or("Rust backend: missing EitherT source argument type")?,
                ),
                result: Box::new(full),
            };
        }
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: EitherT source/result must have value kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let CoreType::Function { parameter, result } = &full else {
            return Err("Rust backend: missing EitherT source stage".into());
        };
        let run = name.ends_with("::run");
        let CoreType::Named {
            name: owner,
            arguments: slots,
        } = (if run { &**parameter } else { &**result })
        else {
            return Err("Rust backend: unproven EitherT owner".into());
        };
        if owner != "std/transformer/either::EitherT" || slots.len() != 3 {
            return Err("Rust backend: EitherT owner/arity mismatch".into());
        }
        let constructor = &slots[1];
        if self.declared_method_type_kind(constructor, scope) != Some(1) {
            return Err("Rust backend: EitherT base must have unary constructor kind".into());
        }
        let error = named("\0either_t_error", vec![]);
        let value = named("\0either_t_value", vec![]);
        let transformed = named(
            "std/transformer/either::EitherT",
            vec![error.clone(), constructor.clone(), value.clone()],
        );
        let variables = BTreeMap::from([
            ("\0either_t_error".into(), 0),
            ("\0either_t_value".into(), 0),
        ]);
        let either = named("std/prelude::Either", vec![error, value.clone()]);
        let (parameter, result) = match name {
            "std/transformer/either::run" => (transformed, applied(constructor, either)?),
            "std/transformer/either::fromEither" => (either, transformed),
            "std/transformer/either::lift" => (applied(constructor, value)?, transformed),
            _ => unreachable!(),
        };
        self.match_local_type(
            &CoreType::Function {
                parameter: Box::new(parameter),
                result: Box::new(result),
            },
            &full,
            &variables,
            &mut BTreeMap::new(),
            scope,
        )?;
        if run {
            if !evidence.is_empty() {
                return Err("Rust backend: unexpected EitherT run dictionary".into());
            }
        } else {
            let [selected] = evidence else {
                return Err("Rust backend: EitherT operation needs selected Monad".into());
            };
            let [selected_base] = selected.constraint.arguments.as_slice() else {
                return Err("Rust backend: EitherT Monad dictionary arity mismatch".into());
            };
            if selected.constraint.name != "Monad"
                || selected.constraint.trait_identity.as_deref() != Some("std/prelude::Monad")
                || self.declared_method_type_kind(selected_base, scope) != Some(1)
                || self.local_canonical_type(selected_base, scope)? != *constructor
            {
                return Err("Rust backend: EitherT base Monad mismatch".into());
            }
            self.validate_local_evidence(
                &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
                evidence,
                scope,
            )?;
        }
        Ok(())
    }

    fn either_transformer_callable(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionaries = self.evidence(evidence, scope)?;
        Ok(format!(
            "V::fun({dictionaries}, |dictionaries, value| native_either_transformer_operation({}, dictionaries.to_vec(), value))",
            quoted(name)
        ))
    }

    pub(super) fn either_transformer_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !supported(name) || scope.contains(name) {
            return Ok(None);
        }
        self.either_transformer_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("transformer-either".into());
        Ok(Some(
            self.either_transformer_callable(name, evidence, scope)?,
        ))
    }

    pub(super) fn either_transformer_call(
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
        if !supported(callee) || scope.contains(callee) {
            return Ok(None);
        }
        if trait_dispatch.is_some()
            || *evidence_argument_index != (arguments.len() > 1).then_some(1)
            || (!deferred_evidence_type_constructor_parameters.is_empty()
                && (deferred_evidence_type_constructor_parameters != &["M".to_owned()]
                    || callee.ends_with("::run")
                    || !arguments.is_empty()
                    || deferred_evidence_parameters.len() != 1))
        {
            return Err("Rust backend: unproven EitherT source stages".into());
        }
        self.either_transformer_type(callee, arguments, type_ref, evidence, scope)?;
        if !deferred_evidence_parameters.is_empty() {
            let CoreType::Function { parameter, .. } = type_ref else {
                return Err("Rust backend: missing EitherT deferred source stage".into());
            };
            if !arguments.is_empty()
                || deferred_evidence_parameters.len() != 1
                || self.local_canonical_type(parameter, scope)?
                    != self.local_canonical_type(&deferred_evidence_parameters[0], scope)?
            {
                return Err("Rust backend: EitherT deferred source type mismatch".into());
            }
        }
        self.capabilities.insert("transformer-either".into());
        if arguments.len() == 1 {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "native_either_transformer_operation({}, {dictionaries}, {})",
                quoted(callee),
                self.expr(&arguments[0], scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.either_transformer_callable(callee, evidence, scope)?;
        let mut body = format!("{{ let mut {value}={callable};");
        for argument in arguments {
            body.push_str(&format!(
                "{value}={value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        body.push_str(&format!("{value} }}"));
        Ok(Some(body))
    }
}
