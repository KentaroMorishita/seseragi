use seseragi_driver::{compile_core_module, compile_module, CompileInput};

#[test]
fn rejects_incomplete_instance_contracts_before_any_backend_output() {
    for (declaration, expected) in [
        (
            "instance Show<Label> { fn show value = \"selected\" }",
            "parser.function-annotations-required",
        ),
        (
            "instance Show<Label> { fn show value: Label = \"selected\" }",
            "parser.function-annotations-required",
        ),
        ("instance Show<Label> {}", "trait.instance-method-missing"),
        (
            "instance Show<Label> { fn show value: Label -> Int = 42 }",
            "trait.instance-method-signature-mismatch",
        ),
        (
            "trait Render<A> { fn render value -> String }\ninstance Render<Label> {}",
            "parser.function-annotations-required",
        ),
    ] {
        let source = format!("// 雫\ntype Label = | Label\n{declaration}\npub effect fn main = println \"must not run\"\n");
        let input = CompileInput::new("methods-λ.ssrg", "fixture/methods", &source);
        let core_errors = compile_core_module(input.clone()).unwrap_err();
        let ts_errors = compile_module(input).unwrap_err();
        assert_eq!(core_errors, ts_errors);
        assert_eq!(core_errors.source, "methods-λ.ssrg");
        assert!(
            core_errors
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message_key == expected),
            "{core_errors:#?}"
        );
        for diagnostic in &core_errors.diagnostics {
            assert!(source.is_char_boundary(diagnostic.primary.start));
            assert!(source.is_char_boundary(diagnostic.primary.end));
            assert!(!source[diagnostic.primary.start..diagnostic.primary.end].is_empty());
        }
    }
}

#[test]
fn preserves_standard_and_shadowed_custom_contracts() {
    let source = "type Label = | Label\ntrait Show<A> { fn describe value: A -> Int }\ninstance Show<Label> { fn describe value: Label -> Int = 42 }\npub fn probe -> Int = describe Label\n";
    let compiled =
        compile_module(CompileInput::new("custom.ssrg", "fixture/custom", source)).unwrap();
    assert!(compiled.generated.typescript.contains("describe"));
    assert!(!compiled.generated.typescript.contains("[\"show\"]"));
}

#[test]
fn standard_json_contracts_keep_the_registered_provider_nominals() {
    let source = "import { Json } from \"std/json\"\ntype Label = | Label\ninstance JsonEncode<Label> { fn encodeJson value: Label -> Json = encodeJson \"label\" }\n";
    compile_core_module(CompileInput::new(
        "json-contract.ssrg",
        "fixture/json-contract",
        source,
    ))
    .unwrap();
    let invalid = source.replace("-> Json = encodeJson \"label\"", "-> Int = 42");
    let diagnostics = compile_core_module(CompileInput::new(
        "json-contract.ssrg",
        "fixture/json-contract",
        &invalid,
    ))
    .unwrap_err();
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message_key == "trait.instance-method-signature-mismatch"),
        "{diagnostics:#?}"
    );
}

#[test]
fn standard_method_contracts_preserve_transparent_aliases() {
    for method in [
        "fn show value: Name -> String = \"label\"",
        "fn show value: Label -> Text = \"label\"",
        "fn show value: Name -> Text = \"label\"",
    ] {
        let source = format!("type Label = | Label\nalias Name = Label\nalias Text = String\ninstance Show<Label> {{ {method} }}\npub effect fn main = println (show Label)\n");
        compile_module(CompileInput::new(
            "aliases.ssrg",
            "fixture/aliases",
            &source,
        ))
        .unwrap();
    }
    let source = "type Box<A> = | Box A\nalias Wrapped<A> = Box<A>\ninstance Functor<Box> { fn map<A, B> f: (A -> B) -> value: Wrapped<A> -> Wrapped<B> = match value { Box item -> Box (f item) } }\n";
    compile_core_module(CompileInput::new(
        "hkt-alias.ssrg",
        "fixture/hkt-alias",
        source,
    ))
    .unwrap();
}
