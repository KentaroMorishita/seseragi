use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/array::length" => Some(1),
        "std/array::filter" | "std/array::find" | "std/array::get" | "std/array::take"
        | "std/array::drop" => Some(2),
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
    fn array_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Array operation requires a target-specific ABI proof".into(),
            );
        }
        let arity = arity(name).expect("known Array operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven Array source application".into());
        }
        // Reconstruct the complete source signature even for a saved reference
        // or partial application. Canonicalization preserves lexical binder
        // identities; spelling Array/Int/Bool/Maybe cannot certify their ABI.
        let parameters = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Array argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&parameters, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven Array argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remainder = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remainder else {
                return Err("Rust backend: unproven Array source arity".into());
            };
            parameters.push(parameter.as_ref());
            remainder = result;
        }
        let CoreType::Named {
            name: array,
            arguments: elements,
        } = parameters[arity - 1]
        else {
            return Err("Rust backend: unproven canonical Array parameter".into());
        };
        if array != "std/prelude::Array" || elements.len() != 1 {
            return Err("Rust backend: unproven canonical Array owner or arity".into());
        }
        let named = |name: &str, arguments: Vec<CoreType>| CoreType::Named {
            name: format!("std/prelude::{name}"),
            arguments,
        };
        let array = named("Array", elements.clone());
        let int = named("Int", vec![]);
        let predicate = function(&elements[..1], named("Bool", vec![]));
        let (expected, result) = match name {
            "std/array::filter" => (vec![predicate, array.clone()], array),
            "std/array::find" => (vec![predicate, array], named("Maybe", elements.clone())),
            "std/array::get" => (vec![int, array], named("Maybe", elements.clone())),
            "std/array::take" | "std/array::drop" => (vec![int, array.clone()], array),
            "std/array::length" => (vec![array], int),
            _ => unreachable!(),
        };
        if full != function(&expected, result) {
            return Err("Rust backend: Array operation canonical type or element mismatch".into());
        }
        Ok(arity)
    }

    fn array_operation_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0array_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_array_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn array_operation_reference(
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
            return Err("Rust backend: Array operation with unsupported evidence".into());
        }
        let arity = self.array_operation_type(name, &[], type_ref, scope)?;
        self.capabilities.insert("array-operations".into());
        Ok(Some(Self::array_operation_callable(name, arity)))
    }

    pub(super) fn array_operation_call(
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
            return Err("Rust backend: unproven Array source application".into());
        }
        let arity = self.array_operation_type(callee, arguments, type_ref, scope)?;
        self.capabilities.insert("array-operations".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_array_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::array_operation_callable(callee, arity)
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
