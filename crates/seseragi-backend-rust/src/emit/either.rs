use super::*;
fn arity(name: &str) -> Option<usize> {
    match name {
        "std/either::mapLeft" | "std/either::mapRight" | "std/either::traverse" => Some(2),
        "std/either::bimap" | "std/either::fold" => Some(3),
        "std/either::swap" | "std/either::sequence" => Some(1),
        _ => None,
    }
}
fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
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
fn applied(constructor: &CoreType, argument: CoreType) -> Result<CoreType> {
    match constructor {
        CoreType::Named { .. } | CoreType::ExternalNamed { .. } => {
            let mut result = constructor.clone();
            match &mut result {
                CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. } => {
                    arguments.push(argument)
                }
                _ => unreachable!(),
            };
            Ok(result)
        }
        CoreType::TypeFunction { .. } => {
            seseragi_syntax::type_functions::apply_type_function(constructor, &[argument])
                .ok_or_else(|| "Rust backend: unproven Either Traversable application".into())
        }
        _ => Err("Rust backend: unproven Either Traversable constructor".into()),
    }
}
impl Backend<'_> {
    fn either_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Either operation needs a target-specific ABI proof".into(),
            );
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven Either source scope".into());
        }
        let supplied = arguments
            .iter()
            .map(|a| {
                local::method_argument_type(a)
                    .ok_or_else(|| "Rust backend: missing Either source argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: Either source/result must have value kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let e = named("\0either_error", vec![]);
        let f = named("\0either_mapped_error", vec![]);
        let a = named("\0either_value", vec![]);
        let b = named("\0either_mapped_value", vec![]);
        let either = |error, value| named("std/prelude::Either", vec![error, value]);
        let input = either(e.clone(), a.clone());
        if !matches!(name, "std/either::sequence" | "std/either::traverse") && !evidence.is_empty()
        {
            return Err("Rust backend: unexpected Either transformation dictionary".into());
        }
        let (parameters, result) = match name {
            "std/either::mapLeft" => (
                vec![function(&[e.clone()], f.clone()), input],
                either(f.clone(), a.clone()),
            ),
            "std/either::mapRight" => (
                vec![function(&[a.clone()], b.clone()), input],
                either(e.clone(), b.clone()),
            ),
            "std/either::bimap" => (
                vec![
                    function(&[e.clone()], f.clone()),
                    function(&[a.clone()], b.clone()),
                    input,
                ],
                either(f.clone(), b.clone()),
            ),
            "std/either::fold" => (
                vec![
                    function(&[e.clone()], b.clone()),
                    function(&[a.clone()], b.clone()),
                    input,
                ],
                b.clone(),
            ),
            "std/either::swap" => (vec![input], either(a.clone(), e.clone())),
            _ => {
                let [selected] = evidence else {
                    return Err("Rust backend: Either traversal needs selected Traversable".into());
                };
                self.validate_traversable_evidence(selected, scope)?;
                let constructor =
                    self.local_canonical_type(&selected.constraint.arguments[0], scope)?;
                if name.ends_with("::sequence") {
                    (
                        vec![applied(&constructor, input)?],
                        either(e.clone(), applied(&constructor, a.clone())?),
                    )
                } else {
                    (
                        vec![
                            function(&[a.clone()], either(e.clone(), b.clone())),
                            applied(&constructor, a.clone())?,
                        ],
                        either(e.clone(), applied(&constructor, b.clone())?),
                    )
                }
            }
        };
        self.match_local_type(
            &function(&parameters, result),
            &full,
            &BTreeMap::from([
                ("\0either_error".into(), 0),
                ("\0either_mapped_error".into(), 0),
                ("\0either_value".into(), 0),
                ("\0either_mapped_value".into(), 0),
            ]),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(arity(name).expect("known Either operation"))
    }

    fn either_operation_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0either_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0either_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_either_operation",
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

    pub(super) fn either_operation_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.either_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("either-operations".into());
        Ok(Some(
            self.either_operation_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn either_operation_call(
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
            || (!deferred_evidence_type_constructor_parameters.is_empty()
                && (deferred_evidence_type_constructor_parameters != &["F".to_owned()]
                    || !matches!(
                        callee.as_str(),
                        "std/either::sequence" | "std/either::traverse"
                    )
                    || deferred_evidence_parameters.is_empty()
                    || arguments.len() >= arity(callee).unwrap()))
            || *evidence_argument_index
                != (arguments.len() > arity(callee).unwrap()).then_some(arity(callee).unwrap())
        {
            return Err("Rust backend: unproven collection source stages".into());
        }
        let arity = self.either_operation_type(callee, arguments, type_ref, evidence, scope)?;
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
        self.capabilities.insert("either-operations".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_either_operation",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.either_operation_callable(callee, arity, evidence, scope)?;
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
