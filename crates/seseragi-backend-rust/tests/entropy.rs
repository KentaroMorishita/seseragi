use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(
        "entropy-雪.ssrg",
        "native/entropy",
        source,
    ))
    .unwrap()
    .core_ir
}

fn cases() -> [(&'static str, &'static str, &'static str); 2] {
    [
        (
            "value:Int",
            "Either<entropy.EntropyConfigError,entropy.EntropySize>",
            "entropy.entropySize value",
        ),
        (
            "value:entropy.EntropySize",
            "Effect<{entropy:entropy.Entropy},entropy.EntropyError,bytes.Bytes>",
            "entropy.secureBytes value",
        ),
    ]
}

#[test]
fn entropy_admits_checked_apis_and_saved_functions() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/entropy.ssrg"
    ));
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "entropy"));
    for name in ["entropySize", "secureBytes"] {
        assert!(artifact
            .source
            .contains(&format!("native_entropy_operation(\"std/entropy::{name}\"")));
    }
}

#[test]
fn entropy_rejects_forged_call_types_evidence_and_source_stages_at_the_original_span() {
    let eq = checked("pub fn main -> Bool = 1 == 1");
    let CoreExpr::Binary { evidence, .. } = &eq.functions[0].body else {
        panic!()
    };
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as entropy from \"std/entropy\"\nimport * as bytes from \"std/bytes\"\npub fn probe {parameter} -> {result} = {expression}\npub fn main -> Int = 0\n"));
        emit_rust_module(&core, "main").unwrap();
        let probe = core
            .functions
            .iter()
            .position(|f| f.symbol.ends_with("::probe"))
            .unwrap();
        let origin = core_expr_origin(&core.functions[probe].body).clone();
        for mutation in 0..9 {
            let mut changed = core.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                evidence: selected,
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
                1 => arguments.clear(),
                2 => arguments.push(arguments[0].clone()),
                3 => selected.push(evidence[0].clone()),
                4 => deferred_evidence_parameters.push(CoreType::Hole),
                5 => deferred_evidence_type_constructor_parameters.push("F".into()),
                6 => *evidence_argument_index = Some(0),
                7 => {
                    arguments[0] = CoreExpr::Boolean {
                        value: true,
                        origin: core_expr_origin(&arguments[0]).clone(),
                    }
                }
                8 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Either".into(),
                        canonical: "fake::Either".into(),
                        arguments: vec![],
                    }
                }
                _ => unreachable!(),
            }
            let error = emit_rust_module(&changed, "main").expect_err("unproven Entropy call");
            assert_eq!(error.code, "SES-RUST-0001", "{expression}/{mutation}");
            assert_eq!(error.origin.unwrap(), origin, "{expression}/{mutation}");
        }
    }
}

#[test]
fn entropy_effects_reject_forged_requirement_rows_owners_results_and_arity() {
    let mut core = checked(
        "import * as entropy from \"std/entropy\"\nimport * as bytes from \"std/bytes\"\npub effect fn probe size:entropy.EntropySize -> bytes.Bytes with Entropy fails entropy.EntropyError = entropy.secureBytes size\npub fn main -> Int = 0\n",
    );
    emit_rust_module(&core, "main").unwrap();
    let probe = core
        .functions
        .iter()
        .position(|f| f.symbol.ends_with("::probe"))
        .unwrap();
    let origin = core_expr_origin(&core.functions[probe].body).clone();
    let CoreExpr::Call {
        callee,
        arguments,
        type_ref,
        evidence,
        ..
    } = core.functions[probe].body.clone()
    else {
        panic!()
    };
    let (CoreType::Named {
        arguments: types, ..
    }
    | CoreType::ExternalNamed {
        arguments: types, ..
    }) = type_ref
    else {
        panic!()
    };
    let [requirements, failure, success]: [CoreType; 3] = types.try_into().unwrap();
    core.functions[probe].body = CoreExpr::EffectInvoke {
        callee,
        callee_signature: None,
        arguments,
        requirements,
        failure,
        success,
        evidence,
        origin: origin.clone(),
    };
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..9 {
        let mut changed = core.clone();
        let CoreExpr::EffectInvoke {
            arguments,
            requirements,
            failure,
            success,
            ..
        } = &mut changed.functions[probe].body
        else {
            panic!()
        };
        let CoreType::Record { closed, fields } = requirements else {
            panic!()
        };
        match mutation {
            0 => *closed = false,
            1 => fields[0].optional = true,
            2 => fields[0].name = "other".into(),
            3 => {
                fields[0].type_ref = CoreType::ExternalNamed {
                    name: "Entropy".into(),
                    canonical: "fake::Entropy".into(),
                    arguments: vec![],
                }
            }
            4 => fields.push(fields[0].clone()),
            5 => *failure = CoreType::Hole,
            6 => *success = CoreType::Hole,
            7 => arguments.clear(),
            8 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                    panic!()
                };
                *type_ref = CoreType::ExternalNamed {
                    name: "EntropySize".into(),
                    canonical: "fake::EntropySize".into(),
                    arguments: vec![],
                }
            }
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "main").expect_err("unproven Entropy effect");
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap(), origin, "mutation {mutation}");
    }
}

#[test]
fn unused_entropy_calls_and_references_keep_program_wasm_admission_unchanged() {
    for (parameter, result, expression) in cases() {
        let core = checked(&format!("import * as entropy from \"std/entropy\"\nimport * as bytes from \"std/bytes\"\nfn hidden {parameter} -> {result} = {expression}\npub fn main -> Int = 42\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
        assert!(error.message.contains("native Entropy"), "{error}");
    }
    for (name, function) in [
        ("entropySize", "Int -> Either<entropy.EntropyConfigError,entropy.EntropySize>"),
        ("secureBytes", "entropy.EntropySize -> Effect<{entropy:entropy.Entropy},entropy.EntropyError,bytes.Bytes>"),
    ] {
        let core = checked(&format!("import * as entropy from \"std/entropy\"\nimport * as bytes from \"std/bytes\"\nfn hidden -> {function} = entropy.{name}\npub fn main -> Int = 42\n"));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert!(error.origin.is_some());
        assert!(error.message.contains("native Entropy"), "{error}");
    }
}
