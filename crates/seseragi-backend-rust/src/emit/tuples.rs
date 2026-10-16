use super::*;

impl Backend<'_> {
    pub(super) fn tuple_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let method = match identity.as_str() {
            "std/tuple::Show" => "Show",
            "std/tuple::Debug" => "Debug",
            _ => return Ok(None),
        };
        // Structural identity comes from CoreType::Tuple, never a named type's
        // spelling or an inspection of the runtime value.
        let [tuple @ CoreType::Tuple { elements }] = type_arguments.as_slice() else {
            return Err("Rust backend: invalid canonical tuple display type".into());
        };
        if elements.len() < 2
            || elements.len() != evidence_arguments.len()
            || !lists::standard_trait(selected, method, tuple)
            || !elements
                .iter()
                .zip(evidence_arguments)
                .all(|(ty, child)| lists::standard_trait(child, method, ty))
        {
            return Err(
                "Rust backend: invalid canonical tuple display identity, arity or child".into(),
            );
        }
        for child in evidence_arguments {
            self.validate_display_child(child, method)?;
        }
        let children = self.evidence(evidence_arguments, scope)?;
        Ok(Some(format!(
            "native_tuple_display_dictionary({children}, {})",
            method == "Debug"
        )))
    }

    fn validate_display_child(&self, child: &CoreCallEvidence, method: &str) -> Result<()> {
        let [ty] = child.constraint.arguments.as_slice() else {
            return Err("Rust backend: tuple child constraint arity mismatch".into());
        };
        if !lists::standard_trait(child, method, ty) {
            return Err("Rust backend: tuple child trait identity mismatch".into());
        }
        match &child.evidence {
            CoreInstanceEvidence::Parameter { index } => {
                let constraint = self
                    .dictionary_constraints
                    .get(*index)
                    .ok_or("Rust backend: tuple child dictionary parameter out of scope")?;
                if constraint.name != method
                    || constraint.arguments != child.constraint.arguments
                    || constraint
                        .trait_identity
                        .as_deref()
                        .is_some_and(|id| id != format!("std/prelude::{method}"))
                {
                    return Err("Rust backend: tuple child dictionary parameter mismatch".into());
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
                    .find(|i| &i.identity == identity)
                    .ok_or("Rust backend: unknown tuple child dictionary")?;
                let owner = &self.owners[identity];
                let trait_identity = format!("std/prelude::{method}");
                if !identity.starts_with(&format!("{trait_identity}<"))
                    || !matches!(instance.trait_name.as_str(), name if name == method || name == trait_identity)
                    || type_arguments.len() != instance.type_parameters.len()
                    || evidence_arguments.len() != instance.constraints.len()
                {
                    return Err(
                        "Rust backend: tuple child instance identity or arity mismatch".into(),
                    );
                }
                let substitutions = instance
                    .type_parameters
                    .iter()
                    .zip(type_arguments)
                    .map(|(parameter, ty)| {
                        (
                            parameter.name.clone(),
                            canonical_type(
                                ty,
                                self.owner,
                                &BTreeMap::new(),
                                self.type_names,
                                false,
                            ),
                        )
                    })
                    .map(|(name, ty)| ty.map(|ty| (name, ty)))
                    .collect::<Option<BTreeMap<_, _>>>()
                    .ok_or("Rust backend: unproven tuple child type argument")?;
                let actual = instance
                    .arguments
                    .iter()
                    .map(|ty| canonical_type(ty, owner, &substitutions, self.type_names, true))
                    .collect::<Option<Vec<_>>>();
                let expected = child
                    .constraint
                    .arguments
                    .iter()
                    .map(|ty| {
                        canonical_type(ty, self.owner, &BTreeMap::new(), self.type_names, false)
                    })
                    .collect::<Option<Vec<_>>>();
                if actual.is_none() || actual != expected {
                    return Err(format!("Rust backend: tuple child instance type mismatch {identity}: {actual:?} != {expected:?}").into());
                }
                for (constraint, argument) in instance.constraints.iter().zip(evidence_arguments) {
                    let actual = constraint
                        .arguments
                        .iter()
                        .map(|ty| canonical_type(ty, owner, &substitutions, self.type_names, true))
                        .collect::<Option<Vec<_>>>();
                    let expected = argument
                        .constraint
                        .arguments
                        .iter()
                        .map(|ty| {
                            canonical_type(ty, self.owner, &BTreeMap::new(), self.type_names, false)
                        })
                        .collect::<Option<Vec<_>>>();
                    if constraint.name != argument.constraint.name
                        || actual.is_none()
                        || actual != expected
                        || (constraint.name == method
                            && constraint
                                .trait_identity
                                .as_deref()
                                .is_some_and(|identity| {
                                    identity != format!("std/prelude::{method}")
                                }))
                    {
                        return Err("Rust backend: tuple child dependency mismatch".into());
                    }
                    // Only Show/Debug children belong to this display proof.
                    // Other checked constraints use the existing evidence path.
                    if constraint.name == method {
                        self.validate_display_child(argument, method)?;
                    }
                }
            }
            CoreInstanceEvidence::Standard {
                evidence_arguments, ..
            } => {
                for argument in evidence_arguments {
                    self.validate_display_child(argument, method)?;
                }
            }
        }
        Ok(())
    }
}

// Compare selected user instance arguments in their defining module with the
// call site's canonical types, after substituting its checked type arguments.
// Instance templates use declaration scope; checked call-site Named scalars
// denote prelude types, and shadowing nominal types retain ExternalNamed identity.
fn canonical_type(
    ty: &CoreType,
    owner: &str,
    substitutions: &BTreeMap<String, CoreType>,
    type_names: &BTreeMap<(String, String), String>,
    declaration_scope: bool,
) -> Option<CoreType> {
    let recur = |ty| canonical_type(ty, owner, substitutions, type_names, declaration_scope);
    Some(match ty {
        CoreType::Hole => return None,
        CoreType::Named { name, arguments } => {
            let arguments = arguments.iter().map(recur).collect::<Option<Vec<_>>>()?;
            if let Some(bound) = substitutions.get(name) {
                let mut bound = bound.clone();
                match &mut bound {
                    CoreType::Named {
                        arguments: applied, ..
                    } => applied.extend(arguments),
                    _ if arguments.is_empty() => (),
                    _ => return None,
                }
                bound
            } else {
                let primitive = matches!(
                    name.as_str(),
                    "Unit" | "Int" | "Bool" | "String" | "Char" | "Float"
                );
                let name = if !declaration_scope && primitive {
                    format!("std/prelude::{name}")
                } else if let Some(canonical) = type_names.get(&(owner.to_owned(), name.clone())) {
                    canonical.clone()
                } else if name.contains("::") {
                    name.clone()
                } else if matches!(
                    name.as_str(),
                    "Unit"
                        | "Int"
                        | "Bool"
                        | "String"
                        | "Char"
                        | "Float"
                        | "List"
                        | "Array"
                        | "Maybe"
                        | "Either"
                        | "Ref"
                        | "Effect"
                ) {
                    format!("std/prelude::{name}")
                } else {
                    format!("{owner}::{name}")
                };
                CoreType::Named { name, arguments }
            }
        }
        CoreType::ExternalNamed {
            canonical,
            arguments,
            ..
        } => CoreType::Named {
            name: canonical.clone(),
            arguments: arguments.iter().map(recur).collect::<Option<Vec<_>>>()?,
        },
        CoreType::Tuple { elements } => CoreType::Tuple {
            elements: elements.iter().map(recur).collect::<Option<Vec<_>>>()?,
        },
        CoreType::Record { closed, fields } => CoreType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| {
                    let type_ref = recur(&field.type_ref)?;
                    let mut output = field.clone();
                    output.type_ref = type_ref;
                    Some(output)
                })
                .collect::<Option<Vec<_>>>()?,
        },
        CoreType::Function { parameter, result } => CoreType::Function {
            parameter: Box::new(recur(parameter)?),
            result: Box::new(recur(result)?),
        },
        CoreType::RequirementMerge { operands } => CoreType::RequirementMerge {
            operands: operands.iter().map(recur).collect::<Option<Vec<_>>>()?,
        },
    })
}

pub(super) fn closed_tuple_partial(
    module: &CoreModule,
    callee: &str,
    arguments: usize,
    evidence: &[CoreCallEvidence],
    deferred: &[CoreType],
) -> bool {
    fn closed(evidence: &CoreCallEvidence) -> bool {
        match &evidence.evidence {
            CoreInstanceEvidence::Parameter { .. } => false,
            CoreInstanceEvidence::Standard {
                evidence_arguments, ..
            }
            | CoreInstanceEvidence::Local {
                evidence_arguments, ..
            }
            | CoreInstanceEvidence::Imported {
                evidence_arguments, ..
            } => evidence_arguments.iter().all(closed),
        }
    }
    let Some(function) = module.functions.iter().find(|f| f.symbol == callee) else {
        return false;
    };
    // Bind only a complete, closed tuple dictionary before a fixed remaining
    // source arrow. Future arguments cannot select evidence or its staging.
    !function.is_effect && arguments < function.parameters.len()
        && function.parameters.len() - arguments == deferred.len()
        && function.parameters[arguments..].iter().zip(deferred).all(|(parameter, ty)| {
            &parameter.type_ref == ty && matches!(ty, CoreType::Named { name, arguments } if arguments.is_empty() && matches!(name.as_str(), "String" | "Int" | "Bool" | "Char" | "Float" | "Unit"))
        })
        && !evidence.is_empty() && evidence.len() == function.constraints.len()
        && evidence.iter().all(|selected| closed(selected) && matches!(&selected.evidence,
            CoreInstanceEvidence::Standard { identity, .. } if matches!(identity.as_str(), "std/tuple::Show" | "std/tuple::Debug")))
}
