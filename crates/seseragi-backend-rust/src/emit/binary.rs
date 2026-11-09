use super::*;

impl Backend<'_> {
    pub(super) fn binary_expression(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Binary {
            operator,
            left,
            right,
            evidence,
            type_ref,
            ..
        } = expr
        else {
            unreachable!()
        };
        Ok({
            if matches!(operator.as_str(), ".." | "..=") {
                return self.range_expression(operator, [left, right], evidence, type_ref, scope);
            }
            if operator == ":" {
                if !evidence.is_empty() || lists::list_element(type_ref).is_none() {
                    return Err(
                        "Rust backend: cons requires canonical List type and no evidence".into(),
                    );
                }
                self.capabilities.insert("list".into());
                return Ok(format!(
                    "V::list_cons({}, {})",
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            if self.program_wasm_library
                && operator == "**"
                && !matches!(evidence.as_slice(), [CoreCallEvidence {
                    evidence: CoreInstanceEvidence::Standard { identity, .. },
                    ..
                }] if identity == "std/int::Pow" || identity == "std/float::Pow")
            {
                return Err(
                    "program WASM: native power requires a target-specific arithmetic proof".into(),
                );
            }
            self.validate_opaque_numeric_operator_provider(
                operator,
                [left, right],
                evidence,
                scope,
            )?;
            if operators::big_int_binary(evidence, operator) {
                self.validate_primitive_operator_binary(
                    operator,
                    [left, right],
                    evidence,
                    type_ref,
                    scope,
                )?;
                self.capabilities.insert("big-int".into());
                return Ok(format!(
                    "native_big_int_arithmetic({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            if operators::decimal_binary(evidence, operator) {
                self.validate_primitive_operator_binary(
                    operator,
                    [left, right],
                    evidence,
                    type_ref,
                    scope,
                )?;
                self.capabilities.insert("decimal".into());
                return Ok(format!(
                    "native_decimal_arithmetic({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            if floats::float_binary(evidence, operator) {
                if operator == "**" {
                    self.validate_primitive_operator_binary(
                        operator,
                        [left, right],
                        evidence,
                        type_ref,
                        scope,
                    )?;
                }
                self.capabilities.insert("float".into());
                return Ok(format!(
                    "native_float_binary({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            if let Some(method) = dictionary_arithmetic(evidence, operator) {
                let dictionary = self.evidence(evidence, scope)?;
                let function = self.fresh();
                // A selected user method can execute after its first
                // source argument, before the second argument is evaluated.
                return Ok(format!("{{ let {function} = ({dictionary}).remove(0).field({}).apply({}); {function}.apply({}) }}", quoted(method), self.expr(left, scope)?, self.expr(right, scope)?));
            }
            if equality::is_equality(operator) {
                return self.equality_binary(operator, left, right, evidence, type_ref, scope);
            }
            if let Some(value) =
                self.descending_int_binary(operator, left, right, evidence, type_ref, scope)?
            {
                return Ok(value);
            }
            if characters::character_comparison(evidence, operator) {
                return Ok(format!(
                    "native_char_binary({}, {}, {})",
                    quoted(operator),
                    self.expr(left, scope)?,
                    self.expr(right, scope)?
                ));
            }
            if let Some(value) =
                self.decimal_comparison_binary(operator, left, right, evidence, type_ref, scope)?
            {
                return Ok(value);
            }
            if let Some(value) =
                self.big_int_comparison_binary(operator, left, right, evidence, type_ref, scope)?
            {
                return Ok(value);
            }
            if let Some(value) =
                self.standard_comparison_binary(operator, left, right, evidence, type_ref, scope)?
            {
                return Ok(value);
            }
            if characters::dictionary_comparison(evidence, operator) {
                let dictionary = self.evidence(evidence, scope)?;
                let a = self.fresh();
                let b = self.fresh();
                let d = self.fresh();
                if comparisons::is_comparison(operator) {
                    let stage = self.fresh();
                    return Ok(format!("{{ let {stage} = ({dictionary}).remove(0).field(\"compare\").apply({}); native_ordering_comparison({}, {stage}.apply({})) }}", self.expr(left, scope)?, quoted(operator), self.expr(right, scope)?));
                }
                return Ok(format!(
                "{{ let {d} = ({dictionary}).remove(0); let {a} = {}; let {b} = {}; native_dictionary_comparison({}, {d}, {a}, {b}) }}",
                self.expr(left, scope)?,
                self.expr(right, scope)?,
                quoted(operator)
            ));
            }
            if matches!(operator.as_str(), "/" | "%" | "**") {
                self.validate_primitive_operator_binary(
                    operator,
                    [left, right],
                    evidence,
                    type_ref,
                    scope,
                )?;
                return Ok({
                    let left = self.expr(left, scope)?;
                    let right = self.expr(right, scope)?;
                    self.located_binary(expr, operator, left, right)
                });
            }
            let standard = evidence.iter().all(|e| matches!(&e.evidence, CoreInstanceEvidence::Standard { identity, .. } if matches!(identity.as_str(), "std/int::Add" | "std/int::Sub" | "std/int::Mul" | "std/int::Eq" | "std/int::Ord" | "std/string::Add")));
            if !standard || !matches!(operator.as_str(), "+" | "-" | "*" | "==" | "<" | "<=") {
                return Err(format!(
                    "Rust backend: unsupported operator/evidence {operator} {evidence:?}"
                )
                .into());
            }
            {
                let left = self.expr(left, scope)?;
                let right = self.expr(right, scope)?;
                self.located_binary(expr, operator, left, right)
            }
        })
    }
}
