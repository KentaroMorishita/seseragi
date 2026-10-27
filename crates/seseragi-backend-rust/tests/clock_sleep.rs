use seseragi_backend_rust::{
    emit_program_wasm, emit_rust_module, emit_rust_module_with_options, NativeClockProvider,
    NativeProviderSelectionOrigin, RustTargetOptions, NATIVE_CLOCK_SLEEP_PROVIDER,
};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreCallSignature, CoreExpr, CoreModule, CoreRecordField, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new("clock-雫🙂.ssrg", "native/clock", source))
        .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
        .core_ir
}
fn options() -> RustTargetOptions {
    RustTargetOptions {
        clock_provider: Some(NativeClockProvider::BuiltinSleep),
        clock_selection_origin: NativeProviderSelectionOrigin::ExplicitApi,
    }
}
fn ty(name: &str) -> CoreType {
    CoreType::Named {
        name: name.into(),
        arguments: vec![],
    }
}
fn clock_row() -> CoreType {
    CoreType::Record {
        closed: true,
        fields: vec![CoreRecordField {
            name: "clock".into(),
            optional: false,
            type_ref: ty("std/clock::Clock"),
        }],
    }
}
fn wait_body(module: &mut CoreModule) -> &mut CoreExpr {
    &mut module
        .functions
        .iter_mut()
        .find(|function| function.symbol.ends_with("::wait"))
        .unwrap()
        .body
}
fn checked_wait() -> CoreModule {
    checked("import * as clock from \"std/clock\"\nimport * as time from \"std/time\"\nimport * as effects from \"std/effect\"\neffect fn wait pause: time.Duration -> Unit with Clock = clock.sleep pause\npub effect fn main = effects.succeed ()\n")
}

#[test]
fn delayed_transform_requires_explicit_finite_binding_and_records_contract() {
    let module = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/filesystem-delayed-transform.ssrg"
    ));
    let missing =
        emit_rust_module(&module, "main").expect_err("no implicit JS provider substitution");
    assert_eq!(missing.code, "SES-RUST-0001");
    assert_eq!(missing.origin.unwrap().source, "clock-雫🙂.ssrg");
    let artifact = emit_rust_module_with_options(&module, "main", &options()).unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "clock-sleep"));
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "filesystem-async"));
    let [provider] = artifact.metadata.providers.as_slice() else {
        panic!()
    };
    assert_eq!(provider.target, "native");
    assert_eq!(provider.provider, NATIVE_CLOCK_SLEEP_PROVIDER);
    assert_eq!(provider.service, "std/clock::Clock");
    assert_eq!(
        provider.selection,
        NativeProviderSelectionOrigin::ExplicitApi
    );
    assert_eq!(
        provider.contract_version,
        seseragi_provider::ContractVersion { major: 1, minor: 0 }
    );
    assert_eq!(provider.supported_operations, ["std/clock::Clock#sleep"]);
    assert_eq!(provider.contract_digest.len(), 64);
    assert!(provider
        .contract_digest
        .bytes()
        .all(|c| c.is_ascii_hexdigit()));
    assert!(!provider.traces.is_empty());
    assert!(provider
        .traces
        .iter()
        .all(|span| span.source == "clock-雫🙂.ssrg" && span.start < span.end));
}

#[test]
fn hidden_sleep_rejects_unproved_duration_row_channels_source_arity_and_evidence() {
    let module = checked_wait();
    emit_rust_module_with_options(&module, "main", &options()).unwrap();
    let original = wait_body(&mut module.clone()).clone();
    let (arguments, origin) = match &original {
        CoreExpr::EffectOperation {
            arguments, origin, ..
        }
        | CoreExpr::EffectInvoke {
            arguments, origin, ..
        } => (arguments.clone(), origin.clone()),
        other => panic!("expected checked Clock Effect body: {other:#?}"),
    };
    assert_eq!(arguments.len(), 1);
    let eq = checked("pub fn equality -> Bool = 1 == 1\n");
    let CoreExpr::Binary { evidence, .. } = &eq.functions[0].body else {
        panic!()
    };
    for mutation in 0..12 {
        let mut changed = module.clone();
        let mut arguments = arguments.clone();
        let mut requirements = clock_row();
        let mut failure = ty("std/prelude::Never");
        let mut success = ty("std/prelude::Unit");
        let mut forged_evidence = vec![];
        match mutation {
            0 => arguments.clear(),
            1 => arguments.push(arguments[0].clone()),
            2 => {
                arguments[0] = CoreExpr::Unit {
                    origin: origin.clone(),
                }
            }
            3 | 4 | 5 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                    panic!()
                };
                *type_ref = match mutation {
                    3 => CoreType::ExternalNamed {
                        name: "Duration".into(),
                        canonical: "other::Duration".into(),
                        arguments: vec![],
                    },
                    4 => CoreType::Hole,
                    _ => CoreType::Named {
                        name: "std/time::Duration".into(),
                        arguments: vec![ty("std/prelude::Int")],
                    },
                };
            }
            6 => {
                let CoreType::Record { closed, .. } = &mut requirements else {
                    panic!()
                };
                *closed = false;
            }
            7 => {
                let CoreType::Record { fields, .. } = &mut requirements else {
                    panic!()
                };
                fields[0].type_ref = CoreType::ExternalNamed {
                    name: "Clock".into(),
                    canonical: "other::Clock".into(),
                    arguments: vec![],
                };
            }
            8 => {
                let CoreType::Record { fields, .. } = &mut requirements else {
                    panic!()
                };
                fields[0].optional = true;
            }
            9 => failure = ty("std/prelude::String"),
            10 => success = ty("std/prelude::Int"),
            _ => forged_evidence = evidence.clone(),
        }
        *wait_body(&mut changed) = CoreExpr::EffectInvoke {
            callee: "std/clock::sleep".into(),
            callee_signature: None,
            requirements,
            failure,
            success,
            arguments,
            evidence: forged_evidence,
            origin: origin.clone(),
        };
        let error = emit_rust_module_with_options(&changed, "main", &options())
            .expect_err("unproved Clock fields never enter runtime");
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap().source, "clock-雫🙂.ssrg");
    }
}

#[test]
fn milliseconds_preserves_canonical_owner_kind_signature_and_one_source_argument() {
    let module = checked("import * as time from \"std/time\"\npub fn probe -> Either<time.DurationError, time.Duration> = time.milliseconds 1\n");
    let artifact = emit_rust_module(&module, "probe").unwrap();
    assert!(
        artifact.metadata.providers.is_empty(),
        "pure Duration conversion does not select Clock"
    );
    let mut authoritative = module.clone();
    if let CoreExpr::Call {
        callee_signature,
        type_ref,
        ..
    } = &mut authoritative.functions[0].body
    {
        callee_signature.get_or_insert_with(|| CoreCallSignature {
            type_parameters: vec![],
            parameters: vec![ty("std/prelude::Int")],
            result: type_ref.clone(),
            argument_types: vec![ty("std/prelude::Int")],
            instantiated_type_arguments: vec![],
        });
    } else {
        panic!("checked pure operation must retain its source call");
    }
    emit_rust_module(&authoritative, "probe")
        .expect("explicit checked signature agrees with the canonical operation");
    for mutation in 0..9 {
        let mut changed = authoritative.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            evidence,
            callee_signature,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => arguments.clear(),
            1 => arguments.push(arguments[0].clone()),
            2 => {
                arguments[0] = CoreExpr::Unit {
                    origin: seseragi_lowering::core_expr_origin(&arguments[0]).clone(),
                }
            }
            3 => *type_ref = CoreType::Hole,
            4 | 5 => {
                let (CoreType::Named { arguments, .. } | CoreType::ExternalNamed { arguments, .. }) =
                    type_ref
                else {
                    panic!()
                };
                arguments[1] = if mutation == 4 {
                    CoreType::ExternalNamed {
                        name: "Duration".into(),
                        canonical: "other::Duration".into(),
                        arguments: vec![],
                    }
                } else {
                    ty("std/prelude::Int")
                };
            }
            6 => {
                callee_signature
                    .as_mut()
                    .expect("checked callable scheme")
                    .parameters
                    .clear();
            }
            7 => {
                callee_signature
                    .as_mut()
                    .expect("checked callable scheme")
                    .result = CoreType::Hole;
            }
            _ => {
                let eq = checked("pub fn equality -> Bool = 1 == 1\n");
                let CoreExpr::Binary { evidence: seed, .. } = &eq.functions[0].body else {
                    panic!()
                };
                evidence.push(seed[0].clone());
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert_eq!(error.origin.unwrap().source, "clock-雫🙂.ssrg");
    }
}

#[test]
fn ordinary_saved_and_returned_clock_functions_retain_source_stages() {
    let module = checked(
        r#"
import * as effects from "std/effect"
import * as clock from "std/clock"
import { Clock } from "std/clock"
import * as time from "std/time"
fn conversion -> (Int -> Either<time.DurationError, time.Duration>) = time.milliseconds
fn sleeper -> (time.Duration -> Effect<{clock: Clock}, Never, Unit>) = clock.sleep
pub effect fn main -> Unit with Clock = do {
  let make = conversion ()
  match make 0 {
    Left _ -> effects.succeed ()
    Right pause -> do { let wait = sleeper (); wait pause }
  }
}
"#,
    );
    let artifact = emit_rust_module_with_options(&module, "main", &options()).unwrap();
    assert_eq!(artifact.metadata.providers.len(), 1);
    let sleeper = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::sleeper"))
        .unwrap();
    assert_eq!(
        sleeper.parameters.len(),
        1,
        "the source Unit call is retained before the returned Duration call"
    );
}

#[test]
fn finite_selection_never_admits_now_or_another_provider_or_program_wasm_clock() {
    for identity in [
        "bun/process#clock",
        "seseragi/runtime-native#clock",
        "other#clock-sleep",
    ] {
        assert!(NativeClockProvider::from_identity(identity).is_err());
    }
    let now = checked("import * as clock from \"std/clock\"\npub effect fn main -> Unit with Clock = do { clock.now (); succeed () }\n");
    let error = emit_rust_module_with_options(&now, "main", &options()).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert_eq!(error.origin.unwrap().source, "clock-雫🙂.ssrg");
    let module = checked_wait();
    let error = emit_program_wasm(&[&module], &module.module, &["main".into()]).unwrap_err();
    assert_eq!(error.code, "SES-RUST-0001");
    assert!(error.origin.is_some());
    for source in [
        "import * as clock from \"std/clock\"\nstruct Duration { value: Int }\neffect fn wait value: Duration -> Unit with Clock = clock.sleep value\npub fn probe -> Unit = ()\n",
        "import * as clock from \"std/clock\"\neffect fn wait<Duration> value: Duration -> Unit with Clock = clock.sleep value\npub fn probe -> Unit = ()\n",
    ] {
        compile_core_module(CompileInput::new("nominal.ssrg", "native/clock", source)).expect_err("spelling cannot supply the Duration owner");
    }
}

#[test]
fn non_clock_metadata_preserves_the_existing_optional_field_contract() {
    let module = checked("pub fn probe -> Int = 42\n");
    let artifact = emit_rust_module(&module, "probe").unwrap();
    let json = serde_json::to_value(&artifact.metadata).unwrap();
    assert!(json.get("providers").is_none());
    assert_eq!(
        artifact.metadata,
        emit_rust_module_with_options(&module, "probe", &options())
            .unwrap()
            .metadata
    );
}
