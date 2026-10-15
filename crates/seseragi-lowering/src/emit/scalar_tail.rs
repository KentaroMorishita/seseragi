use super::{render_conditional_condition, render_typescript_expr};
use crate::{TypeScriptExpr, TypeScriptParameter};

pub(super) fn render(
    body: &TypeScriptExpr,
    self_name: &str,
    parameters: &[TypeScriptParameter],
) -> String {
    let initial = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            format!(
                "let $ssrg$scalarSlot{index}: {} = {};",
                parameter.type_name, parameter.name
            )
        })
        .collect::<String>();
    // The outer curried arguments stay immutable. Every invocation of a saved
    // partial gets fresh slots, and each iteration gets its own source names.
    let snapshots = parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| {
            format!(
                "const {}: {} = $ssrg$scalarSlot{index};",
                parameter.name, parameter.type_name
            )
        })
        .collect::<String>();
    format!(
        "{{ {initial} while (true) {{ {snapshots} {} }} }}",
        tail(body, self_name, parameters.len())
    )
}

fn tail(expr: &TypeScriptExpr, self_name: &str, arity: usize) -> String {
    match expr {
        TypeScriptExpr::Conditional {
            condition,
            then_branch,
            else_branch,
        } => format!(
            "if ({}) {{ {} }} else {{ {} }}",
            render_conditional_condition(condition),
            tail(then_branch, self_name, arity),
            tail(else_branch, self_name, arity),
        ),
        TypeScriptExpr::Call {
            callee, arguments, ..
        } if callee == self_name && arguments.len() == arity => {
            let mut output = String::new();
            // Complete argument evaluation, including checked arithmetic, must
            // precede every loop-state assignment.
            for (index, argument) in arguments.iter().enumerate() {
                output.push_str(&format!(
                    "const $ssrg$scalarNext{index} = {};",
                    render_typescript_expr(argument)
                ));
            }
            for index in 0..arity {
                output.push_str(&format!(
                    "$ssrg$scalarSlot{index} = $ssrg$scalarNext{index};"
                ));
            }
            output.push_str("continue;");
            output
        }
        _ => format!("return {};", render_typescript_expr(expr)),
    }
}
