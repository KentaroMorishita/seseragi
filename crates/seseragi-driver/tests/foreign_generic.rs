use seseragi_driver::{compile_module, CompileInput};

fn compile(source: &str) -> seseragi_driver::CompiledModule {
    compile_module(CompileInput::new(
        "foreign.ssrg",
        "fixture/foreign-generic",
        source,
    ))
    .unwrap()
}
#[test]
fn generic_foreign_binders_and_concrete_codec_evidence_survive_every_ir() {
    let result = compile(
        r#"
pub foreign "typescript" from "./api.mjs" {
  opaque type Schema<A>
  pure fn make<A> value: A -> Schema<A>
  pure fn read<A> schema: Schema<A> -> A
}
pub fn text unit: Unit -> String = read<String> (make<String> "linked")
pub fn flag unit: Unit -> Bool = read<Bool> (make<Bool> True)
"#,
    );
    for name in ["make", "read"] {
        let export = result
            .typed_interface
            .exports
            .iter()
            .find(|e| e.name == name)
            .unwrap();
        assert_eq!(export.scheme.type_parameters.len(), 1);
        assert_eq!(
            export.declaration_kind.as_deref(),
            Some("generic-foreign-function")
        );
    }
    let ts = result.generated.typescript;
    assert!(ts.contains("type Schema<A>"), "{ts}");
    assert!(ts.contains("readonly [A]"), "{ts}");
    assert!(ts.contains("__ssrg$foreign$codec$A"), "{ts}");
    assert!(ts.contains("(\"string\")"), "{ts}");
    assert!(ts.contains("(\"bool\")"), "{ts}");
    assert!(!ts.contains("\"unsupported\""), "{ts}");
}
#[test]
fn unresolved_structural_and_higher_rank_foreign_codecs_fail_before_emission() {
    for source in [
        r#"foreign "typescript" from "./api.mjs" {
  pure fn echo<A> value: A -> A
}
fn bad<A> x: A -> A = echo<A> x"#,
        r#"foreign "typescript" from "./api.mjs" {
  pure fn echo<A> value: A -> A
}
fn bad x: {x: String} -> {x: String} = echo<{x: String}> x"#,
        r#"foreign "typescript" from "./api.mjs" {
  pure fn echo<F<_>, A> value: F<A> -> F<A>
}
fn bad x: Array<String> -> Array<String> = echo<Array, String> x"#,
    ] {
        let error = compile_module(CompileInput::new(
            "foreign.ssrg",
            "fixture/foreign-policy",
            source,
        ))
        .unwrap_err();
        assert!(
            error
                .diagnostics
                .iter()
                .any(|d| d.message_key == "foreign.generic-codec-policy-required"),
            "{error:?}"
        );
    }
}
