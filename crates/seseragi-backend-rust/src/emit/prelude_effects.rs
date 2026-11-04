use super::*;

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
}
