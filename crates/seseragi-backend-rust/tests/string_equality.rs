use seseragi_backend_rust::emit_rust_module;
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreExpr, CoreInstanceEvidence, CoreType};

#[test]
fn checked_string_equality_accepts_original_values_and_dictionary_stages() {
    for source in [
        include_str!("../../../examples/spec/fixtures/rust-backend/string-equality-minimal.ssrg"),
        include_str!("../../../examples/spec/fixtures/rust-backend/string-equality.ssrg"),
    ] {
        let compiled = compile_core_module(CompileInput::new(
            "string-equality-雫.ssrg",
            "native/main",
            source,
        ))
        .unwrap();
        emit_rust_module(&compiled.core_ir, "main").unwrap();
    }
}

#[test]
fn string_equality_rejects_counterfeit_owners_types_and_evidence() {
    let core = compile_core_module(CompileInput::new(
        "string-equality-雫.ssrg",
        "native/main",
        "// 雫🙂\npub fn probe -> Bool = \"left\" == \"right\"",
    ))
    .unwrap()
    .core_ir;
    emit_rust_module(&core, "probe").unwrap();
    for mutation in 0..10 {
        let mut changed = core.clone();
        let CoreExpr::Binary {
            evidence, type_ref, ..
        } = &mut changed.functions[0].body
        else {
            panic!()
        };
        match mutation {
            0 => evidence[0].constraint.trait_identity = Some("fake::Eq".into()),
            1 => evidence[0].constraint.name = "Ord".into(),
            2 => evidence[0].constraint.arguments.clear(),
            3 => evidence[0].constraint.arguments[0] = CoreType::Hole,
            4 => {
                evidence[0].constraint.arguments[0] = CoreType::ExternalNamed {
                    name: "String".into(),
                    canonical: "fake::String".into(),
                    arguments: vec![],
                }
            }
            5 => {
                let CoreInstanceEvidence::Standard { type_arguments, .. } =
                    &mut evidence[0].evidence
                else {
                    panic!()
                };
                type_arguments.push(CoreType::Hole);
            }
            6 => {
                let extra = evidence[0].clone();
                let CoreInstanceEvidence::Standard {
                    evidence_arguments, ..
                } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                evidence_arguments.push(extra);
            }
            7 => {
                let CoreInstanceEvidence::Standard { identity, .. } = &mut evidence[0].evidence
                else {
                    panic!()
                };
                *identity = "std/int::Eq".into();
            }
            8 => evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 0 },
            _ => {
                *type_ref = CoreType::Named {
                    name: "std/prelude::String".into(),
                    arguments: vec![],
                }
            }
        }
        let error = emit_rust_module(&changed, "probe").unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001", "case {mutation}");
        assert_eq!(error.origin.unwrap().source, "string-equality-雫.ssrg");
    }
}
