use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreTraitDispatch, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "grapheme-雪🙂.ssrg",
        "native/grapheme",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn grapheme_accepts_direct_operations() {
    let core = checked(
        "import * as grapheme from \"std/text/grapheme\"\npub fn main -> Int = grapheme.length \"A👍🏽e\\u{301}\"\n",
    );
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "text"));
}

#[test]
fn grapheme_accepts_saved_generic_and_partial_operations() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/text-grapheme.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
}

const APIS: &[(&str, &str)] = &[
    ("length", "String -> Int"),
    ("clusters", "String -> Array<String>"),
    ("byteBoundaries", "String -> Array<Int>"),
    ("at", "Int -> String -> Maybe<String>"),
    (
        "slice",
        "Int -> Int -> String -> Either<grapheme.GraphemeSliceError, String>",
    ),
];

#[test]
fn grapheme_rejects_unproven_call_types_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (parameter, result, expression) in [
        ("String", "Int", "grapheme.length value"),
        ("String", "Array<String>", "grapheme.clusters value"),
        ("String", "Array<Int>", "grapheme.byteBoundaries value"),
        ("String", "Maybe<String>", "grapheme.at 0 value"),
        (
            "String",
            "Either<grapheme.GraphemeSliceError, String>",
            "grapheme.slice 0 1 value",
        ),
        ("Int", "String -> Maybe<String>", "grapheme.at value"),
        (
            "Int",
            "Int -> String -> Either<grapheme.GraphemeSliceError, String>",
            "grapheme.slice value",
        ),
        (
            "Int",
            "String -> Either<grapheme.GraphemeSliceError, String>",
            "grapheme.slice 0 value",
        ),
    ] {
        let core = checked(&format!(
            "import * as grapheme from \"std/text/grapheme\"\npub fn probe value: {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..11 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut changed.functions[probe].body
            else {
                panic!("{expression}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: result.into(),
                        canonical: "fake::Result".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/text/grapheme::GraphemeSliceError".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.extend(vec![arguments[0].clone(); 4]),
                5 => evidence.push(other[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(CoreTraitDispatch {
                        trait_identity: "std/prelude::Eq".into(),
                        method: "eq".into(),
                        method_contract: None,
                    })
                }
                _ => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven grapheme call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn grapheme_saved_functions_require_canonical_signatures_and_no_evidence() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, signature) in APIS {
        let core = checked(&format!(
            "import * as grapheme from \"std/text/grapheme\"\npub fn probe -> {signature} = grapheme.{name}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..4 {
            let mut changed = core.clone();
            let CoreExpr::Variable {
                type_ref, evidence, ..
            } = &mut changed.functions[probe].body
            else {
                panic!("{name}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: (*signature).into(),
                        canonical: "fake::Grapheme".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/text/grapheme::GraphemeSliceError".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                _ => evidence.push(other[0].clone()),
            }
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{name}/{mutation}");
        }
    }
}

#[test]
fn grapheme_keeps_unused_program_wasm_references_rejected() {
    for (operation, signature) in APIS {
        let core = checked(&format!(
            "import * as grapheme from \"std/text/grapheme\"\npub fn probe value: Int -> Int = {{ let saved: {signature} = grapheme.{operation}; value }}\npub fn main -> Int = probe 42\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/grapheme", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific proof"),
            "{operation}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "grapheme-雪🙂.ssrg");
    }
}

#[test]
fn grapheme_invalid_arguments_match_ts_diagnostics() {
    for expression in [
        "grapheme.length 'a'",
        "grapheme.clusters ()",
        "grapheme.byteBoundaries [1]",
        "grapheme.at 0.5 \"a\"",
        "grapheme.at \"0\" \"a\"",
        "grapheme.at 0 'a'",
        "grapheme.slice 0.5 1 \"a\"",
        "grapheme.slice 0 1.5 \"a\"",
        "grapheme.slice 0 1 True",
        "grapheme.slice 0 1 \"\\u{dfff}\"",
        "grapheme.length \"\\u{110000}\"",
    ] {
        let source = format!(
            "import * as grapheme from \"std/text/grapheme\"\npub fn probe = {expression}\n"
        );
        let input = CompileInput::new("grapheme-type.ssrg", "native/grapheme", &source);
        let ts = seseragi_driver::compile_module(input).unwrap_err();
        let native = compile_core_module(input).unwrap_err();
        assert!(!ts.diagnostics.is_empty(), "{expression}");
        assert_eq!(
            serde_json::to_value(ts).unwrap(),
            serde_json::to_value(native).unwrap(),
            "{expression}"
        );
    }
}
