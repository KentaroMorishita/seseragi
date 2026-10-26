use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn native_int_remainder_accepts_checked_canonical_remainder() {
    let checked = compile_core_module(CompileInput::new(
        "rem-雪🙂.ssrg",
        "native/main",
        "pub fn probe -> Int = 7 % 3\n",
    ))
    .unwrap();
    emit_rust_module(&checked.core_ir, "probe").expect("native Int remainder");
}

#[test]
fn native_remainder_does_not_expand_program_wasm_arithmetic_admission() {
    let source = "pub fn quotient n: Int -> Int = n % 2\npub fn probe -> Int = quotient 7\n";
    let core = compile_core_module(CompileInput::new(
        "rem-target-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").expect("ordinary native target supports checked Int Rem");
    let error =
        seseragi_backend_rust::emit_program_wasm(&[&core], "native/main", &["quotient".into()])
            .expect_err("program WASM arithmetic still requires its own proof");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "rem-target-雪.ssrg");
}

#[test]
fn remainder_does_not_substitute_a_lexical_binder_named_int() {
    let source = "fn generic<A> n: A -> (A -> A) where Rem<A,A,A> = rem n\npub fn probe -> Int = { let saved = generic 7; saved 2 }";
    let mut core = compile_core_module(CompileInput::new(
        "rem-binder-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    let function = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    fn rename(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::String(name) if name == "A" => *name = "Int".into(),
            serde_json::Value::Object(fields) => fields.values_mut().for_each(rename),
            serde_json::Value::Array(values) => values.iter_mut().for_each(rename),
            _ => (),
        }
    }
    let mut encoded = serde_json::to_value(&*function).unwrap();
    rename(&mut encoded);
    *function = serde_json::from_value(encoded).unwrap();
    emit_rust_module(&core, "probe").expect("alpha-renamed lexical dictionary remains valid");
    let function = core
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::generic"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &mut function.body else {
        panic!("method")
    };
    evidence[0].evidence = CoreInstanceEvidence::Standard {
        identity: "std/int::Rem".into(),
        type_arguments: vec![],
        evidence_arguments: vec![],
    };
    let error =
        emit_rust_module(&core, "probe").expect_err("lexical Int is not the canonical primitive");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "rem-binder-雪.ssrg");
}

#[test]
fn primitive_remainder_refuses_forged_binary_evidence_and_operand_types() {
    let core = compile_core_module(CompileInput::new(
        "rem-雪🙂.ssrg",
        "native/main",
        "pub fn probe -> Int = 7 % 2\n",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..12 {
        let mut changed = core.clone();
        let CoreExpr::Binary {
            evidence,
            left,
            right,
            type_ref,
            origin,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("checked binary")
        };
        let boolean = CoreType::Named {
            name: "Bool".into(),
            arguments: vec![],
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Rem".into()),
            1 => evidence[0].constraint.name = "Mul".into(),
            2 => {
                evidence[0].constraint.arguments.pop();
            }
            3 => evidence[0].constraint.arguments[0] = boolean,
            4 => *type_ref = CoreType::Hole,
            5 => {
                if let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                {
                    type_arguments.push(boolean);
                }
            }
            6 => {
                let child = evidence[0].clone();
                if let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                {
                    evidence_arguments.push(child);
                }
            }
            7 => evidence.push(evidence[0].clone()),
            8 => evidence.clear(),
            9 => {
                if let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence {
                    *identity = "std/int::Add".into();
                }
            }
            10 => {
                **left = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            _ => {
                **right = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("mutation {mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "rem-雪🙂.ssrg");
    }
}

#[test]
fn remainder_method_refuses_forged_source_stages_and_dispatch() {
    let source = "pub fn probe -> (Int -> Int) = rem 7\n";
    let core = compile_core_module(CompileInput::new(
        "rem-method-雪.ssrg",
        "native/main",
        source,
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..7 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            trait_dispatch,
            type_ref,
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("checked method")
        };
        match mutation {
            0 => trait_dispatch.as_mut().unwrap().trait_identity = "fake::Rem".into(),
            1 => trait_dispatch.as_mut().unwrap().method = "mul".into(),
            2 => *type_ref = CoreType::Hole,
            3 => {
                arguments.push(arguments[0].clone());
                arguments.push(arguments[0].clone());
            }
            4 => deferred_evidence_parameters.push(CoreType::Hole),
            5 => *evidence_argument_index = Some(0),
            _ => {
                evidence[0].constraint.arguments[1] = CoreType::ExternalNamed {
                    name: "Int".into(),
                    canonical: "fake::Int".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").expect_err(&format!("mutation {mutation}"));
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "rem-method-雪.ssrg");
    }
}
