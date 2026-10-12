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
fn bad<String> x: String -> String = echo<String> x"#,
        r#"foreign "typescript" from "./api.mjs" {
  pure fn echo<A> value: A -> A
}
type String = | UserString
fn bad x: String -> String = echo<String> x"#,
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

#[test]
fn primitive_foreign_codec_selection_covers_the_documented_scalar_domain() {
    let source = format!(
        "foreign \"typescript\" from \"./api.mjs\" {{\n  pure fn echo<A> value: A -> A\n}}\n{}",
        ["Unit", "Bool", "Char", "String", "Int", "Float"]
            .into_iter()
            .enumerate()
            .map(|(i, name)| format!("fn scalar{i} value: {name} -> {name} = echo<{name}> value\n"))
            .collect::<String>()
    );
    let result = compile(&source);
    assert!(!result.generated.typescript.contains("unsupported"));
}

#[test]
fn foreign_binder_names_shadow_backend_primitives_without_changing_call_codecs() {
    let result = compile(
        r#"foreign "typescript" from "./api.mjs" {
  pure fn echo<String> value: String -> String
}
pub fn value unit: Unit -> Bool = echo<Bool> True
"#,
    );
    let ts = result.generated.typescript;
    assert!(ts.contains("<String,>(value: String)"), "{ts}");
    assert!(ts.contains("_ssrg_foreign_invokePure<String>"), "{ts}");
    assert!(ts.contains("(\"bool\")"), "{ts}");
}
