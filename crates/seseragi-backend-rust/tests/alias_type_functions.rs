use seseragi_backend_rust::emit_program_wasm;
use seseragi_driver::{compile_core_module, CompileInput};

#[test]
fn new_compile_time_type_functions_do_not_extend_program_wasm_admission() {
    let source = "type Pair<A,B> = | Pair (A,B)\nalias Flip<A,B> = Pair<B,A>\ntrait Tag<F<_,_>> { fn tag<A,B> value:F<A,B> -> String }\ninstance Tag<Flip> { fn tag<A,B> value:Pair<B,A> -> String = \"flip\" }\npub fn probe -> String = tag<Flip,Int,Bool> (Pair (True,42))\n";
    let compiled = compile_core_module(CompileInput::new(
        "type-functions.ssrg",
        "fixture/type-functions",
        source,
    ))
    .unwrap_or_else(|diagnostics| panic!("{diagnostics:#?}"));
    let error = emit_program_wasm(
        &[&compiled.core_ir],
        &compiled.core_ir.module,
        &["probe".into()],
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checked type-function dictionaries are unsupported"),
        "{error}"
    );
}

#[test]
fn structural_alias_heads_do_not_extend_program_wasm_admission() {
    let source = include_str!(
        "../../../examples/spec/fixtures/rust-backend/structural-alias-functions.ssrg"
    );
    let compiled = compile_core_module(CompileInput::new(
        "structural.ssrg",
        "fixture/structural",
        source,
    ))
    .unwrap_or_else(|e| panic!("{e:#?}"));
    let error = emit_program_wasm(
        &[&compiled.core_ir],
        &compiled.core_ir.module,
        &["probe".into()],
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("checked type-function dictionaries are unsupported"),
        "{error}"
    );
}
