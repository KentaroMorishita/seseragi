use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "sequence-雪🙂.ssrg",
        "native/sequence",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:?}"))
    .core_ir
}

fn expressions() -> Vec<(&'static str, &'static str)> {
    vec![
        ("findIndex (\\x: Int -> x == 2) [1, 2]", "Maybe<Int>"),
        ("takeWhile (\\x: Int -> x == 1) [1, 2]", "Array<Int>"),
        ("dropWhile (\\x: Int -> x == 1) [1, 2]", "Array<Int>"),
        ("last [1, 2]", "Maybe<Int>"),
        ("init [1, 2]", "Maybe<Array<Int>>"),
        ("zip [True] [1, 2]", "Array<(Int, Bool)>"),
        (
            "zipWith (\\x: Int -> \\y: Int -> x + y) [3] [1, 2]",
            "Array<Int>",
        ),
        ("unzip [(1, True)]", "(Array<Int>, Array<Bool>)"),
        (
            "chunksOf 2 [1, 2]",
            "Either<collection.SizeError, Array<Array<Int>>>",
        ),
        (
            "windows 2 [1, 2]",
            "Either<collection.SizeError, Array<Array<Int>>>",
        ),
        (
            "reduceRight 0 (\\x: Int -> \\a: Int -> x - a) [1, 2]",
            "Int",
        ),
    ]
}

fn source(module: &str, expression: &str, result: &str) -> String {
    let (expression, result) = if module == "list" {
        (
            expression.replace('[', "`["),
            result.replace("Array", "List"),
        )
    } else {
        (expression.into(), result.into())
    };
    format!("// 雪🙂\nimport * as sequences from \"std/{module}\"\nimport * as collection from \"std/collection\"\npub fn probe -> {result} = sequences.{expression}\n")
}

#[test]
fn sequence_accepts_all_twenty_two_operations_with_exact_result_types() {
    for module in ["array", "list"] {
        for (expression, result) in expressions() {
            let source = source(module, expression, result);
            let artifact = emit_rust_module(&checked(&source), "probe")
                .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
            assert!(artifact
                .metadata
                .capabilities
                .iter()
                .any(|c| c == "sequence"));
        }
    }
    emit_rust_module(&checked("import { NonPositiveSize, SizeError } from \"std/collection\"\npub fn probe -> SizeError = NonPositiveSize (-2)\n"), "probe").unwrap();
}

#[test]
fn sequence_rejects_unproven_results_arguments_evidence_and_source_stages_at_original_span() {
    let eq = checked("pub fn probe -> Bool = 1 == 1\n");
    let CoreExpr::Binary {
        evidence: other, ..
    } = &eq.functions[0].body
    else {
        panic!()
    };
    for module in ["array", "list"] {
        for (expression, result) in expressions() {
            let core = checked(&format!(
                "type Pretender<A> = | Pretender A\n{}",
                source(module, expression, result)
            ));
            let original = core_expr_origin(&core.functions[0].body).clone();
            for mutation in 0..12 {
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
                } = &mut changed.functions[0].body
                else {
                    panic!()
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
                    2 => arguments.clear(),
                    3 => arguments.extend(vec![arguments[0].clone(); 4]),
                    4 => evidence.push(other[0].clone()),
                    5 => deferred_evidence_parameters.push(CoreType::Hole),
                    6 => deferred_evidence_type_constructor_parameters.push("F".into()),
                    7 => *evidence_argument_index = Some(0),
                    8 => {
                        *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                            trait_identity: "std/prelude::Eq".into(),
                            method: "equals".into(),
                            method_contract: None,
                        })
                    }
                    9 => {
                        let last = arguments.last_mut().unwrap();
                        *last = CoreExpr::Unit {
                            origin: core_expr_origin(last).clone(),
                        };
                    }
                    _ => {
                        let (CoreExpr::Array { type_ref, .. } | CoreExpr::List { type_ref, .. }) =
                            arguments.last_mut().unwrap()
                        else {
                            panic!()
                        };
                        *type_ref = if mutation == 10 {
                            CoreType::Named {
                                name: "native/sequence::Pretender".into(),
                                arguments: vec![CoreType::Named {
                                    name: "Int".into(),
                                    arguments: vec![],
                                }],
                            }
                        } else {
                            CoreType::Named {
                                name: if module == "array" { "Array" } else { "List" }.into(),
                                arguments: vec![CoreType::Named {
                                    name: "String".into(),
                                    arguments: vec![],
                                }],
                            }
                        };
                    }
                }
                let error = emit_rust_module(&changed, "probe")
                    .err()
                    .unwrap_or_else(|| panic!("{module}/{expression}/{mutation} accepted"));
                assert_eq!(error.code, "SES-RUST-0001");
                assert_eq!(
                    error.origin.unwrap(),
                    original,
                    "{module}/{expression}/{mutation}"
                );
            }
        }
    }
}

#[test]
fn sequence_keeps_saved_generic_and_partially_applied_source_stages() {
    for module in ["array", "list"] {
        let kind = if module == "array" { "Array" } else { "List" };
        let literal = if module == "array" {
            "[1, 2]"
        } else {
            "`[1, 2]"
        };
        let source = format!("import * as s from \"std/{module}\"\nfn pair<A,B> right: {kind}<B> -> left: {kind}<A> -> {kind}<(A,B)> = s.zip right left\npub fn probe -> {kind}<Int> = {{ let zip: (Int -> Int -> Int) -> {kind}<Int> -> {kind}<Int> -> {kind}<Int> = s.zipWith; let add = zip (\\a: Int -> \\b: Int -> a + b); let saved = add {literal}; match s.unzip (pair {literal} {literal}) {{ (left, _right) -> saved left }} }}\n");
        emit_rust_module(&checked(&source), "probe").unwrap();
    }
}

#[test]
fn sequence_size_error_constructor_rejects_wrong_owner_kind_arity_and_payload_at_original_span() {
    let core = checked("// 雪🙂\nimport * as c from \"std/collection\"\ntype SizeError = | FakeError Int\npub fn probe -> c.SizeError = c.NonPositiveSize 0\n");
    emit_rust_module(&core, "probe").unwrap();
    let origin = core_expr_origin(&core.functions[0].body).clone();
    for mutation in 0..6 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => {
                *type_ref = CoreType::Named {
                    name: "SizeError".into(),
                    arguments: vec![],
                }
            }
            2 => {
                *type_ref = CoreType::Named {
                    name: "std/collection::SizeError".into(),
                    arguments: vec![CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }],
                }
            }
            3 => arguments.clear(),
            4 => arguments.push(arguments[0].clone()),
            _ => {
                arguments[0] = CoreExpr::Unit {
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

#[test]
fn sequence_and_size_error_references_remain_refused_in_program_wasm() {
    for module in ["array", "list"] {
        for (expression, result) in expressions() {
            let original = source(module, expression, result);
            let (imports, call) = original.split_once("pub fn probe -> ").unwrap();
            let (result, body) = call.split_once(" = ").unwrap();
            let wrapped = format!("{imports}pub fn probe value: Int -> Int = {{ let saved: Unit -> {result} = \\_: Unit -> {}; value }}\n", body.trim());
            let core = checked(&wrapped);
            // Scalar export ABI; check the operation refusal rather than an unsupported boundary.
            let error =
                emit_program_wasm(&[&core], "native/sequence", &["probe".into()]).unwrap_err();
            assert!(
                error
                    .message
                    .contains("sequence operation requires a target-specific"),
                "{module}/{expression}: {error}"
            );
            assert_eq!(error.origin.unwrap().source, "sequence-雪🙂.ssrg");
        }
    }
    for (import, expression, signature) in [
        (
            "import { NonPositiveSize, SizeError } from \"std/collection\"",
            "NonPositiveSize",
            "Int -> SizeError",
        ),
        (
            "import * as s from \"std/array\"\nimport * as collection from \"std/collection\"",
            "s.windows",
            "Int -> Array<Int> -> Either<collection.SizeError, Array<Array<Int>>>",
        ),
        (
            "import * as s from \"std/list\"",
            "s.zip",
            "List<Bool> -> List<Int> -> List<(Int,Bool)>",
        ),
    ] {
        let core = checked(&format!("{import}\npub fn probe value: Int -> Int = {{ let saved: {signature} = {expression}; value }}\n"));
        // Scalar export ABI; check the operation refusal rather than an unsupported boundary.
        let error = emit_program_wasm(&[&core], "native/sequence", &["probe".into()]).unwrap_err();
        assert!(
            error
                .message
                .contains("sequence operation requires a target-specific"),
            "{expression}: {error}"
        );
    }
}
