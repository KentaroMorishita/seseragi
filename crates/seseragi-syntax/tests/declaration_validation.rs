use seseragi_syntax::{parse_diagnostics, DiagnosticSeverity};

fn assert_error(source: &str, key: &str, highlighted: &str) {
    let artifact = parse_diagnostics("main.ssrg", source);
    let diagnostic = artifact
        .diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.message_key == key
                && &source[diagnostic.primary.start..diagnostic.primary.end] == highlighted
        })
        .unwrap_or_else(|| panic!("missing {key} for {source:?}: {artifact:?}"));
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(
        &source[diagnostic.primary.start..diagnostic.primary.end],
        highlighted
    );
    assert!(!diagnostic.helps().is_empty());
}

#[test]
fn rejects_unused_invalid_declarations_at_the_header() {
    for (declaration, key, highlighted) in [
        (
            "fn add left -> Int = left",
            "parser.function-annotations-required",
            "fn add left -> Int",
        ),
        (
            "fn add left: Int = left",
            "parser.function-annotations-required",
            "fn add left: Int",
        ),
        (
            "fn add left right: Int -> Int = right",
            "parser.function-annotations-required",
            "fn add left right: Int -> Int",
        ),
        (
            "pub let count = 3",
            "parser.public-let-annotation-required",
            "count",
        ),
        (
            "let 1value = 1",
            "parser.invalid-declaration-name",
            "1value",
        ),
        ("let match = 1", "parser.invalid-declaration-name", "match"),
        ("let where = 1", "parser.invalid-declaration-name", "where"),
        (
            "fn 1value -> Int = 1",
            "parser.invalid-declaration-name",
            "1value",
        ),
        (
            "fn Upper -> Int = 1",
            "parser.invalid-declaration-name",
            "Upper",
        ),
        ("fn = 1", "parser.invalid-declaration-name", "fn"),
        (
            "fn alias -> Int = 1",
            "parser.invalid-declaration-name",
            "alias",
        ),
        (
            "fn value Upper: Int -> Int = 1",
            "parser.invalid-declaration-name",
            "Upper",
        ),
    ] {
        let source = format!("// 雫\n{declaration}\npub effect fn main = println \"check\"\n");
        assert_error(&source, key, highlighted);
    }
}

#[test]
fn rejects_module_expressions_before_between_and_after_declarations() {
    for source in [
        "println \"unexpected\"\n",
        "println \"unexpected\"\npub effect fn main = println \"main\"\n",
        "let value = 1\nprintln \"unexpected\"\npub effect fn main = println \"main\"\n",
        "let value = 1; println \"unexpected\"; pub effect fn main = println \"main\"\n",
        "pub effect fn main = println \"main\"\nprintln \"unexpected\"\n",
        "import * as effects from \"std/effect\"\nprintln \"unexpected\"\n",
        "alias Count = Int\nprintln \"unexpected\"\n",
        "newtype Count = Int\nprintln \"unexpected\"\n",
        "struct Count { value: Int }\nprintln \"unexpected\"\n",
        "type State = | Ready\nprintln \"unexpected\"\n",
        "type State =\n | Ready\n | Waiting Int\nprintln \"unexpected\"\n",
        "type State = | Ready; println \"unexpected\"\n",
        "rec { fn value -> Int = 1 }\nprintln \"unexpected\"\n",
    ] {
        assert_error(
            source,
            "parser.module-expression-statement",
            "println \"unexpected\"",
        );
    }
}

#[test]
fn preserves_annotated_unicode_contextual_names_and_expression_continuations() {
    let source = r#"import * as effects from "std/effect"
pub let count: Int = 3
let 次の値 = count
let account' = 次の値
let pure = account'
let namespace = pure
let _ignored = 1
type State =
  | Ready
  | Waiting Int
fn 次を読む value: Int -> Int = value
fn title value: Int -> String =
  if value == 0 then
    "zero"
  else
    "more"
let pending =
  effects.succeed 1
  |> map (\value -> value + 1)
let grouped = (
  次を読む
    count
)
fn local -> Int = {
  let inner = 1
  inner + count
}
pub effect fn main = do {
  println $ title count
  println (show (local ()))
}
"#;
    let artifact = parse_diagnostics("main.ssrg", source);
    assert!(artifact.diagnostics.is_empty(), "{artifact:?}");
}

#[test]
fn distinguishes_record_type_headers_from_declaration_bodies() {
    for declaration in [
        "instance Debug<{ name: String }> { fn debug value: { name: String } -> String = value.name }",
        "instance Show<Array<{ name: String }>> { fn show value: Array<{ name: String }> -> String = \"names\" }",
        "impl Box<{ name: String }> { fn title -> String = \"name\" }",
    ] {
        let source = format!("{declaration}\n");
        let artifact = parse_diagnostics("main.ssrg", &source);
        assert!(artifact.diagnostics.is_empty(), "{source}\n{artifact:?}");
        assert_error(
            &format!("{source}println \"unexpected\"\n"),
            "parser.module-expression-statement",
            "println \"unexpected\"",
        );
    }
}
