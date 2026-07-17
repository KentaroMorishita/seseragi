use crate::{
    TypeScriptDecisionBinding, TypeScriptDecisionBranch, TypeScriptDecisionProjection,
    TypeScriptDecisionTest, TypeScriptExpr, TypeScriptType,
};

use super::render_typescript_expr;
use crate::typescript::types::render_typescript_type;

const SCRUTINEE_NAME: &str = "$ssrg_match";

pub(super) fn render_decision(
    scrutinee: &TypeScriptExpr,
    scrutinee_type: &TypeScriptType,
    branches: &[TypeScriptDecisionBranch],
    type_ref: &TypeScriptType,
) -> String {
    let body = render_branches(branches, type_ref);
    format!(
        "(({SCRUTINEE_NAME}: {}): {} => {body})({})",
        render_typescript_type(scrutinee_type),
        render_typescript_type(type_ref),
        render_typescript_expr(scrutinee)
    )
}

fn render_branches(branches: &[TypeScriptDecisionBranch], result: &TypeScriptType) -> String {
    let Some((branch, rest)) = branches.split_first() else {
        return "((): never => { throw new Error(\"non-exhaustive Seseragi match\"); })()"
            .to_owned();
    };
    let value = render_with_bindings(
        &branch.bindings,
        &render_typescript_expr(&branch.value),
        result,
    );
    let Some(condition) = render_condition(branch) else {
        return value;
    };
    format!("{condition} ? {value} : {}", render_branches(rest, result))
}

fn render_condition(branch: &TypeScriptDecisionBranch) -> Option<String> {
    let mut parts = branch.tests.iter().map(render_test).collect::<Vec<_>>();
    if let Some(guard) = &branch.guard {
        parts.push(render_with_bindings(
            &branch.bindings,
            &render_typescript_expr(guard),
            &TypeScriptType::Boolean,
        ));
    }
    (!parts.is_empty()).then(|| parts.join(" && "))
}

fn render_test(test: &TypeScriptDecisionTest) -> String {
    match test {
        TypeScriptDecisionTest::BigintEquals { path, value } => format!(
            "{} === {value}n",
            render_projection_path(SCRUTINEE_NAME, path)
        ),
        TypeScriptDecisionTest::StringEquals { path, value } => format!(
            "{} === {value:?}",
            render_projection_path(SCRUTINEE_NAME, path)
        ),
        TypeScriptDecisionTest::BooleanEquals { path, value } => format!(
            "{} === {value}",
            render_projection_path(SCRUTINEE_NAME, path)
        ),
        TypeScriptDecisionTest::TagEquals { path, tag } => format!(
            "{}.tag === {tag:?}",
            render_projection_path(SCRUTINEE_NAME, path)
        ),
        TypeScriptDecisionTest::Invalid => "false".to_owned(),
    }
}

fn render_with_bindings(
    bindings: &[TypeScriptDecisionBinding],
    expression: &str,
    result: &TypeScriptType,
) -> String {
    if bindings.is_empty() {
        return expression.to_owned();
    }
    let parameters = bindings
        .iter()
        .map(|binding| {
            format!(
                "{}: {}",
                binding.name,
                render_typescript_type(&binding.type_ref)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = bindings
        .iter()
        .map(|binding| render_projection_path(SCRUTINEE_NAME, &binding.path))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "(({parameters}): {} => {expression})({arguments})",
        render_typescript_type(result)
    )
}

fn render_projection_path(root: &str, path: &[TypeScriptDecisionProjection]) -> String {
    path.iter()
        .fold(root.to_owned(), |value, projection| match projection {
            TypeScriptDecisionProjection::TupleElement { index } => {
                format!("{value}[{index}]")
            }
            TypeScriptDecisionProjection::RecordField { name } => {
                format!("{value}[{name:?}]")
            }
            TypeScriptDecisionProjection::AdtPayload => format!("{value}.value"),
        })
}

#[cfg(test)]
mod tests {
    use super::render_projection_path;
    use crate::TypeScriptDecisionProjection;

    #[test]
    fn renders_nested_tuple_payload_projection() {
        assert_eq!(
            render_projection_path(
                "$value",
                &[
                    TypeScriptDecisionProjection::TupleElement { index: 1 },
                    TypeScriptDecisionProjection::AdtPayload,
                ],
            ),
            "$value[1].value"
        );
    }
}
