use super::*;

impl Backend<'_> {
    pub(super) fn array_index(&mut self, expression: &CoreExpr, scope: &Scope) -> Result<String> {
        let (receiver, offset) = self.array_index_reads(expression, scope)?;
        Ok(format!(
            "{{ {receiver} {offset} native_array_index({}, {}) }}",
            receiver.0, offset.0
        ))
    }

    /// The same read for a match that tests it at once: the element stays an
    /// `Option<V>`, with the location a Defect inside the read reports.
    pub(super) fn array_index_option(
        &mut self,
        expression: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        let (receiver, offset) = self.array_index_reads(expression, scope)?;
        let value = format!(
            "{{ {receiver} {offset} native_array_get({}, {}) }}",
            receiver.0, offset.0
        );
        Ok(match self.source_location(core_expr_origin(expression)) {
            Some(source) => format!("native_source({source}, || {value})"),
            None => value,
        })
    }

    fn array_index_reads(&mut self, expression: &CoreExpr, scope: &Scope) -> Result<(Read, Read)> {
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
        } = expression
        else {
            unreachable!()
        };
        if scope.has_unproven_type_scope()
            || scope.contains(callee)
            || arguments.len() != 2
            || !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven Array index intrinsic ABI".into());
        }
        let receiver = local::method_argument_type(&arguments[0])
            .ok_or("Rust backend: missing Array index receiver type")?;
        let offset = local::method_argument_type(&arguments[1])
            .ok_or("Rust backend: missing Array index offset type")?;
        let receiver = self.local_canonical_type(&receiver, scope)?;
        let offset = self.local_canonical_type(&offset, scope)?;
        let result = self.local_canonical_type(type_ref, scope)?;
        let (
            CoreType::Named {
                name: array,
                arguments: elements,
            },
            CoreType::Named {
                name: int,
                arguments: int_arguments,
            },
            CoreType::Named {
                name: maybe,
                arguments: results,
            },
        ) = (&receiver, &offset, &result)
        else {
            return Err("Rust backend: unproven Array index types".into());
        };
        if array != "std/prelude::Array"
            || elements.len() != 1
            || int != "std/prelude::Int"
            || !int_arguments.is_empty()
            || maybe != "std/prelude::Maybe"
            || results != elements
        {
            return Err("Rust backend: Array index canonical type or element mismatch".into());
        }
        // This syntax intrinsic has a receiver-first ABI. Keep both reads at
        // their original evaluation position, once each, before selecting an
        // element. It is not the index-first, curried std/array::get function.
        let receiver_name = self.fresh();
        let offset_name = self.fresh();
        let receiver = self.expr(&arguments[0], scope)?;
        let offset = self.expr(&arguments[1], scope)?;
        self.capabilities.insert("array-index".into());
        self.direct_calls += 1;
        Ok((Read(receiver_name, receiver), Read(offset_name, offset)))
    }
}

/// A value read once into a named local, in source order.
struct Read(String, String);

impl std::fmt::Display for Read {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "let {} = {};", self.0, self.1)
    }
}
