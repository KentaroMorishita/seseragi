use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn standard_error_show_supports_source_values_and_saved_dictionary_stages() {
    let source =
        include_str!("../../../examples/spec/fixtures/rust-backend/int-parse-error-show.ssrg");
    let core = compile_core_module(CompileInput::new(
        "parse-error-雫.ssrg",
        "proof/error-show",
        source,
    ))
    .unwrap();
    emit_rust_module(&core.core_ir, "main").unwrap();
}

#[test]
fn standard_error_show_rejects_forged_type_owner_kind_evidence_and_dispatch() {
    let core = compile_core_module(CompileInput::new(
        "parse-error-雫🙂.ssrg",
        "proof/error-show",
        "import * as ints from \"std/int\"\npub fn probe -> String = show ints.EmptyInt",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..11 {
        let mut changed = core.clone();
        let CoreExpr::Call {
            evidence,
            arguments,
            trait_dispatch,
            type_ref,
            ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        let child = evidence[0].clone();
        let selected = &mut evidence[0];
        match mutation {
            0 => selected.constraint.arguments.clear(),
            1 => {
                selected.constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "IntParseError".into(),
                    canonical: "counterfeit::IntParseError".into(),
                    arguments: vec![],
                }
            }
            2 => {
                selected.constraint.arguments[0] = CoreType::Named {
                    name: "std/int::IntParseError".into(),
                    arguments: vec![CoreType::Hole],
                }
            }
            3 => selected.constraint.trait_identity = Some("counterfeit::Show".into()),
            4 => selected.constraint.name = "Debug".into(),
            5 | 6 => {
                let CoreInstanceEvidence::Standard {
                    type_arguments,
                    evidence_arguments,
                    ..
                } = &mut selected.evidence
                else {
                    panic!()
                };
                if mutation == 5 {
                    type_arguments.push(CoreType::Hole);
                } else {
                    evidence_arguments.push(child);
                }
            }
            7 => trait_dispatch.as_mut().unwrap().method = "debug".into(),
            8 => trait_dispatch.as_mut().unwrap().trait_identity = "counterfeit::Show".into(),
            9 => arguments.clear(),
            _ => *type_ref = CoreType::Hole,
        }
        let error = match emit_rust_module(&changed, "probe") {
            Err(error) => error,
            Ok(_) => panic!("mutation {mutation} was admitted"),
        };
        assert_eq!(error.code, "SES-RUST-0001", "mutation {mutation}");
        assert!(error.origin.is_some(), "mutation {mutation}: {error}");
    }
}
