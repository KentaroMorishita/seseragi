#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, Scratch};

#[test]
fn constrained_local_callables_keep_dictionary_frames_and_source_arity() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/local-dictionaries.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/local-dictionaries.stdout")
            .trim_end(),
    );
}

#[test]
fn constrained_locals_preserve_enclosing_self_tail_calls_and_snapshots() {
    let source = r#"
fn carry remaining: Int -> snapshot: (Unit -> String) -> (Unit -> String) = {
  let render = \value -> show value
  let current = \_: Unit -> render remaining
  if remaining == 0 then snapshot else carry (remaining - 1) current
}
pub fn probe -> String = (carry 100000 (\_: Unit -> "initial")) ()
"#;
    let artifact = emit(source);
    assert!(artifact.metadata.self_tail_calls > 0);
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n");
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn constrained_local_evidence_rejects_forged_frames_traits_and_staging() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreStatement, CoreType};
    let source =
        "// 雫🙂\npub fn probe -> String = { let render = \\value -> show value; render 42 }\n";
    let core = compile_core_module(CompileInput::new("local-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    for case in 0..15 {
        let mut changed = core.clone();
        let CoreExpr::Sequence {
            statements, result, ..
        } = &mut changed.functions[0].body
        else {
            panic!("sequence")
        };
        let CoreStatement::LocalFunction {
            constraints,
            type_parameters,
            parameters,
            body,
            rec_group,
            origin,
            ..
        } = &mut statements[0]
        else {
            panic!("local function")
        };
        let CoreExpr::Call {
            evidence,
            evidence_argument_index,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            ..
        } = &mut **result
        else {
            panic!("local call")
        };
        match case {
            0 => evidence.clear(),
            1 => evidence.push(evidence[0].clone()),
            2 => evidence[0].constraint.name = "Debug".into(),
            3 => evidence[0].constraint.trait_identity = Some("other::Show".into()),
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::Named {
                    name: "Bool".into(),
                    arguments: vec![],
                }
            }
            5 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            6 => constraints[0].trait_identity = Some("other::Show".into()),
            7 => type_parameters.clear(),
            8 => parameters.clear(),
            9 => *rec_group = Some(origin.clone()),
            10 => *evidence_argument_index = Some(0),
            11 => deferred_evidence_parameters.push(CoreType::Hole),
            12 => deferred_evidence_type_constructor_parameters.push("F".into()),
            13 | 14 => {
                let CoreExpr::Call { evidence, .. } = body else {
                    panic!("trait call")
                };
                if case == 13 {
                    evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 1 };
                } else {
                    evidence[0].constraint.name = "Debug".into();
                }
            }
            _ => unreachable!(),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {case}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "local-λ.ssrg");
        assert!(
            origin.start < origin.end && origin.end <= source.len(),
            "case {case}"
        );
        assert!(source.is_char_boundary(origin.start) && source.is_char_boundary(origin.end));
    }
}
