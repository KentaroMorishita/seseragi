#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, Scratch};

#[test]
fn float_values_preserve_ieee_arithmetic_selected_evidence_and_canonical_display() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/float-values.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/float-values.stdout").trim_end(),
    );
}

#[test]
fn float_evidence_rejects_mismatched_canonical_constraints() {
    let core = compile_core_module(CompileInput::new(
        "float.ssrg",
        "native/main",
        "pub fn probe -> Float = 1.25 + 2.5\n",
    ))
    .unwrap()
    .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..5 {
        let mut changed = core.clone();
        let CoreExpr::Binary { evidence, .. } = &mut changed.functions[0].body else {
            panic!("binary")
        };
        match mutation {
            0 => {
                evidence[0].constraint.arguments[0] = seseragi_lowering::CoreType::Named {
                    name: "Int".into(),
                    arguments: vec![],
                }
            }
            1 => evidence[0].constraint.trait_identity = Some("user::Add".into()),
            2 => evidence[0].constraint.name = "Sub".into(),
            3 => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                type_arguments.push(seseragi_lowering::CoreType::Named {
                    name: "Float".into(),
                    arguments: vec![],
                });
            }
            _ => {
                let seseragi_lowering::CoreInstanceEvidence::Standard { identity, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!("standard")
                };
                *identity = "std/float::Sub".into();
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}

#[test]
fn generic_arithmetic_uses_the_selected_user_method() {
    run(
        r#"
type Mark = | Mark Float
instance Add<Mark, Mark, Mark> {
  fn add left: Mark -> right: Mark -> Mark = Mark 42.0
}
fn plus<A> left: A -> right: A -> A where Add<A, A, A> = left + right
pub fn probe -> Float = match plus (Mark 1.0) (Mark 2.0) { Mark value -> value }
"#,
        "42.0",
    );
}

#[test]
fn float_canonical_dispatch_rejects_another_trait_or_method() {
    let core = compile_core_module(CompileInput::new(
        "float.ssrg",
        "native/main",
        "pub fn probe -> String = show 1.0\n",
    ))
    .unwrap()
    .core_ir;
    for method in [false, true] {
        let mut changed = core.clone();
        let CoreExpr::Call {
            trait_dispatch: Some(dispatch),
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("dispatch")
        };
        if method {
            dispatch.method = "debug".into()
        } else {
            dispatch.trait_identity = "user::Show".into()
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(error.origin.is_some());
    }
}

#[test]
fn float_dictionary_arithmetic_preserves_a_user_methods_first_source_stage() {
    let artifact = emit(
        r#"
type Mark = | Mark Int
instance Add<Mark, Mark, Mark> {
  fn add left: Mark -> (Mark -> Mark) = {
    let bad = 9007199254740991 + 1
    \right -> Mark 0
  }
}
pub fn probe -> Mark = (Mark 0) + (Mark (9007199254740991 * 9007199254740991))
"#,
    );
    let scratch = Scratch::new();
    let output = Command::new(scratch.compile(&artifact.source))
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!output.status.success());
    // The method's first stage raises the safe-range defect, before the
    // second source argument can raise its different i64 multiplication defect.
    assert!(String::from_utf8_lossy(&output.stderr).contains("Seseragi Int overflow"));
}
