use super::*;
use seseragi_lowering::{CoreInstanceMethod, CoreTraitDispatch, CoreTraitMethodContract};

impl Backend<'_> {
    pub(super) fn validate_bare_method(
        &self,
        dispatch: &CoreTraitDispatch,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        let [selected] = evidence else {
            return Err("Rust backend: bare method requires one primary dictionary".into());
        };
        let mut declared = selected.constraint.clone();
        declared.trait_identity = Some(dispatch.trait_identity.clone());
        if !self.valid_constraint_label(&declared)
            || selected
                .constraint
                .trait_identity
                .as_deref()
                .is_some_and(|identity| identity != dispatch.trait_identity)
        {
            return Err("Rust backend: bare method dictionary trait mismatch".into());
        }
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[declared], &[]),
            evidence,
            scope,
        )?;
        let (identity, type_arguments) = match &selected.evidence {
            CoreInstanceEvidence::Parameter { .. } => {
                return self.validate_parameter_method_value(dispatch, selected, type_ref, scope);
            }
            CoreInstanceEvidence::Local {
                identity,
                type_arguments,
                ..
            }
            | CoreInstanceEvidence::Imported {
                identity,
                type_arguments,
                ..
            } => (identity, type_arguments),
            _ => return Ok(()), // Standard dictionaries have their own validators.
        };
        let instance = self
            .module
            .instances
            .iter()
            .find(|i| &i.identity == identity)
            .ok_or("Rust backend: unknown bare method instance")?;
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return if dispatch.trait_identity == "std/prelude::Show" && dispatch.method == "show" {
                Ok(())
            } else {
                Err("Rust backend: unproven derived method reference".into())
            };
        };
        let method = methods
            .iter()
            .find(|m| m.name == dispatch.method)
            .ok_or("Rust backend: unknown selected method")?;
        if !method.constraints.is_empty() {
            return Err("Rust backend: missing bare method constraint stages".into());
        }
        for (index, (parameter, actual)) in instance
            .type_parameters
            .iter()
            .zip(type_arguments)
            .enumerate()
        {
            if parameter.arity > 0 {
                let variable = format!("\0bare_instance_type_{index}");
                self.match_local_type(
                    &CoreType::Named {
                        name: variable.clone(),
                        arguments: vec![],
                    },
                    actual,
                    &BTreeMap::from([(variable, parameter.arity)]),
                    &mut BTreeMap::new(),
                    scope,
                )?;
            }
        }
        let mut substitutions = instance
            .type_parameters
            .iter()
            .zip(type_arguments)
            .map(|(p, ty)| {
                self.local_canonical_type(ty, scope)
                    .map(|ty| (p.name.clone(), ty))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        let mut variables = BTreeMap::new();
        for (index, parameter) in method.type_parameters.iter().enumerate() {
            if substitutions.contains_key(&parameter.name) {
                return Err("Rust backend: unproven overlapping method type scope".into());
            }
            let name = format!("\0bare_method_type_{index}");
            variables.insert(name.clone(), parameter.arity);
            substitutions.insert(
                parameter.name.clone(),
                CoreType::Named {
                    name,
                    arguments: vec![],
                },
            );
        }
        // Reconstruct source stages from the implementation's parameters and
        // checked body result, including a function returned by that body.
        // Dictionary stages are deliberately absent from this value type.
        let mut method_type = local::method_argument_type(&method.body)
            .ok_or("Rust backend: unproven selected method result")?;
        for parameter in method.parameters.iter().rev() {
            method_type = CoreType::Function {
                parameter: Box::new(parameter.type_ref.clone()),
                result: Box::new(method_type),
            };
        }
        let method_type = display::canonical_type(
            &method_type,
            &self.owners[identity],
            &substitutions,
            self.type_names,
            true,
        )
        .ok_or("Rust backend: unproven selected method value type")?;
        self.match_local_type(
            &method_type,
            type_ref,
            &variables,
            &mut BTreeMap::new(),
            scope,
        )
    }

    fn validate_parameter_method_value(
        &self,
        dispatch: &CoreTraitDispatch,
        selected: &CoreCallEvidence,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        // Eq is checked by equality_call. The remaining admitted source
        // parameter methods have the canonical A -> B -> C arithmetic ABI.
        if !matches!(
            (dispatch.trait_identity.as_str(), dispatch.method.as_str()),
            ("std/prelude::Add", "add")
                | ("std/prelude::Sub", "sub")
                | ("std/prelude::Mul", "mul")
                | ("std/prelude::Div", "div")
                | ("std/prelude::Rem", "rem")
                | ("std/prelude::Pow", "pow")
        ) || selected.constraint.arguments.len() != 3
        {
            return Err("Rust backend: unproven scoped method value signature".into());
        }
        let [left, right, result] = selected.constraint.arguments.as_slice() else {
            unreachable!()
        };
        let expected = CoreType::Function {
            parameter: Box::new(left.clone()),
            result: Box::new(CoreType::Function {
                parameter: Box::new(right.clone()),
                result: Box::new(result.clone()),
            }),
        };
        self.match_local_type(
            &expected,
            type_ref,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )
    }

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
                        callable_signature: None,
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
            let mut inner = scope.clone();
            for parameter in &parameters {
                inner.insert(parameter.id.clone());
            }
            let CoreExpr::Call {
                arguments: completed_arguments,
                ..
            } = &completed
            else {
                unreachable!()
            };
            self.validate_method_contract(
                dispatch,
                contract,
                completed_arguments,
                evidence,
                &inner,
            )?;
            let primary = self.evidence(&evidence[..1], scope)?;
            let stage = format!("\0method_stage_{}", self.fresh());
            let mut output = format!(
                "{{ let mut {} = ({primary}).remove(0).field({});",
                id(&stage),
                quoted(&dispatch.method)
            );
            for argument in arguments {
                output.push_str(&format!(
                    "{0} = {0}.apply({1});",
                    id(&stage),
                    self.expr(argument, scope)?
                ));
            }
            let result = self.fresh();
            let mut body = format!("{{ let mut {result} = {}.clone();", id(&stage));
            for parameter in &parameters {
                body.push_str(&format!(
                    "{result} = {result}.apply({}.clone());",
                    id(&parameter.id)
                ));
            }
            for selected in &evidence[1..] {
                let dictionary = self.evidence(std::slice::from_ref(selected), &inner)?;
                body.push_str(&format!(
                    "{result} = {result}.apply(({dictionary}).remove(0));"
                ));
            }
            body.push_str(&format!("{result} }}"));
            let mut captures = scope.clone();
            captures.retain(|name| name.starts_with("\0dictionary_"));
            captures.insert(stage.clone());
            for (index, parameter) in parameters.iter().enumerate().rev() {
                let mut previous = captures.clone();
                for earlier in &parameters[..index] {
                    previous.insert(earlier.id.clone());
                }
                body = Self::closure(&parameter.id, &body, &previous);
            }
            output.push_str(&body);
            output.push_str(" }");
            return Ok(output);
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
