use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreCallSignature, CoreExpr, CoreModule, CoreType};

fn checked(source: &str) -> CoreModule {
    compile_core_module(CompileInput::new(
        "calendar-雫🙂.ssrg",
        "native/calendar",
        source,
    ))
    .unwrap_or_else(|error| panic!("{source}\n{error:#?}"))
    .core_ir
}

#[test]
fn calendar_calls_and_saved_partials_keep_canonical_types() {
    let core = checked(
        r#"
import * as time from "std/time"
fn saved -> (Int -> Int -> Int -> Either<time.DateTimeError, time.LocalDate>) = time.localDate
fn partial year: Int -> (Int -> Int -> Either<time.DateTimeError, time.LocalDate>) = time.localDate year
fn render value: time.LocalDate -> String = time.formatLocalDate value
pub fn main -> String = match partial 2024 2 29 {
  Left error -> show error
  Right value -> render value
}
"#,
    );
    let artifact = emit_rust_module(&core, "main").unwrap();
    assert!(artifact
        .metadata
        .capabilities
        .iter()
        .any(|c| c == "time-calendar"));
    assert!(artifact.source.contains("native_time_operation"));
}

#[test]
fn calendar_rejects_forged_owners_kinds_signatures_and_stages() {
    let mut core = checked(
        "import * as time from \"std/time\"\nfn probe value: time.LocalDate -> String = time.formatLocalDate value\npub fn main -> Int = 1\n",
    );
    emit_rust_module(&core, "main").unwrap();
    let CoreExpr::Call {
        callee_signature,
        type_ref,
        ..
    } = &mut core.functions[0].body
    else {
        panic!()
    };
    *callee_signature = Some(CoreCallSignature {
        type_parameters: vec![],
        parameters: vec![CoreType::Named {
            name: "std/time::LocalDate".into(),
            arguments: vec![],
        }],
        result: type_ref.clone(),
        argument_types: vec![CoreType::Named {
            name: "std/time::LocalDate".into(),
            arguments: vec![],
        }],
        instantiated_type_arguments: vec![],
    });
    emit_rust_module(&core, "main").unwrap();
    for mutation in 0..10 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            arguments,
            type_ref,
            callee_signature,
            deferred_evidence_parameters,
            evidence_argument_index,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => *type_ref = CoreType::Hole,
            1..=3 => {
                let CoreExpr::Variable { type_ref, .. } = &mut arguments[0] else {
                    panic!()
                };
                *type_ref = match mutation {
                    1 => CoreType::Hole,
                    2 => CoreType::Named {
                        name: "fake::LocalDate".into(),
                        arguments: vec![],
                    },
                    _ => CoreType::Named {
                        name: "std/time::LocalDate".into(),
                        arguments: vec![CoreType::Hole],
                    },
                };
            }
            4 => arguments.clear(),
            5 => arguments.push(arguments[0].clone()),
            6 => callee_signature.as_mut().unwrap().parameters.clear(),
            7 => callee_signature.as_mut().unwrap().result = CoreType::Hole,
            8 => deferred_evidence_parameters.push(CoreType::Hole),
            9 => *evidence_argument_index = Some(0),
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "main").unwrap_err();
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
    }
}

#[test]
fn calendar_is_not_admitted_in_unused_program_wasm_functions() {
    let core = checked(
        "import * as time from \"std/time\"\nfn unused value: time.LocalDate -> String = time.formatLocalDate value\npub fn main -> Int = 1\n",
    );
    let error = emit_program_wasm(&[&core], &core.module, &["main".into()]).unwrap_err();
    assert!(error.origin.is_some());
}
