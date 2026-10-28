use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

fn native(source: &str) {
    let checked =
        compile_core_module(CompileInput::new("power-雪🙂.ssrg", "native/power", source)).unwrap();
    emit_rust_module(&checked.core_ir, "probe").expect("checked native power");
}

#[test]
fn native_int_power_accepts_the_canonical_dictionary() {
    native("pub fn probe -> Int = 2 ** 3\n");
}

#[test]
fn native_float_power_accepts_the_canonical_dictionary() {
    native("pub fn probe -> Float = 2.0 ** 3.0\n");
    // Retain the exact former target-refusal input as a newly supported case.
    native("pub fn probe -> Float = 1.5 ** 2.0\n");
}

#[test]
fn native_float_power_function_preserves_exponent_first_source_arity() {
    native("import * as floats from \"std/float\"\npub fn probe -> Float = floats.power 3.0 2.0\n");
}

#[test]
fn power_does_not_replace_a_scoped_binder_named_int_or_float() {
    for (scalar, base, exponent, provider) in [
        ("Int", "2", "3", "std/int::Pow"),
        ("Float", "2.0", "3.0", "std/float::Pow"),
    ] {
        let source = format!("fn generic<A> base: A -> (A -> A) where Pow<A,A,A> = pow base\npub fn probe -> {scalar} = {{ let saved = generic {base}; saved {exponent} }}");
        let mut core = compile_core_module(CompileInput::new(
            "power-binder-雪.ssrg",
            "native/power",
            &source,
        ))
        .unwrap()
        .core_ir;
        let function = core
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::generic"))
            .unwrap();
        fn rename(value: &mut serde_json::Value, scalar: &str) {
            match value {
                serde_json::Value::String(name) if name == "A" => *name = scalar.into(),
                serde_json::Value::Object(fields) => {
                    fields.values_mut().for_each(|v| rename(v, scalar))
                }
                serde_json::Value::Array(values) => {
                    values.iter_mut().for_each(|v| rename(v, scalar))
                }
                _ => (),
            }
        }
        let mut encoded = serde_json::to_value(&*function).unwrap();
        rename(&mut encoded, scalar);
        *function = serde_json::from_value(encoded).unwrap();
        emit_rust_module(&core, "probe").expect("alpha-renamed lexical dictionary remains valid");
        let function = core
            .functions
            .iter_mut()
            .find(|f| f.symbol.ends_with("::generic"))
            .unwrap();
        let CoreExpr::Call { evidence, .. } = &mut function.body else {
            panic!("saved method")
        };
        evidence[0].evidence = CoreInstanceEvidence::Standard {
            identity: provider.into(),
            type_arguments: vec![],
            evidence_arguments: vec![],
        };
        let error = emit_rust_module(&core, "probe")
            .expect_err("a lexical binder is not a canonical primitive");
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap().source, "power-binder-雪.ssrg");
    }
}

#[test]
fn power_preserves_saved_methods_generic_dictionaries_and_returned_functions() {
    native("fn selected<A> base: A -> (A -> A) where Pow<A,A,A> = pow base\npub fn probe -> Int = { let saved = selected 2; saved 3 + saved 4 }\n");
    native("fn selected<A> base: A -> (A -> A) where Pow<A,A,A> = \\exponent -> base ** exponent\npub fn probe -> Float = { let saved = selected 2.0; saved 3.0 + saved 4.0 }\n");
    native("import * as floats from \"std/float\"\npub fn probe -> Float = { let saved = floats.power 3.0; saved 2.0 }\n");
}

#[test]
fn power_rejects_forged_canonical_evidence_operand_and_result_types() {
    for (scalar, values) in [("Int", "2 ** 3"), ("Float", "2.0 ** 3.0")] {
        let source = format!("pub fn probe -> {scalar} = {values}\n");
        let core = compile_core_module(CompileInput::new(
            "power-雪🙂.ssrg",
            "native/power",
            &source,
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        for mutation in 0..13 {
            let mut changed = core.clone();
            let CoreExpr::Binary {
                operator,
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
                0 => evidence[0].constraint.trait_identity = Some("fake::Pow".into()),
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
                    if let CoreInstanceEvidence::Standard { identity, .. } =
                        &mut evidence[0].evidence
                    {
                        *identity = "std/int::Add".into();
                    }
                }
                10 => {
                    **left = CoreExpr::Unit {
                        origin: origin.clone(),
                    }
                }
                11 => {
                    **right = CoreExpr::Unit {
                        origin: origin.clone(),
                    }
                }
                12 => *operator = "+".into(),
                _ => unreachable!(),
            }
            let error =
                emit_rust_module(&changed, "probe").expect_err("unproven power must not emit");
            assert_eq!(error.code, "SES-RUST-0001", "{scalar}/{mutation}");
            assert_eq!(
                error.origin.unwrap().source,
                "power-雪🙂.ssrg",
                "{scalar}/{mutation}"
            );
        }
    }
}

#[test]
fn native_power_does_not_expand_program_wasm_admission() {
    for source in [
        "pub fn square value: Int -> Int = value ** 2\npub fn probe -> Int = square 3\n",
        "fn selected<A> base: A -> exponent: A -> A where Pow<A,A,A> = base ** exponent\npub fn square value: Int -> Int = selected value 2\npub fn probe -> Int = square 3\n",
    ] {
        let core = compile_core_module(CompileInput::new(
            "power-wasm-雪.ssrg", "native/power", source,
        )).unwrap().core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let error = seseragi_backend_rust::emit_program_wasm(&[&core], "native/power", &["square".into()])
            .expect_err("program WASM needs a separate arithmetic proof");
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.message.contains("arithmetic proof"), "{error:?}");
        assert_eq!(error.origin.unwrap().source, "power-wasm-雪.ssrg");
    }
}

#[test]
fn float_namespace_power_rejects_forged_source_stages_types_and_evidence() {
    let core = compile_core_module(CompileInput::new(
        "namespace-power-雪🙂.ssrg",
        "native/power",
        "import * as floats from \"std/float\"\npub fn probe -> Float = floats.power 3.0 2.0\n",
    ))
    .unwrap()
    .core_ir;
    let proof = compile_core_module(CompileInput::new(
        "proof.ssrg",
        "native/proof",
        "pub fn probe -> Float = 2.0 ** 3.0\n",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary {
        evidence: dictionary,
        ..
    } = &proof.functions[0].body
    else {
        panic!("power proof")
    };
    for mutation in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            evidence,
            type_ref,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            evidence_argument_index,
            origin,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("namespace call")
        };
        match mutation {
            0 => {
                arguments.pop();
            }
            1 => arguments.push(arguments[0].clone()),
            2 => {
                arguments[0] = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            3 => *type_ref = CoreType::Hole,
            4 => evidence.extend(dictionary.clone()),
            5 => deferred_evidence_parameters.push(CoreType::Hole),
            6 => deferred_evidence_type_constructor_parameters.push("A".into()),
            7 => *evidence_argument_index = Some(0),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").expect_err("unproven namespace power");
        assert_eq!(error.code, "SES-RUST-0001", "{mutation}");
        assert_eq!(error.origin.unwrap().source, "namespace-power-雪🙂.ssrg");
    }
}
