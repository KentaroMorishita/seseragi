use seseragi_driver::{compile_module, CompileInput};

fn compiles(source: &str) -> bool {
    let result = compile_module(CompileInput::new(
        "variance.ssrg",
        "fixture/variance",
        source,
    ));
    if let Err(error) = &result {
        eprintln!("{source}\n{error:?}");
    }
    result.is_ok()
}

fn rejects_type(source: &str) {
    let diagnostics = compile_module(CompileInput::new(
        "variance.ssrg",
        "fixture/variance",
        source,
    ))
    .expect_err("an invariant conversion must be rejected");
    assert!(!diagnostics.diagnostics.is_empty());
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "SES-T0101"),
        "{source}\n{diagnostics:?}"
    );
}

#[test]
fn invariance_preserves_partial_constructor_supertrait_resolution() {
    assert!(compiles(include_str!(
        "../../../examples/spec/artifacts/schema-1/applicative-validation/main.ssrg"
    )));
}

#[test]
fn record_width_does_not_cross_collection_or_nominal_arguments() {
    for (declaration, container, value) in [
        ("", "Array", "[{ name: \"Aki\", id: 1 }]"),
        ("", "List", "`[{ name: \"Aki\", id: 1 }]"),
        (
            "type Box<A> = | Box A",
            "Box",
            "Box { name: \"Aki\", id: 1 }",
        ),
        (
            "struct Box<A> { value: A }",
            "Box",
            "Box { value: { name: \"Aki\", id: 1 } }",
        ),
    ] {
        let source = format!("{declaration}\nlet users: {container}<{{ name: String, id: Int }}> = {value}\npub let wrong: {container}<{{ name: String }}> = users");
        rejects_type(&source);
        let same = format!("{declaration}\nlet users: {container}<{{ name: String, id: Int }}> = {value}\npub let same: {container}<{{ id: Int, name: String }}> = users");
        assert!(compiles(&same), "reordered fields are identical: {same}");
    }
}

#[test]
fn direct_width_and_explicit_map_remain_valid_but_depth_is_rejected() {
    assert!(compiles(
        r#"
fn nameOnly value: { name: String, id: Int } -> { name: String } = { name: value.name }
let user = { name: "Aki", id: 1 }
pub let name: { name: String } = user
pub let names: Array<{ name: String }> = map nameOnly [user]
"#
    ));
    rejects_type(
        r#"
let user = { child: { name: "Aki", id: 1 } }
pub let wrong: { child: { name: String } } = user
"#,
    );
}

#[test]
fn capability_widening_does_not_widen_payloads_or_non_never_failures() {
    for container in ["Effect", "Stream"] {
        let imports = if container == "Stream" {
            "import { Stream } from \"std/stream\"\n"
        } else {
            ""
        };
        assert!(compiles(&format!("{imports}fn widen value: {container}<{{}}, Never, Int> -> {container}<{{ console: Console }}, String, Int> = value")));
        for (actual, expected) in [
            (
                "{}, Never, { name: String, id: Int }",
                "{}, Never, { name: String }",
            ),
            (
                "{}, { name: String, id: Int }, Int",
                "{}, { name: String }, Int",
            ),
        ] {
            let source = format!(
                "{imports}fn wrong value: {container}<{actual}> -> {container}<{expected}> = value"
            );
            rejects_type(&source);
        }
    }
    assert!(compiles(
        "import { MutableSignal, Signal } from \"std/signal\"\nfn read value: MutableSignal<Int> -> Signal<Int> = value"
    ));
    rejects_type("import { MutableSignal, Signal } from \"std/signal\"\nfn wrong value: MutableSignal<{ name: String, id: Int }> -> Signal<{ name: String }> = value");
}
