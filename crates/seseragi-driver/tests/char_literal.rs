use seseragi_driver::{compile_module, CompileInput};

#[test]
fn char_literals_keep_identity_through_generic_calls_and_containers() {
    let source = r#"
fn identity<A> value: A -> A = value
fn character value: Char -> Char = value
let account' = '瀬'
fn classify value: Char -> Int = match value { 'a' -> 1; '\u{03BB}' -> 2; _ -> 0 }
let escaped: Char = '\u{03BB}'
let nested: Array<Maybe<Char>> = [Just 'a', Just escaped]
let tuple: (Char, Char) = (identity account', character '\'')
pub let result = `${tuple}, ${nested}, ${'λ'}`
"#;
    let result = compile_module(CompileInput::new("char.ssrg", "fixture/char", source));
    assert!(result.is_ok(), "{result:#?}");
}

#[test]
fn char_and_string_are_not_implicitly_interchangeable() {
    for source in [
        "pub let value: String = 'a'",
        "pub let value: Char = \"a\"",
        "fn use value: String -> String = value\npub let bad = use 'a'",
        "struct Char { value: String }\npub let bad: Char = 'a'",
    ] {
        let result = compile_module(CompileInput::new("bad-char.ssrg", "fixture/char", source));
        assert!(result.is_err(), "{source}\n{result:#?}");
    }
}
