#[path = "support/native.rs"]
mod support;

use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{core_free_variables, is_closed_local_evidence, CoreExpr};
use support::{emit, run};

#[test]
fn maybe_display_rejects_counterfeit_dictionary_owners_and_children() {
    use seseragi_lowering::{CoreInstanceEvidence, CoreType};
    let source = "// 雫🙂\npub fn probe -> String = show [42][0]\n";
    let core = compile_core_module(CompileInput::new("maybe-λ.ssrg", "native/main", source))
        .unwrap()
        .core_ir;
    for case in 0..8 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            trait_dispatch,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!("display")
        };
        match case {
            0 => evidence[0].constraint.trait_identity = Some("other::Show".into()),
            1 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "Maybe".into(),
                    canonical: "other::Maybe".into(),
                    arguments: vec![CoreType::Named {
                        name: "Int".into(),
                        arguments: vec![],
                    }],
                }
            }
            2 => trait_dispatch.as_mut().unwrap().trait_identity = "other::Show".into(),
            _ => {
                let CoreInstanceEvidence::Standard {
                    type_arguments,
                    evidence_arguments,
                    ..
                } = &mut evidence[0].evidence
                else {
                    panic!("Maybe")
                };
                match case {
                    3 => type_arguments.clear(),
                    4 => evidence_arguments.clear(),
                    5 => type_arguments[0] = CoreType::Hole,
                    6 => evidence_arguments[0].constraint.name = "Debug".into(),
                    7 => {
                        evidence_arguments[0].constraint.arguments[0] = CoreType::Named {
                            name: "Bool".into(),
                            arguments: vec![],
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {case}");
        let origin = error.origin.unwrap();
        assert_eq!(origin.source, "maybe-λ.ssrg");
        assert!(origin.start < origin.end && origin.end <= source.len());
    }
}

#[test]
fn parameter_dependent_dictionaries_do_not_reuse_another_instance() {
    let source = format!("{}\npub fn probe -> (Int, Int, Int, Int) = (nested 35, returned 35, parameterized 35, other 25)\n", include_str!("../../../examples/spec/fixtures/rust-backend/dictionaries.ssrg"));
    run(&source, "(42, 84, 42, 42)");
    let artifact = emit(&source);
    assert!(artifact.metadata.closed_dictionary_sites > 0);
}

#[test]
fn closed_fact_excludes_parameter_evidence() {
    let module = compile_core_module(CompileInput::new(
        "main.ssrg",
        "native/main",
        include_str!("../../../examples/spec/fixtures/rust-backend/dictionaries.ssrg"),
    ))
    .unwrap()
    .core_ir;
    assert!(!is_closed_local_evidence(
        &module,
        &seseragi_lowering::CoreInstanceEvidence::Parameter { index: 0 }
    ));
    let function = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::parameterized"))
        .unwrap();
    let CoreExpr::Call { evidence, .. } = &function.body else {
        panic!("expected trait call");
    };
    assert!(!is_closed_local_evidence(&module, &evidence[0].evidence));
    let captured = module
        .functions
        .iter()
        .find(|f| f.symbol.ends_with("::captured"))
        .unwrap();
    assert!(core_free_variables(&captured.body)
        .unwrap()
        .evidence_parameters
        .contains(&0));
}
