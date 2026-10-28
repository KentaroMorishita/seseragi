use super::*;

pub(super) fn is_equality(operator: &str) -> bool {
    matches!(operator, "==" | "!=")
}

impl Backend<'_> {
    fn equality_builtin(&self, ty: &CoreType, name: &str, scope: &Scope) -> Result<bool> {
        Ok(
            matches!(self.local_canonical_type(ty, scope)?, CoreType::Named { name: actual, arguments }
            if actual == format!("std/prelude::{name}") && arguments.is_empty()),
        )
    }

    pub(super) fn primitive_equality_dictionary(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Option<Result<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return None;
        };
        let primitive = match identity.as_str() {
            "std/int::Eq" => "Int",
            "std/bool::Eq" => "Bool",
            "std/unit::Eq" => "Unit",
            "std/string::Eq" => "String",
            _ => return None,
        };
        Some((|| {
            self.validate_eq_evidence(selected, scope)?;
            if !type_arguments.is_empty()
                || !evidence_arguments.is_empty()
                || !self.equality_builtin(&selected.constraint.arguments[0], primitive, scope)?
            {
                return Err("Rust backend: invalid canonical primitive Eq evidence".into());
            }
            Ok(format!("native_operator_dictionary({})", quoted(identity)))
        })())
    }

    fn validate_eq_evidence(&self, selected: &CoreCallEvidence, scope: &Scope) -> Result<()> {
        if selected.constraint.arguments.len() != 1
            || selected
                .constraint
                .trait_identity
                .as_deref()
                .is_some_and(|id| id != "std/prelude::Eq")
            || (selected.constraint.name != "Eq"
                && self
                    .trait_names
                    .get(&(self.owner.to_owned(), selected.constraint.name.clone()))
                    .map(String::as_str)
                    != Some("std/prelude::Eq"))
        {
            return Err("Rust backend: equality requires canonical Eq evidence".into());
        }
        // Normalize only the proof label after checking its canonical owner.
        // The selected dictionary and its scoped argument identities stay intact.
        self.validate_parameter_evidence(selected, scope)?;
        let mut normalized = selected.clone();
        normalized.constraint.name = "Eq".into();
        normalized.constraint.trait_identity = Some("std/prelude::Eq".into());
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[normalized.constraint.clone()], &[]),
            std::slice::from_ref(&normalized),
            scope,
        )?;
        if let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        {
            if !type_arguments.is_empty() || !evidence_arguments.is_empty() {
                return Err("Rust backend: unexpected Eq type or child evidence arguments".into());
            }
            let primitive = match identity.as_str() {
                "std/int::Eq" => "Int",
                "std/bool::Eq" => "Bool",
                "std/unit::Eq" => "Unit",
                "std/char::Eq" => "Char",
                "std/string::Eq" => "String",
                _ => return Err("Rust backend: unsupported canonical Eq provider".into()),
            };
            if !self.equality_builtin(&selected.constraint.arguments[0], primitive, scope)? {
                return Err(
                    "Rust backend: primitive Eq cannot stand for a nominal or scoped binder".into(),
                );
            }
        }
        Ok(())
    }

    pub(super) fn validate_eq_instance(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<()> {
        if !instance.identity.starts_with("std/prelude::Eq<") {
            return Ok(());
        }
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven Eq implementation".into());
        };
        let [method] = methods.as_slice() else {
            return Err("Rust backend: invalid Eq method count".into());
        };
        if instance.arguments.len() != 1
            || instance.supertrait_count != 0
            || method.name != "eq"
            || !method.type_parameters.is_empty()
            || !method.constraints.is_empty()
            || !(1..=2).contains(&method.parameters.len())
        {
            return Err("Rust backend: unproven Eq method source arity".into());
        }
        for parameter in &method.parameters {
            self.match_local_type(
                &instance.arguments[0],
                &parameter.type_ref,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        let mut result = local::method_argument_type(&method.body)
            .ok_or("Rust backend: unproven Eq method result")?;
        if method.parameters.len() == 1 {
            let CoreType::Function {
                parameter,
                result: next,
            } = result
            else {
                return Err("Rust backend: unproven returned Eq stage".into());
            };
            self.match_local_type(
                &instance.arguments[0],
                &parameter,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
            result = *next;
        }
        if !self.equality_builtin(&result, "Bool", scope)? {
            return Err("Rust backend: Eq method result must be canonical Bool".into());
        }
        Ok(())
    }

    pub(super) fn equality_result_type(
        &self,
        type_ref: &CoreType,
        supplied: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        let [selected] = evidence else {
            return Err("Rust backend: equality requires one Eq dictionary".into());
        };
        self.validate_eq_evidence(selected, scope)?;
        if supplied > 2 {
            return Err("Rust backend: equality source arity mismatch".into());
        }
        let mut result = type_ref;
        for _ in supplied..2 {
            let CoreType::Function {
                parameter,
                result: next,
            } = result
            else {
                return Err("Rust backend: equality result stage mismatch".into());
            };
            self.match_local_type(
                &selected.constraint.arguments[0],
                parameter,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
            result = next;
        }
        if !self.equality_builtin(result, "Bool", scope)? {
            return Err("Rust backend: equality result must be canonical Bool".into());
        }
        Ok(())
    }

    pub(super) fn equality_reference(
        &mut self,
        operator: &str,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let [selected] = evidence else {
            return Err("Rust backend: equality requires one Eq dictionary".into());
        };
        self.validate_eq_evidence(selected, scope)?;
        let dictionary = self.evidence(evidence, scope)?;
        let method = format!("({dictionary}).remove(0).field(\"eq\")");
        if operator == "==" {
            return Ok(method);
        }
        // The selected method runs when left is applied. Store its returned
        // callable as a managed capture; negate only the eventual Bool result.
        Ok(format!("V::fun(vec![{method}], |c, left| {{ let stage = c[0].apply(left); V::fun(vec![stage], |c, right| V::Bool(!c[0].apply(right).boolean())) }})"))
    }

    pub(super) fn equality_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            type_ref,
            ..
        } = expr
        else {
            unreachable!()
        };
        if arguments.len() > 2
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
            || !matches!(trait_dispatch, Some(d) if d.trait_identity == "std/prelude::Eq" && d.method == "eq" && d.method_contract.is_none())
        {
            return Err("Rust backend: unproven equality source or dictionary staging".into());
        }
        self.equality_result_type(type_ref, arguments.len(), evidence, scope)?;
        for argument in arguments {
            self.equality_argument(&evidence[0], argument, scope)?;
        }
        let function =
            self.equality_reference(if callee == "!=" { "!=" } else { "==" }, evidence, scope)?;
        let stage = self.fresh();
        let mut value = format!("{{ let mut {stage} = {function};");
        for argument in arguments {
            value.push_str(&format!(
                "{stage} = {stage}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{stage} }}"));
        Ok(value)
    }

    fn equality_argument(
        &self,
        selected: &CoreCallEvidence,
        argument: &CoreExpr,
        scope: &Scope,
    ) -> Result<()> {
        let actual = local::method_argument_type(argument)
            .ok_or("Rust backend: unproven equality argument type")?;
        self.match_local_type(
            &selected.constraint.arguments[0],
            &actual,
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )?;
        Ok(())
    }

    pub(super) fn equality_binary(
        &mut self,
        operator: &str,
        left: &CoreExpr,
        right: &CoreExpr,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<String> {
        self.equality_result_type(type_ref, 2, evidence, scope)?;
        self.equality_argument(&evidence[0], left, scope)?;
        self.equality_argument(&evidence[0], right, scope)?;
        if let CoreInstanceEvidence::Standard { identity, .. } = &evidence[0].evidence {
            // These checked primitive first stages only capture left. Keep the
            // existing allocation-free fully applied intrinsic evaluation.
            if identity == "std/char::Eq" {
                return Ok(format!(
                    "native_char_binary({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            let value = format!(
                "binary(\"==\", {}, {})",
                self.expr(left, scope)?,
                self.expr(right, scope)?
            );
            return Ok(if operator == "==" {
                value
            } else {
                format!("V::Bool(!({value}).boolean())")
            });
        }
        let function = self.equality_reference(operator, evidence, scope)?;
        let stage = self.fresh();
        Ok(format!(
            "{{ let {stage} = ({function}).apply({}); {stage}.apply({}) }}",
            self.expr(left, scope)?,
            self.expr(right, scope)?
        ))
    }
}
