use super::*;
use seseragi_lowering::CoreInstanceConstraint;

impl Backend<'_> {
    /// Validate providers even when their only use passes through an abstract
    /// primary dictionary. A forged method table must not bypass the source
    /// contract merely because the method value is checked in another frame.
    pub(in crate::emit) fn validate_method_value_provider(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        method: &CoreInstanceMethod,
        scope: &Scope,
    ) -> Result<()> {
        let Some(declaration) = self.module.trait_methods.iter().find(|d| {
            instance
                .identity
                .starts_with(&format!("{}<", d.trait_identity))
                && d.method == method.name
                && !d.constraints.is_empty()
                && !d.signature.parameters.is_empty()
        }) else {
            return Ok(());
        };
        if instance.arguments.len() != declaration.trait_parameters.len()
            || method.type_parameters.len() != declaration.signature.type_parameters.len()
            || method.parameters.is_empty()
            || method
                .type_parameters
                .iter()
                .zip(&declaration.signature.type_parameters)
                .any(|(actual, expected)| actual.arity != expected.arity)
        {
            return Err(
                "Rust backend: constrained method value provider binder or arity mismatch".into(),
            );
        }
        let owner = declaration
            .trait_identity
            .rsplit_once("::")
            .map(|(o, _)| o)
            .ok_or("Rust backend: missing method value declaration owner")?;
        let substitutions = declaration
            .trait_parameters
            .iter()
            .zip(&instance.arguments)
            .map(|(p, actual)| (p.name.clone(), actual.clone()))
            .chain(
                declaration
                    .signature
                    .type_parameters
                    .iter()
                    .zip(&method.type_parameters)
                    .map(|(p, actual)| {
                        (
                            p.name.clone(),
                            CoreType::Named {
                                name: actual.name.clone(),
                                arguments: vec![],
                            },
                        )
                    }),
            )
            .collect::<BTreeMap<_, _>>();
        let canonical = |ty| {
            display::canonical_type(ty, owner, &substitutions, self.type_names, true).ok_or_else(
                || RustBackendError::from("Rust backend: unproven method value provider owner"),
            )
        };
        let primary = CoreInstanceConstraint {
            name: instance.trait_name.clone(),
            trait_identity: Some(declaration.trait_identity.clone()),
            arguments: instance.arguments.clone(),
        };
        let mut constraints = vec![primary.clone()];
        for c in &declaration.constraints {
            constraints.push(CoreInstanceConstraint {
                arguments: c
                    .arguments
                    .iter()
                    .map(canonical)
                    .collect::<Result<Vec<_>>>()?,
                ..c.clone()
            });
        }
        let mut evidence = vec![CoreCallEvidence {
            constraint: primary,
            evidence: CoreInstanceEvidence::Local {
                identity: instance.identity.clone(),
                type_arguments: instance
                    .type_parameters
                    .iter()
                    .map(|p| CoreType::Named {
                        name: p.name.clone(),
                        arguments: vec![],
                    })
                    .collect(),
                evidence_arguments: instance
                    .constraints
                    .iter()
                    .enumerate()
                    .map(|(index, c)| CoreCallEvidence {
                        constraint: c.clone(),
                        evidence: CoreInstanceEvidence::Parameter { index },
                    })
                    .collect(),
            },
        }];
        let mut inner = scope.clone();
        for c in &method.constraints {
            let index = inner.dictionaries.len();
            inner.dictionaries.push(c.clone());
            evidence.push(CoreCallEvidence {
                constraint: c.clone(),
                evidence: CoreInstanceEvidence::Parameter { index },
            });
        }
        let type_ref = method.parameters.iter().rev().fold(
            local::method_argument_type(&method.body)
                .ok_or("Rust backend: unproven method value provider result")?,
            |result, p| CoreType::Function {
                parameter: Box::new(p.type_ref.clone()),
                result: Box::new(result),
            },
        );
        self.validate_constrained_method_value(
            &CoreExpr::Call {
                callee: String::new(),
                callee_signature: None,
                arguments: vec![],
                evidence,
                deferred_evidence_parameters: vec![],
                deferred_evidence_type_constructor_parameters: vec![],
                evidence_argument_index: None,
                trait_dispatch: Some(CoreTraitDispatch {
                    trait_identity: declaration.trait_identity.clone(),
                    method: method.name.clone(),
                    method_contract: Some(CoreTraitMethodContract {
                        parameters: declaration
                            .signature
                            .parameters
                            .iter()
                            .map(canonical)
                            .collect::<Result<Vec<_>>>()?,
                        constraints,
                    }),
                }),
                type_ref,
                origin: method.origin.clone(),
            },
            &inner,
        )
    }

    /// A method value specializes dictionaries before exposing source stages.
    /// The ordinary method entry retains its existing source-first call ABI.
    pub(super) fn constrained_method_value(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<String> {
        self.validate_constrained_method_value(expr, scope)?;
        let CoreExpr::Call {
            evidence,
            trait_dispatch: Some(dispatch),
            ..
        } = expr
        else {
            unreachable!()
        };
        let primary = self.evidence(&evidence[..1], scope)?;
        let stage = self.fresh();
        let mut output = format!(
            "{{ let mut {stage} = ({primary}).remove(0).field({});",
            quoted(&seseragi_lowering::method_value_factory_name(
                &dispatch.method
            ))
        );
        for selected in &evidence[1..] {
            let child = self.evidence(std::slice::from_ref(selected), scope)?;
            output.push_str(&format!("{stage} = {stage}.apply(({child}).remove(0));"));
        }
        output.push_str(&format!("{stage} }}"));
        Ok(output)
    }

    fn validate_constrained_method_value(&self, expr: &CoreExpr, scope: &Scope) -> Result<()> {
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            type_ref,
            ..
        } = expr
        else {
            unreachable!()
        };
        let contract = dispatch
            .method_contract
            .as_ref()
            .expect("method value contract");
        if !arguments.is_empty()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
            || scope.has_unproven_type_scope()
        {
            return Err("Rust backend: unproven constrained method value staging".into());
        }
        let declaration = self
            .module
            .trait_methods
            .iter()
            .find(|d| d.trait_identity == dispatch.trait_identity && d.method == dispatch.method)
            .ok_or("Rust backend: missing constrained method value declaration")?;
        if declaration.constraints.is_empty()
            || declaration.signature.parameters.is_empty()
            || !declaration.signature.argument_types.is_empty()
            || contract.parameters.len() != declaration.signature.parameters.len()
            || contract.constraints.len() != declaration.constraints.len() + 1
            || contract
                .constraints
                .first()
                .and_then(|c| c.trait_identity.as_deref())
                != Some(dispatch.trait_identity.as_str())
            || contract
                .constraints
                .iter()
                .any(|c| !self.valid_constraint_label(c))
        {
            return Err("Rust backend: unproven constrained method value declaration".into());
        }
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &contract.constraints, &[]),
            evidence,
            scope,
        )?;
        let selected = evidence
            .first()
            .ok_or("Rust backend: missing method value primary dictionary")?;
        if selected.constraint.arguments.len() != declaration.trait_parameters.len() {
            return Err("Rust backend: constrained method value trait arity mismatch".into());
        }
        let owner = declaration
            .trait_identity
            .rsplit_once("::")
            .map(|(o, _)| o)
            .ok_or("Rust backend: missing method value owner")?;
        let mut names = BTreeSet::new();
        let mut substitutions = BTreeMap::new();
        for (parameter, actual) in declaration
            .trait_parameters
            .iter()
            .zip(&selected.constraint.arguments)
        {
            if !names.insert(parameter.name.clone())
                || self.declared_method_type_kind(actual, scope) != Some(parameter.arity as usize)
            {
                return Err("Rust backend: constrained method value trait kind mismatch".into());
            }
            substitutions.insert(
                parameter.name.clone(),
                self.local_canonical_type(actual, scope)?,
            );
        }
        let variables = Self::method_value_binders(
            declaration
                .signature
                .type_parameters
                .iter()
                .map(|p| (&p.name, p.arity)),
            &mut substitutions,
            &mut names,
        )?;
        let expected = declaration.signature.parameters.iter().rev().fold(
            declaration.signature.result.clone(),
            |result, parameter| CoreType::Function {
                parameter: Box::new(parameter.clone()),
                result: Box::new(result),
            },
        );
        self.validate_value_signature_and_constraints(
            &expected,
            &declaration.constraints,
            owner,
            &substitutions,
            &variables,
            type_ref,
            &contract.constraints[1..],
            scope,
        )?;
        let canonical = |ty| {
            display::canonical_type(ty, owner, &substitutions, self.type_names, true).ok_or_else(
                || RustBackendError::from("Rust backend: unproven method value parameter owner"),
            )
        };
        let mut bindings = BTreeMap::new();
        for (expected, actual) in declaration
            .signature
            .parameters
            .iter()
            .zip(&contract.parameters)
        {
            self.match_local_type(
                &canonical(expected)?,
                actual,
                &variables,
                &mut bindings,
                scope,
            )?;
        }
        // The same instantiation must explain the complete returned method
        // type, declaration parameters and every method-owned obligation.
        self.match_local_type(
            &canonical(&expected)?,
            type_ref,
            &variables,
            &mut bindings,
            scope,
        )?;
        self.match_value_constraints(
            &declaration.constraints,
            &contract.constraints[1..],
            owner,
            &substitutions,
            &variables,
            &mut bindings,
            scope,
        )?;
        match &selected.evidence {
            CoreInstanceEvidence::Parameter { .. } => {}
            CoreInstanceEvidence::Local {
                identity,
                type_arguments,
                ..
            }
            | CoreInstanceEvidence::Imported {
                identity,
                type_arguments,
                ..
            } => {
                let instance = self
                    .module
                    .instances
                    .iter()
                    .find(|i| &i.identity == identity)
                    .ok_or("Rust backend: unknown constrained method value instance")?;
                let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
                else {
                    return Err("Rust backend: unproven constrained method value provider".into());
                };
                let method = methods
                    .iter()
                    .find(|m| m.name == dispatch.method)
                    .ok_or("Rust backend: missing constrained method value implementation")?;
                if method.parameters.is_empty()
                    || instance.type_parameters.len() != type_arguments.len()
                {
                    return Err(
                        "Rust backend: unproven constrained method value source arity".into(),
                    );
                }
                let mut substitutions = BTreeMap::new();
                let mut names = BTreeSet::new();
                for (p, actual) in instance.type_parameters.iter().zip(type_arguments) {
                    if !names.insert(p.name.clone())
                        || self.declared_method_type_kind(actual, scope) != Some(p.arity as usize)
                    {
                        return Err(
                            "Rust backend: constrained method value instance kind mismatch".into(),
                        );
                    }
                    substitutions.insert(p.name.clone(), self.local_canonical_type(actual, scope)?);
                }
                let variables = Self::method_value_binders(
                    method.type_parameters.iter().map(|p| (&p.name, p.arity)),
                    &mut substitutions,
                    &mut names,
                )?;
                let expected = method.parameters.iter().rev().fold(
                    local::method_argument_type(&method.body)
                        .ok_or("Rust backend: unproven constrained method value result")?,
                    |result, p| CoreType::Function {
                        parameter: Box::new(p.type_ref.clone()),
                        result: Box::new(result),
                    },
                );
                self.validate_value_signature_and_constraints(
                    &expected,
                    &method.constraints,
                    &self.owners[identity],
                    &substitutions,
                    &variables,
                    type_ref,
                    &contract.constraints[1..],
                    scope,
                )?;
            }
            _ => return Err("Rust backend: unproven constrained method value evidence".into()),
        }
        Ok(())
    }

    fn method_value_binders<'a>(
        parameters: impl Iterator<Item = (&'a String, u32)>,
        substitutions: &mut BTreeMap<String, CoreType>,
        names: &mut BTreeSet<String>,
    ) -> Result<BTreeMap<String, u32>> {
        let mut variables = BTreeMap::new();
        for (index, (parameter, arity)) in parameters.enumerate() {
            if !names.insert(parameter.clone()) {
                return Err("Rust backend: overlapping constrained method value binder".into());
            }
            let name = format!("\0constrained_method_type_{index}");
            variables.insert(name.clone(), arity);
            substitutions.insert(
                parameter.clone(),
                CoreType::Named {
                    name,
                    arguments: vec![],
                },
            );
        }
        Ok(variables)
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_value_signature_and_constraints(
        &self,
        expected: &CoreType,
        constraints: &[CoreInstanceConstraint],
        owner: &str,
        substitutions: &BTreeMap<String, CoreType>,
        variables: &BTreeMap<String, u32>,
        actual: &CoreType,
        actual_constraints: &[CoreInstanceConstraint],
        scope: &Scope,
    ) -> Result<()> {
        let expected =
            display::canonical_type(expected, owner, substitutions, self.type_names, true)
                .ok_or("Rust backend: unproven constrained method value type")?;
        self.validate_method_value_type(&expected, actual, variables, scope)?;
        let mut bindings = BTreeMap::new();
        self.match_local_type(&expected, actual, variables, &mut bindings, scope)?;
        self.match_value_constraints(
            constraints,
            actual_constraints,
            owner,
            substitutions,
            variables,
            &mut bindings,
            scope,
        )?;
        for (name, arity) in variables {
            if bindings.get(name).is_some_and(|ty| {
                self.declared_method_type_kind(ty, scope) != Some(*arity as usize)
            }) {
                return Err(
                    "Rust backend: constrained method obligation binder kind mismatch".into(),
                );
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn match_value_constraints(
        &self,
        expected: &[CoreInstanceConstraint],
        actual: &[CoreInstanceConstraint],
        owner: &str,
        substitutions: &BTreeMap<String, CoreType>,
        variables: &BTreeMap<String, u32>,
        bindings: &mut BTreeMap<String, CoreType>,
        scope: &Scope,
    ) -> Result<()> {
        if expected.len() != actual.len() {
            return Err("Rust backend: method value obligation count mismatch".into());
        }
        for (expected, actual) in expected.iter().zip(actual) {
            if expected.trait_identity.is_none()
                || expected.trait_identity != actual.trait_identity
                || !self.valid_constraint_label(expected)
                || expected.arguments.len() != actual.arguments.len()
            {
                return Err("Rust backend: method value obligation identity mismatch".into());
            }
            for (expected, actual) in expected.arguments.iter().zip(&actual.arguments) {
                let expected =
                    display::canonical_type(expected, owner, substitutions, self.type_names, true)
                        .ok_or("Rust backend: unproven method obligation owner")?;
                self.match_local_type(&expected, actual, variables, bindings, scope)?;
            }
        }
        Ok(())
    }
}
