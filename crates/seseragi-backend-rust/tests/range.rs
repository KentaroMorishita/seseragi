use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_expr_origin, CoreExpr, CoreInstanceEvidence, CoreType};
use std::{fs, process::Command};

#[test]
fn range_fixture_executes_with_native_iterable_reducible_and_display() {
    let core = compile_core_module(CompileInput::new(
        "range.ssrg",
        "native/range",
        include_str!("../../../examples/spec/fixtures/rust-backend/range.ssrg"),
    ))
    .unwrap();
    let artifact = emit_rust_module(&core.core_ir, "main").unwrap();
    let scratch =
        std::env::temp_dir().join(format!("seseragi-native-range-{}", std::process::id()));
    fs::create_dir(&scratch).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(scratch.clone());
    let source = scratch.join("program.rs");
    let binary = scratch.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&source, artifact.source).unwrap();
    let build = Command::new("rustc")
        .args(["--edition=2021", "-O", "-o"])
        .arg(&binary)
        .arg(&source)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output = Command::new(binary)
        .env("PATH", "")
        .env("SESERAGI_NATIVE_METRICS", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.stdout,
        include_bytes!("../../../examples/spec/fixtures/rust-backend/range.stdout")
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("managed-live=0"));
}

#[test]
fn range_operators_reject_unproven_bounds_results_and_evidence() {
    let equality = compile_core_module(CompileInput::new(
        "range.ssrg",
        "native/range",
        "pub fn equal -> Bool = 1 == 1",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary {
        evidence: other, ..
    } = &equality.functions[0].body
    else {
        panic!()
    };
    for operator in ["..", "..="] {
        let core = compile_core_module(CompileInput::new(
            "range.ssrg",
            "native/range",
            &format!("pub fn probe -> Range<Int> = 1 {operator} 3"),
        ))
        .unwrap()
        .core_ir;
        emit_rust_module(&core, "probe").unwrap();
        let origin = core_expr_origin(&core.functions[0].body).clone();
        for mutation in 0..5 {
            let mut bad = core.clone();
            let CoreExpr::Binary {
                left,
                right,
                type_ref,
                evidence,
                ..
            } = &mut bad.functions[0].body
            else {
                panic!()
            };
            match mutation {
                0 => *type_ref = CoreType::Hole,
                1 => {
                    *type_ref = CoreType::ExternalNamed {
                        name: "Range".into(),
                        canonical: "fake::Range".into(),
                        arguments: vec![CoreType::Named {
                            name: "Int".into(),
                            arguments: vec![],
                        }],
                    }
                }
                2 => {
                    **left = CoreExpr::Boolean {
                        value: true,
                        origin: origin.clone(),
                    }
                }
                3 => {
                    **right = CoreExpr::Boolean {
                        value: false,
                        origin: origin.clone(),
                    }
                }
                _ => evidence.push(other[0].clone()),
            }
            let error = emit_rust_module(&bad, "probe").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(error.origin.unwrap(), origin);
        }
    }
}

#[test]
fn range_iterable_requires_int_elements_and_closed_provider_arity() {
    let core = compile_core_module(CompileInput::new(
        "range.ssrg", "native/range", "import * as arrays from \"std/array\"\npub fn main -> Array<Int> = arrays.fromIterable (1 ..= 3)",
    )).unwrap().core_ir;
    emit_rust_module(&core, "main").unwrap();
    let origin = core_expr_origin(&core.functions[0].body).clone();
    for mutation in 0..4 {
        let mut bad = core.clone();
        let CoreExpr::Call { evidence, .. } = &mut bad.functions[0].body else {
            panic!()
        };
        let selected = &mut evidence[0];
        let CoreInstanceEvidence::Standard {
            identity,
            type_arguments,
            evidence_arguments,
        } = &mut selected.evidence
        else {
            panic!()
        };
        match mutation {
            0 => type_arguments.push(CoreType::Hole),
            1 => evidence_arguments.push(other_evidence()),
            2 => *identity = "fake::Iterable".into(),
            _ => {
                let float = CoreType::Named {
                    name: "Float".into(),
                    arguments: vec![],
                };
                selected.constraint.arguments[0] = CoreType::Named {
                    name: "Range".into(),
                    arguments: vec![float.clone()],
                };
                selected.constraint.arguments[1] = float;
            }
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(error.origin.unwrap(), origin);
    }
}

fn other_evidence() -> seseragi_lowering::CoreCallEvidence {
    let core = compile_core_module(CompileInput::new(
        "range.ssrg",
        "native/range",
        "pub fn main -> Bool = 1 == 1",
    ))
    .unwrap()
    .core_ir;
    let CoreExpr::Binary { evidence, .. } = &core.functions[0].body else {
        panic!()
    };
    evidence[0].clone()
}
