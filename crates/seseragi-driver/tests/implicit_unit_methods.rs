use seseragi_driver::{compile_core_module, compile_module, CompileInput};

#[test]
fn implicit_unit_methods_share_checked_signatures_and_provider_provenance() {
    for prefix in ["", "type Unit = | LocalUnit\n"] {
        for body in ["ready<Int> ()", "{ let saved = ready<Int>; saved () }"] {
            let source = format!("{prefix}trait Ready<A> {{ fn ready -> Int }}\ninstance Ready<Int> {{ fn ready -> Int = 7 }}\npub fn probe -> Int = {body}\n");
            let input = CompileInput::new("unit-λ.ssrg", "fixture/unit", &source);
            let core = compile_core_module(input.clone()).unwrap().core_ir;
            let compiled = compile_module(input).unwrap();
            assert_eq!(core, compiled.core_ir);
            assert_eq!(core.trait_methods[0].signature.parameters.len(), 1);
            assert!(compiled.diagnostics.diagnostics.is_empty());
        }
    }
}

#[test]
fn nominal_unit_argument_is_rejected_before_either_backend() {
    let source = "// 雫\ntype Unit = | LocalUnit\ntrait Ready<A> { fn ready -> Int }\ninstance Ready<Int> { fn ready -> Int = 7 }\npub fn probe -> Int = ready<Int> LocalUnit\n";
    let input = CompileInput::new("unit-owner-λ.ssrg", "fixture/unit-owner", source);
    let core = compile_core_module(input.clone()).unwrap_err();
    let ts = compile_module(input).unwrap_err();
    assert_eq!(core, ts);
    assert_eq!(core.diagnostics.len(), 1);
    assert_eq!(
        core.diagnostics[0].message_key,
        "call.argument-type-mismatch"
    );
    let span = core.diagnostics[0].primary;
    assert_eq!(&source[span.start..span.end], "LocalUnit");
}
