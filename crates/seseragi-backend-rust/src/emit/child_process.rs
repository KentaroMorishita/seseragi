use super::*;

fn ty(name: &str) -> CoreType {
    applied(name, vec![])
}
fn applied(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn arrow(parameters: &[CoreType], result: CoreType) -> CoreType {
    parameters
        .iter()
        .rev()
        .fold(result, |result, parameter| CoreType::Function {
            parameter: Box::new(parameter.clone()),
            result: Box::new(result),
        })
}
fn environment() -> CoreType {
    CoreType::Record {
        closed: true,
        fields: vec![seseragi_lowering::CoreRecordField {
            name: "childProcesses".into(),
            optional: false,
            type_ref: ty("std/child-process::ChildProcesses"),
        }],
    }
}
fn signature(name: &str, full: &CoreType) -> Result<Option<(Vec<CoreType>, CoreType)>> {
    let command = || ty("std/child-process::Command");
    let string = || ty("std/prelude::String");
    let config = |result| {
        applied(
            "std/prelude::Either",
            vec![ty("std/child-process::ChildProcessConfigError"), result],
        )
    };
    let effect = |result| {
        applied(
            "std/prelude::Effect",
            vec![
                environment(),
                ty("std/child-process::ChildProcessError"),
                result,
            ],
        )
    };
    Ok(Some(match name {
        "std/child-process::command" => {
            (vec![ty("std/child-process::Executable")], config(command()))
        }
        "std/child-process::addArgument" => (vec![string(), command()], config(command())),
        "std/child-process::addArguments" => (
            vec![applied("std/prelude::Array", vec![string()]), command()],
            config(command()),
        ),
        "std/child-process::inDirectory" => (vec![ty("std/path::Path"), command()], command()),
        "std/child-process::setEnvironment" => {
            (vec![string(), string(), command()], config(command()))
        }
        "std/child-process::unsetEnvironment" => (vec![string(), command()], config(command())),
        "std/child-process::clearEnvironment" => (vec![command()], command()),
        "std/child-process::terminationGrace" => {
            (vec![ty("std/time::Duration"), command()], command())
        }
        "std/child-process::outputBuffer" => {
            (vec![ty("std/stream::BufferCapacity"), command()], command())
        }
        "std/child-process::captureLimit" => (
            vec![ty("std/prelude::Int")],
            config(ty("std/child-process::CaptureLimit")),
        ),
        "std/child-process::defaultCaptureLimit" => (
            vec![ty("std/prelude::Unit")],
            ty("std/child-process::CaptureLimit"),
        ),
        "std/child-process::runCaptured" => (
            vec![
                ty("std/child-process::CaptureLimit"),
                ty("std/bytes::Bytes"),
                command(),
            ],
            effect(ty("std/child-process::CapturedProcess")),
        ),
        "std/child-process::runInherited" => (
            vec![command()],
            effect(ty("std/child-process::ChildExitStatus")),
        ),
        "std/child-process::runStreaming" => {
            let CoreType::Function { parameter, .. } = full else {
                return Err("Rust backend: missing child input Stream stage".into());
            };
            let CoreType::Named { name, arguments } = parameter.as_ref() else {
                return Err("Rust backend: child input requires a canonical Stream".into());
            };
            if name != "std/stream::Stream" || arguments.len() != 3 {
                return Err("Rust backend: unproven child input Stream".into());
            }
            let mut operands = match &arguments[0] {
                CoreType::RequirementMerge { operands } => operands.clone(),
                other => vec![other.clone()],
            };
            operands.push(environment());
            let mut fields = BTreeMap::new();
            let mut other = Vec::new();
            for operand in operands {
                match operand {
                    CoreType::Record {
                        closed: true,
                        fields: row,
                    } => {
                        for field in row {
                            if fields
                                .insert(field.name.clone(), field.clone())
                                .is_some_and(|previous| previous != field)
                            {
                                return Err(
                                    "Rust backend: conflicting ChildProcesses requirement".into()
                                );
                            }
                        }
                    }
                    operand => other.push(operand),
                }
            }
            let row = CoreType::Record {
                closed: true,
                fields: fields.into_values().collect(),
            };
            let merged = if other.is_empty() {
                row
            } else {
                other.push(row);
                CoreType::RequirementMerge { operands: other }
            };
            (
                vec![
                    applied(
                        "std/stream::Stream",
                        vec![
                            arguments[0].clone(),
                            arguments[1].clone(),
                            ty("std/child-process::ChildInput"),
                        ],
                    ),
                    command(),
                ],
                applied(
                    "std/stream::Stream",
                    vec![
                        merged,
                        applied(
                            "std/prelude::Either",
                            vec![
                                arguments[1].clone(),
                                ty("std/child-process::ChildProcessError"),
                            ],
                        ),
                        ty("std/child-process::ChildEvent"),
                    ],
                ),
            )
        }
        _ => return Ok(None),
    }))
}

impl Backend<'_> {
    fn child_process_type(
        &self,
        name: &str,
        full: &CoreType,
        scope: &Scope,
    ) -> Result<Option<usize>> {
        if !name.starts_with("std/child-process::") || scope.contains(name) {
            return Ok(None);
        }
        let full = self.local_canonical_type(full, scope)?;
        let Some((parameters, result)) = signature(name, &full)? else {
            return Ok(None);
        };
        if self.program_wasm_library
            || scope.has_unproven_type_scope()
            // The result requirement merge is derived from these checked inputs.
            || parameters
                .iter()
                .any(|parameter| self.declared_method_type_kind(parameter, scope) != Some(0))
            || full != arrow(&parameters, self.local_canonical_type(&result, scope)?)
        {
            return Err(format!(
                "Rust backend: unproven native child process argument or result: {name}"
            )
            .into());
        }
        Ok(Some(parameters.len()))
    }
    pub(super) fn child_process_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some(arity) = self.child_process_type(name, type_ref, scope)? else {
            return Ok(None);
        };
        if !evidence.is_empty() {
            return Err("Rust backend: unproven child process reference evidence".into());
        }
        self.capabilities
            .extend(["child-process".into(), "effect-sync".into()]);
        let names: Vec<_> = (0..arity)
            .map(|i| format!("\0child_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|n| format!("{}.clone()", id(n)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_child_process_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..arity).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }
    pub(super) fn child_process_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, valid, saturated) = match expr {
            CoreExpr::Call {
                callee,
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } => (
                callee,
                arguments,
                type_ref.clone(),
                evidence.is_empty()
                    && trait_dispatch.is_none()
                    && deferred_evidence_parameters.is_empty()
                    && deferred_evidence_type_constructor_parameters.is_empty()
                    && evidence_argument_index.is_none(),
                false,
            ),
            CoreExpr::EffectInvoke {
                callee,
                arguments,
                requirements,
                failure,
                success,
                evidence,
                ..
            } => (
                callee,
                arguments,
                applied(
                    "std/prelude::Effect",
                    vec![requirements.clone(), failure.clone(), success.clone()],
                ),
                evidence.is_empty(),
                true,
            ),
            _ => return Ok(None),
        };
        let supplied = if name.starts_with("std/child-process::") && !scope.contains(name) {
            arguments
                .iter()
                .map(|a| {
                    local::method_argument_type(a)
                        .ok_or_else(|| "Rust backend: missing child process argument type".into())
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            return Ok(None);
        };
        let full = arrow(&supplied, result);
        let Some(arity) = self.child_process_type(name, &full, scope)? else {
            return Ok(None);
        };
        if !valid || arguments.len() > arity || (saturated && arguments.len() != arity) {
            return Err("Rust backend: unproven child process source application".into());
        }
        let function = self
            .child_process_reference(name, &[], &full, scope)?
            .unwrap();
        let temporary = self.fresh();
        let mut value = format!("{{ let mut {temporary} = {function};");
        for argument in arguments {
            value.push_str(&format!(
                "{temporary} = {temporary}.apply({});",
                self.expr(argument, scope)?
            ));
        }
        value.push_str(&format!("{temporary} }}"));
        Ok(Some(value))
    }
}
