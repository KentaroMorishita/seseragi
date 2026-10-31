use super::*;

fn is_stdin(name: &str) -> bool {
    matches!(
        name,
        "std/prelude::readLine" | "std/stdin::readLine" | "stdin.readLine"
    )
}

fn primitive(name: &str) -> CoreType {
    CoreType::Named {
        name: format!("std/prelude::{name}"),
        arguments: vec![],
    }
}

fn result_type() -> CoreType {
    CoreType::Named {
        name: "std/prelude::Effect".into(),
        arguments: vec![
            CoreType::Record {
                closed: true,
                fields: vec![seseragi_lowering::CoreRecordField {
                    name: "stdin".into(),
                    optional: false,
                    type_ref: primitive("Stdin"),
                }],
            },
            CoreType::Named {
                name: "std/prelude::StdinError".into(),
                arguments: vec![],
            },
            CoreType::Named {
                name: "std/prelude::Maybe".into(),
                arguments: vec![primitive("String")],
            },
        ],
    }
}

impl Backend<'_> {
    pub(super) fn prelude_effect_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if let Some(value) = self.stdin_reference(name, evidence, type_ref, scope)? {
            return Ok(Some(value));
        }
        self.console_reference(name, evidence, type_ref, scope)
    }

    pub(super) fn prelude_effect_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if let Some(value) = self.stdin_call(expr, scope)? {
            return Ok(Some(value));
        }
        if let Some(value) = self.from_either_call(expr, scope)? {
            return Ok(Some(value));
        }
        self.console_call(expr, scope)
    }

    fn validate_stdin_type(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err(
                "Rust backend: readLine requires its checked canonical Stdin Effect contract"
                    .into(),
            );
        }
        Ok(())
    }

    pub(super) fn stdin_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !is_stdin(name) || scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native readLine requires a target-specific ABI proof".into(),
            );
        }
        if !evidence.is_empty() {
            return Err("Rust backend: Stdin operation with unsupported evidence".into());
        }
        self.validate_stdin_type(type_ref, &result_type(), scope)?;
        self.require_operation(name);
        self.capabilities.insert("stdin".into());
        Ok(Some(format!("native_operation({}, vec![])", quoted(name))))
    }

    pub(super) fn stdin_call(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<Option<String>> {
        let (name, arguments, output, valid) = match expr {
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
                CoreType::ExternalNamed {
                    name: "Effect".into(),
                    canonical: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
                evidence.is_empty(),
            ),
            CoreExpr::EffectOperation {
                operation,
                arguments,
                requirements,
                failure,
                success,
                ..
            } => (
                operation,
                arguments,
                CoreType::ExternalNamed {
                    name: "Effect".into(),
                    canonical: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
                true,
            ),
            _ => return Ok(None),
        };
        if !is_stdin(name) || scope.contains(name) {
            return Ok(None);
        }
        if self.program_wasm_library {
            return Err(
                "program WASM: native readLine requires a target-specific ABI proof".into(),
            );
        }
        if !valid || !arguments.is_empty() {
            return Err("Rust backend: unproven readLine source application".into());
        }
        self.stdin_reference(name, &[], &output, scope)
    }
}
