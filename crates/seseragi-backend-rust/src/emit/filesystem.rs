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
        "std/fs::readTextUtf8" => (vec![path()], effect(ty("std/fs::FileTextError"), string())),
        "std/fs::writeTextUtf8" => (
            vec![ty("std/fs::WriteMode"), string(), path()],
            effect(ty("std/fs::FileSystemError"), unit()),
        ),
        _ => return None,
    })
}
fn effect(failure: CoreType, success: CoreType) -> CoreType {
    applied(
        "std/prelude::Effect",
        vec![
            CoreType::Record {
                closed: true,
                fields: vec![seseragi_lowering::CoreRecordField {
                    name: "fileSystem".into(),
                    optional: false,
                    type_ref: ty("std/fs::FileSystem"),
                }],
            },
            failure,
            success,
        ],
    )
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
        let Some((parameters, result)) = signature(name) else {
            return Ok(None);
        };
        if scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty()
            || self.local_canonical_type(type_ref, scope)? != arrow(&parameters, result)
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
        if scope.contains(name) || signature(name).is_none() {
            return Ok(None);
        }
        if !valid {
            return Err("Rust backend: unproven native filesystem call evidence".into());
        }
        let (parameters, expected) = signature(name).unwrap();
        if arguments.len() > parameters.len() {
            return Err("Rust backend: native filesystem source arity mismatch".into());
        }
        for (argument, expected) in arguments.iter().zip(&parameters) {
            let actual = local::method_argument_type(argument)
                .ok_or("Rust backend: missing filesystem argument type")?;
            if self.local_canonical_type(&actual, scope)? != *expected {
                return Err("Rust backend: native filesystem canonical argument mismatch".into());
            }
        }
        if self.local_canonical_type(&result, scope)?
            != arrow(&parameters[arguments.len()..], expected.clone())
        {
            return Err("Rust backend: native filesystem canonical result mismatch".into());
        }
        let reference_type = arrow(&parameters, expected);
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
