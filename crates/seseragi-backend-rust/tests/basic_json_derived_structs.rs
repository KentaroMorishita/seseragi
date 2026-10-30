use seseragi_backend_rust::{emit_program_wasm, emit_rust_module};
use seseragi_driver::{compile_core_module, CompileInput};
use seseragi_lowering::{CoreInstanceEvidence, CoreInstanceImplementation, CoreType};

fn checked(source: &str) -> seseragi_lowering::CoreModule {
    compile_core_module(CompileInput::new(
        "derived-雪🙂.ssrg",
        "native/json-derived",
        source,
    ))
    .unwrap_or_else(|e| panic!("{source}\n{e:?}"))
    .core_ir
}

#[test]
fn derived_structs_keep_generic_recursive_and_saved_method_dictionaries() {
    let core = checked(include_str!(
        "../../../examples/spec/fixtures/rust-backend/basic-json-derived-structs.ssrg"
    ));
    emit_rust_module(&core, "main").unwrap_or_else(|e| panic!("{e:?}"));
}

#[test]
fn derived_struct_dictionaries_reject_unproven_heads_and_fields_at_the_declaration() {
    let core = checked("// 雪🙂\nstruct Row<A> deriving JsonEncode, JsonDecode { z: A, a: Int }\npub fn main -> Int = 0\n");
    emit_rust_module(&core, "main").unwrap();
    for index in 0..core.instances.len() {
        for mutation in 0..15 {
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
                0 => *adt_symbol = "fake::Row".into(),
                1 => *transparent_newtype = true,
                2 => payload_evidence.clear(),
                3 => payload_evidence.push(payload_evidence[0].clone()),
                4 => payload_evidence[0].variant_symbol = "fake".into(),
                5 => {
                    payload_evidence[1].variant_symbol = payload_evidence[0].variant_symbol.clone()
                }
                6 => payload_evidence[0].evidence = CoreInstanceEvidence::Parameter { index: 99 },
                7 => {
                    let CoreInstanceEvidence::Standard { identity, .. } =
                        &mut payload_evidence[1].evidence
                    else {
                        panic!()
                    };
                    *identity = "std/bool::JsonEncode".into();
                }
                8 => instance.arguments[0] = CoreType::Hole,
                9 => instance.supertrait_count = 1,
                10 => instance.type_parameters.clear(),
                11 => instance.trait_name = "Show".into(),
                12 => instance.identity = "fake::JsonEncode<Row<A>>".into(),
                13 => instance.type_identity = Some("fake::Row<$0>".into()),
                _ => instance.type_identity = None,
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
}

#[test]
fn unused_derived_json_structs_stay_out_of_program_wasm() {
    for method in ["JsonEncode", "JsonDecode"] {
        let core = checked(&format!(
            "struct Row deriving {method} {{ value: Int }}\npub fn main -> Int = 0\n"
        ));
        emit_rust_module(&core, "main").unwrap();
        let error =
            emit_program_wasm(&[&core], "native/json-derived", &["main".into()]).unwrap_err();
        assert_eq!(error.code, "SES-RUST-0001");
        assert!(
            error.message.contains("require a target-specific"),
            "{error:?}"
        );
    }
}
