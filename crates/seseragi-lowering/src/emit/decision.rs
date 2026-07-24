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
    render_decision_with_value_renderer(
        scrutinee,
        scrutinee_type,
        branches,
        type_ref,
        &render_typescript_expr,
    )
}

pub(super) fn render_decision_with_value_renderer(
    scrutinee: &TypeScriptExpr,
    scrutinee_type: &TypeScriptType,
    branches: &[TypeScriptDecisionBranch],
    type_ref: &TypeScriptType,
    render_value: &dyn Fn(&TypeScriptExpr) -> String,
) -> String {
    let body = render_branches(branches, type_ref, render_value);
    format!(
        "(({SCRUTINEE_NAME}: {}): {} => {body})({})",
        render_typescript_type(scrutinee_type),
        render_typescript_type(type_ref),
        render_typescript_expr(scrutinee)
    )
}

fn render_branches(
    branches: &[TypeScriptDecisionBranch],
    result: &TypeScriptType,
    render_value: &dyn Fn(&TypeScriptExpr) -> String,
) -> String {
    let Some((branch, rest)) = branches.split_first() else {
        return "((): never => { throw new Error(\"non-exhaustive Seseragi match\"); })()"
            .to_owned();
    };
    let value = render_with_bindings(&branch.bindings, &render_value(&branch.value), result);
    let Some(condition) = render_condition(branch) else {
        return value;
    };
    format!(
        "{condition} ? {value} : {}",
        render_branches(rest, result, render_value)
    )
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
        TypeScriptDecisionTest::ArrayLength {
            path,
            length,
            minimum,
        } => format!(
            "{}.length {} {length}",
            render_projection_path(SCRUTINEE_NAME, path),
            if *minimum { ">=" } else { "===" }
        ),
        TypeScriptDecisionTest::ListLength {
            path,
            length,
            minimum,
        } => render_list_length_test(
            &render_projection_path(SCRUTINEE_NAME, path),
            *length,
            *minimum,
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
            TypeScriptDecisionProjection::ArrayElement { index } => {
                format!("{value}[{index}]")
            }
            TypeScriptDecisionProjection::ArrayRest { start } => {
                format!("{value}.slice({start})")
            }
            TypeScriptDecisionProjection::ListElement { index } => {
                format!("{}.head", render_list_tail(&value, *index))
            }
            TypeScriptDecisionProjection::ListRest { start } => render_list_tail(&value, *start),
            TypeScriptDecisionProjection::RecordField { name } => {
                format!("{value}[{name:?}]")
            }
            TypeScriptDecisionProjection::AdtPayload => format!("{value}.value"),
        })
}

fn render_list_length_test(value: &str, length: usize, minimum: bool) -> String {
    let mut parts = (0..length)
        .map(|index| format!("{}.tag === \"Cons\"", render_list_tail(value, index)))
        .collect::<Vec<_>>();
    if !minimum {
        parts.push(format!(
            "{}.tag === \"Empty\"",
            render_list_tail(value, length)
        ));
    }
    if parts.is_empty() {
        "true".to_owned()
    } else {
        parts.join(" && ")
    }
}

fn render_list_tail(value: &str, count: usize) -> String {
    (0..count).fold(value.to_owned(), |value, _| format!("{value}.tail"))
}

#[cfg(test)]
mod tests {
    use super::{render_list_length_test, render_projection_path};
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

    #[test]
    fn renders_array_and_list_rest_projections() {
        assert_eq!(
            render_projection_path(
                "$value",
                &[TypeScriptDecisionProjection::ArrayRest { start: 2 }],
            ),
            "$value.slice(2)"
        );
        assert_eq!(
            render_projection_path(
                "$value",
                &[TypeScriptDecisionProjection::ListElement { index: 2 }],
            ),
            "$value.tail.tail.head"
        );
        assert_eq!(
            render_projection_path(
                "$value",
                &[TypeScriptDecisionProjection::ListRest { start: 2 }],
            ),
            "$value.tail.tail"
        );
    }

    #[test]
    fn renders_list_length_tests_without_measuring_the_list() {
        assert_eq!(
            render_list_length_test("$value", 0, false),
            "$value.tag === \"Empty\""
        );
        assert_eq!(
            render_list_length_test("$value", 2, true),
            "$value.tag === \"Cons\" && $value.tail.tag === \"Cons\""
        );
    }
}
