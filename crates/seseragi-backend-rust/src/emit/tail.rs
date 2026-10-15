use super::*;
use seseragi_lowering::CoreFunction;

struct TailTarget<'a> {
    symbol: &'a str,
    slots: Vec<String>,
    label: String,
}

impl Backend<'_> {
    pub(super) fn direct_function(
        &mut self,
        function: &CoreFunction,
        setup: &str,
        scope: &Scope,
    ) -> Result<String> {
        let target = TailTarget {
            symbol: &function.symbol,
            slots: function.parameters.iter().map(|_| self.fresh()).collect(),
            label: self.fresh(),
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
                && !scope.contains(callee)
                && trait_dispatch.is_none()
                && deferred_evidence_parameters.is_empty()
                && deferred_evidence_type_constructor_parameters.is_empty()
                && !(evidence_argument_index.is_some() && !evidence.is_empty())
                && known_pure_call_shape(self.module, callee, arguments.len())
                    == Some(CoreCallShape::Saturated) =>
            {
                // Evaluate evidence, then every source argument, before any
                // loop storage changes. Captures and later arguments must see
                // the caller's values, and defects must preserve source order.
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
                branches,
                ..
            } => {
                let value = self.fresh();
                let mut body = format!("{{ let {value} = {};", self.expr(scrutinee, scope)?);
                for branch in branches {
                    let conditions = branch
                        .tests
                        .iter()
                        .map(|test| self.test(test, &value))
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
                            Self::project(&value, &binding.path)?,
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
        let (name, value) = match first {
            CoreStatement::PureLet { name, value, .. } => (name, self.expr(value, scope)?),
            CoreStatement::LocalFunction {
                name,
                constraints,
                parameters,
                body,
                rec_group,
                ..
            } if constraints.is_empty() && rec_group.is_none() => {
                (name, self.function(parameters, body, scope)?)
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
        let mut inner = scope.clone();
        inner.insert(name.clone());
        Ok(format!(
            "{{ let {} = {value}; {} }}",
            id(name),
            self.tail_sequence(rest, result, &inner, target)?,
        ))
    }
}
