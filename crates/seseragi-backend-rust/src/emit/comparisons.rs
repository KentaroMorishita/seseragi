use super::*;

pub(super) fn is_comparison(operator: &str) -> bool {
    matches!(operator, "<" | "<=" | ">" | ">=")
}

pub(super) fn int_ord_dictionary(selected: &CoreCallEvidence) -> Option<Result<String>> {
    let CoreInstanceEvidence::Standard {
        identity,
        type_arguments,
        evidence_arguments,
    } = &selected.evidence
    else {
        return None;
    };
    if identity != "std/int::Ord" {
        return None;
    }
    Some(
        if selected.constraint.name == "Ord"
            && selected
                .constraint
                .trait_identity
                .as_deref()
                .is_none_or(|id| id == "std/prelude::Ord")
            && matches!(selected.constraint.arguments.as_slice(), [ty] if canonical::primitive(ty, "Int"))
            && type_arguments.is_empty()
            && evidence_arguments.is_empty()
        {
            Ok("native_int_ord_dictionary()".into())
        } else {
            Err("Rust backend: invalid canonical Int Ord evidence".into())
        },
    )
}

impl Backend<'_> {
    pub(super) fn validate_ord_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        if selected.constraint.name != "Ord"
            || selected
                .constraint
                .trait_identity
                .as_deref()
                .is_some_and(|id| id != "std/prelude::Ord")
            || selected.constraint.arguments.len() != 1
        {
            return Err("Rust backend: comparison requires canonical Ord evidence".into());
        }
        let mut declared = selected.constraint.clone();
        declared.trait_identity = Some("std/prelude::Ord".into());
        self.validate_local_evidence(
            &LocalCallable::new(std::iter::empty(), &[declared], &[]),
            std::slice::from_ref(selected),
            scope,
        )?;
        if matches!(&selected.evidence, CoreInstanceEvidence::Standard { identity, .. } if identity == "std/int::Ord")
            && !self.comparison_builtin(&selected.constraint.arguments[0], "Int", scope)?
        {
            return Err("Rust backend: Int Ord cannot stand for a scoped type binder".into());
        }
        Ok(())
    }

    fn comparison_builtin(&self, ty: &CoreType, name: &str, scope: &Scope) -> Result<bool> {
        Ok(
            matches!(self.local_canonical_type(ty, scope)?, CoreType::Named { name: canonical, arguments } if canonical == format!("std/prelude::{name}") && arguments.is_empty()),
        )
    }
    pub(super) fn ord_supertrait(
        &self,
        instance: &seseragi_lowering::CoreInstance,
        scope: &Scope,
    ) -> Result<bool> {
        if instance.supertrait_count == 0 {
            return Ok(false);
        }
        let Some(parent) = instance.constraints.first() else {
            return Err("Rust backend: missing Ord parent evidence".into());
        };
        if instance.supertrait_count != 1
            || instance.trait_name != "Ord"
            || !instance.identity.starts_with("std/prelude::Ord<")
            || instance.arguments.len() != 1
            || parent.name != "Eq"
            || parent.trait_identity.as_deref() != Some("std/prelude::Eq")
            || parent.arguments.len() != 1
        {
            return Err("Rust backend: unproven canonical Ord supertrait".into());
        }
        self.match_local_type(
            &instance.arguments[0],
            &parent.arguments[0],
            &BTreeMap::new(),
            &mut BTreeMap::new(),
            scope,
        )?;
        let CoreInstanceImplementation::UserDefined { methods } = &instance.implementation else {
            return Err("Rust backend: unproven Ord implementation".into());
        };
        let [method] = methods.as_slice() else {
            return Err("Rust backend: invalid Ord method count".into());
        };
        if method.name != "compare"
            || !method.constraints.is_empty()
            || !method.type_parameters.is_empty()
            || !(1..=2).contains(&method.parameters.len())
        {
            return Err("Rust backend: unproven Ord method source arity".into());
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
        let mut result_type = local::method_argument_type(&method.body)
            .ok_or("Rust backend: unproven Ord method result")?;
        if method.parameters.len() == 1 {
            let CoreType::Function { parameter, result } = result_type else {
                return Err("Rust backend: unproven returned Ord stage".into());
            };
            self.match_local_type(
                &instance.arguments[0],
                &parameter,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
            result_type = *result;
        }
        if !self.comparison_builtin(&result_type, "Ordering", scope)? {
            return Err("Rust backend: Ord method result must be Ordering".into());
        }
        Ok(true)
    }
    pub(super) fn comparison_result_type(
        &self,
        type_ref: &CoreType,
        supplied: usize,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<()> {
        let [selected] = evidence else {
            return Err("Rust backend: comparison requires one Ord dictionary".into());
        };
        let [expected] = selected.constraint.arguments.as_slice() else {
            return Err("Rust backend: invalid comparison constraint arity".into());
        };
        let mut result_type = type_ref;
        for _ in supplied..2 {
            let CoreType::Function { parameter, result } = result_type else {
                return Err("Rust backend: invalid comparison result arity".into());
            };
            self.match_local_type(
                expected,
                parameter,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
            result_type = result;
        }
        if !self.comparison_builtin(result_type, "Bool", scope)? {
            return Err("Rust backend: comparison result must be Bool".into());
        }
        Ok(())
    }
    pub(super) fn comparison_reference(
        &mut self,
        operator: &str,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<String> {
        let [selected] = evidence else {
            return Err("Rust backend: comparison requires one Ord dictionary".into());
        };
        self.validate_ord_evidence(selected, scope)?;
        let dictionary = self.evidence(evidence, scope)?;
        // The first source stage runs on application of left, not on right.
        // Capture the resulting callable as a normal managed graph edge.
        Ok(format!("V::fun(vec![({dictionary}).remove(0).field(\"compare\")], |c, left| {{ let stage = c[0].apply(left); V::fun(vec![stage], |c, right| native_ordering_comparison({}, c[0].apply(right))) }})", quoted(operator)))
    }

    pub(super) fn comparison_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
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
            || !matches!(trait_dispatch, Some(d) if d.trait_identity == "std/prelude::Ord" && d.method == "compare" && d.method_contract.is_none())
        {
            return Err("Rust backend: unproven comparison source or dictionary staging".into());
        }
        let function = self.comparison_reference(callee, evidence, scope)?;
        let expected = &evidence[0].constraint.arguments[0];
        for argument in arguments {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: unproven comparison argument type")?;
            self.match_local_type(
                expected,
                &actual,
                &BTreeMap::new(),
                &mut BTreeMap::new(),
                scope,
            )?;
        }
        self.comparison_result_type(type_ref, arguments.len(), evidence, scope)?;
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
}
