#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::CoreExpr;
use std::process::Command;
use support::{emit, run, Scratch};

#[test]
fn integer_bases_patterns_and_safe_boundaries_use_shared_canonical_values() {
    run(
        include_str!("../../../examples/spec/fixtures/rust-backend/integer-literals.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/integer-literals.stdout")
            .trim_end(),
    );
}

#[test]
fn primitive_partial_binding_rejects_unproven_staging_and_source_arity() {
    for source in [
        "fn same<A> left: A -> right: A -> Bool where Eq<A> = left == right\npub fn probe -> Bool = { let partial = same 'λ'; partial 'λ' }\n",
        "fn same<A> left: A -> right: A -> A where Add<A, A, A> = left + right\npub fn probe -> Float = { let partial = same 1.25; partial 2.5 }\n",
    ] {
    let core = compile_core_module(CompileInput::new("char.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    assert!(emit_rust_module(&core, "probe").is_ok());
    for mutation in 0..5 {
        let mut changed = core.clone();
        let function = changed
            .functions
            .iter_mut()
            .find(|function| function.symbol.ends_with("::probe"))
            .unwrap();
        let CoreExpr::Sequence { statements, .. } = &mut function.body else {
            panic!("expected sequence")
        };
        let seseragi_lowering::CoreStatement::PureLet { value, .. } = &mut statements[0] else {
            panic!("expected partial binding")
        };
        let CoreExpr::Call {
            callee,
            arguments,
            evidence,
            deferred_evidence_parameters,
            deferred_evidence_type_constructor_parameters,
            origin,
            ..
        } = value
        else {
            panic!("expected partial call")
        };
        match mutation {
            0 => deferred_evidence_parameters.push(seseragi_lowering::CoreType::Named {
                name: "Char".into(),
                arguments: vec![],
            }),
            1 => deferred_evidence_type_constructor_parameters.push("F".into()),
            2 => evidence[0].constraint.trait_identity = Some("user::Eq".into()),
            3 => *callee = "unknown::same".into(),
            _ => arguments.push(CoreExpr::Char {
                value: "λ".into(),
                origin: origin.clone(),
            }),
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "mutation={mutation}");
        assert!(error.origin.is_some());
    }
}
}

#[test]
fn defects_keep_checked_int_bounds_and_application_order() {
    run("pub fn probe -> Int = -7 + 49\n", "42");
    for (source, expected) in [
        ("pub fn probe -> Int = 9007199254740991 + 1\n", "Seseragi Int overflow"),
        // returned's body must run before evaluating the second argument.
        ("fn returned seed: Int -> (Int -> Int) = { let bad = 9007199254740991 + 1; \\value -> value }\npub fn probe -> Int = returned 0 (9007199254740991 * 9007199254740991)\n", "Seseragi Int overflow"),
    ] {
        let artifact = emit(source); let scratch = Scratch::new();
        let output = Command::new(scratch.compile(&artifact.source)).env("PATH", "").output().unwrap();
        assert!(!output.status.success()); assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}
