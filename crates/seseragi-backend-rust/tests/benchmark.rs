use seseragi_backend_rust::{emit_program_wasm, emit_rust_benchmarks, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(
        "benchmark.ssrg",
        "native/benchmark",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn benchmark_apis_accept_saved_partial_generic_and_cold_values() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/std-benchmark.ssrg"
    ));
    let artifact = emit_rust_module(&module, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|capability| capability == "benchmark"));
}

#[test]
fn benchmark_runner_checks_export_type_and_retains_module_order() {
    let module = checked("import * as b from \"std/benchmark\"\nimport * as e from \"std/effect\"\npub let benchmarks:b.Benchmark = b.benchmark \"case\" (e.succeed ())\n");
    let artifact = emit_rust_benchmarks(
        &[&module],
        &[("main".into(), "native/benchmark::benchmarks".into())],
    )
    .unwrap();
    assert_eq!(artifact.metadata.entry_kind, "benchmark");
    assert_eq!(artifact.metadata.modules, ["native/benchmark"]);
    assert!(artifact.source.contains("run_native_benchmarks_entry"));
    let wrong = checked("pub let benchmarks:Int = 42\n");
    assert!(emit_rust_benchmarks(
        &[&wrong],
        &[("main".into(), "native/benchmark::benchmarks".into())]
    )
    .is_err());
}

#[test]
fn benchmark_rejects_forged_types_evidence_and_source_stages() {
    let core = checked("import * as b from \"std/benchmark\"\nfn probe value:Int -> Int = b.blackBox value\npub fn main -> Int = probe 42\n");
    emit_rust_module(&core, "main")
        .unwrap_or_else(|error| panic!("{:?}: {error}", core.functions[0].body));
    for mutation in 0..14 {
        let mut changed = core.clone();
        let probe = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let CoreExpr::Call {
            type_ref,
            arguments,
            evidence,
            trait_dispatch,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            callee_signature,
            ..
        } = &mut probe.body
        else {
            panic!("checked blackBox call");
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1 => {
                *type_ref = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            2 => arguments.push(arguments[0].clone()),
            3 => arguments.clear(),
            4 => *evidence_argument_index = Some(0),
            5 => deferred_evidence_parameters.push(CoreType::Hole),
            6 => deferred_evidence_type_constructor_parameters.push("F".into()),
            7 => {
                *trait_dispatch = Some(seseragi_lowering::CoreTraitDispatch {
                    trait_identity: "std/prelude::Eq".into(),
                    method: "eq".into(),
                    method_contract: None,
                });
            }
            8 => {
                *callee_signature = Some(seseragi_lowering::CoreCallSignature {
                    type_parameters: vec![],
                    parameters: vec![],
                    result: type_ref.clone(),
                    argument_types: vec![],
                    instantiated_type_arguments: vec![],
                });
            }
            9 => callee_signature.as_mut().unwrap().type_parameters[0].arity = 1,
            10 => {
                callee_signature
                    .as_mut()
                    .unwrap()
                    .instantiated_type_arguments[0] = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            11 => {
                callee_signature.as_mut().unwrap().result = CoreType::Named {
                    name: "String".into(),
                    arguments: vec![],
                }
            }
            12 => {
                callee_signature.as_mut().unwrap().argument_types[0] = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            13 => callee_signature
                .as_mut()
                .unwrap()
                .instantiated_type_arguments
                .clear(),
            _ => unreachable!(),
        }
        assert!(evidence.is_empty());
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
    }
}

#[test]
fn benchmark_bodies_require_canonical_services_failure_and_unit_result() {
    let core = checked("import * as b from \"std/benchmark\"\nfn probe body:Effect<{},b.BenchmarkFailure,Unit> -> b.Benchmark = b.benchmark \"case\" body\npub fn main -> Int = 42\n");
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..8 {
        let mut changed = core.clone();
        let probe = changed
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let CoreExpr::Call {
            arguments,
            type_ref,
            ..
        } = &mut probe.body
        else {
            panic!("checked benchmark call");
        };
        if mutation == 0 {
            *type_ref = CoreType::ExternalNamed {
                name: "Benchmark".into(),
                canonical: "counterfeit::Benchmark".into(),
                arguments: vec![],
            };
        } else {
            let CoreExpr::Variable { type_ref, .. } = &mut arguments[1] else {
                panic!("checked body argument");
            };
            match mutation {
                1 => *type_ref = CoreType::Hole,
                2 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Effect".into(),
                        canonical: "counterfeit::Effect".into(),
                        arguments: vec![],
                    };
                }
                _ => {
                    let types = match type_ref {
                        CoreType::Named { arguments, .. }
                        | CoreType::ExternalNamed { arguments, .. } => arguments,
                        _ => panic!("Effect type"),
                    };
                    match mutation {
                        3 => {
                            types[0] = CoreType::Record {
                                closed: false,
                                fields: vec![],
                            }
                        }
                        4 => {
                            types[0] = CoreType::Record {
                                closed: true,
                                fields: vec![seseragi_lowering::CoreRecordField {
                                    name: "clock".into(),
                                    optional: false,
                                    type_ref: CoreType::ExternalNamed {
                                        name: "Clock".into(),
                                        canonical: "std/clock::Clock".into(),
                                        arguments: vec![],
                                    },
                                }],
                            }
                        }
                        5 => {
                            types[1] = CoreType::Named {
                                name: "String".into(),
                                arguments: vec![],
                            }
                        }
                        6 => {
                            types[2] = CoreType::Named {
                                name: "Int".into(),
                                arguments: vec![],
                            }
                        }
                        7 => {
                            types.pop();
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
    }
}

#[test]
fn unused_benchmark_apis_do_not_widen_program_wasm() {
    for (parameters, result, expression) in [
        ("value:Int", "Int", "b.blackBox value"),
        (
            "name:String",
            "b.Benchmark",
            "b.benchmark name (e.succeed ())",
        ),
        ("name:String", "b.Benchmark", "b.suite name []"),
        (
            "size:Int",
            "b.Benchmark",
            "b.inputSize size (b.suite \"empty\" [])",
        ),
        (
            "message:String",
            "Effect<{},b.BenchmarkFailure,Unit>",
            "b.fail message",
        ),
    ] {
        let core = checked(&format!("import * as b from \"std/benchmark\"\nimport * as e from \"std/effect\"\nfn unused {parameters} -> {result} = {expression}\npub fn main value:Int -> Int = value\n"));
        assert!(emit_program_wasm(&[&core], "native/benchmark", &["main".into()]).is_err());
    }
}
