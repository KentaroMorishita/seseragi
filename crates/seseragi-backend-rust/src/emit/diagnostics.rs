use super::*;

impl Backend<'_> {
    pub(super) fn source_location(&self, origin: &seseragi_lowering::SourceSpan) -> Option<String> {
        let source = self.options.sources.get(self.owner)?;
        let prefix = source.text.get(..origin.start)?;
        let line = prefix.bytes().filter(|b| *b == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next()?.chars().count() + 1;
        Some(format!(
            "&NativeSource {{ file: {}, line: {line}, column: {column} }}",
            quoted(&source.path)
        ))
    }

    pub(super) fn located_expr(&self, expr: &CoreExpr, value: String, scope: &Scope) -> String {
        if self.program_wasm_library {
            return value;
        }
        let Some(source) = self.source_location(core_expr_origin(expr)) else {
            return value;
        };
        match expr {
            CoreExpr::Call { callee, .. } | CoreExpr::EffectInvoke { callee, .. }
                if !scope.contains(callee)
                    && (self.functions.contains(callee)
                        || self.constructors.contains_key(callee)) =>
            {
                value
            }
            CoreExpr::EffectOperation { .. } => format!("native_locate_value({source}, {value})"),
            CoreExpr::Call { .. } | CoreExpr::EffectInvoke { .. } => {
                format!("native_source_value({source}, || {{ {value} }})")
            }
            CoreExpr::Variable { name, .. }
                if !scope.contains(name)
                    && (self.globals.contains_key(name)
                        || self
                            .globals
                            .contains_key(&format!("{}::{name}", self.owner))) =>
            {
                let index = self.globals.get(name).or_else(|| self.globals.get(&format!("{}::{name}", self.owner))).unwrap();
                format!("get_global_at({index}, {source})")
            }
            CoreExpr::Binary { operator, evidence, .. }
                if matches!(operator.as_str(), "+" | "-" | "*" | "/" | "%" | "**")
                    && !floats::float_binary(evidence, operator)
                    && !evidence.iter().all(|selected| matches!(
                        &selected.evidence,
                        CoreInstanceEvidence::Standard { identity, .. }
                            if matches!(identity.as_str(), "std/int::Add" | "std/int::Sub" | "std/int::Mul" | "std/int::Div" | "std/int::Rem" | "std/int::Pow" | "std/string::Add")
                    )) =>
            {
                format!("native_source({source}, || {{ {value} }})")
            }
            _ => value,
        }
    }

    pub(super) fn located_binary(
        &self,
        expr: &CoreExpr,
        operator: &str,
        left: String,
        right: String,
    ) -> String {
        match self
            .source_location(core_expr_origin(expr))
            .filter(|_| !self.program_wasm_library)
        {
            Some(source) if matches!(operator, "+" | "-" | "*" | "/" | "%" | "**") => format!(
                "binary_at({}, {left}, {right}, Some({source}))",
                quoted(operator)
            ),
            _ => format!("binary({}, {left}, {right})", quoted(operator)),
        }
    }
}

impl Backend<'_> {
    pub(super) fn entry_failure_show(
        &mut self,
        entry: &seseragi_lowering::CoreFunction,
    ) -> Result<String> {
        let scope = Scope::new();
        let ty = local::method_argument_type(&entry.body)
            .ok_or("Rust backend: unproven entry Effect type")?;
        let CoreType::Named { name, arguments } = self.local_canonical_type(&ty, &scope)? else {
            return Err("Rust backend: unproven entry Effect type".into());
        };
        if name != "std/prelude::Effect" || arguments.len() != 3 {
            return Err("Rust backend: unproven entry Effect type".into());
        }
        let failure = &arguments[1];
        if matches!(failure, CoreType::Named { name, arguments } if name == "std/prelude::Never" && arguments.is_empty())
        {
            return Ok(
                "fn entry_failure_show(_: V) -> V { panic!(\"Rust ABI: Never failure\") }".into(),
            );
        }
        let selected = self.failure_show_evidence(failure, 0)?;
        let dictionary = self.evidence(&[selected], &scope)?;
        Ok(format!("fn entry_failure_show(value: V) -> V {{ ({dictionary}).remove(0).field(\"show\").apply(value) }}"))
    }

    fn failure_show_evidence(&self, ty: &CoreType, depth: usize) -> Result<CoreCallEvidence> {
        if depth > 64 {
            return Err("Rust backend: recursive entry Show selection".into());
        }
        let scope = Scope::new();
        let mut candidates = Vec::new();
        for instance in &self.module.instances {
            if instance.arguments.len() != 1
                || (!instance.identity.starts_with("std/prelude::Show<")
                    && !self.derived_identity(instance, "std/prelude::Show"))
            {
                continue;
            }
            let owner = &self.owners[&instance.identity];
            let parameters = instance
                .type_parameters
                .iter()
                .map(|p| (p.name.clone(), p.arity))
                .collect::<BTreeMap<_, _>>();
            let names = parameters
                .keys()
                .map(|name| {
                    (
                        name.clone(),
                        CoreType::Named {
                            name: name.clone(),
                            arguments: vec![],
                        },
                    )
                })
                .collect();
            let Some(template) = display::canonical_type(
                &instance.arguments[0],
                owner,
                &names,
                self.type_names,
                true,
            ) else {
                continue;
            };
            let mut bindings = BTreeMap::new();
            if self
                .match_local_type(&template, ty, &parameters, &mut bindings, &scope)
                .is_err()
            {
                continue;
            }
            let type_arguments = instance
                .type_parameters
                .iter()
                .map(|p| {
                    bindings
                        .get(&p.name)
                        .cloned()
                        .ok_or("Rust backend: unbound entry Show type argument".into())
                })
                .collect::<Result<Vec<_>>>()?;
            let mut evidence_arguments = Vec::new();
            for constraint in &instance.constraints {
                if constraint.name != "Show"
                    || constraint.arguments.len() != 1
                    || constraint
                        .trait_identity
                        .as_deref()
                        .is_some_and(|id| id != "std/prelude::Show")
                {
                    return Err("Rust backend: unsupported entry Show prerequisite".into());
                }
                let child = display::canonical_type(
                    &constraint.arguments[0],
                    owner,
                    &bindings,
                    self.type_names,
                    true,
                )
                .ok_or("Rust backend: unproven entry Show prerequisite")?;
                evidence_arguments.push(self.failure_show_evidence(&child, depth + 1)?);
            }
            candidates.push(CoreInstanceEvidence::Imported {
                identity: instance.identity.clone(),
                provider_module: owner.clone(),
                type_arguments,
                evidence_arguments,
            });
        }
        let evidence = if candidates.len() == 1 {
            candidates.remove(0)
        } else if !candidates.is_empty() {
            return Err("Rust backend: ambiguous entry Show instance".into());
        } else {
            let (identity, type_arguments, children) = match ty {
                CoreType::Named { name, arguments } if arguments.is_empty() => {
                    (format!("Show<{name}>"), vec![], vec![])
                }
                CoreType::Named { name, arguments } => {
                    let owner =
                        match name.as_str() {
                            "std/prelude::Array" => "std/array",
                            "std/prelude::List" => "std/list",
                            "std/prelude::Maybe" => "std/maybe",
                            "std/prelude::Either" => "std/either",
                            _ => return Err(
                                "Rust backend: entry failure requires a supported Show instance"
                                    .into(),
                            ),
                        };
                    (
                        format!("{owner}::Show"),
                        arguments.clone(),
                        arguments.clone(),
                    )
                }
                CoreType::Tuple { elements } => {
                    ("std/tuple::Show".into(), vec![ty.clone()], elements.clone())
                }
                _ => {
                    return Err(
                        "Rust backend: entry failure requires a supported Show instance".into(),
                    )
                }
            };
            CoreInstanceEvidence::Standard {
                identity,
                type_arguments,
                evidence_arguments: children
                    .iter()
                    .map(|child| self.failure_show_evidence(child, depth + 1))
                    .collect::<Result<_>>()?,
            }
        };
        Ok(CoreCallEvidence {
            constraint: seseragi_lowering::CoreInstanceConstraint {
                name: "Show".into(),
                trait_identity: Some("std/prelude::Show".into()),
                arguments: vec![diagnostic_evidence_type(ty)],
            },
            evidence: diagnostic_evidence(evidence),
        })
    }
}

fn diagnostic_evidence_type(ty: &CoreType) -> CoreType {
    match ty {
        CoreType::Named { name, arguments } if name.contains("::") => CoreType::ExternalNamed {
            name: name.rsplit("::").next().unwrap().into(),
            canonical: name.clone(),
            arguments: arguments.iter().map(diagnostic_evidence_type).collect(),
        },
        CoreType::Tuple { elements } => CoreType::Tuple {
            elements: elements.iter().map(diagnostic_evidence_type).collect(),
        },
        CoreType::Record { closed, fields } => CoreType::Record {
            closed: *closed,
            fields: fields
                .iter()
                .map(|field| {
                    let mut field = field.clone();
                    field.type_ref = diagnostic_evidence_type(&field.type_ref);
                    field
                })
                .collect(),
        },
        _ => ty.clone(),
    }
}
fn diagnostic_evidence(evidence: CoreInstanceEvidence) -> CoreInstanceEvidence {
    match evidence {
        CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } => CoreInstanceEvidence::Standard {
            identity,
            type_arguments: type_arguments
                .iter()
                .map(diagnostic_evidence_type)
                .collect(),
            evidence_arguments,
        },
        other => other,
    }
}
