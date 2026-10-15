use super::*;
use seseragi_lowering::{CoreFunction, CoreScalarKind, CoreScalarTailLoop};

impl Backend<'_> {
    pub(super) fn scalar_direct_function(
        &mut self,
        function: &CoreFunction,
        plan: &CoreScalarTailLoop,
    ) -> Result<String> {
        let slots = function
            .parameters
            .iter()
            .map(|_| self.fresh())
            .collect::<Vec<_>>();
        let parameters = function
            .parameters
            .iter()
            .map(|p| format!(", {}: V", id(&p.id)))
            .collect::<String>();
        let mut setup = String::new();
        let mut snapshots = String::new();
        for ((parameter, kind), slot) in
            function.parameters.iter().zip(&plan.parameters).zip(&slots)
        {
            let tag = scalar_tag(*kind);
            setup.push_str(&format!(
                "let V::{tag}(mut {slot}) = {} else {{ panic!(\"Rust ABI: expected {tag}\") }};",
                id(&parameter.id)
            ));
            snapshots.push_str(&format!("let {} = {slot};", id(&parameter.id)));
        }
        let body = self.scalar_tail(&function.body, &slots, plan.result)?;
        self.self_tail_calls += plan.self_tail_calls;
        // V parameters/return and the generated curried wrappers keep their
        // ABI. Only the private saturated body uses immediate Rust scalars.
        Ok(format!("\nfn {}_direct(dicts: Vec<V>{parameters}) -> V {{ {setup} loop {{ {snapshots} {body} }} }}\n", id(&function.symbol)))
    }

    fn scalar_tail(
        &mut self,
        expr: &CoreExpr,
        slots: &[String],
        result: CoreScalarKind,
    ) -> Result<String> {
        match expr {
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => Ok(format!(
                "if {} {{ {} }} else {{ {} }}",
                self.scalar_expr(condition)?,
                self.scalar_tail(then_branch, slots, result)?,
                self.scalar_tail(else_branch, slots, result)?,
            )),
            // The shared analysis has excluded all calls except saturated self
            // calls at this exact tail position, with no hidden evidence.
            CoreExpr::Call { arguments, .. } => {
                let mut output = String::new();
                let mut next = Vec::new();
                for argument in arguments {
                    let temporary = self.fresh();
                    output.push_str(&format!(
                        "let {temporary} = {};",
                        self.scalar_expr(argument)?
                    ));
                    next.push(temporary);
                }
                for (slot, value) in slots.iter().zip(next) {
                    output.push_str(&format!("{slot} = {value};"));
                }
                output.push_str("continue;");
                Ok(output)
            }
            _ => Ok(format!(
                "return V::{}({});",
                scalar_tag(result),
                self.scalar_expr(expr)?
            )),
        }
    }

    fn scalar_expr(&mut self, expr: &CoreExpr) -> Result<String> {
        Ok(match expr {
            CoreExpr::Integer { value, .. } => format!(
                "{}_i64",
                value
                    .parse::<i64>()
                    .map_err(|_| "Rust backend: invalid scalar integer")?
            ),
            CoreExpr::Boolean { value, .. } => value.to_string(),
            CoreExpr::Variable { name, .. } => id(name),
            CoreExpr::Unary { operand, .. } => format!(
                "checked_int_value(({}).checked_neg().expect(\"Int overflow\"))",
                self.scalar_expr(operand)?
            ),
            CoreExpr::Binary {
                operator,
                left,
                right,
                ..
            } => {
                let a = self.fresh();
                let b = self.fresh();
                let left = self.scalar_expr(left)?;
                let right = self.scalar_expr(right)?;
                let operation = match operator.as_str() {
                    "+" => "int_add_value",
                    "-" => "int_subtract_value",
                    "*" => "int_multiply_value",
                    "==" => "int_equal_value",
                    "<" => "int_less_value",
                    "<=" => "int_less_equal_value",
                    _ => return Err("Rust backend: invalid certified scalar operator".into()),
                };
                format!("{{ let {a} = {left}; let {b} = {right}; {operation}({a}, {b}) }}")
            }
            CoreExpr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => format!(
                "if {} {{ {} }} else {{ {} }}",
                self.scalar_expr(condition)?,
                self.scalar_expr(then_branch)?,
                self.scalar_expr(else_branch)?,
            ),
            _ => return Err("Rust backend: invalid certified scalar expression".into()),
        })
    }
}

fn scalar_tag(kind: CoreScalarKind) -> &'static str {
    match kind {
        CoreScalarKind::Int => "Int",
        CoreScalarKind::Bool => "Bool",
    }
}
