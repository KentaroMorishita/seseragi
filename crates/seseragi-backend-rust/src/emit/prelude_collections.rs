use super::*;
fn arity(name: &str) -> Option<usize> {
    match name {
        "std/prelude::sum" => Some(1),
        "std/prelude::join" | "std/prelude::forEach" => Some(2),
        _ => None,
    }
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: format!("std/prelude::{name}"),
        arguments,
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
    pub(super) fn reducible_supertrait(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<bool> {
        if !instance.identity.starts_with("std/prelude::Reducible<") {
            return Ok(false);
        }
        if self.program_wasm_library {
            return Err("program WASM: native Reducible instance needs an ABI proof".into());
        }
        let Some(parent) = instance.constraints.first() else {
            return Err("Rust backend: missing Reducible parent".into());
        };
        if instance.trait_name != "Reducible"
            || instance.supertrait_count != 1
            || instance.arguments.len() != 2
            || parent.arguments.len() != 2
            || parent.name != "Iterable"
            || parent.trait_identity.as_deref() != Some("std/prelude::Iterable")
        {
            return Err("Rust backend: unproven canonical Reducible supertrait".into());
        }
        for (argument, inherited) in instance.arguments.iter().zip(&parent.arguments) {
            if self.declared_method_type_kind(argument, scope) != Some(0)
                || self.local_canonical_type(argument, scope)?
                    != self.local_canonical_type(inherited, scope)?
            {
                return Err("Rust backend: Reducible parent type/kind mismatch".into());
            }
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven Reducible implementation".into());
        };
        let [method] = methods.as_slice() else {
            return Err("Rust backend: Reducible method count".into());
        };
        if method.name != "reduce"
            || method.type_parameters.len() != 1
            || method.type_parameters[0].arity != 0
            || !method.constraints.is_empty()
            || !(1..=3).contains(&method.parameters.len())
        {
            return Err("Rust backend: unproven Reducible method scheme".into());
        }
        let mut method_scope = scope.clone();
        method_scope.type_parameters(
            method
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &format!("{}::{}", instance.identity, method.name),
        );
        let accumulator = CoreType::Named {
            name: method.type_parameters[0].name.clone(),
            arguments: vec![],
        };
        let actual = function(
            &method
                .parameters
                .iter()
                .map(|p| p.type_ref.clone())
                .collect::<Vec<_>>(),
            local::method_argument_type(&method.body)
                .ok_or("Rust backend: missing Reducible result type")?,
        );
        let expected = function(
            &[
                accumulator.clone(),
                function(
                    &[accumulator.clone(), instance.arguments[1].clone()],
                    accumulator.clone(),
                ),
                instance.arguments[0].clone(),
            ],
            accumulator,
        );
        if self.declared_method_type_kind(&actual, &method_scope) != Some(0)
            || self.local_canonical_type(&actual, &method_scope)?
                != self.local_canonical_type(&expected, &method_scope)?
        {
            return Err("Rust backend: Reducible method signature mismatch".into());
        }
        Ok(true)
    }

    fn validate_prelude_constraint(
        &self,
        selected: &CoreCallEvidence,
        name: &str,
        arguments: &[CoreType],
        scope: &Scope,
    ) -> Result<()> {
        if selected.constraint.name != name
            || selected.constraint.trait_identity.as_deref()
                != Some(format!("std/prelude::{name}").as_str())
            || selected.constraint.arguments.len() != arguments.len()
        {
            return Err("Rust backend: Prelude collection constraint mismatch".into());
        }
        for (actual, expected) in selected.constraint.arguments.iter().zip(arguments) {
            if self.declared_method_type_kind(actual, scope) != Some(0)
                || self.local_canonical_type(actual, scope)? != *expected
            {
                return Err("Rust backend: Prelude collection evidence type/kind mismatch".into());
            }
        }
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
            std::slice::from_ref(selected),
            scope,
        )?;
        Ok(())
    }

    fn validate_prelude_reducible(&self, selected: &CoreCallEvidence, scope: &Scope) -> Result<()> {
        let arguments = selected
            .constraint
            .arguments
            .iter()
            .map(|ty| self.local_canonical_type(ty, scope))
            .collect::<Result<Vec<_>>>()?;
        if arguments.len() != 2 {
            return Err("Rust backend: Reducible collection/element arity mismatch".into());
        }
        self.validate_prelude_constraint(selected, "Reducible", &arguments, scope)?;
        if let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence {
            // Reducible and Iterable have identical collection/element heads
            // for these native providers. Reuse their checked ABI head proof.
            let mut iterable = selected.clone();
            iterable.constraint.name = "Iterable".into();
            iterable.constraint.trait_identity = Some("std/prelude::Iterable".into());
            if let CoreInstanceEvidence::Standard {
                identity: provider, ..
            } = &mut iterable.evidence
            {
                *provider = match identity.as_str() {
                    "std/array::Reducible" => "std/array::Iterable",
                    "std/list::Reducible" => "std/list::Iterable",
                    "std/map::Reducible" => "std/map::Iterable",
                    "std/set::Reducible" => "std/set::Iterable",
                    _ => {
                        return Err(
                            "Rust backend: native Reducible provider needs an ABI proof".into()
                        )
                    }
                }
                .into();
            }
            self.validate_iterable_evidence(&iterable, scope)?;
        }
        Ok(())
    }

    pub(super) fn prelude_collection_dictionary(
        &self,
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
        if selected.constraint.trait_identity.as_deref() == Some("std/prelude::Reducible") {
            if self.program_wasm_library {
                return Err(
                    "program WASM: native Reducible requires a target-specific ABI proof".into(),
                );
            }
            self.validate_prelude_reducible(selected, scope)?;
            return Ok(Some(format!(
                "native_prelude_reducible_dictionary({})",
                quoted(identity)
            )));
        }
        if identity == "std/int::Zero" {
            if self.program_wasm_library {
                return Err(
                    "program WASM: native Int Zero requires a target-specific ABI proof".into(),
                );
            }
            self.validate_prelude_constraint(selected, "Zero", &[named("Int", vec![])], scope)?;
            if !type_arguments.is_empty() || !evidence_arguments.is_empty() {
                return Err("Rust backend: Int Zero provider arity mismatch".into());
            }
            return Ok(Some(
                "V::dict(vec![(\"zero\", V::fun(vec![], |_, _| V::Int(0)))])".into(),
            ));
        }
        Ok(None)
    }

    fn prelude_collection_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Prelude collection requires a target-specific ABI proof"
                    .into(),
            );
        }
        let arity = arity(name).expect("known Prelude collection");
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven Prelude collection source stage".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Prelude collection argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err(
                "Rust backend: Prelude collection requires value-kind source/result".into(),
            );
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remaining = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: missing Prelude collection source arity".into());
            };
            parameters.push(parameter.as_ref().clone());
            remaining = result;
        }
        let first = evidence
            .first()
            .ok_or("Rust backend: Prelude collection requires selected evidence")?;
        let iterable = name == "std/prelude::forEach";
        if iterable {
            self.validate_iterable_evidence(first, scope)?;
        } else {
            self.validate_prelude_reducible(first, scope)?;
        }
        let collection = self.local_canonical_type(&first.constraint.arguments[0], scope)?;
        let element = self.local_canonical_type(&first.constraint.arguments[1], scope)?;
        let (expected, output) = match name {
            "std/prelude::sum" => {
                if evidence.len() != 3 {
                    return Err("Rust backend: sum requires Reducible/Zero/Add".into());
                }
                for (selected, trait_name, arguments) in [
                    (&evidence[1], "Zero", vec![element.clone()]),
                    (&evidence[2], "Add", vec![element.clone(); 3]),
                ] {
                    self.validate_prelude_constraint(selected, trait_name, &arguments, scope)?;
                }
                (vec![collection], element)
            }
            "std/prelude::join" => {
                if evidence.len() != 1 || element != named("String", vec![]) {
                    return Err("Rust backend: join requires Reducible<C,String>".into());
                }
                (
                    vec![named("String", vec![]), collection],
                    named("String", vec![]),
                )
            }
            _ => {
                if evidence.len() != 1 {
                    return Err("Rust backend: forEach requires one Iterable dictionary".into());
                }
                let effect = remaining.clone();
                if !matches!(&effect, CoreType::Named { name, arguments } if name == "std/prelude::Effect" && arguments.len() == 3 && arguments[2] == named("Unit", vec![]))
                {
                    return Err("Rust backend: forEach requires an Effect<R,E,Unit> result".into());
                }
                (
                    vec![function(&[element], effect.clone()), collection],
                    effect,
                )
            }
        };
        if full != function(&expected, output) {
            return Err(
                "Rust backend: Prelude collection canonical type, accumulator or source mismatch"
                    .into(),
            );
        }
        Ok(arity)
    }

    fn prelude_collection_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0prelude_collection_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0prelude_collection_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_prelude_collection",
            quoted(name),
            id(&dictionary)
        );
        for i in (0..arity).rev() {
            let mut captures: Scope = names[..i].iter().cloned().collect();
            captures.insert(dictionary.clone());
            body = Self::closure(&names[i], &body, &captures);
        }
        let dictionaries = self.evidence(evidence, scope)?;
        Ok(format!(
            "{{ let {}=V::Tuple(Shared::new({dictionaries})); {body} }}",
            id(&dictionary)
        ))
    }

    pub(super) fn prelude_collection_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.prelude_collection_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("prelude-collections".into());
        Ok(Some(self.prelude_collection_callable(
            name, arity, evidence, scope,
        )?))
    }

    pub(super) fn prelude_collection_call(
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
        if trait_dispatch.is_some()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || *evidence_argument_index
                != (arguments.len() > arity(callee).unwrap()).then_some(arity(callee).unwrap())
        {
            return Err("Rust backend: unproven collection source stages".into());
        }
        let arity = self.prelude_collection_type(callee, arguments, type_ref, evidence, scope)?;
        // A generic partial application can retain its remaining source types.
        // Its selected dictionaries are already checked in the caller's scope;
        // the metadata must describe exactly the ordinary curried suffix.
        if !deferred_evidence_parameters.is_empty() {
            if arguments.len() + deferred_evidence_parameters.len() != arity {
                return Err("Rust backend: collection deferred source arity mismatch".into());
            }
            let mut remaining = type_ref;
            for deferred in deferred_evidence_parameters {
                let CoreType::Function { parameter, result } = remaining else {
                    return Err("Rust backend: missing collection deferred source stage".into());
                };
                if self.local_canonical_type(parameter, scope)?
                    != self.local_canonical_type(deferred, scope)?
                {
                    return Err("Rust backend: collection deferred source type mismatch".into());
                }
                remaining = result;
            }
        }
        self.capabilities.insert("prelude-collections".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_prelude_collection",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.prelude_collection_callable(callee, arity, evidence, scope)?;
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
