use super::*;

fn arity(name: &str) -> Option<usize> {
    match name {
        "std/non-empty-list::singleton"
        | "std/non-empty-list::fromList"
        | "std/non-empty-list::toList"
        | "std/non-empty-list::head"
        | "std/non-empty-list::tail" => Some(1),
        "std/non-empty-list::cons" | "std/non-empty-list::reduce1" => Some(2),
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

impl Backend<'_> {
    fn non_empty_list_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<usize> {
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven NonEmptyList source scope".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing NonEmptyList argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&supplied, type_ref.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven NonEmptyList argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let a = named("\0non_empty_list_element", vec![]);
        let list = named("std/prelude::List", vec![a.clone()]);
        let non_empty = named("std/non-empty-list::NonEmptyList", vec![a.clone()]);
        let (parameters, result) = match name {
            "std/non-empty-list::singleton" => (vec![a.clone()], non_empty),
            "std/non-empty-list::cons" => (vec![a.clone(), list], non_empty),
            "std/non-empty-list::fromList" => {
                (vec![list], named("std/prelude::Maybe", vec![non_empty]))
            }
            "std/non-empty-list::toList" | "std/non-empty-list::tail" => (vec![non_empty], list),
            "std/non-empty-list::head" => (vec![non_empty], a.clone()),
            "std/non-empty-list::reduce1" => (
                vec![function(&[a.clone(), a.clone()], a.clone()), non_empty],
                a.clone(),
            ),
            _ => unreachable!(),
        };
        self.match_local_type(
            &function(&parameters, result),
            &full,
            &BTreeMap::from([("\0non_empty_list_element".into(), 0)]),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(arity(name).expect("known NonEmptyList operation"))
    }

    fn non_empty_list_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0non_empty_list_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_non_empty_list_operation({}, vec![{args}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        body
    }

    pub(super) fn non_empty_list_reference(
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
            return Err("Rust backend: unexpected NonEmptyList dictionary".into());
        }
        let arity = self.non_empty_list_type(name, &[], type_ref, scope)?;
        self.capabilities.insert("non-empty-list".into());
        Ok(Some(Self::non_empty_list_callable(name, arity)))
    }

    pub(super) fn non_empty_list_call(
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
            || *evidence_argument_index
                != (arguments.len() > arity(callee).unwrap()).then_some(arity(callee).unwrap())
        {
            return Err("Rust backend: unproven NonEmptyList source stages".into());
        }
        let arity = self.non_empty_list_type(callee, arguments, type_ref, scope)?;
        self.capabilities.insert("non-empty-list".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_non_empty_list_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::non_empty_list_callable(callee, arity)
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
