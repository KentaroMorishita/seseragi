use super::*;

fn operation(name: &str) -> Option<(&str, usize)> {
    if name == "std/collection::NonPositiveSize" {
        return Some(("NonPositiveSize", 1));
    }
    let (owner, name) = name.split_once("::")?;
    if !matches!(owner, "std/array" | "std/list") {
        return None;
    }
    Some((
        name,
        match name {
            "last" | "init" | "unzip" | "concat" => 1,
            "findIndex" | "takeWhile" | "dropWhile" | "zip" | "chunksOf" | "windows" => 2,
            "zipWith" | "reduceRight" => 3,
            _ => return None,
        },
    ))
}

fn named(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}

fn function(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}

fn element<'a>(ty: &'a CoreType, owner: &str) -> Result<&'a CoreType> {
    let CoreType::Named { name, arguments } = ty else {
        return Err("Rust backend: unproven sequence collection type".into());
    };
    if name != owner || arguments.len() != 1 {
        return Err("Rust backend: unproven sequence canonical owner or arity".into());
    }
    Ok(&arguments[0])
}

impl Backend<'_> {
    fn sequence_type(
        &self,
        name: &str,
        arguments: &[CoreExpr],
        result: &CoreType,
        scope: &Scope,
    ) -> Result<usize> {
        if self.program_wasm_library {
            return Err(
                "program WASM: native sequence operation requires a target-specific ABI proof"
                    .into(),
            );
        }
        let (operation, arity) = operation(name).expect("known sequence operation");
        if scope.has_unproven_type_scope() || arguments.len() > arity {
            return Err("Rust backend: unproven sequence source application".into());
        }
        let actual = arguments
            .iter()
            .map(|a| {
                local::method_argument_type(a)
                    .ok_or_else(|| "Rust backend: missing sequence argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let full = function(&actual, result.clone());
        if self.declared_method_type_kind(&full, scope) != Some(0) {
            return Err("Rust backend: unproven sequence argument/result kind".into());
        }
        let full = self.local_canonical_type(&full, scope)?;
        let mut remaining = &full;
        let mut parameters = Vec::new();
        for _ in 0..arity {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: unproven sequence source arity".into());
            };
            parameters.push(parameter.as_ref().clone());
            remaining = result;
        }
        let int = named("std/prelude::Int", vec![]);
        let (expected, output) = if operation == "NonPositiveSize" {
            (vec![int], named("std/collection::SizeError", vec![]))
        } else {
            let owner = if name.starts_with("std/list::") {
                "std/prelude::List"
            } else {
                "std/prelude::Array"
            };
            let a = element(&parameters[arity - 1], owner)?.clone();
            let collection = |a| named(owner, vec![a]);
            let maybe = |a| named("std/prelude::Maybe", vec![a]);
            let predicate = function(&[a.clone()], named("std/prelude::Bool", vec![]));
            match operation {
                "findIndex" => (vec![predicate, collection(a)], maybe(int)),
                "takeWhile" | "dropWhile" => {
                    (vec![predicate, collection(a.clone())], collection(a))
                }
                "last" => (vec![collection(a.clone())], maybe(a)),
                "init" => (vec![collection(a.clone())], maybe(collection(a))),
                "concat" => {
                    let inner = element(&a, owner)?.clone();
                    (vec![collection(a)], collection(inner))
                }
                "zip" => {
                    let b = element(&parameters[0], owner)?.clone();
                    (
                        vec![collection(b.clone()), collection(a.clone())],
                        collection(CoreType::Tuple {
                            elements: vec![a, b],
                        }),
                    )
                }
                "zipWith" => {
                    let b = element(&parameters[1], owner)?.clone();
                    let c = element(remaining, owner)?.clone();
                    (
                        vec![
                            function(&[a.clone(), b.clone()], c.clone()),
                            collection(b),
                            collection(a),
                        ],
                        collection(c),
                    )
                }
                "unzip" => {
                    let CoreType::Tuple { elements: parts } = &a else {
                        return Err("Rust backend: sequence unzip requires a pair".into());
                    };
                    if parts.len() != 2 {
                        return Err("Rust backend: sequence unzip requires a pair".into());
                    }
                    (
                        vec![collection(a.clone())],
                        CoreType::Tuple {
                            elements: parts.iter().cloned().map(collection).collect(),
                        },
                    )
                }
                "chunksOf" | "windows" => (
                    vec![int, collection(a.clone())],
                    named(
                        "std/prelude::Either",
                        vec![
                            named("std/collection::SizeError", vec![]),
                            collection(collection(a)),
                        ],
                    ),
                ),
                "reduceRight" => {
                    let b = parameters[0].clone();
                    (
                        vec![
                            b.clone(),
                            function(&[a.clone(), b.clone()], b.clone()),
                            collection(a),
                        ],
                        b,
                    )
                }
                _ => unreachable!(),
            }
        };
        if full != function(&expected, output) {
            return Err("Rust backend: sequence canonical type, element or stage mismatch".into());
        }
        Ok(arity)
    }

    fn sequence_callable(name: &str, arity: usize) -> String {
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0sequence_argument_{i}"))
            .collect();
        let args = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut value = format!("native_sequence_operation({}, vec![{args}])", quoted(name));
        for i in (0..arity).rev() {
            value = Self::closure(&names[i], &value, &names[..i].iter().cloned().collect());
        }
        value
    }

    pub(super) fn sequence_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        ty: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if operation(name).is_none() || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: sequence operation with unsupported evidence".into());
        }
        let arity = self.sequence_type(name, &[], ty, scope)?;
        self.capabilities.insert("sequence".into());
        Ok(Some(Self::sequence_callable(name, arity)))
    }

    pub(super) fn sequence_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let CoreExpr::Call {
            callee,
            arguments,
            type_ref,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            ..
        } = expr
        else {
            return Ok(None);
        };
        if operation(callee).is_none() || scope.contains(callee) {
            return Ok(None);
        }
        if !evidence.is_empty()
            || trait_dispatch.is_some()
            || !deferred_evidence_parameters.is_empty()
            || !deferred_evidence_type_constructor_parameters.is_empty()
            || evidence_argument_index.is_some()
        {
            return Err("Rust backend: unproven sequence source application".into());
        }
        let arity = self.sequence_type(callee, arguments, type_ref, scope)?;
        self.capabilities.insert("sequence".into());
        if arguments.len() == arity {
            self.direct_calls += 1;
            return Ok(Some(format!(
                "native_sequence_operation({}, vec![{}])",
                quoted(callee),
                self.expressions(arguments, scope)?
            )));
        }
        let value = self.fresh();
        let mut output = format!(
            "{{ let mut {value} = {};",
            Self::sequence_callable(callee, arity)
        );
        for arg in arguments {
            output.push_str(&format!(
                "{value} = {value}.apply({});",
                self.expr(arg, scope)?
            ));
        }
        output.push_str(&format!("{value} }}"));
        Ok(Some(output))
    }
}
