use seseragi_backend_rust::{
    emit_program_wasm, emit_rust_module, emit_rust_module_with_options, NativeClockProvider,
    RustTargetOptions, NATIVE_CLOCK_PROVIDER,
};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(
        "clock-duration.ssrg",
        "native/time",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

fn options() -> RustTargetOptions {
    RustTargetOptions {
        clock_provider: Some(NativeClockProvider::BuiltinClock),
        ..RustTargetOptions::default()
    }
}

fn probe(module: &mut CoreModule) -> &mut CoreExpr {
    &mut module
        .functions
        .iter_mut()
        .find(|f| f.symbol.ends_with("::probe"))
        .unwrap()
        .body
}

#[test]
fn duration_fixture_admits_all_operations_without_selecting_clock() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/clock-duration.ssrg"
    ));
    let artifact = emit_rust_module(&module, "main").unwrap();
    assert!(artifact.metadata.providers.is_empty());
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|name| name == "duration"));
    for name in [
        "zeroDuration",
        "nanoseconds",
        "milliseconds",
        "seconds",
        "minutes",
        "hours",
        "toNanoseconds",
        "addDuration",
    ] {
        assert!(
            artifact.source.contains(&format!("std/time::{name}")),
            "{name}"
        );
    }
}

#[test]
fn all_duration_operations_reject_forged_result_types_evidence_and_source_arity() {
    let equality = checked("pub fn equality -> Bool = 1 == 1\n");
    let CoreExpr::Binary {
        evidence: dictionary,
        ..
    } = &equality.functions[0].body
    else {
        panic!()
    };
    for declaration in [
        "fn probe -> time.Duration = time.zeroDuration ()",
        "fn probe -> Either<time.DurationError, time.Duration> = time.nanoseconds 1",
        "fn probe -> Either<time.DurationError, time.Duration> = time.milliseconds 1",
        "fn probe -> Either<time.DurationError, time.Duration> = time.seconds 1",
        "fn probe -> Either<time.DurationError, time.Duration> = time.minutes 1",
        "fn probe -> Either<time.DurationError, time.Duration> = time.hours 1",
        "fn probe value: time.Duration -> Int = time.toNanoseconds value",
        "fn probe right: time.Duration -> left: time.Duration -> Either<time.DurationError, time.Duration> = time.addDuration right left",
    ] {
        let module = checked(&format!(
            "import * as time from \"std/time\"\n{declaration}\npub effect fn main = succeed ()\n"
        ));
        emit_rust_module(&module, "main").unwrap();
        for mutation in 0..5 {
            let mut changed = module.clone();
            let CoreExpr::Call {
                arguments,
                type_ref,
                callee_signature,
                evidence,
                ..
            } = probe(&mut changed) else {
                panic!("{declaration}")
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => arguments.push(arguments[0].clone()),
                2 => {
                    let origin = seseragi_lowering::core_expr_origin(&arguments[0]).clone();
                    arguments[0] = if declaration.contains("zeroDuration") {
                        CoreExpr::Integer { value: "0".into(), origin }
                    } else {
                        CoreExpr::Unit { origin }
                    };
                }
                3 => {
                    *callee_signature = Some(seseragi_lowering::CoreCallSignature {
                        type_parameters: vec![],
                        parameters: vec![],
                        result: type_ref.clone(),
                        argument_types: vec![],
                        instantiated_type_arguments: vec![],
                    });
                }
                _ => evidence.extend(dictionary.clone()),
            }
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001", "{declaration}/{mutation}");
            assert_eq!(error.origin.unwrap().source, "clock-duration.ssrg");
        }
    }
}

#[test]
fn now_validates_explicit_selection_and_records_both_contract_operations() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/clock-now.ssrg"
    ));
    let missing = emit_rust_module(&module, "main").unwrap_err();
    assert_eq!(missing.code, "SES-RUST-0001");
    assert_eq!(missing.origin.unwrap().source, "clock-duration.ssrg");
    let artifact = emit_rust_module_with_options(&module, "main", &options()).unwrap();
    let [provider] = artifact.metadata.providers.as_slice() else {
        panic!()
    };
    assert_eq!(provider.provider, NATIVE_CLOCK_PROVIDER);
    assert_eq!(provider.service, "std/clock::Clock");
    assert_eq!(
        provider.supported_operations,
        ["std/clock::Clock#now", "std/clock::Clock#sleep"]
    );
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|name| name == "clock-now"));
    assert!(!provider.traces.is_empty());
    assert_eq!(
        emit_program_wasm(&[&module], &module.module, &["main".into()])
            .unwrap_err()
            .code,
        "SES-RUST-0001"
    );
}

#[test]
fn now_rejects_forged_unit_requirements_channels_and_nominal_instant() {
    let mut module = checked("import * as clock from \"std/clock\"\nimport * as time from \"std/time\"\neffect fn probe -> time.Instant with Clock = clock.now ()\npub effect fn main = succeed ()\n");
    emit_rust_module_with_options(&module, "main", &options()).unwrap();
    let original = probe(&mut module).clone();
    let (CoreExpr::Call {
        arguments, origin, ..
    }
    | CoreExpr::EffectOperation {
        arguments, origin, ..
    }
    | CoreExpr::EffectInvoke {
        arguments, origin, ..
    }) = original
    else {
        panic!("{original:#?}")
    };
    let named = |name: &str| CoreType::Named {
        name: name.into(),
        arguments: vec![],
    };
    *probe(&mut module) = CoreExpr::EffectInvoke {
        callee: "std/clock::now".into(),
        callee_signature: None,
        requirements: CoreType::Record {
            closed: true,
            fields: vec![seseragi_lowering::CoreRecordField {
                name: "clock".into(),
                optional: false,
                type_ref: named("std/clock::Clock"),
            }],
        },
        failure: named("std/prelude::Never"),
        success: named("std/time::Instant"),
        arguments,
        evidence: vec![],
        origin,
    };
    emit_rust_module_with_options(&module, "main", &options()).unwrap();
    for mutation in 0..6 {
        let mut changed = module.clone();
        let (CoreExpr::EffectOperation {
            arguments,
            requirements,
            failure,
            success,
            ..
        }
        | CoreExpr::EffectInvoke {
            arguments,
            requirements,
            failure,
            success,
            ..
        }) = probe(&mut changed)
        else {
            panic!()
        };
        match mutation {
            0 => arguments.clear(),
            1 => arguments.push(arguments[0].clone()),
            2 => {
                arguments[0] = CoreExpr::Integer {
                    value: "0".into(),
                    origin: seseragi_lowering::core_expr_origin(&arguments[0]).clone(),
                }
            }
            3 => *requirements = CoreType::Hole,
            4 => {
                *failure = CoreType::Named {
                    name: "std/prelude::String".into(),
                    arguments: vec![],
                }
            }
            _ => {
                *success = CoreType::ExternalNamed {
                    name: "Instant".into(),
                    canonical: "other::Instant".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module_with_options(&changed, "main", &options()).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "{mutation}");
        assert_eq!(error.origin.unwrap().source, "clock-duration.ssrg");
    }
}
