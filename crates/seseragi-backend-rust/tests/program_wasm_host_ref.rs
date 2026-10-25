use seseragi_backend_rust::{emit_program_wasm, PROGRAM_WASM_HOST_EFFECT_ABI};
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn closed_host_effect_factory_can_own_private_data_state() {
    let module = compile_core_module(CompileInput::new(
        "host-ref-雪.ssrg",
        "fixture/host-ref",
        include_str!("../../../examples/spec/fixtures/rust-backend/program-wasm-host-ref.ssrg"),
    ))
    .unwrap()
    .core_ir;
    let artifact = emit_program_wasm(&[&module], &module.module, &["create".into()]).unwrap();
    assert_eq!(artifact.metadata.abi, PROGRAM_WASM_HOST_EFFECT_ABI);
    assert_eq!(artifact.metadata.exports[0].source_arity, 1);
    assert!(artifact.source.contains("PROGRAM_TRACK_REFS: bool = true"));
    assert!(artifact
        .declarations
        .contains("HostEffect<string, string, string>"));
    assert!(!artifact.declarations.contains("Ref<"));
}

#[test]
fn host_leases_and_public_cells_do_not_become_private_data_codecs() {
    for source in [
        "import * as refs from \"std/ref\"\nimport { Ref } from \"std/ref\"\npub effect fn value request: (String -> Effect<{}, String, String>) -> Ref<String> = refs.make \"seed\"\n",
        "import * as refs from \"std/ref\"\npub effect fn value request: (String -> Effect<{}, String, String>) -> String fails String = do { cell <- refs.make request; saved <- refs.get cell; saved \"input\" }\n",
    ] {
        let module = compile_core_module(CompileInput::new("host-cell-refusal.ssrg", "fixture/host-cell-refusal", source)).unwrap().core_ir;
        let error = emit_program_wasm(&[&module], &module.module, &["value".into()])
            .expect_err("a public cell or a cell containing a host lease acquired a codec");
        assert_eq!(error.code, "SES-RUST-0001");
        let origin = error.origin.expect("refusal must retain source range");
        assert_eq!(origin.source, "host-cell-refusal.ssrg");
        assert!(origin.end > origin.start);
    }
}
