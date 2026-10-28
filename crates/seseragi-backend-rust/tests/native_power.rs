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
}

#[test]
fn native_float_power_function_preserves_exponent_first_source_arity() {
    native("import * as floats from \"std/float\"\npub fn probe -> Float = floats.power 3.0 2.0\n");
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
    let core = compile_core_module(CompileInput::new(
        "power-wasm-雪.ssrg",
        "native/power",
        "pub fn square value: Int -> Int = value ** 2\npub fn probe -> Int = square 3\n",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    let error =
        seseragi_backend_rust::emit_program_wasm(&[&core], "native/power", &["square".into()])
            .expect_err("program WASM needs a separate arithmetic proof");
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "power-wasm-雪.ssrg");
}
