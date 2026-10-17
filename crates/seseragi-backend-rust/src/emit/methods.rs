use super::*;
use seseragi_lowering::{CoreInstanceMethod, CoreTraitDispatch, CoreTraitMethodContract};

impl Backend<'_> {
    pub(super) fn method_function(
        &mut self,
        method: &CoreInstanceMethod,
        scope: &Scope,
    ) -> Result<String> {
        if !method.constraints.is_empty() && scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven overlapping method type scope".into());
        }
        let mut inner = scope.clone();
        let mut parameters = method.parameters.clone();
        for constraint in &method.constraints {
            if !self.valid_constraint_label(constraint) {
                return Err("Rust backend: unproven method constraint identity".into());
            }
            let index = inner.dictionaries.len();
            inner.dictionaries.push(constraint.clone());
            // Implementation-only stages follow the source parameters. They
            // use the combined instance/method frame, but never source arity.
            parameters.push(CoreParameter {
                id: format!("\0dictionary_{index}"),
                kind: "evidence".into(),
                type_ref: CoreType::Hole,
            });
        }
        self.function(&parameters, &method.body, &inner)
    }

    pub(super) fn reject_missing_method_contract(
        &self,
        dispatch: &CoreTraitDispatch,
        evidence: &[CoreCallEvidence],
    ) -> Result<()> {
        let constrained = evidence
            .first()
            .and_then(|selected| match &selected.evidence {
                CoreInstanceEvidence::Local { identity, .. }
                | CoreInstanceEvidence::Imported { identity, .. } => self
                    .module
                    .instances
                    .iter()
                    .find(|i| &i.identity == identity),
                _ => None,
            })
            .and_then(|instance| match &instance.implementation {
                CoreInstanceImplementation::UserDefined { methods } => {
                    methods.iter().find(|method| method.name == dispatch.method)
                }
                _ => None,
            })
            .is_some_and(|method| !method.constraints.is_empty());
        if constrained {
            return Err("Rust backend: missing constrained method contract".into());
        }
        Ok(())
    }

    fn validate_method_contract(
        &self,
        dispatch: &CoreTraitDispatch,
        contract: &CoreTraitMethodContract,
        arguments: &[CoreExpr],
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        if contract.parameters.is_empty()
            || contract.constraints.len() < 2
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
            return Err(
                "Rust backend: invalid constrained method contract identity or arity".into(),
            );
        }
        let callable = LocalCallable::new(std::iter::empty(), &contract.constraints, &[]);
        self.validate_local_evidence(&callable, evidence, scope)?;
        for (expected, actual) in contract.parameters.iter().zip(arguments) {
            let actual = local::method_argument_type(actual)
                .ok_or("Rust backend: unproven method argument type")?;
            self.match_local_type(
                expected,
                &actual,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        if let Some(CoreCallEvidence {
            evidence:
                CoreInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    ..
                }
                | CoreInstanceEvidence::Imported {
                    identity,
                    type_arguments,
                    ..
                },
            ..
        }) = evidence.first()
        {
            let instance = self
                .module
                .instances
                .iter()
                .find(|i| &i.identity == identity)
                .ok_or("Rust backend: unknown method instance")?;
            let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation
            else {
                return Err("Rust backend: unproven constrained method implementation".into());
            };
            let method = methods
                .iter()
                .find(|method| method.name == dispatch.method)
                .ok_or("Rust backend: unknown constrained method")?;
            if method.parameters.len() != contract.parameters.len()
                || method.constraints.len() + 1 != evidence.len()
                || instance.type_parameters.len() != type_arguments.len()
            {
                return Err(
                    "Rust backend: constrained method source or dictionary arity mismatch".into(),
                );
            }
            let mut substitutions = instance
                .type_parameters
                .iter()
                .zip(type_arguments)
                .map(|(parameter, ty)| {
                    self.local_canonical_type(ty, scope)
                        .map(|ty| (parameter.name.clone(), ty))
                })
                .collect::<Result<BTreeMap<_, _>>>()?;
            let mut variables = Vec::new();
            for (index, parameter) in method.type_parameters.iter().enumerate() {
                if substitutions.contains_key(&parameter.name) {
                    return Err("Rust backend: unproven overlapping method type scope".into());
                }
                let name = format!("\0method_type_{index}");
                variables.push((name.clone(), parameter.arity));
                substitutions.insert(
                    parameter.name.clone(),
                    CoreType::Named {
                        name,
                        arguments: vec![],
                    },
                );
            }
            let canonical = |ty| {
                display::canonical_type(
                    ty,
                    &self.owners[identity],
                    &substitutions,
                    self.type_names,
                    true,
                )
                .ok_or_else(|| {
                    RustBackendError::from("Rust backend: unproven selected method type")
                })
            };
            let parameters = method
                .parameters
                .iter()
                .map(|parameter| {
                    Ok(CoreParameter {
                        type_ref: canonical(&parameter.type_ref)?,
                        ..parameter.clone()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let constraints = method
                .constraints
                .iter()
                .map(|constraint| {
                    Ok(seseragi_lowering::CoreInstanceConstraint {
                        arguments: constraint
                            .arguments
                            .iter()
                            .map(canonical)
                            .collect::<Result<Vec<_>>>()?,
                        ..constraint.clone()
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let callable = LocalCallable::new(variables.into_iter(), &constraints, &parameters);
            self.validate_local_arguments(&callable, arguments, &evidence[1..], scope)?;
        }
        Ok(())
    }

    pub(super) fn method_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch: Some(dispatch),
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            unreachable!()
        };
        let contract = dispatch
            .method_contract
            .as_ref()
            .expect("checked method contract");
        let arity = contract.parameters.len();
        if !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || *evidence_argument_index != (arguments.len() > arity).then_some(arity)
        {
            return Err("Rust backend: unproven constrained method source staging".into());
        }
        if arguments.len() < arity {
            let mut completed = expr.clone();
            let CoreExpr::Call {
                arguments: completed_arguments,
                ..
            } = &mut completed
            else {
                unreachable!()
            };
            let parameters = contract
                .parameters
                .iter()
                .skip(arguments.len())
                .map(|type_ref| {
                    let id = format!("\0method_partial_{}", self.fresh());
                    completed_arguments.push(CoreExpr::Variable {
                        name: id.clone(),
                        evidence: vec![],
                        type_ref: type_ref.clone(),
                        origin: core_expr_origin(expr).clone(),
                    });
                    CoreParameter {
                        id,
                        kind: "named".into(),
                        type_ref: type_ref.clone(),
                    }
                })
                .collect::<Vec<_>>();
            return self.function(&parameters, &completed, scope);
        }
        self.validate_method_contract(dispatch, contract, arguments, evidence, scope)?;
        let primary = self.evidence(&evidence[..1], scope)?;
        let temporary = self.fresh();
        let mut value = format!(
            "{{ let mut {temporary} = ({primary}).remove(0).field({});",
            quoted(&dispatch.method)
        );
        // Source stages can fail or return closures. Run them at the same
        // position as TS, before materializing each method dictionary stage.
        for argument in &arguments[..arity] {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        for selected in &evidence[1..] {
            let dictionary = self.evidence(std::slice::from_ref(selected), scope)?;
            value.push_str(&format!(
                "{temporary} = {temporary}.apply(({dictionary}).remove(0));"
            ));
        }
        for argument in &arguments[arity..] {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(value)
    }
}
