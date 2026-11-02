use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreCallSignature, CoreExpr, CoreType};

#[test]
fn timezone_service_and_saved_conversions_are_admitted() {
    let source = r#"
import * as time from "std/time"
fn saved -> (time.Instant -> time.TimeZone -> time.ZonedDateTime) = time.atTimeZone
fn resolve local: time.LocalDateTime -> zone: time.TimeZone -> time.LocalResolution = time.resolveLocal local zone
effect fn load id: String -> time.TimeZone with TimeZones fails time.TimeZoneError = time.loadTimeZone id
effect fn version -> String with TimeZones = time.databaseVersion ()
pub fn main -> Int = 0
"#;
    let core = compile_core_module(CompileInput::new("timezones.ssrg", "native/zones", source))
        .unwrap()
        .core_ir;
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "time-zones"));
}

const API_FUNCTIONS: &[&str] = &[
    "effect fn probe -> String with TimeZones = time.databaseVersion ()",
    "effect fn probe id: String -> time.TimeZone with TimeZones fails time.TimeZoneError = time.loadTimeZone id",
    "fn probe zone: time.TimeZone -> String = time.timeZoneId zone",
    "fn probe zone: time.TimeZone -> String = time.timeZoneVersion zone",
    "fn probe instant: time.Instant -> zone: time.TimeZone -> time.ZonedDateTime = time.atTimeZone instant zone",
    "fn probe local: time.LocalDateTime -> zone: time.TimeZone -> time.LocalResolution = time.resolveLocal local zone",
    "fn probe zoned: time.ZonedDateTime -> time.Instant = time.zonedInstant zoned",
    "fn probe zoned: time.ZonedDateTime -> time.LocalDateTime = time.zonedLocalDateTime zoned",
    "fn probe zoned: time.ZonedDateTime -> time.UtcOffset = time.zonedOffset zoned",
    "fn probe zoned: time.ZonedDateTime -> time.TimeZone = time.zonedTimeZone zoned",
];

#[test]
fn all_timezone_apis_reject_unproven_argument_result_and_effect_types() {
    for function in API_FUNCTIONS {
        let source =
            format!("import * as time from \"std/time\"\n{function}\npub fn main -> Int = 0\n");
        let core = compile_core_module(CompileInput::new("zones.ssrg", "native/zones", &source))
            .unwrap()
            .core_ir;
        emit_rust_module(&core, "main").unwrap();
        for mutation in 0..5 {
            let mut changed = core.clone();
            let probe = changed
                .functions
                .iter_mut()
                .find(|f| f.symbol == "native/zones::probe")
                .unwrap();
            match &mut probe.body {
                CoreExpr::Call {
                    arguments,
                    type_ref,
                    callee_signature,
                    deferred_evidence_parameters,
                    ..
                } => match mutation {
                    0 => *type_ref = CoreType::Hole,
                    1 => match &mut arguments[0] {
                        CoreExpr::Variable { type_ref, .. } => {
                            *type_ref = CoreType::Named {
                                name: "fake::TimeZone".into(),
                                arguments: vec![],
                            }
                        }
                        CoreExpr::Unit { origin } => {
                            arguments[0] = CoreExpr::String {
                                value: "wrong Unit".into(),
                                origin: origin.clone(),
                            }
                        }
                        other => panic!("unexpected argument: {other:#?}"),
                    },
                    2 => arguments.push(arguments[0].clone()),
                    3 => {
                        let signature = callee_signature.get_or_insert_with(|| CoreCallSignature {
                            type_parameters: vec![],
                            parameters: vec![],
                            result: CoreType::Hole,
                            argument_types: vec![],
                            instantiated_type_arguments: vec![],
                        });
                        signature.result = CoreType::Hole;
                    }
                    _ => deferred_evidence_parameters.push(CoreType::Hole),
                },
                CoreExpr::EffectInvoke {
                    arguments,
                    requirements,
                    failure,
                    success,
                    ..
                } => match mutation {
                    0 => *success = CoreType::Hole,
                    1 => {
                        *failure = CoreType::Named {
                            name: "fake::TimeZoneError".into(),
                            arguments: vec![],
                        }
                    }
                    2 => arguments.push(arguments[0].clone()),
                    3 => {
                        *requirements = CoreType::Record {
                            closed: false,
                            fields: vec![],
                        }
                    }
                    _ => {
                        *requirements = CoreType::Record {
                            closed: true,
                            fields: vec![seseragi_lowering::CoreRecordField {
                                name: "timeZones".into(),
                                optional: false,
                                type_ref: CoreType::Named {
                                    name: "fake::TimeZones".into(),
                                    arguments: vec![],
                                },
                            }],
                        };
                    }
                },
                other => panic!("unexpected probe body: {other:#?}"),
            }
            let error = emit_rust_module(&changed, "main").unwrap_err();
            assert!(error.origin.is_some(), "{function}/{mutation}: {error}");
        }
        let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
        assert!(error.origin.is_some(), "{function}: {error}");
    }
}

#[test]
fn packed_timezone_data_tracks_the_committed_ts_bundle() {
    use sha2::{Digest, Sha256};
    let digest = format!(
        "{:x}",
        Sha256::digest(include_bytes!("../../../runtime/ts/src/timezone-rules.js"))
    );
    assert!(include_str!("../../seseragi-native-runtime/src/timezone_data.rs").contains(&digest));
}
