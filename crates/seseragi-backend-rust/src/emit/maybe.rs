use super::*;
fn arity(name: &str) -> Option<usize> {
    match name {
        "std/maybe::withDefault" | "std/maybe::orElse" | "std/maybe::traverse" => Some(2),
        "std/maybe::sequence" => Some(1),
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
fn apply(constructor: &CoreType, argument: CoreType) -> Result<CoreType> {
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
                .ok_or_else(|| {
                    "Rust backend: unproven Traversable type-function application".into()
                })
        }
        _ => Err("Rust backend: unproven Traversable constructor application".into()),
    }
}
impl Backend<'_> {
    pub(super) fn traversable_supertrait(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<bool> {
        if !instance.identity.starts_with("std/prelude::Traversable<") {
            return Ok(false);
        }
        if self.program_wasm_library {
            return Err("program WASM: native Traversable instance needs an ABI proof".into());
        }
        let [constructor] = instance.arguments.as_slice() else {
            return Err("Rust backend: Traversable instance constructor arity".into());
        };
        let Some(parent) = instance.constraints.first() else {
            return Err("Rust backend: missing canonical Traversable parent".into());
        };
        if instance.trait_name != "Traversable"
            || instance.supertrait_count != 1
            || parent.name != "Functor"
            || parent.trait_identity.as_deref() != Some("std/prelude::Functor")
            || parent.arguments.len() != 1
            || self.declared_method_type_kind(constructor, scope) != Some(1)
            || self.declared_method_type_kind(&parent.arguments[0], scope) != Some(1)
            || self.local_canonical_type(constructor, scope)?
                != self.local_canonical_type(&parent.arguments[0], scope)?
        {
            return Err("Rust backend: unproven canonical Traversable supertrait".into());
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven Traversable implementation".into());
        };
        let [method] = methods.as_slice() else {
            return Err("Rust backend: invalid Traversable method count".into());
        };
        if method.name != "traverse"
            || method.type_parameters.len() != 3
            || method
                .type_parameters
                .iter()
                .map(|p| p.arity)
                .collect::<Vec<_>>()
                != [1, 0, 0]
            || !(1..=2).contains(&method.parameters.len())
            || method.constraints.len() != 1
        {
            return Err("Rust backend: unproven Traversable method source scheme".into());
        }
        let mut method_scope = scope.clone();
        method_scope.type_parameters(
            method
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity)),
            &format!("{}::{}", instance.identity, method.name),
        );
        let g = named(&method.type_parameters[0].name, vec![]);
        let a = named(&method.type_parameters[1].name, vec![]);
        let b = named(&method.type_parameters[2].name, vec![]);
        let selected = &method.constraints[0];
        if selected.name != "Applicative"
            || selected.trait_identity.as_deref() != Some("std/prelude::Applicative")
            || selected.arguments.len() != 1
            || self.local_canonical_type(&selected.arguments[0], &method_scope)?
                != self.local_canonical_type(&g, &method_scope)?
        {
            return Err("Rust backend: unproven Traversable target dictionary".into());
        }
        let expected = function(
            &[
                function(&[a.clone()], apply(&g, b.clone())?),
                apply(constructor, a)?,
            ],
            apply(&g, apply(constructor, b)?)?,
        );
        let result = local::method_argument_type(&method.body)
            .ok_or("Rust backend: missing Traversable method result")?;
        let actual = function(
            &method
                .parameters
                .iter()
                .map(|p| p.type_ref.clone())
                .collect::<Vec<_>>(),
            result,
        );
        self.match_local_type(
            &expected,
            &actual,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            &method_scope,
        )?;
        Ok(true)
    }

    fn validate_traversable_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        let [constructor] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: Traversable dictionary arity".into());
        };
        if selected.constraint.trait_identity.as_deref() != Some("std/prelude::Traversable")
            || !self.valid_constraint_label(&selected.constraint)
            || self.declared_method_type_kind(constructor, scope) != Some(1)
        {
            return Err("Rust backend: unproven canonical Traversable constraint/kind".into());
        }
        self.validate_parameter_evidence(selected, scope)?;
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[selected.constraint.clone()], &[]),
            std::slice::from_ref(selected),
            scope,
        )?;
        if let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        {
            let owner = match identity.as_str() {
                "std/array::Traversable" => "std/prelude::Array",
                "std/list::Traversable" => "std/prelude::List",
                _ => {
                    return Err(
                        "Rust backend: native Traversable provider needs an ABI proof".into(),
                    )
                }
            };
            if !type_arguments.is_empty() || !evidence_arguments.is_empty() {
                return Err("Rust backend: Traversable standard type/child arity mismatch".into());
            }
            let constructor = self.local_canonical_type(constructor, scope)?;
            let expected = named(owner, vec![]);
            let lambda = CoreType::TypeFunction {
                parameter_kinds: vec![0],
                body: Box::new(named(
                    owner,
                    vec![CoreType::TypeFunctionParameter { depth: 0, index: 0 }],
                )),
            };
            if constructor != expected && constructor != lambda {
                return Err("Rust backend: Traversable canonical constructor mismatch".into());
            }
        }
        Ok(())
    }
    pub(super) fn traversable_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard { identity, .. } = &selected.evidence else {
            return Ok(None);
        };
        if selected.constraint.trait_identity.as_deref() != Some("std/prelude::Traversable") {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native Traversable dictionary needs a target-specific ABI proof"
                    .into(),
            );
        }
        self.validate_traversable_evidence(selected, scope)?;
        self.capabilities.insert("traversable".into());
        Ok(Some(format!(
            "native_sequence_traversable_dictionary({})",
            identity == "std/list::Traversable"
        )))
    }
    fn maybe_operation_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native Maybe standard operation needs a target-specific ABI proof"
                    .into(),
            );
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven Maybe source scope".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing Maybe source argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err(
                "Rust backend: Maybe standard call requires value-kind source/result".into(),
            );
        }
        let full = self.local_canonical_type(&full, scope)?;
        let a = named("\0maybe_source_a", vec![]);
        let b = named("\0maybe_source_b", vec![]);
        let maybe = |value| named("std/prelude::Maybe", vec![value]);
        let (parameters, result) = match name {
            "std/maybe::withDefault" | "std/maybe::orElse" => {
                if !evidence.is_empty() {
                    return Err("Rust backend: unexpected Maybe fallback dictionary".into());
                }
                if name.ends_with("::withDefault") {
                    (vec![a.clone(), maybe(a.clone())], a.clone())
                } else {
                    (vec![maybe(a.clone()), maybe(a.clone())], maybe(a.clone()))
                }
            }
            _ => {
                let [selected] = evidence else {
                    return Err(
                        "Rust backend: Maybe traversal requires selected Traversable".into(),
                    );
                };
                self.validate_traversable_evidence(selected, scope)?;
                let constructor =
                    self.local_canonical_type(&selected.constraint.arguments[0], scope)?;
                if name.ends_with("::sequence") {
                    (
                        vec![apply(&constructor, maybe(a.clone()))?],
                        maybe(apply(&constructor, a.clone())?),
                    )
                } else {
                    (
                        vec![
                            function(&[a.clone()], maybe(b.clone())),
                            apply(&constructor, a.clone())?,
                        ],
                        maybe(apply(&constructor, b.clone())?),
                    )
                }
            }
        };
        self.match_local_type(
            &function(&parameters, result),
            &full,
            &BTreeMap::from([
                ("\0maybe_source_a".into(), 0),
                ("\0maybe_source_b".into(), 0),
            ]),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(arity(name).expect("known Maybe standard operation"))
    }

    fn maybe_operation_callable(
        &mut self,
        name: &str,
        arity: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let dictionary = "\0maybe_dictionaries".to_owned();
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0maybe_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "{}({}, native_map_dictionaries(&{}), vec![{args}])",
            "native_maybe_operation",
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

    pub(super) fn maybe_operation_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if arity(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        let arity = self.maybe_operation_type(name, &[], type_ref, evidence, scope)?;
        self.capabilities.insert("maybe-operations".into());
        Ok(Some(
            self.maybe_operation_callable(name, arity, evidence, scope)?,
        ))
    }

    pub(super) fn maybe_operation_call(
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
                        "std/maybe::sequence" | "std/maybe::traverse"
                    )
                    || deferred_evidence_parameters.is_empty()
                    || arguments.len() >= arity(callee).unwrap()))
            || *evidence_argument_index
                != (arguments.len() > arity(callee).unwrap()).then_some(arity(callee).unwrap())
        {
            return Err("Rust backend: unproven collection source stages".into());
        }
        let arity = self.maybe_operation_type(callee, arguments, type_ref, evidence, scope)?;
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
        self.capabilities.insert("maybe-operations".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            let dictionaries = self.evidence(evidence, scope)?;
            return Ok(Some(format!(
                "{}({}, {dictionaries}, vec![{}])",
                "native_maybe_operation",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let callable = self.maybe_operation_callable(callee, arity, evidence, scope)?;
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
