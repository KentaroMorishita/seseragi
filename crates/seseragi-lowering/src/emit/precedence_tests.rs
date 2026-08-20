use super::render_typescript_expr;
use crate::TypeScriptExpr;

fn identifier(name: &str) -> TypeScriptExpr {
    TypeScriptExpr::Identifier {
        name: name.to_owned(),
    }
}

fn number(value: &str) -> TypeScriptExpr {
    TypeScriptExpr::Number {
        value: value.to_owned(),
    }
}

fn binary(operator: &str, left: TypeScriptExpr, right: TypeScriptExpr) -> TypeScriptExpr {
    TypeScriptExpr::Binary {
        operator: operator.to_owned(),
        left: Box::new(left),
        right: Box::new(right),
    }
}

#[test]
fn preserves_grouped_calculator_arithmetic() {
    let numerator = binary("-", identifier("value"), number("1.0"));
    let denominator = binary("+", identifier("value"), number("1.0"));
    let ratio = binary("/", numerator, denominator);

    assert_eq!(
        render_typescript_expr(&ratio),
        "(value - 1.0) / (value + 1.0)"
    );

    let average = binary(
        "/",
        binary("-", identifier("positive"), identifier("negative")),
        number("2.0"),
    );
    assert_eq!(
        render_typescript_expr(&average),
        "(positive - negative) / 2.0"
    );
}

#[test]
fn preserves_equal_precedence_tree_shape_for_both_associativities() {
    let left_associative = binary(
        "-",
        identifier("a"),
        binary("-", identifier("b"), identifier("c")),
    );
    assert_eq!(render_typescript_expr(&left_associative), "a - (b - c)");

    let right_associative = binary(
        "**",
        binary("**", identifier("a"), identifier("b")),
        identifier("c"),
    );
    assert_eq!(render_typescript_expr(&right_associative), "(a ** b) ** c");

    let canonical_right_power = binary(
        "**",
        identifier("a"),
        binary("**", identifier("b"), identifier("c")),
    );
    assert_eq!(
        render_typescript_expr(&canonical_right_power),
        "a ** b ** c"
    );
}

#[test]
fn preserves_comparison_and_logical_precedence() {
    let comparison = binary(
        "===",
        binary("<", identifier("a"), identifier("b")),
        binary(">", identifier("c"), identifier("d")),
    );
    assert_eq!(render_typescript_expr(&comparison), "a < b === c > d");

    let nested_comparison = binary(
        "<",
        identifier("a"),
        binary(">", identifier("b"), identifier("c")),
    );
    assert_eq!(render_typescript_expr(&nested_comparison), "a < (b > c)");

    let logical = binary(
        "&&",
        binary("||", identifier("a"), identifier("b")),
        identifier("c"),
    );
    assert_eq!(render_typescript_expr(&logical), "(a || b) && c");

    let nullish = binary(
        "??",
        identifier("a"),
        binary("||", identifier("b"), identifier("c")),
    );
    assert_eq!(render_typescript_expr(&nullish), "a ?? (b || c)");
}

#[test]
fn parenthesizes_conditional_and_unary_power_operands() {
    let conditional = TypeScriptExpr::Conditional {
        condition: Box::new(identifier("condition")),
        then_branch: Box::new(identifier("left")),
        else_branch: Box::new(identifier("right")),
    };
    let sum = binary("+", conditional, number("1.0"));
    assert_eq!(
        render_typescript_expr(&sum),
        "(condition ? left : right) + 1.0"
    );

    let negative = TypeScriptExpr::Unary {
        operator: "-".to_owned(),
        operand: Box::new(number("2.0")),
    };
    let power = binary("**", negative, number("2.0"));
    assert_eq!(render_typescript_expr(&power), "(-(2.0)) ** 2.0");
}
