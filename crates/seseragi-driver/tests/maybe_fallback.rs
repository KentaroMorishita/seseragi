use seseragi_driver::{compile_module, CompileInput};

#[test]
fn fallback_infers_maybe_payload_and_propagates_expected_types() {
    let source = r#"
fn unwrap<A> value: Maybe<A> -> fallback: A -> A = value ?? fallback
let cached: Maybe<String> = Nothing
let requested: Maybe<String> = Just "request"
let display: String = cached ?? requested ?? "anonymous"
let inferred = Nothing ?? 7
let inner: Maybe<Int> = Just Nothing ?? Just 1
let nested: Array<Int> = Nothing ?? []
let array = [Just 7 ?? 9, Nothing ?? 8]
let boolean: Bool = Nothing ?? 1 < 2 && True
pub let output = (unwrap (Just "hit") "fallback", display, inferred, nested, array, boolean)
"#;
    let result = compile_module(CompileInput::new(
        "fallback.ssrg",
        "fixture/fallback",
        source,
    ));
    assert!(result.is_ok(), "{result:#?}");
    let generated = result.unwrap().generated.typescript;
    assert!(
        !generated.contains(" ?? "),
        "must not use host nullish semantics: {generated}"
    );
    assert!(generated.contains("Just"), "{generated}");
}

#[test]
fn fallback_rejects_non_maybe_and_mismatched_payloads() {
    for source in [
        "pub let bad = 7 ?? 9",
        "pub let bad = Right 7 ?? 9",
        "pub let bad = Just 7 ?? \"no\"",
        "struct Maybe<A> { value: A }\nfn bad value: Maybe<Int> -> Int = value ?? 9",
        "fn bad<Maybe> value: Maybe<Int> -> Int = value ?? 9",
        "struct Item { value: Int }\nstruct Other { value: Int }\npub let bad = Just (Item {value: 1}) ?? Other {value: 2}",
    ] {
        let result = compile_module(CompileInput::new("bad-fallback.ssrg", "fixture/fallback", source));
        assert!(result.is_err(), "{source}\n{result:#?}");
    }
}

#[test]
fn fallback_remains_reserved_and_non_referenceable() {
    for source in [
        "pub let operation = (??)",
        "pub operator infixr 0 ?? left: Int -> right: Int -> Int = right",
    ] {
        assert!(compile_module(CompileInput::new("bad.ssrg", "fixture/fallback", source)).is_err());
    }
    let catalog = seseragi_semantics::standard_library_catalog();
    let entry = catalog.iter().find(|entry| entry.name == "??").unwrap();
    assert_eq!(entry.signature.as_deref(), Some("Maybe<A> ?? A -> A"));
    assert!(entry.description.contains("only for Nothing"));
}

#[test]
fn fallback_diagnostics_keep_operand_ranges_and_type_differences() {
    for (source, operand) in [
        ("pub let bad = 7 ?? 9", "7"),
        ("pub let bad = Just 7 ?? True", "True"),
    ] {
        let result = seseragi_semantics::semantic_diagnostics("bad.ssrg", source);
        let diagnostic = result
            .diagnostics
            .iter()
            .find(|d| d.message_key == "call.argument-type-mismatch")
            .expect("operand diagnostic");
        assert_eq!(diagnostic.code, "SES-T0101");
        assert_eq!(
            &source[diagnostic.primary.start..diagnostic.primary.end],
            operand
        );
        assert!(diagnostic.type_difference.is_some());
    }
}
