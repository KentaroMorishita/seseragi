use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "big-key-雪🙂.ssrg",
        "native/big-int-keys",
        source,
    ))
    .unwrap()
    .core_ir
}

#[test]
fn big_show_checks_selected_identity_children_argument_result_and_source_stages() {
    let core=checked("// 雪🙂\nimport * as big from \"std/big-int\"\nfn render value: big.BigInt -> String = show value\npub fn main -> Int = 0\n");
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::render"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..18 {
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
                *type_ref = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                }
            }
            2 => evidence.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => evidence[0].constraint.name = "Eq".into(),
            5 => evidence[0].constraint.trait_identity = Some("std/prelude::Debug".into()),
            6 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "Show<std/prelude::String>".into();
            }
            7 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            8 => {
                let extra = evidence[0].clone();
                let seseragi_lowering::CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            9 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            10 => trait_dispatch.as_mut().unwrap().trait_identity = "std/prelude::Debug".into(),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 => *evidence_argument_index = Some(0),
            14 => {
                arguments[0] = CoreExpr::Unit {
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
            15 => arguments.push(arguments[0].clone()),
            16 => evidence.push(evidence[0].clone()),
            _ => *trait_dispatch = None,
        }
        let error = emit_rust_module(&bad, "main").expect_err("forged BigInt Show");
        assert_eq!(error.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn big_saved_show_has_its_exact_function_type_and_keeps_program_wasm_closed() {
    let saved = checked("import * as big from \"std/big-int\"\nfn formatter -> (big.BigInt -> String) = show\npub fn main -> Int = 0\n");
    emit_rust_module(&saved, "main").unwrap();
    let index = saved
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::formatter"))
        .unwrap();
    let origin = core_expr_origin(&saved.functions[index].body).clone();
    for result in [
        CoreType::Hole,
        CoreType::Named {
            name: "std/prelude::Int".into(),
            arguments: vec![],
        },
    ] {
        let mut bad = saved.clone();
        let CoreExpr::Call { type_ref, .. } = &mut bad.functions[index].body else {
            panic!()
        };
        let CoreType::Function { result: actual, .. } = type_ref else {
            panic!()
        };
        **actual = result;
        let error = emit_rust_module(&bad, "main").expect_err("forged saved BigInt Show result");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
    let core=checked("import * as big from \"std/big-int\"\npub fn probe value: Int -> Int = { let saved: big.BigInt -> String = show; value }\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let error = emit_program_wasm(&[&core], "native/big-int-keys", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("requires a target-specific"));
    assert_eq!(error.origin.unwrap().source, "big-key-雪🙂.ssrg");
}
#[test]
fn big_debug_checks_selected_identity_children_argument_result_and_source_stages() {
    let core=checked("// 雪🙂\nimport * as big from \"std/big-int\"\nfn render value: big.BigInt -> String = debug value\npub fn main -> Int = 0\n");
    let index = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::render"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[index].body).clone();
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..18 {
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
                *type_ref = CoreType::Named {
                    name: "std/prelude::Int".into(),
                    arguments: vec![],
                }
            }
            2 => evidence.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => evidence[0].constraint.name = "Eq".into(),
            5 => evidence[0].constraint.trait_identity = Some("std/prelude::Show".into()),
            6 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "Debug<std/prelude::String>".into();
            }
            7 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            8 => {
                let extra = evidence[0].clone();
                let seseragi_lowering::CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            9 => trait_dispatch.as_mut().unwrap().method = "show".into(),
            10 => trait_dispatch.as_mut().unwrap().trait_identity = "std/prelude::Show".into(),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 => *evidence_argument_index = Some(0),
            14 => {
                arguments[0] = CoreExpr::Unit {
                    origin: core_expr_origin(&arguments[0]).clone(),
                }
            }
            15 => arguments.push(arguments[0].clone()),
            16 => evidence.push(evidence[0].clone()),
            _ => *trait_dispatch = None,
        }
        let error = emit_rust_module(&bad, "main").expect_err("forged BigInt Debug");
        assert_eq!(error.code, "SES-RUST-0001", "mutation{mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation{mutation}");
    }
}

#[test]
fn big_saved_debug_has_its_exact_function_type_and_keeps_program_wasm_closed() {
    let saved = checked("import * as big from \"std/big-int\"\nfn formatter -> (big.BigInt -> String) = debug\npub fn main -> Int = 0\n");
    emit_rust_module(&saved, "main").unwrap();
    let index = saved
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::formatter"))
        .unwrap();
    let origin = core_expr_origin(&saved.functions[index].body).clone();
    for result in [
        CoreType::Hole,
        CoreType::Named {
            name: "std/prelude::Int".into(),
            arguments: vec![],
        },
    ] {
        let mut bad = saved.clone();
        let CoreExpr::Call { type_ref, .. } = &mut bad.functions[index].body else {
            panic!()
        };
        let CoreType::Function { result: actual, .. } = type_ref else {
            panic!()
        };
        **actual = result;
        let error = emit_rust_module(&bad, "main").expect_err("forged saved BigInt Debug result");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
    let core=checked("import * as big from \"std/big-int\"\npub fn probe value: Int -> Int = { let saved: big.BigInt -> String = debug; value }\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    let error = emit_program_wasm(&[&core], "native/big-int-keys", &["probe".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.message.contains("requires a target-specific"));
    assert_eq!(error.origin.unwrap().source, "big-key-雪🙂.ssrg");
}
