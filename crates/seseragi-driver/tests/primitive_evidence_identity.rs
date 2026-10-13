use serde_json::Value;
use seseragi_driver::{compile_module, CompileInput};
use seseragi_semantics::TypedDecl;

fn evidence(source: &str) -> Value {
    let compiled = compile_module(CompileInput::new(
        "identity.ssrg",
        "fixture/identity",
        source,
    ))
    .unwrap_or_else(|errors| panic!("{source}\n{errors:#?}"));
    let body = compiled
        .typed_hir
        .declarations
        .iter()
        .find_map(|decl| match decl {
            TypedDecl::Fn { symbol, body, .. } | TypedDecl::EffectFn { symbol, body, .. }
                if symbol.ends_with("::main") =>
            {
                Some(body)
            }
            _ => None,
        })
        .expect("main body");
    serde_json::to_value(body).unwrap()
}

fn identities(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::Object(fields) => {
            if let Some(identity) = fields.get("identity").and_then(Value::as_str) {
                output.push(identity.into());
            }
            for value in fields.values() {
                identities(value, output);
            }
        }
        Value::Array(values) => {
            for value in values {
                identities(value, output);
            }
        }
        _ => {}
    }
}

#[test]
fn primitive_show_keeps_its_dictionary_under_nonderived_and_derived_nominal_shadow() {
    for (name, literal, field_type) in [
        ("Bool", "True", "String"),
        ("Int", "1", "Bool"),
        ("Float", "(-1.0)", "Bool"),
        ("Char", "'a'", "Bool"),
        ("String", "\"ok\"", "Bool"),
        ("Unit", "()", "Bool"),
    ] {
        for deriving in ["", " deriving Show"] {
            let source = format!("struct {name}{deriving} {{value: {field_type}}}\nfn keep<A> value: A -> A where Show<A> = {{let rendered = show value; value}}\npub effect fn main = do {{println (show {literal}); println (show (keep {literal})); println `${{{literal}}}`}}\n");
            let mut found = Vec::new();
            identities(&evidence(&source), &mut found);
            let expected = format!("Show<std/prelude::{name}>");
            assert_eq!(
                found
                    .iter()
                    .filter(|identity| **identity == expected)
                    .count(),
                4,
                "{source}\n{found:?}"
            );
            assert!(
                !found
                    .iter()
                    .any(|identity| identity.starts_with("fixture/identity::")),
                "primitive selected local dictionary: {found:?}"
            );
        }
    }
}

#[test]
fn nominal_show_still_requires_and_selects_its_own_instance() {
    let rejected = compile_module(CompileInput::new("identity.ssrg", "fixture/identity", "struct Float {value: Bool}\npub effect fn main = do {println (show (Float {value: True}))}\n")).unwrap_err();
    assert!(rejected
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "SES-T0201"));
    let source = "struct Float deriving Show {value: Bool}\nfn render<A> value: A -> String where Show<A> = show value\npub effect fn main = do {println (render (-1.0)); println (render (Float {value: True}))}\n";
    let mut found = Vec::new();
    identities(&evidence(source), &mut found);
    assert!(
        found
            .iter()
            .any(|identity| identity == "Show<std/prelude::Float>"),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|identity| identity.contains("fixture/identity::Float")),
        "{found:?}"
    );
}

#[test]
fn nominal_primitive_spelling_is_not_a_scalar_foreign_codec() {
    let source = "struct Bool {value: Int}\nforeign \"typescript\" from \"./host.js\" {\n  pure fn echo<A> value: A -> A\n}\npub fn main -> Bool = echo (Bool {value: 1})\n";
    let rejected = compile_module(CompileInput::new(
        "identity.ssrg",
        "fixture/identity",
        source,
    ))
    .unwrap_err();
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-F0102"),
        "{rejected:#?}"
    );
}

#[test]
fn ordinary_primitive_programs_keep_the_existing_typed_evidence_shape() {
    for (name, literal) in [
        ("Bool", "True"),
        ("Int", "1"),
        ("Float", "(-1.0)"),
        ("Char", "'a'"),
        ("String", "\"ok\""),
        ("Unit", "()"),
    ] {
        let body = evidence(&format!(
            "pub effect fn main = do {{println (show {literal})}}\n"
        ));
        assert!(
            !body
                .to_string()
                .contains(&format!("\"canonical\":\"std/prelude::{name}\"")),
            "{body}"
        );
    }
}

#[test]
fn proven_scalar_foreign_codecs_remain_available_under_nominal_shadow() {
    for (name, literal, field_type, codec) in [
        ("Bool", "True", "String", "bool"),
        ("Int", "1", "Bool", "int"),
        ("Float", "(-1.0)", "Bool", "float"),
        ("Char", "'a'", "Bool", "char"),
        ("String", "\"ok\"", "Bool", "string"),
        ("Unit", "()", "Bool", "unit"),
    ] {
        let source = format!("struct {name} {{value: {field_type}}}\nforeign \"typescript\" from \"./host.mjs\" {{\n pure fn echo<A> value: A -> A\n}}\npub effect fn main = do {{println (show (echo {literal}))}}\n");
        let compiled = compile_module(CompileInput::new(
            "identity.ssrg",
            "fixture/identity",
            &source,
        ))
        .unwrap_or_else(|errors| panic!("{source}\n{errors:#?}"));
        assert!(
            compiled
                .generated
                .typescript
                .contains(&format!("(\"{codec}\")")),
            "{}",
            compiled.generated.typescript
        );
    }
}

#[test]
fn composite_dictionaries_keep_distinct_primitive_and_nominal_element_identity() {
    let source = "struct Float deriving Show {value: Bool}\npub effect fn main = do {println (show [(-1.0)]); println (show ((-1.0), True)); println (show {x: (-1.0)}); println (show [Float {value: True}]); println (show (Float {value: True}, True)); println (show {x: Float {value: True}})}\n";
    let mut found = Vec::new();
    identities(&evidence(source), &mut found);
    assert_eq!(
        found
            .iter()
            .filter(|identity| *identity == "Show<std/prelude::Float>")
            .count(),
        3,
        "{found:?}"
    );
    assert_eq!(
        found
            .iter()
            .filter(|identity| identity.contains("fixture/identity::Float"))
            .count(),
        3,
        "{found:?}"
    );
}

#[test]
fn scalar_codec_type_argument_is_not_captured_by_a_same_name_generic_binder() {
    let source = "foreign \"typescript\" from \"./host.mjs\" {\n pure fn echo<A> value: A -> A\n}\nfn tricky<Bool> value: Bool -> String where Show<Bool> = show (echo True)\npub effect fn main = do {println (tricky 1)}\n";
    let compiled = compile_module(CompileInput::new(
        "identity.ssrg",
        "fixture/identity",
        source,
    ))
    .unwrap();
    assert!(
        compiled
            .generated
            .typescript
            .contains("echo(true)(\"bool\")) as boolean"),
        "{}",
        compiled.generated.typescript
    );
}

#[test]
fn nominal_primitive_spelling_does_not_inherit_standard_equality() {
    let source = "struct Bool {value: Int}\npub effect fn main = do {let equal = Bool {value: 1} == Bool {value: 2}; println \"typed\"}\n";
    let rejected = compile_module(CompileInput::new(
        "identity.ssrg",
        "fixture/identity",
        source,
    ))
    .unwrap_err();
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0201"),
        "{rejected:#?}"
    );
}

#[test]
fn generic_evidence_inference_does_not_merge_primitive_and_nominal_arguments() {
    let source = "struct Bool deriving Show {value: Int}\nfn same<A> left: A -> right: A -> String where Show<A> = show left\npub effect fn main = do {println (same True (Bool {value: 1}))}\n";
    let rejected = compile_module(CompileInput::new(
        "identity.ssrg",
        "fixture/identity",
        source,
    ))
    .unwrap_err();
    assert!(
        rejected
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SES-T0101"),
        "{rejected:#?}"
    );
}

#[test]
fn imported_scalar_codec_result_is_not_captured_by_a_caller_generic_binder() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let foreign =
        "pub foreign \"typescript\" from \"./host.mjs\" {\n pure fn echo<A> value: A -> A\n}\n";
    let source = "import {echo} from \"./foreign\"\nfn tricky<Bool> value: Bool -> String where Show<Bool> = show (echo True)\npub effect fn main = do {println (tricky 1)}\n";
    let mut graph = ModuleGraph::new();
    graph
        .add_module(
            "fixture/identity::foreign".to_owned(),
            std::iter::empty::<(String, String)>(),
        )
        .unwrap();
    graph
        .add_module(
            "fixture/identity::main".to_owned(),
            [(
                "./foreign".to_owned(),
                "fixture/identity::foreign".to_owned(),
            )],
        )
        .unwrap();
    let compiled = compile_project(
        graph,
        [
            ProjectModuleInput::new(
                "foreign.ssrg",
                "fixture/identity::foreign",
                foreign,
                "dist/foreign.js",
            ),
            ProjectModuleInput::new(
                "main.ssrg",
                "fixture/identity::main",
                source,
                "dist/main.js",
            ),
        ],
    )
    .unwrap();
    let emitted = &compiled.modules["fixture/identity::main"]
        .generated
        .typescript;
    assert!(
        emitted.contains("echo(true)(\"bool\")) as boolean"),
        "{emitted}"
    );
}

#[test]
fn foreign_result_proof_follows_import_aliases_and_reexports() {
    use seseragi_driver::{compile_project, ProjectModuleInput};
    use seseragi_project::ModuleGraph;
    let foreign = "pub foreign \"typescript\" from \"./host.mjs\" {\n pure fn choose<A, B> left: A -> right: B -> B\n}\n";
    let facade = "pub import {choose as again} from \"./foreign\"\n";
    for (import, call, target, emitted_call) in [
        (
            "import {choose as selected} from \"./foreign\"",
            "selected",
            "fixture/proof::foreign",
            "selected",
        ),
        (
            "import * as foreign from \"./foreign\"",
            "foreign.choose",
            "fixture/proof::foreign",
            "foreign$member$choose",
        ),
        (
            "import {again as selected} from \"./facade\"",
            "selected",
            "fixture/proof::facade",
            "selected",
        ),
    ] {
        let source = format!("{import}\nfn tricky<Bool> value: Bool -> String where Show<Bool> = show ({call} \"ok\" True)\npub effect fn main = do {{println (tricky 1)}}\n");
        let mut graph = ModuleGraph::new();
        graph
            .add_module(
                "fixture/proof::foreign".to_owned(),
                std::iter::empty::<(String, String)>(),
            )
            .unwrap();
        graph
            .add_module(
                "fixture/proof::facade".to_owned(),
                [("./foreign".to_owned(), "fixture/proof::foreign".to_owned())],
            )
            .unwrap();
        let specifier = if target.ends_with("facade") {
            "./facade"
        } else {
            "./foreign"
        };
        graph
            .add_module(
                "fixture/proof::main".to_owned(),
                [(specifier.to_owned(), target.to_owned())],
            )
            .unwrap();
        let compiled = compile_project(
            graph,
            [
                ProjectModuleInput::new(
                    "foreign.ssrg",
                    "fixture/proof::foreign",
                    foreign,
                    "dist/foreign.js",
                ),
                ProjectModuleInput::new(
                    "facade.ssrg",
                    "fixture/proof::facade",
                    facade,
                    "dist/facade.js",
                ),
                ProjectModuleInput::new("main.ssrg", "fixture/proof::main", source, "dist/main.js"),
            ],
        )
        .unwrap();
        let emitted = &compiled.modules["fixture/proof::main"].generated.typescript;
        assert!(
            emitted.contains(&format!(
                "{emitted_call}(\"ok\")(true)(\"string\")(\"bool\")) as boolean"
            )),
            "{emitted}"
        );
    }
}
