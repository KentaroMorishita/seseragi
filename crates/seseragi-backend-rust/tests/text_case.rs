use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreTraitDispatch, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "text-case-雪🙂.ssrg",
        "native/text-case",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

#[test]
fn casing_accepts_direct_saved_generic_and_returned_operations() {
    let artifact = emit_rust_module(
        &checked(include_str!(
            "../../../examples/spec/fixtures/rust-backend/text-case.ssrg"
        )),
        "main",
    )
    .unwrap();
    assert!(artifact.metadata.capabilities.iter().any(|c| c == "text"));
}

#[test]
fn casing_rejects_unproven_argument_result_evidence_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for operation in ["toLower", "toUpper", "caseFold"] {
        let core = checked(&format!(
            "import * as text from \"std/text\"\npub fn probe value: String -> String = text.{operation} value\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
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
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "String".into(),
                        canonical: "fake::String".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/prelude::String".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => arguments.clear(),
                4 => arguments.push(arguments[0].clone()),
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
                    arguments[0] = CoreExpr::Unit {
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven casing call");
            assert_eq!(error.code, "SES-RUST-0001", "{operation}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{operation}/{mutation}");
        }
    }
}

#[test]
fn casing_keeps_unused_program_wasm_references_rejected() {
    for operation in ["toLower", "toUpper", "caseFold"] {
        let core = checked(&format!(
            "import * as text from \"std/text\"\npub fn probe value: Int -> Int = {{ let saved: String -> String = text.{operation}; value }}\npub fn main -> Int = probe 42\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/text-case", &["probe".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("requires a target-specific proof"),
            "{operation}: {error}"
        );
        assert_eq!(error.origin.unwrap().source, "text-case-雪🙂.ssrg");
    }
}

#[test]
fn casing_type_errors_match_the_ts_target() {
    for operation in ["toLower", "toUpper", "caseFold"] {
        for argument in [
            "0",
            "'A'",
            "True",
            "[]",
            "()",
            "\"a\" \"b\"",
            "\"\\u{d800}\"",
            "\"\\u{110000}\"",
        ] {
            let source = format!("import * as text from \"std/text\"\npub fn main -> String = text.{operation} {argument}\n");
            let input = CompileInput::new("text-case-type.ssrg", "native/text-case", &source);
            let ts = seseragi_driver::compile_module(input).unwrap_err();
            let native = compile_core_module(input).unwrap_err();
            assert!(!ts.diagnostics.is_empty(), "{operation}/{argument}");
            assert_eq!(
                serde_json::to_value(ts).unwrap(),
                serde_json::to_value(native).unwrap(),
                "{operation}/{argument}"
            );
        }
    }
}
