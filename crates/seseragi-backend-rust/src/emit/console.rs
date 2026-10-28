use super::*;

fn is_console(name: &str) -> bool {
    matches!(
        name,
        "std/prelude::print"
            | "std/prelude::println"
            | "console.print"
            | "console.println"
            | "std/console::print"
            | "std/console::println"
            | "std/console::error"
            | "std/console::errorLine"
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
                    name: "console".into(),
                    optional: false,
                    type_ref: primitive("Console"),
                }],
            },
            primitive("ConsoleError"),
            primitive("Unit"),
        ],
    }
}

impl Backend<'_> {
    fn validate_console_type(
        &self,
        actual: &CoreType,
        expected: &CoreType,
        scope: &Scope,
    ) -> Result<()> {
        if self.declared_method_type_kind(actual, scope) != Some(0)
            || self.local_canonical_type(actual, scope)? != *expected
        {
            return Err("Rust backend: Console requires canonical String and its checked Console Effect contract; render values with Show or std/console.printValue".into());
        }
        Ok(())
    }

    pub(super) fn console_reference(
        &mut self,
        name: &str,
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<Option<String>> {
        if !is_console(name) || scope.contains(name) {
            return Ok(None);
        }
        if !evidence.is_empty() {
            return Err("Rust backend: Console operation with unsupported evidence".into());
        }
        self.validate_console_type(
            type_ref,
            &CoreType::Function {
                parameter: Box::new(primitive("String")),
                result: Box::new(result_type()),
            },
            scope,
        )?;
        self.require_operation(name);
        Ok(Some(format!(
            "V::fun(vec![], |_, text| native_operation({}, vec![text]))",
            quoted(name)
        )))
    }

    pub(super) fn console_call(
        &mut self,
        expr: &CoreExpr,
        scope: &Scope,
    ) -> Result<Option<String>> {
        let (name, arguments, output, valid, saturated) = match expr {
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
                CoreType::ExternalNamed {
                    name: "Effect".into(),
                    canonical: "std/prelude::Effect".into(),
                    arguments: vec![requirements.clone(), failure.clone(), success.clone()],
                },
                evidence.is_empty(),
                true,
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
                true,
            ),
            _ => return Ok(None),
        };
        if !is_console(name) || scope.contains(name) {
            return Ok(None);
        }
        if !valid || arguments.len() > 1 || (saturated && arguments.len() != 1) {
            return Err("Rust backend: unproven Console source application".into());
        }
        let Some(argument) = arguments.first() else {
            return self.console_reference(name, &[], &output, scope);
        };
        let input = local::method_argument_type(argument)
            .ok_or("Rust backend: missing Console text argument type")?;
        self.validate_console_type(&input, &primitive("String"), scope)?;
        self.validate_console_type(&output, &result_type(), scope)?;
        self.require_operation(name);
        // Evaluation stays at the call site exactly once. The native leaf
        // retains the String and only writes when the cold Effect is run.
        Ok(Some(format!(
            "native_operation({}, vec![{}])",
            quoted(name),
            self.expr(argument, scope)?
        )))
    }
}
