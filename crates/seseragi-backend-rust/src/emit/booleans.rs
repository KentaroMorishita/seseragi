use super::*;

impl Backend<'_> {
    pub(super) fn boolean_not(
        &mut self,
        operand: &CoreExpr,
        result: &CoreType,
        scope: &Scope,
    ) -> Result<String> {
        let input = local::method_argument_type(operand)
            .ok_or("Rust backend: unproven Bool negation operand")?;
        for ty in [&input, result] {
            if self.declared_method_type_kind(ty, scope) != Some(0)
                || !matches!(self.local_canonical_type(ty, scope)?,
                    CoreType::Named { name, arguments }
                    if name == "std/prelude::Bool" && arguments.is_empty())
            {
                return Err(
                    "Rust backend: Bool negation requires canonical Bool input and result".into(),
                );
            }
        }
        // Prefix ! has one source operand and no callable/partial form.
        // Keep its strict evaluation once before negating the checked value.
        Ok(format!(
            "V::Bool(!({}).boolean())",
            self.expr(operand, scope)?
        ))
    }
}
