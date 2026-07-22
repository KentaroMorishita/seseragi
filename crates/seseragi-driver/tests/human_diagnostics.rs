use seseragi_driver::{compile_module, CompileInput};
use seseragi_syntax::Diagnostic;

fn first_diagnostic(source_name: &str, source: &str) -> Diagnostic {
    compile_module(CompileInput::new(source_name, source_name, source))
        .expect_err("invalid source must stop before code generation")
        .diagnostics
        .into_iter()
        .next()
        .expect("invalid source must produce a diagnostic")
}

#[test]
fn presents_the_first_eight_diagnostic_families_without_raw_message_keys() {
    const MISSING_INSTANCE: &str = include_str!(
        "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/template-missing-show/main.ssrg"
    );
    const EFFECT_REQUIRED: &str = include_str!(
        "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/effect-compact-not-effect/main.ssrg"
    );
    const NON_EXHAUSTIVE: &str = include_str!(
        "../../../examples/spec/artifacts/semantic-diagnostics-schema-1/match-non-exhaustive/main.ssrg"
    );
    let cases = [
        (
            "unresolved.ssrg",
            "pub fn main -> Int = missing\n",
            "name.unresolved",
        ),
        (
            "arity.ssrg",
            "fn one value: Int -> Int = value\npub fn main -> Int = one 1 2\n",
            "call.arity-mismatch",
        ),
        (
            "argument.ssrg",
            "fn one value: Int -> Int = value\npub fn main -> Int = one \"no\"\n",
            "call.argument-type-mismatch",
        ),
        ("instance.ssrg", MISSING_INSTANCE, "instance.missing"),
        (
            "field.ssrg",
            "pub fn main user: { name: String } -> String = user.nmae\n",
            "record.field-unresolved",
        ),
        (
            "effect.ssrg",
            EFFECT_REQUIRED,
            "effect.compact-body-not-effect",
        ),
        ("match.ssrg", NON_EXHAUSTIVE, "match.non-exhaustive"),
        (
            "parser.ssrg",
            "pub let broken: Int =\n",
            "parser.expected-expression",
        ),
    ];

    for (source_name, source, message_key) in cases {
        let diagnostic = first_diagnostic(source_name, source);
        assert_eq!(diagnostic.message_key, message_key, "{source_name}");
        assert_ne!(
            diagnostic.message(),
            diagnostic.message_key,
            "{source_name}"
        );
        assert!(
            !diagnostic.message().contains(&diagnostic.message_key),
            "{source_name}"
        );
        assert!(!diagnostic.helps().is_empty(), "{source_name}");

        let wire = serde_json::to_value(diagnostic).unwrap();
        assert!(wire["message"].is_string(), "{source_name}");
        assert!(wire["labels"].is_array(), "{source_name}");
        assert!(wire["notes"].is_array(), "{source_name}");
        assert!(wire["helps"].is_array(), "{source_name}");
        assert!(wire["fixes"].is_array(), "{source_name}");
    }
}

#[test]
fn exposes_expected_actual_types_and_a_spelling_fix() {
    let arity = first_diagnostic(
        "arity.ssrg",
        "fn one value: Int -> Int = value\npub fn main -> Int = one 1 2\n",
    );
    assert_eq!(arity.expected_actual_types(), (None, None));

    let mismatch = first_diagnostic(
        "mismatch.ssrg",
        "fn one value: Int -> Int = value\npub fn main -> Int = one \"no\"\n",
    );
    assert_eq!(
        mismatch.expected_actual_types(),
        (Some("Int".to_owned()), Some("String".to_owned()))
    );

    let field = first_diagnostic(
        "field.ssrg",
        "pub struct User { name: String }\npub fn main -> User = User { nmae: \"A\" }\n",
    );
    assert_eq!(field.fixes.len(), 1);
    assert_eq!(field.fixes[0].edits[0].replacement, "name");
}
