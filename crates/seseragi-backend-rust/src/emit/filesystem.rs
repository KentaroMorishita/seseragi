use super::*;

fn ty(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}
fn applied(name: &str, arguments: Vec<CoreType>) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments,
    }
}
fn signature(name: &str) -> Option<(Vec<CoreType>, CoreType)> {
    let path = || ty("std/path::Path");
    let string = || ty("std/prelude::String");
    let unit = || ty("std/prelude::Unit");
    let either_path = || {
        applied(
            "std/prelude::Either",
            vec![ty("std/path::PathError"), path()],
        )
    };
    Some(match name {
        "std/path::current" => (vec![unit()], path()),
        "std/path::child" => (vec![string(), path()], either_path()),
        "std/path::parse" => (vec![string()], either_path()),
        "std/path::render" => (vec![path()], string()),
        "std/path::normalize" => (vec![path()], path()),
        "std/path::isAbsolute" => (vec![path()], ty("std/prelude::Bool")),
        "std/path::join" => (vec![path(), path()], either_path()),
        "std/path::parent" => (vec![path()], applied("std/prelude::Maybe", vec![path()])),
        "std/path::fileName" | "std/path::extension" => {
            (vec![path()], applied("std/prelude::Maybe", vec![string()]))
        }
        "std/fs::Replace" | "std/fs::CreateNew" | "std/fs::Append" => {
            (vec![], ty("std/fs::WriteMode"))
        }
        "std/fs::RegularFile"
        | "std/fs::Directory"
        | "std/fs::SymbolicLink"
        | "std/fs::OtherFileType" => (vec![], ty("std/fs::FileType")),
        "std/fs::exists" => (
            vec![path()],
            effect(ty("std/fs::FileSystemError"), ty("std/prelude::Bool")),
        ),
        "std/fs::metadata" | "std/fs::symlinkMetadata" => (
            vec![path()],
            effect(ty("std/fs::FileSystemError"), ty("std/fs::FileMetadata")),
        ),
        "std/fs::canonicalize" => (vec![path()], effect(ty("std/fs::FileSystemError"), path())),
        "std/fs::readTextUtf8" => (vec![path()], effect(ty("std/fs::FileTextError"), string())),
        "std/fs::readBytes" => (
            vec![path()],
            effect(ty("std/fs::FileSystemError"), ty("std/bytes::Bytes")),
        ),
        "std/fs::readChunks" => (
            vec![ty("std/stream::BufferCapacity"), path()],
            applied(
                "std/stream::Stream",
                vec![
                    filesystem_environment(),
                    ty("std/fs::FileSystemError"),
                    ty("std/bytes::Bytes"),
                ],
            ),
        ),
        "std/fs::writeBytes" => (
            vec![ty("std/fs::WriteMode"), ty("std/bytes::Bytes"), path()],
            effect(ty("std/fs::FileSystemError"), unit()),
        ),
        "std/fs::writeTextUtf8" => (
            vec![ty("std/fs::WriteMode"), string(), path()],
            effect(ty("std/fs::FileSystemError"), unit()),
        ),
        "std/fs::writeAtomic" => (
            vec![ty("std/bytes::Bytes"), path()],
            effect(ty("std/fs::FileSystemError"), unit()),
        ),
        "std/fs::createDirectory"
        | "std/fs::createDirectories"
        | "std/fs::removeFile"
        | "std/fs::removeDirectory" => {
            (vec![path()], effect(ty("std/fs::FileSystemError"), unit()))
        }
        "std/fs::move" => (
            vec![path(), path()],
            effect(ty("std/fs::FileSystemError"), unit()),
        ),
        "std/fs::list" => (
            vec![path()],
            applied(
                "std/stream::Stream",
                vec![
                    filesystem_environment(),
                    ty("std/fs::FileSystemError"),
                    ty("std/fs::DirectoryEntry"),
                ],
            ),
        ),
        _ => return None,
    })
}
fn effect(failure: CoreType, success: CoreType) -> CoreType {
    applied(
        "std/prelude::Effect",
        vec![filesystem_environment(), failure, success],
    )
}
fn filesystem_environment() -> CoreType {
    CoreType::Record {
        closed: true,
        fields: vec![seseragi_lowering::CoreRecordField {
            name: "fileSystem".into(),
            optional: false,
            type_ref: ty("std/fs::FileSystem"),
        }],
    }
}
fn merge_requirements(environment: CoreType) -> Result<CoreType> {
    let mut operands = match environment {
        CoreType::RequirementMerge { operands } => operands,
        environment => vec![environment],
    };
    operands.push(filesystem_environment());
    let mut fields = BTreeMap::new();
    let mut other = Vec::new();
    for operand in operands {
        match operand {
            CoreType::Record {
                closed: true,
                fields: row,
            } => {
                for field in row {
                    if let Some(previous) = fields.insert(field.name.clone(), field.clone()) {
                        if previous != field {
                            return Err(
                                "Rust backend: conflicting writeChunks filesystem requirement"
                                    .into(),
                            );
                        }
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
    if other.is_empty() {
        Ok(row)
    } else {
        other.push(row);
        Ok(CoreType::RequirementMerge { operands: other })
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

pub(super) fn display_contract(identity: &str) -> Option<(&'static str, &'static str)> {
    ["Show", "Debug"].into_iter().find_map(|method| {
        [
            "std/path::PathError",
            "std/fs::FileSystemOperation",
            "std/fs::FileSystemErrorKind",
            "std/fs::FileSystemError",
            "std/fs::FileType",
            "std/fs::FileMetadata",
            "std/fs::DirectoryEntry",
            "std/fs::WriteMode",
            "std/fs::FileTextError",
            "std/text::Utf8DecodeError",
        ]
        .into_iter()
        .find(|name| identity == format!("{method}<{name}>"))
        .map(|name| (method, name))
    })
}

impl Backend<'_> {
    fn filesystem_canonical_type(
        &self,
        name: &str,
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<CoreType> {
        fn normalize(type_ref: &mut CoreType) -> Result<()> {
            match type_ref {
                CoreType::Function { parameter, result } => {
                    normalize(parameter)?;
                    normalize(result)?;
                }
                CoreType::Named { name, arguments } => {
                    for argument in arguments.iter_mut() {
                        normalize(argument)?;
                    }
                    if name == "std/prelude::Effect" && arguments.len() == 3 {
                        if let CoreType::Record {
                            closed: true,
                            fields,
                        } = &mut arguments[0]
                        {
                            let mut unique = BTreeMap::new();
                            for field in fields.iter() {
                                if let Some(previous) =
                                    unique.insert(field.name.clone(), field.clone())
                                {
                                    if previous != *field {
                                        return Err("Rust backend: conflicting temporary filesystem requirement".into());
                                    }
                                }
                            }
                            *fields = unique.into_values().collect();
                        }
                    }
                }
                _ => (),
            }
            Ok(())
        }
        let mut canonical = self.local_canonical_type(type_ref, scope)?;
        if matches!(
            name,
            "std/fs::withTemporaryDirectory" | "std/fs::withTemporaryFile"
        ) {
            normalize(&mut canonical)?;
        }
        Ok(canonical)
    }

    fn filesystem_signature(
        &self,
        name: &str,
        full: &CoreType,
        scope: &Scope,
    ) -> Result<Option<(Vec<CoreType>, CoreType)>> {
        if matches!(
            name,
            "std/fs::withTemporaryDirectory" | "std/fs::withTemporaryFile"
        ) {
            if scope.has_unproven_type_scope() {
                return Err("Rust backend: unproven temporary filesystem kinds".into());
            }
            let full = self.filesystem_canonical_type(name, full, scope)?;
            let CoreType::Function { result, .. } = &full else {
                return Err("Rust backend: missing temporary prefix stage".into());
            };
            let CoreType::Function {
                parameter: callback,
                ..
            } = result.as_ref()
            else {
                return Err("Rust backend: missing temporary callback stage".into());
            };
            if self.declared_method_type_kind(callback, scope) != Some(0) {
                return Err("Rust backend: unproven temporary callback kinds".into());
            }
            let CoreType::Function { result, .. } = callback.as_ref() else {
                return Err("Rust backend: temporary callback must return Effect".into());
            };
            let CoreType::Named { name, arguments } = result.as_ref() else {
                return Err("Rust backend: temporary callback must return canonical Effect".into());
            };
            if name != "std/prelude::Effect" || arguments.len() != 3 {
                return Err("Rust backend: unproven temporary callback Effect".into());
            }
            return Ok(Some((
                vec![
                    ty("std/prelude::String"),
                    arrow(&[ty("std/path::Path")], result.as_ref().clone()),
                ],
                applied(
                    "std/prelude::Effect",
                    vec![
                        merge_requirements(arguments[0].clone())?,
                        applied(
                            "std/prelude::Either",
                            vec![ty("std/fs::FileSystemError"), arguments[1].clone()],
                        ),
                        arguments[2].clone(),
                    ],
                ),
            )));
        }
        if name != "std/fs::writeChunks" {
            return Ok(signature(name));
        }
        if scope.has_unproven_type_scope() {
            return Err("Rust backend: unproven filesystem type scope".into());
        }
        let mut remaining = full;
        let mut parameters = Vec::new();
        for _ in 0..3 {
            let CoreType::Function { parameter, result } = remaining else {
                return Err("Rust backend: missing writeChunks source stage".into());
            };
            parameters.push(*parameter.clone());
            remaining = result;
        }
        if self.declared_method_type_kind(&parameters[1], scope) != Some(0) {
            return Err("Rust backend: unproven writeChunks Stream kinds".into());
        }
        let parameters = parameters
            .iter()
            .map(|parameter| self.local_canonical_type(parameter, scope))
            .collect::<Result<Vec<_>>>()?;
        let CoreType::Named { name, arguments } = &parameters[1] else {
            return Err("Rust backend: writeChunks requires a canonical Stream".into());
        };
        if name != "std/stream::Stream" || arguments.len() != 3 {
            return Err("Rust backend: unproven writeChunks Stream type".into());
        }
        let result = applied(
            "std/prelude::Effect",
            vec![
                merge_requirements(arguments[0].clone())?,
                applied(
                    "std/prelude::Either",
                    vec![arguments[1].clone(), ty("std/fs::FileSystemError")],
                ),
                ty("std/prelude::Unit"),
            ],
        );
        Ok(Some((
            vec![
                ty("std/fs::WriteMode"),
                applied(
                    "std/stream::Stream",
                    vec![
                        arguments[0].clone(),
                        arguments[1].clone(),
                        ty("std/bytes::Bytes"),
                    ],
                ),
                ty("std/path::Path"),
            ],
            result,
        )))
    }

    pub(super) fn filesystem_display_dictionary(
        &mut self,
        selected: &CoreCallEvidence,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let seseragi_lowering::CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &selected.evidence
        else {
            return Ok(None);
        };
        let Some((method, name)) = display_contract(identity) else {
            return Ok(None);
        };
        if !type_arguments.is_empty()
            || !evidence_arguments.is_empty()
            || selected.constraint.arguments.len() != 1
            || !lists::standard_trait(selected, method, &selected.constraint.arguments[0])
            || self.local_canonical_type(&selected.constraint.arguments[0], scope)? != ty(name)
        {
            return Err("Rust backend: unproven native filesystem display evidence".into());
        }
        Ok(Some(format!(
            "native_filesystem_display_dictionary({}, {})",
            quoted(name),
            method == "Debug"
        )))
    }

    pub(super) fn filesystem_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let Some((parameters, result)) = self.filesystem_signature(name, type_ref, scope)? else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty()
            || self.filesystem_canonical_type(name, type_ref, scope)? != arrow(&parameters, result)
        {
            return Err("Rust backend: unproven native filesystem reference contract".into());
        }
        self.capabilities.insert(
            if name.starts_with("std/path::") {
                "path"
            } else {
                "filesystem-async"
            }
            .into(),
        );
        let names: Vec<_> = (0..parameters.len())
            .map(|i| format!("\0filesystem_argument_{i}"))
            .collect();
        let arguments = names
            .iter()
            .map(|name| format!("{}.clone()", id(name)))
            .collect::<Vec<_>>()
            .join(",");
        let mut body = format!(
            "native_filesystem_operation({}, vec![{arguments}])",
            quoted(name)
        );
        for i in (0..names.len()).rev() {
            body = Self::closure(&names[i], &body, &names[..i].iter().cloned().collect());
        }
        Ok(Some(body))
    }

    pub(super) fn filesystem_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, result, valid) = match expr {
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
            ),
            _ => return Ok(None),
        };
        if scope.contains(name)
            || (!matches!(
                name.as_str(),
                "std/fs::writeChunks"
                    | "std/fs::withTemporaryDirectory"
                    | "std/fs::withTemporaryFile"
            ) && signature(name).is_none())
        {
            return Ok(None);
        }
        if !valid {
            return Err("Rust backend: unproven native filesystem call evidence".into());
        }
        let supplied = arguments
            .iter()
            .map(|argument| {
                local::method_argument_type(argument)
                    .ok_or_else(|| "Rust backend: missing filesystem argument type".into())
            })
            .collect::<Result<Vec<_>>>()?;
        let (parameters, expected) = self
            .filesystem_signature(name, &arrow(&supplied, result.clone()), scope)?
            .unwrap();
        if arguments.len() > parameters.len() {
            return Err("Rust backend: native filesystem source arity mismatch".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing filesystem argument type")?;
            if self.filesystem_canonical_type(name, &actual, scope)? != *expected {
                return Err("Rust backend: native filesystem canonical argument mismatch".into());
            }
        }
        if self.filesystem_canonical_type(name, &result, scope)?
            != arrow(&parameters[arguments.len()..], expected.clone())
        {
            return Err("Rust backend: native filesystem canonical result mismatch".into());
        }
        let reference_type = arrow(&supplied, result);
        let function = self
            .filesystem_reference(name, &[], &reference_type, scope)?
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
