use super::*;
use seseragi_lowering::CoreFunction;

struct TailTarget<'a> {
    symbol: &'a str,
    slots: Vec<String>,
    label: String,
    local: Option<(LocalCallable, String)>,
}

impl Backend<'_> {
    pub(super) fn direct_function(
        &mut self,
        function: &CoreFunction,
        setup: &str,
        scope: &Scope,
    ) -> Result<String> {
        if let Some(plan) = seseragi_lowering::scalar_self_tail_loop_with_globals(
            function,
            self.owner,
            self.scalar_globals,
        ) {
            return self.scalar_direct_function(function, &plan);
        }
        let target = TailTarget {
            symbol: &function.symbol,
            slots: function.parameters.iter().map(|_| self.fresh()).collect(),
            label: self.fresh(),
            local: None,
        };
        let before = self.self_tail_calls;
        let body = if function.is_effect {
            format!("return {};", self.expr(&function.body, scope)?)
        } else {
            self.tail_body(&function.body, scope, &target)?
        };
        let looping = self.self_tail_calls > before;
        let parameters = function
            .parameters
            .iter()
            .zip(&target.slots)
            .map(|(p, slot)| {
                if looping {
                    format!(", mut {slot}: V")
                } else {
                    format!(", {}: V", id(&p.id))
                }
            })
            .collect::<String>();
        let body = if looping {
            // Source names are immutable snapshots of this iteration. Loop
            // storage has fresh identities: a lexical binding can shadow a
            // parameter without intercepting the next iteration's assignment.
            let restore = function
                .parameters
                .iter()
                .zip(&target.slots)
                .map(|(p, slot)| format!("let {} = {slot}.clone();", id(&p.id)))
                .collect::<String>();
            format!(
                "'{label}: loop {{ {restore} {setup} {body} }}",
                label = target.label
            )
        } else {
            format!("{setup} {body}")
        };
        Ok(format!(
            "\nfn {}_direct({}dicts: Vec<V>{parameters}) -> V {{ {body} }}\n",
            id(&function.symbol),
            if looping { "mut " } else { "" },
        ))
    }

    pub(super) fn local_tail_body(
        &mut self,
        callable: &LocalCallable,
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        let name = callable.self_name.as_ref().expect("recursive local name");
        let target = TailTarget {
            symbol: name,
            slots: callable.parameters.iter().map(|_| self.fresh()).collect(),
            label: self.fresh(),
            local: Some((callable.clone(), scope.recursive[name].factory.clone())),
        };
        let before = self.self_tail_calls;
        let value = self.tail_body(body, scope, &target)?;
        if self.self_tail_calls == before {
            return Ok(format!("{{ {value} }}"));
        }
        let mut setup = String::new();
        let mut restore = String::new();
        for (parameter, slot) in callable.parameters.iter().zip(&target.slots) {
            setup.push_str(&format!("let mut {slot} = {}.clone();", id(&parameter.id)));
            restore.push_str(&format!("let {} = {slot}.clone();", id(&parameter.id)));
        }
        let dictionaries = (0..callable.constraints.len())
            .map(|index| {
                let name = id(&format!("\0dictionary_{index}"));
                restore.push_str(&format!("let {name} = dicts[{index}].clone();"));
                format!("{name}.clone()")
            })
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!("{{ {setup} let mut dicts: Vec<V> = vec![{dictionaries}]; '{}: loop {{ {restore} {value} }} }}", target.label))
    }

    fn tail_body(&mut self, expr: &CoreExpr, scope: &Scope, target: &TailTarget) -> Result<String> {
        self.tail_inner(expr, scope, target).map_err(|mut error| {
            if error.origin.is_none() {
                error.origin = Some(core_expr_origin(expr).clone());
            }
            error
        })
    }

    fn tail_inner(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
        target: &TailTarget,
    ) -> Result<String> {
        match expr {
            CoreExpr::Call {
                callee,
                arguments,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } if callee == target.symbol
                && trait_dispatch.is_none()
                && deferred_evidence_parameters.is_empty()
                && deferred_evidence_type_constructor_parameters.is_empty()
                && !(evidence_argument_index.is_some() && !evidence.is_empty())
                && match &target.local {
                    Some((callable, factory)) => {
                        scope
                            .recursive
                            .get(callee)
                            .is_some_and(|recursive| &recursive.factory == factory)
                            && arguments.len() == callable.parameters.len()
                            && evidence_argument_index
                                .is_none_or(|index| index == callable.parameters.len())
                    }
                    None => {
                        !scope.contains(callee)
                            && known_pure_call_shape(self.module, callee, arguments.len())
                                == Some(CoreCallShape::Saturated)
                    }
                } =>
            {
                // Evaluate evidence, then every source argument, before any
                // loop storage changes. Captures and later arguments must see
                // the caller's values, and defects must preserve source order.
                if let Some((callable, _)) = &target.local {
                    self.validate_local_arguments(callable, arguments, evidence, scope)?;
                }
                let dictionaries = self.evidence(evidence, scope)?;
                let next_dicts = self.fresh();
                let mut body = format!("let {next_dicts} = {dictionaries};");
                let mut values = Vec::new();
                for arg in arguments {
                    let name = self.fresh();
                    body.push_str(&format!("let {name} = {};", self.expr(arg, scope)?));
                    values.push(name);
                }
                body.push_str(&format!("dicts = {next_dicts};"));
                for (slot, value) in target.slots.iter().zip(values) {
                    body.push_str(&format!("{slot} = {value};"));
                }
                body.push_str(&format!("continue '{};", target.label));
                self.self_tail_calls += 1;
                Ok(body)
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => Ok(format!(
                "if ({}).boolean() {{ {} }} else {{ {} }}",
                self.expr(condition, scope)?,
                self.tail_body(then_branch, scope, target)?,
                self.tail_body(else_branch, scope, target)?,
            )),
            CoreExpr::Decision {
                scrutinee,
                scrutinee_type,
                branches,
                ..
            } => {
                let value = self.fresh();
                let mut body = format!("{{ let {value} = {};", self.expr(scrutinee, scope)?);
                for branch in branches {
                    let plan = seseragi_lowering::core_list_prefix_plan(
                        scrutinee_type,
                        branch,
                        scope.bound_type_names(),
                    );
                    let prefix = plan.as_ref().map(|plan| {
                        let prefix = self.fresh();
                        let label = self.fresh();
                        body.push_str(&Self::prefix_checks(plan, &value, &prefix, &label));
                        prefix
                    });
                    let prefix = prefix.as_deref();
                    let conditions = branch
                        .tests
                        .iter()
                        .skip(usize::from(plan.is_some()))
                        .map(|test| self.test(test, &value, prefix))
                        .collect::<Result<Vec<_>>>()?;
                    body.push_str(&format!(
                        "if {} {{",
                        if conditions.is_empty() {
                            "true".into()
                        } else {
                            conditions.join(" && ")
                        }
                    ));
                    let mut inner = scope.clone();
                    for binding in &branch.bindings {
                        body.push_str(&format!(
                            "let {} = {};",
                            id(&binding.name),
                            Self::project_prefix(&value, &binding.path, prefix)?,
                        ));
                        inner.insert(binding.name.clone());
                    }
                    let guard = branch
                        .guard
                        .as_ref()
                        .map(|g| self.expr(g, &inner))
                        .transpose()?
                        .map(|g| format!("({g}).boolean()"))
                        .unwrap_or_else(|| "true".into());
                    body.push_str(&format!(
                        "if {guard} {{ {} }} }}",
                        self.tail_body(&branch.value, &inner, target)?
                    ));
                    if plan.is_some() {
                        body.push('}');
                    }
                }
                body.push_str("panic!(\"Rust backend: unmatched checked decision\"); }");
                Ok(body)
            }
            CoreExpr::Sequence {
                statements, result, ..
            } => self.tail_sequence(statements, result, scope, target),
            // A returned lambda and a call through an alias keep the normal
            // closure ABI. Their bodies are not this function's tail position.
            _ => Ok(format!("return {};", self.expr(expr, scope)?)),
        }
    }

    fn tail_sequence(
        &mut self,
        statements: &[CoreStatement],
        result: &CoreExpr,
        scope: &Scope,
        target: &TailTarget,
    ) -> Result<String> {
        let Some((first, rest)) = statements.split_first() else {
            return self.tail_body(result, scope, target);
        };
        if let Some((declarations, inner, count)) =
            self.recursive_local_prefix(statements, scope)?
        {
            return Ok(format!(
                "{{ {declarations} {} }}",
                self.tail_sequence(&statements[count..], result, &inner, target)?
            ));
        }
        let mut inner = scope.clone();
        let (name, value) = match first {
            CoreStatement::PureLet { name, value, .. } => {
                let value = self.expr(value, scope)?;
                inner.insert(name.clone());
                (name, value)
            }
            CoreStatement::LocalFunction {
                initializer_in_outer_scope,
                name,
                constraints,
                type_parameters,
                parameters,
                body,
                rec_group,
                origin,
                ..
            } if rec_group.is_none() => {
                let mut callable = LocalCallable::new(
                    type_parameters.iter().map(|p| (p.name.clone(), p.arity)),
                    constraints,
                    parameters,
                );
                if !initializer_in_outer_scope {
                    callable.self_name = Some(name.clone());
                }
                if *initializer_in_outer_scope && parameters.is_empty() {
                    callable.method_value = local::method_argument_type(body)
                        .filter(|ty| matches!(ty, CoreType::Function { .. }));
                }
                let value = self
                    .local_function(&callable, body, scope)
                    .map_err(|error| crate::project::at(&error.message, origin, self.owner))?;
                inner.insert_callable(name.clone(), callable);
                (name, value)
            }
            // Effect continuations have their own interpreter frames. Never
            // continue a pure caller loop from inside one of those closures.
            _ => {
                return Ok(format!(
                    "return {};",
                    self.sequence(statements, result, scope)?
                ))
            }
        };
        Ok(format!(
            "{{ let {} = {value}; {} }}",
            id(name),
            self.tail_sequence(rest, result, &inner, target)?,
        ))
    }
}
