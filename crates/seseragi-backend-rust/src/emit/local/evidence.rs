use super::*;

fn label_matches(declared: &CoreInstanceConstraint, label: &str) -> bool {
    declared.name == label
        || declared.trait_identity.as_ref().is_some_and(|identity| {
            identity == &format!("std/prelude::{label}")
                || identity.ends_with(&format!("::trait({label})"))
        })
}

impl Backend<'_> {
    pub(in crate::emit) fn inherited_parameter_constraint(
        &self,
        declared: &CoreInstanceConstraint,
        selected: &CoreInstanceConstraint,
        scope: &Scope,
    ) -> bool {
        if declared.trait_identity.as_deref() == Some("std/prelude::Reducible")
            && selected.trait_identity.as_deref() == Some("std/prelude::Iterable")
            && !self.program_wasm_library
        {
            return self.valid_constraint_label(declared)
                && self.valid_constraint_label(selected)
                && declared.arguments.len() == 2
                && selected.arguments.len() == 2
                && declared
                    .arguments
                    .iter()
                    .chain(&selected.arguments)
                    .all(|ty| self.declared_method_type_kind(ty, scope) == Some(0));
        }
        // Canonical supertraits project fields from the same scoped frame.
        // Argument equality is checked by the caller, never by display name.
        let kind = match (
            declared.trait_identity.as_deref(),
            selected.trait_identity.as_deref(),
        ) {
            (Some("std/prelude::Ord"), Some("std/prelude::Eq")) => 0,
            (Some("std/prelude::Applicative"), Some("std/prelude::Functor"))
            | (
                Some("std/prelude::Monad"),
                Some("std/prelude::Applicative" | "std/prelude::Functor"),
            )
            | (Some("std/prelude::Traversable"), Some("std/prelude::Functor"))
                if !self.program_wasm_library =>
            {
                1
            }
            _ => return false,
        };
        self.valid_constraint_label(selected)
            && matches!(declared.arguments.as_slice(),[argument] if self.declared_method_type_kind(argument,scope)==Some(kind))
            && matches!(selected.arguments.as_slice(),[argument] if self.declared_method_type_kind(argument,scope)==Some(kind))
    }

    pub(in crate::emit) fn valid_constraint_label(
        &self,
        constraint: &CoreInstanceConstraint,
    ) -> bool {
        let Some(identity) = &constraint.trait_identity else {
            return false;
        };
        identity == &format!("std/prelude::{}", constraint.name)
            || identity.ends_with(&format!("::trait({})", constraint.name))
            || self
                .trait_names
                .get(&(self.owner.to_owned(), constraint.name.clone()))
                == Some(identity)
    }

    pub(in crate::emit) fn validate_parameter_evidence(
        &self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<()> {
        let CoreInstanceEvidence::Parameter { index } = selected.evidence else {
            return Ok(());
        };
        let declared = scope
            .dictionaries
            .get(index)
            .ok_or("Rust backend: dictionary parameter out of scope")?;
        let declared_arguments = declared
            .arguments
            .iter()
            .map(|ty| self.local_canonical_type(ty, scope))
            .collect::<Result<Vec<_>>>()?;
        let selected_arguments = selected
            .constraint
            .arguments
            .iter()
            .map(|ty| self.local_canonical_type(ty, scope))
            .collect::<Result<Vec<_>>>()?;
        let inherited = self.inherited_parameter_constraint(declared, &selected.constraint, scope);
        if !self.valid_constraint_label(declared)
            || declared_arguments != selected_arguments
            || (!inherited
                && (!label_matches(declared, &selected.constraint.name)
                    || selected
                        .constraint
                        .trait_identity
                        .as_ref()
                        .is_some_and(|identity| {
                            declared.trait_identity.as_ref() != Some(identity)
                        })))
        {
            return Err("Rust backend: dictionary parameter constraint mismatch".into());
        }
        Ok(())
    }

    pub(in crate::emit) fn validate_local_evidence(
        &self,
        callable: &LocalCallable,
        evidence: &[CoreCallEvidence],
        scope: &Scope,
    ) -> Result<BTreeMap<String, CoreType>> {
        if callable.constraints.len() != evidence.len() {
            return Err("Rust backend: local callable dictionary arity mismatch".into());
        }
        let mut bindings = BTreeMap::new();
        for (declared, selected) in callable.constraints.iter().zip(evidence) {
            if declared.arguments.len() != selected.constraint.arguments.len()
                || selected
                    .constraint
                    .trait_identity
                    .as_ref()
                    .is_some_and(|id| declared.trait_identity.as_ref() != Some(id))
            {
                return Err("Rust backend: local callable dictionary trait mismatch".into());
            }
            for (template, actual) in declared
                .arguments
                .iter()
                .zip(&selected.constraint.arguments)
            {
                self.match_local_type(template, actual, &callable.variables, &mut bindings, scope)?;
            }
            self.validate_parameter_evidence(selected, scope)?;
            match &selected.evidence {
                CoreInstanceEvidence::Parameter { index } => {
                    let parameter = &scope.dictionaries[*index];
                    if parameter.trait_identity != declared.trait_identity
                        && !self.inherited_parameter_constraint(
                            parameter,
                            &selected.constraint,
                            scope,
                        )
                    {
                        return Err("Rust backend: local callable parameter trait mismatch".into());
                    }
                }
                CoreInstanceEvidence::Local {
                    identity,
                    type_arguments,
                    evidence_arguments,
                }
                | CoreInstanceEvidence::Imported {
                    identity,
                    type_arguments,
                    evidence_arguments,
                    ..
                } => {
                    let instance = self
                        .module
                        .instances
                        .iter()
                        .find(|item| &item.identity == identity)
                        .ok_or("Rust backend: unknown local callable dictionary")?;
                    if declared.trait_identity.as_ref().is_none_or(|trait_id| {
                        !identity.starts_with(&format!("{trait_id}<"))
                            && !self.json_derived_identity(instance, trait_id)
                    }) || (!label_matches(declared, &selected.constraint.name)
                        && instance.trait_name != selected.constraint.name)
                        || instance.type_parameters.len() != type_arguments.len()
                        || instance.constraints.len() != evidence_arguments.len()
                    {
                        return Err(
                            "Rust backend: local callable instance identity or arity mismatch"
                                .into(),
                        );
                    }
                    let arguments = instance
                        .type_parameters
                        .iter()
                        .zip(type_arguments)
                        .map(|(p, ty)| {
                            self.local_canonical_type(ty, scope)
                                .map(|ty| (p.name.clone(), ty))
                        })
                        .collect::<Result<BTreeMap<_, _>>>()?;
                    let canonical = |ty| {
                        display::canonical_type(
                            ty,
                            &self.owners[identity],
                            &arguments,
                            self.type_names,
                            true,
                        )
                    };
                    let instantiated = instance
                        .arguments
                        .iter()
                        .map(canonical)
                        .collect::<Option<Vec<_>>>();
                    let actual = selected
                        .constraint
                        .arguments
                        .iter()
                        .map(|ty| self.local_canonical_type(ty, scope))
                        .collect::<Result<Vec<_>>>()?;
                    if instantiated.as_ref() != Some(&actual) {
                        return Err("Rust backend: local callable instance type mismatch".into());
                    }
                    for (child, argument) in instance.constraints.iter().zip(evidence_arguments) {
                        let expected = child
                            .arguments
                            .iter()
                            .map(canonical)
                            .collect::<Option<Vec<_>>>();
                        let actual = argument
                            .constraint
                            .arguments
                            .iter()
                            .map(|ty| self.local_canonical_type(ty, scope))
                            .collect::<Result<Vec<_>>>()?;
                        if expected.as_ref() != Some(&actual)
                            || argument
                                .constraint
                                .trait_identity
                                .as_ref()
                                .is_some_and(|id| child.trait_identity.as_ref() != Some(id))
                        {
                            return Err(
                                "Rust backend: local callable child dictionary mismatch".into()
                            );
                        }
                        let mut child = child.clone();
                        child.arguments = expected.expect("checked canonical child arguments");
                        self.validate_local_evidence(
                            &LocalCallable::new(std::iter::empty(), &[child], &[]),
                            std::slice::from_ref(argument),
                            scope,
                        )?;
                    }
                }
                CoreInstanceEvidence::Standard { .. } => {
                    if declared.trait_identity.as_deref()
                        != Some(format!("std/prelude::{}", selected.constraint.name).as_str())
                    {
                        return Err(
                            "Rust backend: local callable standard trait identity mismatch".into(),
                        );
                    }
                }
            }
        }
        Ok(bindings)
    }
}
