use super::*;

impl Backend<'_> {
    pub(super) fn range_expression(
        &mut self,
        operator: &str,
        operands: [&CoreExpr; 2],
        evidence: &[CoreCallEvidence],
        type_ref: &CoreType,
        scope: &Scope,
    ) -> Result<String> {
        if self.program_wasm_library {
            return Err("program WASM: native Range requires a target-specific ABI proof".into());
        }
        let int = CoreType::Named {
            name: "std/prelude::Int".into(),
            arguments: vec![],
        };
        let range = CoreType::Named {
            name: "std/prelude::Range".into(),
            arguments: vec![int.clone()],
        };
        if !evidence.is_empty() || self.local_canonical_type(type_ref, scope)? != range {
            return Err("Rust backend: Range requires canonical Range<Int> and no evidence".into());
        }
        for operand in operands {
            let actual = local::method_argument_type(operand)
                .ok_or("Rust backend: missing Range bound type")?;
            if self.declared_method_type_kind(&actual, scope) != Some(0)
                || self.local_canonical_type(&actual, scope)? != int
            {
                return Err("Rust backend: Range requires canonical Int bounds".into());
            }
        }
        self.capabilities.insert("range".into());
        Ok(format!(
            "native_range({}, {}, {})",
            self.expr(operands[0], scope)?,
            self.expr(operands[1], scope)?,
            operator == "..="
        ))
    }
}
