use super::*;

impl RecursiveLocal {
    pub(in crate::emit) fn reference(&self) -> String {
        format!("{}({}.clone())", self.factory, id(&self.environment))
    }
}

impl Scope {
    /// A lifted recursive reference reconstructs its closure from these outer
    /// values. They are real capture edges even if only a sibling uses them.
    pub(in crate::emit) fn retain_free(&mut self, free: &BTreeSet<String>) {
        let mut needed = free.clone();
        loop {
            let before = needed.len();
            for (name, recursive) in &self.recursive {
                if needed.contains(name) {
                    needed.insert(recursive.environment.clone());
                }
            }
            if needed.len() == before {
                break;
            }
        }
        self.retain(|name| needed.contains(name));
    }
}

impl Backend<'_> {
    pub(in crate::emit) fn local_source_function(
        &mut self,
        callable: &LocalCallable,
        body: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        let recursive = callable
            .self_name
            .as_ref()
            .is_some_and(|name| scope.recursive.contains_key(name));
        self.function_inner(
            &callable.parameters,
            body,
            scope,
            recursive.then_some(callable),
        )
    }

    /// Lift a named recursive declaration (or a complete explicit rec group)
    /// into code-only Rust factories. Re-entering a member passes its immutable
    /// lexical environment, never a strong reference to itself or a sibling.
    /// Escaping and partially applied members therefore use the ordinary traced
    /// closure ABI without introducing an Rc cycle or a weak dangling callback.
    pub(in crate::emit) fn recursive_local_prefix(
        &mut self,
        statements: &[CoreStatement],
        scope: &Scope,
    ) -> Result<Option<(String, Scope, usize)>> {
        let Some(CoreStatement::LocalFunction {
            initializer_in_outer_scope,
            rec_group,
            name,
            parameters,
            body,
            origin,
            ..
        }) = statements.first()
        else {
            return Ok(None);
        };
        if *initializer_in_outer_scope {
            if rec_group.is_some() {
                return Err(crate::project::at(
                    "Rust backend: let initializer cannot belong to a recursive group",
                    origin,
                    self.owner,
                ));
            }
            return Ok(None);
        }
        // A parameter with the function's own name shadows it, so a call
        // through that parameter is not recursion.
        if rec_group.is_none()
            && core_free_variables(body).is_some_and(|free| {
                !free.names.contains(name)
                    || parameters.iter().any(|parameter| &parameter.id == name)
            })
        {
            return Ok(None);
        }
        let count = if let Some(group) = rec_group {
            statements.iter().take_while(|statement| matches!(statement,
                CoreStatement::LocalFunction { rec_group: Some(other), .. } if other == group
            )).count()
        } else {
            1
        };
        let mut members = Vec::new();
        let mut free_names = BTreeSet::new();
        for statement in &statements[..count] {
            let CoreStatement::LocalFunction {
                initializer_in_outer_scope,
                name,
                type_parameters,
                constraints,
                parameters,
                body,
                origin,
                ..
            } = statement
            else {
                unreachable!()
            };
            if *initializer_in_outer_scope || members.iter().any(|(member, _, _, _)| member == name)
            {
                return Err(crate::project::at(
                    "Rust backend: invalid recursive local group",
                    origin,
                    self.owner,
                ));
            }
            let mut callable = LocalCallable::new(
                type_parameters.iter().map(|p| (p.name.clone(), p.arity)),
                constraints,
                parameters,
            );
            callable.self_name = Some(name.clone());
            if let Some(mut free) = core_free_variables(body) {
                for parameter in parameters {
                    free.names.remove(&parameter.id);
                }
                if constraints.is_empty() {
                    for index in free.evidence_parameters {
                        free.names.insert(format!("\0dictionary_{index}"));
                    }
                }
                free_names.extend(free.names);
            } else {
                // As with ordinary closures, an expression not covered by the
                // free-variable analysis (notably Effect plans) conservatively
                // retains the lexical scope. Expression emission still checks
                // every operation and its evidence.
                free_names.extend(scope.iter().cloned());
            }
            members.push((name.clone(), callable, body, origin));
        }
        // Group names shadow outer bindings, including same-named globals.
        let mut captures = scope.clone();
        for (name, _, _, _) in &members {
            captures.block(name.clone());
            free_names.remove(name);
        }
        captures.retain_free(&free_names);
        let capture_names = captures.iter().cloned().collect::<Vec<_>>();
        let environment = format!("\0recursive_environment_{}", self.fresh());
        let mut factory_scope = captures.clone();
        factory_scope.insert(environment.clone());
        let mut factories = Vec::new();
        for (name, callable, _, _) in &members {
            let recursive = RecursiveLocal {
                factory: self.fresh(),
                environment: environment.clone(),
            };
            factories.push(recursive.clone());
            factory_scope.blocked.remove(name);
            factory_scope.recursive.insert(name.clone(), recursive);
            factory_scope
                .callables
                .insert(name.clone(), callable.clone());
        }
        let restore = capture_names
            .iter()
            .enumerate()
            .map(|(index, name)| format!("let {} = captures.get()[{index}].clone();", id(name)))
            .collect::<String>();
        let values = capture_names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut declarations = format!(
            "let {} = V::Tuple(Shared::new(vec![{values}]));",
            id(&environment)
        );
        let mut inner = scope.clone();
        for ((name, callable, body, origin), recursive) in members.iter().zip(&factories) {
            let value = self
                .local_function(callable, body, &factory_scope)
                .map_err(|error| crate::project::at(&error.message, origin, self.owner))?;
            declarations.push_str(&format!(
                "fn {}({}: V) -> V {{ let V::Tuple(captures) = &{} else {{ unreachable!() }}; {restore} {value} }} let {} = {};",
                recursive.factory, id(&environment), id(&environment),
                id(name),
                recursive.reference()
            ));
            inner.insert_callable(name.clone(), callable.clone());
        }
        Ok(Some((declarations, inner, count)))
    }
}
