use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/iterator::unfold" => Some(2),
        "std/iterator::next" => Some(1),
        _ => None,
    }
}

fn function(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

impl Backend<'_> {
    fn iterator_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Iterator operation requires a target-specific ABI proof"
                    .into(),
            );
        }
        let arity = arity(name).expect("known Iterator operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven Iterator source application".into());
        }
        // Reconstruct the complete source signature even for a saved reference
        // or partial application. Canonicalization preserves lexical binder
        // identities; spelling Iterator/Int/Bool/Maybe cannot certify their ABI.
        let parameters = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Iterator argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&parameters, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven Iterator argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remainder = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remainder else {
                return Err("Rust backend: unproven Iterator source arity".into());
            };
            parameters.push(parameter.as_ref());
            remainder = result;
        }
        let named = |name: &str, arguments: Vec<CoreType>| CoreType::Named {
            name: format!("std/prelude::{name}"),
            arguments,
        };
        let iterator_element = |ty: &CoreType| -> Result<CoreType> {
            match ty {
                CoreType::Named { name, arguments }
                    if name == "std/prelude::Iterator" && arguments.len() == 1 =>
                {
                    Ok(arguments[0].clone())
                }
                _ => Err("Rust backend: unproven canonical Iterator owner or arity".into()),
            }
        };
        let (expected, result) = if name == "std/iterator::unfold" {
            let element = iterator_element(remainder)?;
            let state = parameters[1].clone();
            let step = function(
                &[state.clone()],
                named(
                    "Maybe",
                    vec![CoreType::Tuple {
                        elements: vec![element.clone(), state.clone()],
                    }],
                ),
            );
            (vec![step, state], named("Iterator", vec![element]))
        } else {
            let element = iterator_element(parameters[0])?;
            let iterator = named("Iterator", vec![element.clone()]);
            (
                vec![iterator.clone()],
                named(
                    "Maybe",
                    vec![CoreType::Tuple {
                        elements: vec![element, iterator],
                    }],
                ),
            )
        };
        if full != function(&expected, result) {
            return Err(
                "Rust backend: Iterator operation canonical type or element mismatch".into(),
            );
        }
        Ok(arity)
    }

    fn iterator_operation_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0iterator_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_iterator_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn iterator_operation_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: Iterator operation with unsupported evidence".into());
        }
        let arity = self.iterator_operation_type(name, &[], type_ref, scope)?;
        self.capabilities.insert("iterator".into());
        Ok(Some(Self::iterator_operation_callable(name, arity)))
    }

    pub(super) fn iterator_operation_call(
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
        if arity(callee).is_none() || scope.contains(callee) {
            return Ok(None);
        }
        if !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Iterator source application".into());
        }
        let arity = self.iterator_operation_type(callee, arguments, type_ref, scope)?;
        self.capabilities.insert("iterator".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_iterator_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::iterator_operation_callable(callee, arity)
        );
        for argument in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(Some(output))
    }
}
