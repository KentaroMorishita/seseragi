use super::render_typescript_expr;
use crate::TypeScriptExpr;

#[test]
fn nul_before_each_ascii_digit_has_an_unambiguous_escape() {
    for digit in '0'..='9' {
        let value = format!("\0{digit}");
        let rendered = render_typescript_expr(&TypeScriptExpr::String {
            value: value.clone(),
        });
        assert_eq!(rendered, format!("\"\\u0000{digit}\""));
        assert_eq!(serde_json::from_str::<String>(&rendered).unwrap(), value);
    }
}

#[test]
fn string_literals_preserve_controls_backslashes_quotes_and_unicode() {
    for value in [
        "",
        "plain 'text'",
        "\0",
        "\0x1",
        "\0\01",
        "\01\09",
        "\\01",
        "\\\01",
        "\"\01\"",
        "雫\09🙂e\u{301}\u{2028}\u{2029}",
        "\u{1}\u{8}\u{b}\u{c}\n\r\t\\\"",
    ] {
        let rendered = render_typescript_expr(&TypeScriptExpr::String {
            value: value.to_owned(),
        });
        assert_eq!(serde_json::from_str::<String>(&rendered).unwrap(), value);
    }
}
