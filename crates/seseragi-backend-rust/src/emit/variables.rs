//! Validate references outside the recursive expression-dispatch frame.
use super::*;

impl Backend<'_> {
    pub(super) fn variable(&mut self, expr: &CoreExpr, scope: &Scope) -> Result<String> {
        let CoreExpr::Variable {
            name,
            evidence,
            type_ref,
            ..
        } = expr
        else {
            unreachable!("variable dispatch")
        };
        if let Some(value) =
            self.standard_error_constructor(name, &[], evidence, type_ref, scope)?
        {
            return Ok(value);
        }
        if let Some(value) =
            self.clock_reference(name, evidence, type_ref, core_expr_origin(expr), scope)?
        {
            return Ok(value);
        }
        if let Some(value) = self.prelude_effect_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.die_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.concurrency_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.from_either_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.filesystem_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.text_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.html_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.bytes_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.big_int_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.decimal_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.json_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.json_codec_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.json_decoder_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.collection_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.array_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.sequence_order_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.stream_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.validation_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.signal_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.iterator_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.iterable_build_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.collection_control_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.maybe_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.either_operation_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.sequence_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.math_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if let Some(value) = self.number_parse_reference(name, evidence, type_ref, scope)? {
            return Ok(value);
        }
        if equality::is_equality(name) && !scope.contains(name) {
            self.equality_result_type(type_ref, 0, evidence, scope)?;
        }
        if comparisons::is_comparison(name) && !scope.contains(name) {
            self.comparison_result_type(type_ref, 0, evidence, scope)?;
        }
        if operators::primitive_reference(name, evidence).is_some() && !scope.contains(name) {
            self.validate_primitive_operator_result(&evidence[0], 0, type_ref, scope)?;
        }
        if name == "std/float::power" && !scope.contains(name) {
            self.validate_float_power_type(type_ref, 0, scope)?;
        }
        self.reference(name, evidence, scope)
    }
}
