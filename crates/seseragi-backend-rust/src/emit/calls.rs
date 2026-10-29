//! Keep call validation and emission off the recursive expression dispatch
//! frame, including when the expression being emitted is a deep binary tree.
use super::*;

impl Backend<'_> {
    pub(super) fn call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            type_ref,
            ..
        } = expr
        else {
            unreachable!("call dispatch");
        };
        if callee == "std/float::power" && !scope.contains(callee) {
            if !evidence.is_empty()
                || trait_dispatch.is_some()
                || !deferred_evidence_parameters.is_empty()
                || !deferred_evidence_type_constructor_parameters.is_empty()
                || evidence_argument_index.is_some()
            {
                return Err("Rust backend: unproven Float power evidence or staging".into());
            }
            self.validate_float_power_type(type_ref, arguments.len(), scope)?;
            for argument in arguments {
                let actual = local::method_argument_type(argument)
                    .ok_or("Rust backend: unproven Float power argument type")?;
                self.validate_float_power_type(&actual, 2, scope)?;
            }
        }
        if let Some(value) = self.clock_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.console_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.die_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.concurrency_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.from_either_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.filesystem_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.text_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.bytes_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.big_int_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.collection_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.array_operation_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.math_call(expr, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.number_parse_call(expr, scope)? {
            return Ok(value);
        }
        if matches!(callee.as_str(), "<$>" | "<*>" | ">>=") && !scope.contains(callee) {
            return Err("Rust backend: unproven trait operator source staging".into());
        }
        if callee == "builtin::array-index" {
            return self.array_index(expr, scope);
        }
        if comparisons::is_comparison(callee) && !scope.contains(callee) {
            return self.comparison_call(expr, scope);
        }
        if equality::is_equality(callee) && !scope.contains(callee) {
            return self.equality_call(expr, scope);
        }
        if trait_dispatch.is_none() && scope.callables.contains_key(callee) {
            return self.local_call(expr, scope);
        }
        if trait_dispatch.is_none() && !scope.contains(callee) {
            if let Some(factory) = self.global_method_factory(callee)? {
                return self.call_local_value(expr, &factory, scope);
            }
        }
        if let Some(dispatch) = trait_dispatch {
            self.validate_hash_call(expr, scope)?;
            if dispatch.trait_identity == "std/prelude::Eq" {
                return self.equality_call(expr, scope);
            }
            self.validate_int_parse_show_call(expr, scope)?;
            self.validate_primitive_operator_call(expr, scope)?;
            if dispatch.method_contract.is_some() {
                return self.method_call(expr, scope);
            }
            self.reject_missing_method_contract(dispatch, evidence)?;
            if arguments.is_empty() {
                self.validate_bare_method(dispatch, evidence, type_ref, scope)?;
            }
        }
        let bound_primitive_partial = trait_dispatch.is_none()
            && !scope.contains(callee)
            && (characters::closed_character_partial(
                self.module,
                callee,
                arguments.len(),
                evidence,
                deferred_evidence_parameters,
            ) || floats::closed_float_partial(
                self.module,
                callee,
                arguments.len(),
                evidence,
                deferred_evidence_parameters,
            ) || display::closed_display_partial(
                self.module,
                callee,
                arguments.len(),
                evidence,
                deferred_evidence_parameters,
            ));
        let returned_evidence_call = self.validate_returned_evidence_call(expr, scope)?;
        if (!deferred_evidence_parameters.is_empty() && !bound_primitive_partial)
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || (evidence_argument_index.is_some()
                && !evidence.is_empty()
                && !returned_evidence_call)
        {
            return Err("Rust backend: deferred callable evidence is unsupported".into());
        }
        if trait_dispatch.is_none()
            && !scope.contains(callee)
            && evidence.is_empty()
            && deferred_evidence_parameters.is_empty()
            && evidence_argument_index.is_none()
            && lists::list_operation_arity(callee) == Some(arguments.len())
        {
            self.capabilities.insert("list".into());
            self.direct_calls += 1;
            return Ok(format!(
                "native_list_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            ));
        }
        if trait_dispatch.is_none()
            && !scope.contains(callee)
            && evidence.is_empty()
            && deferred_evidence_parameters.is_empty()
            && evidence_argument_index.is_none()
            && floats::float_operation_arity(callee) == Some(arguments.len())
        {
            self.capabilities.insert("float".into());
            self.direct_calls += 1;
            return Ok(format!(
                "native_float_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            ));
        }
        if trait_dispatch.is_none()
            && !scope.contains(callee)
            && evidence.is_empty()
            && deferred_evidence_parameters.is_empty()
            && evidence_argument_index.is_none()
            && arguments.len() == 1
        {
            if let Some(function) = characters::character_operation(callee) {
                self.direct_calls += 1;
                return Ok(format!("{function}({})", self.expr(&arguments[0], scope)?));
            }
        }
        if trait_dispatch.is_none()
            && !scope.contains(callee)
            && known_pure_call_shape(self.module, callee, arguments.len())
                == Some(CoreCallShape::Saturated)
        {
            let dictionaries = self.evidence(evidence, scope)?;
            let args = self.expressions(arguments, scope)?;
            self.direct_calls += 1;
            return Ok(format!(
                "{}_direct({dictionaries}{}{})",
                id(callee),
                if args.is_empty() { "" } else { "," },
                args
            ));
        }
        let function = if let Some(dispatch) = trait_dispatch {
            if evidence.len() != 1 {
                return Err("Rust backend: trait dispatch requires one dictionary".into());
            }
            if let CoreInstanceEvidence::Parameter { index } = evidence[0].evidence {
                self.validate_parameter_evidence(&evidence[0], scope)?;
                if scope.dictionaries[index].trait_identity.as_deref()
                    != Some(dispatch.trait_identity.as_str())
                {
                    return Err(
                        "Rust backend: dictionary parameter dispatch identity mismatch".into(),
                    );
                }
            }
            if !floats::valid_float_dispatch(
                &evidence[0],
                &dispatch.trait_identity,
                &dispatch.method,
            ) {
                return Err("Rust backend: mismatched canonical Float trait dispatch".into());
            }
            if !lists::valid_display_dispatch(
                &evidence[0],
                &dispatch.trait_identity,
                &dispatch.method,
            ) {
                return Err("Rust backend: mismatched canonical display dispatch".into());
            }
            let dictionary = self.evidence(evidence, scope)?;
            format!(
                "({dictionary}).remove(0).field({})",
                quoted(&dispatch.method)
            )
        } else {
            self.reference(callee, evidence, scope)?
        };
        // Apply each stage before evaluating the next source argument.
        let temporary = self.fresh();
        let mut value = format!("{{ let mut {temporary} = {function};");
        for arg in arguments {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(arg, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(value)
    }
}
