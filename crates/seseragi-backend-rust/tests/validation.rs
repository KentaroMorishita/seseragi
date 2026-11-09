use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "validation-雪🙂.ssrg",
        "native/validation",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn inferred_validation_results_preserve_the_declared_native_payload_types() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend-regressions/validation-inferred-return.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap();
}

#[test]
fn invalid_many_requires_a_non_empty_list_including_for_empty_inputs() {
    for (collection, argument) in [
        ("Array<String>", "[]"),
        ("List<String>", "`[]"),
        ("Array<String>", "[\"bad\"]"),
        ("List<String>", "`[\"bad\"]"),
    ] {
        let source = format!(
            "import * as v from \"std/validation\"\npub fn probe -> v.Validation<String,Int> = {{let errors:{collection} = {argument};v.invalidMany errors}}\n"
        );
        let error = compile_core_module(CompileInput::new(
            "validation-雪🙂.ssrg",
            "native/validation",
            &source,
        ))
        .unwrap_err();
        assert!(
            error.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "SES-T0101"
                    && diagnostic.message_key == "call.argument-type-mismatch"
            }),
            "{argument}: {error:#?}"
        );
    }
}

fn cases() -> [(&'static str, &'static str, &'static str, &'static str); 7] {
    [
        (
            "Valid",
            "1",
            "v.Validation<String,Int>",
            "Int -> v.Validation<String,Int>",
        ),
        (
            "valid",
            "1",
            "v.Validation<String,Int>",
            "Int -> v.Validation<String,Int>",
        ),
        (
            "Invalid",
            "pure \"bad\"",
            "v.Validation<String,Int>",
            "NonEmptyList<String> -> v.Validation<String,Int>",
        ),
        (
            "invalid",
            "\"bad\"",
            "v.Validation<String,Int>",
            "String -> v.Validation<String,Int>",
        ),
        (
            "invalidMany",
            "pure \"bad\"",
            "v.Validation<String,Int>",
            "NonEmptyList<String> -> v.Validation<String,Int>",
        ),
        (
            "fromEither",
            "Right 1",
            "v.Validation<String,Int>",
            "Either<String,Int> -> v.Validation<String,Int>",
        ),
        (
            "toEither",
            "v.valid 1",
            "Either<NonEmptyList<String>,Int>",
            "v.Validation<String,Int> -> Either<NonEmptyList<String>,Int>",
        ),
    ]
}

#[test]
fn validation_operations_require_checked_argument_result_owners_kinds_and_source_stages() {
    let eq = checked("pub fn probe -> Bool = 1 == 1");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for (name, argument, result, _) in cases() {
        let core = checked(&format!(
            "import * as v from \"std/validation\"\nimport {{ NonEmptyList }} from \"std/non-empty-list\"\npub fn probe -> {result} = v.{name} ({argument})\n"
        ));
        let artifact = emit_rust_module(&core, "probe").unwrap();
        assert!(artifact
            .metadata
            .capabilities
            .iter()
            .any(|value| value == "validation"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[index].body).clone();
        for mutation in 0..11 {
            let mut bad = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence,
                trait_dispatch,
                deferred_evidence_parameters,
                deferred_evidence_type_constructor_parameters,
                evidence_argument_index,
                ..
            } = &mut bad.functions[index].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Validation".into(),
                        canonical: "counterfeit::Validation".into(),
                        arguments: vec![],
                    }
                }
                2 => {
                    *type_ref = CoreType::Named {
                        name: "std/validation::Validation".into(),
                        arguments: vec![CoreType::Hole],
                    }
                }
                3 => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                4 => arguments.clear(),
                5 => arguments.push(arguments[0].clone()),
                6 => deferred_evidence_parameters.push(CoreType::Hole),
                7 => deferred_evidence_type_constructor_parameters.push("F".into()),
                8 => *evidence_argument_index = Some(0),
                9 => {
                    *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                        trait_identity: "std/prelude::Applicative".into(),
                        method: "pure".into(),
                        method_contract: None,
                    })
                }
                10 => evidence.push(other[0].clone()),
                _ => unreachable!(),
            }
            let error = emit_rust_module(&bad, "probe").expect_err(&format!("{name}/{mutation}"));
            assert_eq!(error.code, "SES-RUST-0001", "{name}/{mutation}: {error:?}");
            assert_eq!(
                error.origin.as_ref(),
                Some(&origin),
                "{name}/{mutation}: {error:?}"
            );
        }
    }
}

#[test]
fn saved_validation_functions_require_full_signatures() {
    for (name, _, _, signature) in cases() {
        let core = checked(&format!(
            "import * as v from \"std/validation\"\nimport {{ NonEmptyList }} from \"std/non-empty-list\"\nfn saved -> ({signature}) = v.{name}\npub fn probe -> Int = 0\n"
        ));
        emit_rust_module(&core, "probe").unwrap();
        emit_program_wasm(&[&core], "native/validation", &["probe".into()])
            .unwrap_or_else(|error| panic!("{error:?}"));
        let index = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::saved"))
            .unwrap();
        for invalid in [
            CoreType::Hole,
            CoreType::Function {
                parameter: Box::new(CoreType::Named {
                    name: "std/prelude::Bool".into(),
                    arguments: vec![],
                }),
                result: Box::new(CoreType::Hole),
            },
        ] {
            let mut bad = core.clone();
            let mut body = &mut bad.functions[index].body;
            while let CoreExpr::Lambda { body: inner, .. } = body {
                body = inner;
            }
            match body {
                CoreExpr::Variable { type_ref, .. } | CoreExpr::Call { type_ref, .. } => {
                    *type_ref = invalid
                }
                expression => panic!("{expression:?}"),
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{name}: {error:?}");
            assert_eq!(error.origin.unwrap().source, "validation-雪🙂.ssrg");
        }
    }
}
