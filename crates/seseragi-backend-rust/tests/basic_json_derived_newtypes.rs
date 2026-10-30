use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreInstanceEvidence, CoreInstanceImplementation, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "newtype-雪🙂.ssrg",
        "native/json-newtypes",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}
#[test]
fn transparent_newtypes_keep_generic_recursive_and_saved_child_codecs() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-derived-newtypes.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
}
#[test]
fn newtype_dictionaries_reject_forged_heads_constructor_payload_and_children() {
    let core = checked(
        "// 雪🙂\nnewtype Id deriving JsonEncode, JsonDecode = Int\npub fn main -> Int = 0\n",
    );
    emit_rust_module(&core, "main").unwrap();
    for index in 0..core.instances.len() {
        for mutation in 0..13 {
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
                0 => instance.identity = "fake::JsonEncode<Id>".into(),
                1 => *transparent_newtype = false,
                2 => *adt_symbol = "fake::Id".into(),
                3 => payload_evidence.clear(),
                4 => payload_evidence.push(payload_evidence[0].clone()),
                5 => payload_evidence[0].variant_symbol = "fake::Id".into(),
                6 => {
                    payload_evidence[0].evidence = CoreInstanceEvidence::Standard {
                        identity: "std/bool::JsonEncode".into(),
                        type_arguments: vec![],
                        evidence_arguments: vec![],
                    }
                }
                7 => instance.arguments[0] = CoreType::Hole,
                8 => instance.supertrait_count = 1,
                9 => instance
                    .type_parameters
                    .push(seseragi_syntax::TypeParameter::value("A")),
                10 => instance.type_identity = None,
                11 => instance.trait_name = "Show".into(),
                _ => payload_evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
            }
            let error = emit_rust_module(&bad, "main").unwrap_err();
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
            0 => bad.adts[0].newtype = false,
            1 => bad.adts[0].variants[0].symbol = "fake::Id".into(),
            2 => bad.adts[0].variants[0].payload = None,
            _ => {
                let duplicate = bad.adts[0].variants[0].clone();
                bad.adts[0].variants.push(duplicate);
            }
        }
        let error = emit_rust_module(&bad, "main").unwrap_err();
        let expected = if matches!(mutation, 1 | 3) {
            &core.adts[0].variants[0].origin
        } else {
            &core.instances[0].origin
        };
        assert_eq!(error.code, "SES-RUST-0001");
        assert_eq!(
            error.origin.as_ref(),
            Some(expected),
            "{mutation}:{error:?}"
        );
    }
}
#[test]
fn ignored_newtype_json_dictionaries_do_not_admit_program_wasm() {
    for method in ["JsonEncode", "JsonDecode"] {
        let core = checked(&format!(
            "newtype Id deriving {method} = Int\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/json-newtypes", &["main".into()]).unwrap_err();
        assert!(
            error.message.contains("require a target-specific"),
            "{error:?}"
        );
    }
}
