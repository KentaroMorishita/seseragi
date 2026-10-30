use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreInstanceEvidence, CoreInstanceImplementation, CoreType};
fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "adt-雪🙂.ssrg",
        "native/json-adts",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn derived_adts_keep_generic_recursive_saved_and_no_payload_unit_distinctions() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-derived-adts.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
}
#[test]
fn adt_dictionaries_reject_forged_heads_variants_payloads_and_children_at_source() {
    let core=checked("// 雪🙂\ntype Choice<A> deriving JsonEncode, JsonDecode = | Empty | Loaded A | Other Int\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    for index in 0..core.instances.len() {
        for mutation in 0..14 {
            let mut bad = core.clone();
            let instance = &mut bad.instances[index];
            let CoreInstanceImplementation::DerivedJson {
                adt_symbol,
                payload_evidence,
                transparent_newtype,
            } = &mut instance.implementation
            else {
                panic!()
            };
            match mutation {
                0 => *adt_symbol = "fake::Choice".into(),
                1 => *transparent_newtype = true,
                2 => payload_evidence.clear(),
                3 => payload_evidence.push(payload_evidence[0].clone()),
                4 => payload_evidence[0].variant_symbol = "fake::Loaded".into(),
                5 => {
                    payload_evidence[1].variant_symbol = payload_evidence[0].variant_symbol.clone()
                }
                6 => payload_evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                7 => {
                    payload_evidence[1].evidence = CoreInstanceEvidence::Standard {
                        identity: "std/bool::JsonEncode".into(),
                        type_arguments: vec![],
                        evidence_arguments: vec![],
                    }
                }
                8 => instance.arguments[0] = CoreType::Hole,
                9 => instance.supertrait_count = 1,
                10 => instance.type_parameters.clear(),
                11 => instance.trait_name = "Show".into(),
                12 => instance.type_identity = None,
                _ => instance.identity = "fake::JsonEncode<Choice<A>>".into(),
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
            assert_eq!(error.code, "SES-RUST-0001");
            assert_eq!(
                error.origin.as_ref(),
                Some(&core.instances[index].origin),
                "{index}:{mutation}:{error:?}"
            );
        }
    }
    for mutation in 0..4 {
        let mut bad = core.clone();
        match mutation {
            0 => bad.adts[0].newtype = true,
            1 => bad.adts[0].variants[1].name = "Other".into(),
            2 => bad.adts[0].variants[1].payload = Some(CoreType::Hole),
            _ => bad.adts[0].variants[1].symbol = "native/json-adts::Ghost".into(),
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        assert_eq!(
            error.origin.as_ref(),
            Some(&core.instances[0].origin),
            "{mutation}:{error:?}"
        );
    }
}
#[test]
fn ignored_derived_adt_dictionaries_do_not_admit_program_wasm() {
    for method in ["JsonEncode", "JsonDecode"] {
        let core = checked(&format!(
            "type Choice deriving {method} = | Empty | Loaded Int\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error = emit_program_wasm(&[&core], "native/json-adts", &["main".into()]).unwrap_err();
        assert!(
            error.message.contains("require a target-specific"),
            "{error:?}"
        );
    }
}
